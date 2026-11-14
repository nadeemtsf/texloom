"""Synthetic infrastructure checks; these values are not app observations."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import admit_records
import evidence_io
import study_records
import timing_records


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def record():
    row = json.loads((Path(__file__).resolve().parents[1] / "evaluation/record-template-v1.json").read_text())
    row.update(run_id="synthetic-run", case_id="case", intent_id="intent", document_id="document",
        study_arm="natural_request", decision_basis="canonical_commit", entry_layer="pdf_selection",
        contract_family="displacement", variant="synthetic", fixture_family="repeated", engine_version="not-a-real-engine",
        language="english", evidence_origin="automated_observation", operation="move", declared_in_scope=True,
        oracle_truth="valid", system_decision="accepted", commit_status="committed", presentation_status="published",
        save_status="not_attempted", terminal_event="completed", adjudicated_correct_if_accepted=True)
    row["required_postconditions"] = list(study_records.POSTCONDITIONS[:3])
    row["postconditions"].update({key: True for key in row["required_postconditions"]})
    row["annotation"] = {"preparer": "synthetic-preparer", "reviewer": "synthetic-reviewer", "status": "reviewed"}
    row["bindings"] = {key: "a" * (40 if key == "implementation_commit" else 64) for key in study_records.BINDINGS}
    row["raw_evidence"] = [{"path": "raw.json", "sha256": sha(b"{}") }]
    return row


class RecordChecks(unittest.TestCase):
    def test_failed_presentation_does_not_erase_a_known_commit(self):
        row = record();row.update(presentation_status="failed",terminal_event="presentation_failure")
        group = study_records.summarize([row])["groups"][0]
        self.assertEqual(group["decisions"], {"accepted": 1})
        self.assertEqual(group["metrics"]["committed_preview_failures"], 1)
        self.assertEqual(group["metrics"]["established_correct_published"]["numerator"], 0)

    def test_unknown_commit_cannot_be_relabelled_a_refusal(self):
        row = record();row.update(system_decision="refused",commit_status="unknown",presentation_status="unknown",
            refusal_stage="commit",refusal_reason="unknown",adjudicated_correct_if_accepted=None)
        with self.assertRaises(ValueError): study_records.summarize([row])

    def test_unrelated_change_fails_safety_even_with_a_valid_intention(self):
        row=record();row["postconditions"]["source_outside_patch_exact"]=False;row["adjudicated_correct_if_accepted"]=False
        group=study_records.summarize([row])["groups"][0]
        self.assertEqual(group["safety_gate"],"fail")
        self.assertEqual(group["known_incorrect_acceptances"],1)

    def test_relabelled_families_and_repeated_intents_do_not_create_independent_cases(self):
        for changes in ({"fixture_family":"other"},{"case_id":"another-case"}):
            first=record();second=copy.deepcopy(first);second.update(changes)
            with self.assertRaises(ValueError):study_records.summarize([first,second])

    def test_deny_all_has_no_accepted_error_fraction(self):
        row=record();row.update(system_decision="refused",commit_status="not_attempted",presentation_status="not_attempted",
            terminal_event="refused",refusal_stage="mapping",refusal_reason="ambiguous_mapping",adjudicated_correct_if_accepted=None)
        row["postconditions"]={key:None for key in study_records.POSTCONDITIONS}
        group=study_records.summarize([row])["groups"][0]
        self.assertIsNone(group["metrics"]["incorrect_acceptance_fraction"]["fraction"])
        self.assertEqual(group["metrics"]["valid_refusal"]["fraction"],1)


class ReferenceChecks(unittest.TestCase):
    def create(self,root):
        data={"runner.bin":(b"synthetic, never executed","runner"),"protocol.json":(b"{}","protocol"),
              "fixture.tex":(b"Synthetic fixture input only","fixture_input"),"expected.tex":(b"Expected source only","oracle_input"),
              "raw.json":(b"{}","raw_evidence")}
        for name,kind,input_name in (("fixtures.json","synthetic_fixture_inputs","fixture.tex"),("oracle.json","independent_oracle_inputs","expected.tex")):
            raw=data[input_name][0]
            data[name]=(json.dumps({"schema_version":1,"kind":kind,"files":[{"path":input_name,"bytes":len(raw),"sha256":sha(raw)}]}).encode(),"fixture_manifest" if name=="fixtures.json" else "oracle_manifest")
        entries=[]
        for name,(raw,role) in data.items():
            (root/name).write_bytes(raw);entries.append({"path":name,"bytes":len(raw),"sha256":sha(raw),"role":role})
        row=record()
        for entry in entries:
            if entry["role"] in admit_records.ROLE_BINDINGS:row["bindings"][admit_records.ROLE_BINDINGS[entry["role"]]]=entry["sha256"]
        manifest={"schema_version":1,"kind":"development_candidate_not_frozen","run_id":row["run_id"],"variant":row["variant"],"bindings":row["bindings"],"files":entries}
        return row,manifest

    def test_matching_bytes_do_not_become_final_admission(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);row,manifest=self.create(root)
            result=admit_records.admit([row],manifest,root)
            self.assertEqual(result["status"],"complete_reference_check")
            self.assertFalse(result["final_evaluation_admitted"]);self.assertFalse(result["experiments_executed"])
            (root/"raw.json").write_bytes(b'{"changed":true}')
            self.assertEqual(admit_records.admit([row],manifest,root)["status"],"unresolved_references")

    def test_fixture_manifest_omission_cannot_pass_declared_closure(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);row,manifest=self.create(root)
            manifest["files"]=[entry for entry in manifest["files"] if entry["role"]!="fixture_input"]
            self.assertEqual(admit_records.admit([row],manifest,root)["status"],"unresolved_references")

    def test_strict_input_rejects_duplicate_keys_overflow_and_nonportable_paths(self):
        for raw in (b'{"a":1,"a":2}',b'{"n":1e999}',b'{"n":NaN}'):
            with self.assertRaises(ValueError):evidence_io.strict_json(raw)
        for name in ("../actual.tex","C:/actual.tex","a:stream","CON.txt","a//b","a\\b","a?.json"):
            with self.assertRaises(ValueError):evidence_io.relative_name(name)


class FingerprintChecks(unittest.TestCase):
    def test_windows_handle_ctime_difference_does_not_reject_unchanged_bytes(self):
        if evidence_io.os.name != "nt":
            self.skipTest("Windows handle/path metadata regression")
        real_fstat = evidence_io.os.fstat
        def handle_info(fd):
            info = real_fstat(fd)
            return SimpleNamespace(**{name: getattr(info, name) for name in
                ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_mode")},
                st_ctime_ns=info.st_ctime_ns + 10000)
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "unchanged.bin"
            path.write_bytes(b"unchanged evidence")
            with patch.object(evidence_io.os, "fstat", side_effect=handle_info):
                receipt, raw = evidence_io.read_fingerprint(path, retain=True)
            self.assertEqual(raw, b"unchanged evidence")
            self.assertEqual(receipt["sha256"], sha(raw))

    def test_handle_metadata_change_during_read_is_still_rejected(self):
        real_fstat = evidence_io.os.fstat
        calls = 0
        def handle_info(fd):
            nonlocal calls
            calls += 1
            info = real_fstat(fd)
            return SimpleNamespace(**{name: getattr(info, name) for name in
                ("st_dev", "st_ino", "st_size", "st_ctime_ns", "st_mode")},
                st_mtime_ns=info.st_mtime_ns + (calls > 1))
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "changing.bin"
            path.write_bytes(b"controlled evidence")
            with patch.object(evidence_io.os, "fstat", side_effect=handle_info):
                with self.assertRaisesRegex(ValueError, "changed during fingerprinting"):
                    evidence_io.read_fingerprint(path)


class TimingChecks(unittest.TestCase):
    def create(self,root):
        snapshot={"schema":2,"clock":"performance.now","endpoint":"two-animation-frames-proxy",
            "collector_id":"12345678-1234-4123-8123-123456789abc","epoch":0,"first_id":1,"last_id":1,
            "clear_history":{"completed":0,"pending":0},"enabled":True,"capacity":256,"evicted":0,"dropped":0,"discarded":0,"invalid_starts":0,
            "active":0,"pending":[],"records":[{"id":1,"kind":"text-move","started_ms":100,"phases":[],"truncated_phases":0,
                "outcome":"published","operation_ms":10,"frame_proxy_ms":16,"frame_status":"observed"}]}
        context={key:"synthetic" for key in timing_records.CONTEXT}
        context.update(source_commit="a"*40,executable_sha256="a"*64,protocol_sha256="b"*64,environment_sha256="c"*64,cache_state="warm",primary_kind="text-move")
        raw=json.dumps(snapshot).encode();(root/"trace.json").write_bytes(raw)
        batch={"context":context,"snapshot":{"path":"trace.json","bytes":len(raw),"sha256":sha(raw)},"trials":[{"trial_id":"trial","record_id":1,"role":"measured"}]}
        return {"schema_version":1,"kind":"development_timing_batches_not_final","batches":[batch]}

    def test_endpoints_are_separate_and_published_is_not_correctness(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);result=timing_records.summarize(self.create(root),root)
            observed=result["groups"][0]["outcomes"]["published"]
            self.assertEqual(observed["conditional_operation_distribution"]["median_ms"],10)
            self.assertEqual(observed["conditional_frame_proxy_distribution"]["median_ms"],16)
            self.assertFalse(result["correctness_computed"]);self.assertFalse(result["physical_latency_computed"])

    def test_missing_trial_suppresses_distributions_instead_of_zero_filling(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);manifest=self.create(root)
            manifest["batches"][0]["trials"].append({"trial_id":"missing","record_id":None,"role":"measured"})
            group=timing_records.summarize(manifest,root)["groups"][0]
            self.assertEqual(group["not_observed"],1);self.assertFalse(group["coverage_complete_current_epochs"])
            self.assertIsNone(group["outcomes"]["published"]["conditional_operation_distribution"])

    def test_overlapping_exports_cannot_be_counted_as_new_trials(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);manifest=self.create(root);other=copy.deepcopy(manifest["batches"][0])
            (root/"other.json").write_bytes((root/"trace.json").read_bytes());other["snapshot"]["path"]="other.json";other["trials"][0]["trial_id"]="other-trial"
            manifest["batches"].append(other)
            with self.assertRaisesRegex(ValueError,"Overlapping"):timing_records.summarize(manifest,root)


if __name__=="__main__":unittest.main()
