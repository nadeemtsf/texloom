"""Development annotation records with separate commit, display and Save outcomes.

Implementation draft: not executed or validated during the no-tests continuation.
Supplied labels and artifact references are not independently established here.
This tool cannot freeze, run or certify a final evaluation.
"""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

if __package__:
    from .evidence_io import read_json
else:
    from evidence_io import read_json


GROUP_FIELDS = ("run_id", "study_arm", "decision_basis", "entry_layer",
                "contract_family", "variant", "fixture_family", "engine_version",
                "language", "evidence_origin")
POSTCONDITIONS = ("target_correct", "source_outside_patch_exact",
                  "unrelated_rendered_content_preserved", "undo_source_exact",
                  "redo_source_exact", "save_reopen_exact")
CORE_POSTCONDITIONS = set(POSTCONDITIONS[:3])
BINDINGS = ("implementation_commit", "runner_sha256", "protocol_sha256",
            "fixture_manifest_sha256", "oracle_manifest_sha256")
DECISIONS = {"accepted", "refused", "compile_failure", "timeout", "crash",
             "cancelled", "commit_unknown", "not_run"}
PRESENTATION = {"published", "failed", "unknown", "not_attempted", "not_applicable"}
SAVE = {"saved", "failed", "unknown", "not_attempted", "not_applicable"}
TERMINAL_EVENTS = {"completed", "refused", "compile_failure", "presentation_failure",
                   "timeout", "crash", "cancelled", "unknown", "not_run"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def nonempty(value):
    return isinstance(value, str) and bool(value.strip()) and len(value) <= 512 and "\0" not in value


def choice(row, key, allowed):
    require(isinstance(row.get(key), str) and row[key] in allowed, f"Unknown {key}")


def exact_keys(value, keys, label):
    require(isinstance(value, dict) and set(value) == set(keys), f"Invalid {label} fields")


def validate(row):
    require(isinstance(row, dict), "A study record must be an object")
    require(type(row.get("schema_version")) is int and row["schema_version"] == 1,
            "Use record-template-v1.json; legacy rows require the legacy analysis tool")
    for key in (*GROUP_FIELDS, "case_id", "intent_id", "document_id", "operation"):
        require(nonempty(row.get(key)), f"Missing annotation field: {key}")
    choice(row, "study_arm", {"natural_request", "injected_candidate_diagnostic"})
    choice(row, "decision_basis", {"canonical_commit", "component_predicate"})
    choice(row, "entry_layer", {"pdf_selection", "source_mapping", "source_plan",
                                "candidate_build", "comparator"})
    choice(row, "contract_family", {"displacement", "reflow", "semantic", "legacy"})
    choice(row, "evidence_origin", {"automated_observation", "native_observation", "user_reported"})
    choice(row, "oracle_truth", {"valid", "unsafe", "unresolved"})
    choice(row, "system_decision", DECISIONS)
    choice(row, "terminal_event", TERMINAL_EVENTS)
    choice(row, "presentation_status", PRESENTATION)
    choice(row, "save_status", SAVE)
    choice(row, "commit_status", {"committed", "not_committed", "unknown", "not_attempted", "not_applicable"})
    require(type(row.get("declared_in_scope")) is bool, "Declare support before observing outcomes")
    if row["study_arm"] == "natural_request":
        require(row["decision_basis"] == "canonical_commit" and row["entry_layer"] == "pdf_selection",
                "Natural visual requests must enter at PDF selection with canonical-commit decisions")

    decision = row["system_decision"]
    if decision == "refused":
        choice(row, "refusal_stage", {"mapping", "identity", "source_plan", "native_validation",
                                       "frontend_validation", "commit", "component_predicate", "unknown"})
        choice(row, "refusal_reason", {"ambiguous_mapping", "unsupported_structure", "source_plan_mismatch",
                                        "preservation_mismatch", "resource_budget", "stale", "other", "unknown"})
    else:
        require(row.get("refusal_stage") is None and row.get("refusal_reason") is None,
                "Non-refusal outcomes must not be assigned refusal labels")
    canonical = row["decision_basis"] == "canonical_commit"
    if canonical:
        require(row["commit_status"] != "not_applicable", "Canonical commit status must be recorded")
        require(row["presentation_status"] != "not_applicable" and row["save_status"] != "not_applicable",
                "Canonical records must retain display and Save observations or explicit unknown/not-attempted")
        if row["commit_status"] == "committed":
            require(decision == "accepted", "A known commit remains accepted after a display error or crash")
        elif decision == "accepted":
            raise ValueError("Canonical acceptance needs an established commit")
        if row["commit_status"] == "unknown" or decision == "commit_unknown":
            require(row["commit_status"] == "unknown" and decision == "commit_unknown",
                    "An uncertain commit must not become a refusal or a definite acceptance")
        if decision == "not_run":
            require(row["commit_status"] == "not_attempted", "Unrun requests cannot have commit observations")
        elif decision not in {"accepted", "commit_unknown"}:
            require(row["commit_status"] in {"not_committed", "not_attempted"},
                    "A pre-commit failure needs a known absence of commit")
        if row["presentation_status"] == "published" or row["save_status"] in {"saved", "failed"}:
            require(decision == "accepted", "Display/Save observations must concern this accepted edit, not its baseline")
    else:
        require(row["study_arm"] == "injected_candidate_diagnostic", "A predicate replay is a diagnostic")
        require(row["commit_status"] == "not_applicable" and
                row["presentation_status"] == "not_applicable" and row["save_status"] == "not_applicable",
                "Component decisions must not claim a canonical commit, presentation or Save")
        require(decision != "commit_unknown", "A component predicate has no canonical commit status")
    if decision == "not_run":
        require(row["terminal_event"] == "not_run", "Unrun records have no terminal execution event")
        require(row["presentation_status"] in {"not_attempted", "not_applicable"} and
                row["save_status"] in {"not_attempted", "not_applicable"}, "Unrun outcomes must remain unobserved")
    else:
        require(row["terminal_event"] != "not_run", "Executed records need their actual terminal event")

    required = row.get("required_postconditions")
    require(isinstance(required, list) and bool(required) and
            all(isinstance(key, str) and key in POSTCONDITIONS for key in required), "Declare the required postconditions")
    require(len(required) == len(set(required)), "Duplicate required postcondition")
    if canonical:
        require(CORE_POSTCONDITIONS <= set(required), "Canonical correctness needs target, source and rendered preservation")
    post = row.get("postconditions")
    exact_keys(post, POSTCONDITIONS, "postcondition")
    require(all(value is None or type(value) is bool for value in post.values()), "Postconditions are booleans or unresolved")
    if decision == "not_run":
        require(all(value is None for value in post.values()), "Unrun postconditions cannot pass or fail")
    adjudication = row.get("adjudicated_correct_if_accepted")
    require(adjudication is None or type(adjudication) is bool, "Invalid accepted-edit adjudication")
    if decision != "accepted":
        require(adjudication is None, "Nonaccepted requests have no accepted-edit adjudication")
    if adjudication is True:
        require(row["oracle_truth"] == "valid" and all(post[key] is True for key in required),
                "Correct-acceptance adjudication conflicts with the oracle or required postconditions")

    annotation = row.get("annotation")
    exact_keys(annotation, ("preparer", "reviewer", "status"), "annotation provenance")
    choice(annotation, "status", {"draft", "reviewed", "unresolved"})
    for name in ("preparer", "reviewer"):
        require(annotation[name] is None or nonempty(annotation[name]), "Invalid annotation provenance label")
    if annotation["status"] == "reviewed":
        require(nonempty(annotation["preparer"]) and nonempty(annotation["reviewer"]), "Reviewed labels need both roles")

    bindings = row.get("bindings")
    exact_keys(bindings, BINDINGS, "artifact binding")
    for name, value in bindings.items():
        length = 40 if name == "implementation_commit" else 64
        require(value is None and decision == "not_run" or
                isinstance(value, str) and re.fullmatch(f"[0-9a-f]{{{length}}}", value), f"Missing or malformed binding: {name}")
    evidence = row.get("raw_evidence")
    require(isinstance(evidence, list), "Raw evidence references must be explicit")
    require(bool(evidence) or decision == "not_run", "Executed records need raw evidence references")
    for item in evidence:
        exact_keys(item, ("path", "sha256"), "raw evidence")
        path = item["path"]
        require(nonempty(path) and "\\" not in path and ":" not in path and
                not PurePosixPath(path).is_absolute() and ".." not in path.split("/") and
                all(part not in {"", "."} for part in path.split("/")), "Use a relative raw evidence path without traversal")
        require(isinstance(item["sha256"], str) and re.fullmatch("[0-9a-f]{64}", item["sha256"]), "Invalid evidence digest")


def rate(count, total):
    return {"numerator": count, "denominator": total, "fraction": count / total if total else None}


def established_correct(row):
    return (row["system_decision"] == "accepted" and row["oracle_truth"] == "valid" and
            row["annotation"]["status"] == "reviewed" and
            row["adjudicated_correct_if_accepted"] is True and
            all(row["postconditions"][key] is True for key in row["required_postconditions"]))


def established_incorrect(row):
    return (row["system_decision"] == "accepted" and
            (row["oracle_truth"] == "unsafe" or row["adjudicated_correct_if_accepted"] is False or
             any(row["postconditions"][key] is False for key in row["required_postconditions"])))


def grouped_records(rows):
    require(isinstance(rows, list) and len(rows) <= 10000, "Expected at most 10,000 annotation records")
    groups, identities, natural_intents = defaultdict(list), set(), set()
    for row in rows:
        validate(row)
        # Family/contract changes do not create a new correctness request.
        identity = tuple(row[key] for key in GROUP_FIELDS if key not in {"fixture_family", "contract_family"}) + (row["case_id"],)
        require(identity not in identities, "Duplicate correctness request; do not count timing repetitions or relabelled families")
        identities.add(identity)
        if row["study_arm"] == "natural_request":
            intent_identity = identity[:-1] + (row["intent_id"],)
            require(intent_identity not in natural_intents, "Repeated natural intent in one stratum; timing repetitions are separate")
            natural_intents.add(intent_identity)
        groups[tuple(row[key] for key in GROUP_FIELDS)].append(row)
    return groups


def summarize(rows):
    groups = grouped_records(rows)
    result = {"schema": 1, "kind": "development_annotation_summary_v1",
              "status": "development_only" if rows else "empty_no_results",
              "final_evaluation_executed": False, "groups": [],
              "limits": ["Labels, role identifiers and raw file references are supplied; this tool does not verify their truth or file bytes.",
                         "No groups or different entry layers/runs/bases are pooled; repeated runs are not independent replications.",
                         "Adjudication bounds are deterministic, not statistical confidence intervals.",
                         "No latency, physical presentation, tracing overhead or population accuracy is estimated."]}
    for group, data in sorted(groups.items()):
        binding_sets = {tuple(row["bindings"][key] for key in BINDINGS) for row in data}
        require(len(binding_sets) == 1, "Artifact/protocol bindings changed within a stratum; use a separate run/variant")
        decisions = Counter(row["system_decision"] for row in data)
        accepted = [row for row in data if row["system_decision"] == "accepted"]
        valid = [row for row in data if row["oracle_truth"] == "valid"]
        unsafe = [row for row in data if row["oracle_truth"] == "unsafe"]
        in_scope = [row for row in valid if row["declared_in_scope"]]
        bad = sum(established_incorrect(row) for row in accepted)
        unknown = sum(not established_incorrect(row) and not established_correct(row) for row in accepted)
        unresolved_oracles = sum(row["oracle_truth"] == "unresolved" for row in data)
        record = {**dict(zip(GROUP_FIELDS, group)), "case_count": len(data),
                  "intent_count": len({row["intent_id"] for row in data}),
                  "document_count": len({row["document_id"] for row in data}),
                  "decisions": dict(decisions), "terminal_events": dict(Counter(row["terminal_event"] for row in data)),
                  "refusal_stages": dict(Counter(row["refusal_stage"] for row in data if row["system_decision"] == "refused")),
                  "valid_refusal_reasons": dict(Counter(row["refusal_reason"] for row in valid if row["system_decision"] == "refused")),
                  "bindings": data[0]["bindings"], "known_incorrect_acceptances": bad,
                  "accepted_unresolved": unknown, "oracle_unresolved": unresolved_oracles,
                  "labels_without_review": sum(row["annotation"]["status"] != "reviewed" for row in data),
                  "same_label_preparer_and_reviewer": sum(row["annotation"]["status"] == "reviewed" and
                      row["annotation"]["preparer"] == row["annotation"]["reviewer"] for row in data), "metrics": None}
        if bad:
            record["safety_gate"] = "fail"
        elif decisions["not_run"]:
            record["safety_gate"] = "incomplete"
        elif unknown or unresolved_oracles or decisions["commit_unknown"] or record["labels_without_review"]:
            record["safety_gate"] = "unresolved"
        else:
            record["safety_gate"] = "no_observed_incorrect_acceptances"
        if not decisions["not_run"]:
            errors = rate(bad, len(accepted))
            errors.update(lower_bound=errors["fraction"], upper_bound=(bad + unknown) / len(accepted) if accepted else None)
            if unknown:
                errors["fraction"] = None
            record["metrics"] = {
                "incorrect_acceptance_fraction": errors,
                "established_correct_valid_acceptance": rate(sum(established_correct(row) for row in valid), len(valid)),
                "established_correct_in_scope_acceptance": rate(sum(established_correct(row) for row in in_scope), len(in_scope)),
                "valid_refusal": rate(sum(row["system_decision"] == "refused" for row in valid), len(valid)),
                "unsafe_requests_accepted": rate(sum(row["system_decision"] == "accepted" for row in unsafe), len(unsafe)),
                "unsafe_requests_refused": rate(sum(row["system_decision"] == "refused" for row in unsafe), len(unsafe)),
                "required_postcondition_failures_among_accepted": {key: sum(key in row["required_postconditions"] and
                    row["postconditions"][key] is False for row in accepted) for key in POSTCONDITIONS}}
            if data[0]["decision_basis"] == "canonical_commit":
                workflow = [row for row in valid if set(row["required_postconditions"]) == set(POSTCONDITIONS)]
                record["metrics"].update(
                    committed_preview_failures=sum(row["presentation_status"] == "failed" for row in accepted),
                    committed_save_failures=sum(row["save_status"] == "failed" for row in accepted),
                    established_correct_published=rate(sum(established_correct(row) and row["presentation_status"] == "published" for row in valid), len(valid)),
                    full_declared_workflow_success=rate(sum(established_correct(row) and row["presentation_status"] == "published" and
                        row["save_status"] == "saved" for row in workflow), len(workflow)))
        result["groups"].append(record)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rows", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--development", action="store_true", required=True)
    args = parser.parse_args()
    rows, input_receipt = read_json(args.rows)
    result = summarize(rows)
    result.update(recorded_utc=datetime.now(timezone.utc).isoformat(),
                  input_sha256=input_receipt["sha256"],
                  analysis_file_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  data_io_file_sha256=hashlib.sha256(Path(__file__).with_name("evidence_io.py").read_bytes()).hexdigest())
    with args.output.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print("Development annotation summary written; no experiments or file-reference verification performed.")


if __name__ == "__main__":
    main()
