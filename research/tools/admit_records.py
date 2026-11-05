"""Check declared development record/file relations without running experiments.

Unexecuted implementation draft. A complete byte-reference check is not proof
of actual execution, pre-run sealing, source/binary provenance or correct labels.
This tool cannot admit a final evaluation or change its freeze record.
"""
import argparse
from datetime import datetime, timezone
import hashlib
from pathlib import Path
import re

if __package__:
    from . import study_records
    from .evidence_io import (MAX_FILES, MAX_JSON_BYTES, MAX_FILE_BYTES, MAX_TOTAL_BYTES, contained_file,
                             read_fingerprint, read_json, relative_name, require, strict_json, write_json)
else:
    import study_records
    from evidence_io import (MAX_FILES, MAX_JSON_BYTES, MAX_FILE_BYTES, MAX_TOTAL_BYTES, contained_file,
                            read_fingerprint, read_json, relative_name, require, strict_json, write_json)


ROLE_BINDINGS = {"runner": "runner_sha256", "protocol": "protocol_sha256",
                 "fixture_manifest": "fixture_manifest_sha256", "oracle_manifest": "oracle_manifest_sha256"}
ROLES = set(ROLE_BINDINGS) | {"fixture_input", "oracle_input", "raw_evidence", "source_snapshot"}


def hash_value(value, length=64):
    return isinstance(value, str) and re.fullmatch(f"[0-9a-f]{{{length}}}", value) is not None


def file_entry(value, *, role=True):
    keys = {"path", "bytes", "sha256"} | ({"role"} if role else set())
    require(isinstance(value, dict) and set(value) == keys, "Invalid declared artifact fields")
    relative_name(value["path"])
    require(type(value["bytes"]) is int and 0 <= value["bytes"] <= MAX_FILE_BYTES, "Invalid declared artifact byte size")
    require(hash_value(value["sha256"]), "Invalid declared artifact digest")
    if role:
        require(isinstance(value["role"], str) and value["role"] in ROLES, "Unknown artifact role")
        if value["role"] in ROLE_BINDINGS:
            require(value["bytes"] > 0, "Required manifest/runner/protocol artifact is empty")
        if value["role"] in {"fixture_manifest", "oracle_manifest"}:
            require(value["bytes"] <= MAX_JSON_BYTES, "Input manifest exceeds JSON budget")


def manifest_entries(manifest):
    require(isinstance(manifest, dict) and set(manifest) ==
            {"schema_version", "kind", "run_id", "variant", "bindings", "files"}, "Invalid candidate manifest fields")
    require(type(manifest["schema_version"]) is int and manifest["schema_version"] == 1 and
            manifest["kind"] == "development_candidate_not_frozen", "This checker accepts only development candidate manifests")
    require(study_records.nonempty(manifest["run_id"]) and study_records.nonempty(manifest["variant"]), "Missing run/variant identity")
    bindings = manifest["bindings"]
    require(isinstance(bindings, dict) and set(bindings) == set(study_records.BINDINGS), "Invalid manifest bindings")
    require(all(hash_value(value, 40 if key == "implementation_commit" else 64)
                for key, value in bindings.items()), "Incomplete candidate bindings")
    entries = manifest["files"]
    require(isinstance(entries, list) and 0 < len(entries) <= MAX_FILES, "Invalid declared artifact count")
    names, roles, total = {}, {}, 0
    for entry in entries:
        file_entry(entry)
        key = entry["path"].casefold()
        require(key not in names, "Duplicate or case-colliding artifact path")
        names[key] = entry
        total += entry["bytes"]
        require(total <= MAX_TOTAL_BYTES, "Declared artifact total exceeds budget")
        if entry["role"] in ROLE_BINDINGS:
            require(entry["role"] not in roles, "Duplicate binding artifact role")
            roles[entry["role"]] = entry
            require(entry["sha256"] == bindings[ROLE_BINDINGS[entry["role"]]], "Binding artifact digest disagrees with manifest")
    require(set(roles) == set(ROLE_BINDINGS), "A required binding artifact is missing")
    return names, roles


def input_declarations(raw, kind):
    value = strict_json(raw)
    require(isinstance(value, dict) and set(value) == {"schema_version", "kind", "files"}, "Invalid input manifest fields")
    require(type(value["schema_version"]) is int and value["schema_version"] == 1 and value["kind"] == kind,
            "Wrong input manifest kind")
    require(isinstance(value["files"], list) and 0 < len(value["files"]) <= MAX_FILES, "Input manifest has no declared files or exceeds budget")
    names = set()
    for entry in value["files"]:
        file_entry(entry, role=False)
        require(entry["path"].casefold() not in names, "Duplicate input manifest path")
        names.add(entry["path"].casefold())
    return value["files"]


def admit(rows, manifest, root):
    entries, roles = manifest_entries(manifest)
    # Validate shared record rules and duplicates without computing accuracy.
    study_records.grouped_records(rows)
    for row in rows:
        require(row["run_id"] == manifest["run_id"] and row["variant"] == manifest["variant"], "Record run/variant differs from candidate manifest")
        require(row["bindings"] == manifest["bindings"], "Record artifact/protocol bindings differ from manifest")
        for reference in row["raw_evidence"]:
            entry = entries.get(reference["path"].casefold())
            require(entry is not None and entry["path"] == reference["path"] and
                    entry["sha256"] == reference["sha256"], "A record references undeclared or differently identified evidence")

    checks, raw_manifests = [], {}
    for key, entry in sorted(entries.items()):
        check = {"path": entry["path"], "role": entry["role"], "expected_bytes": entry["bytes"],
                 "expected_sha256": entry["sha256"], "status": "unresolved"}
        try:
            path = contained_file(root, entry["path"])
            retain = entry["role"] in {"fixture_manifest", "oracle_manifest"}
            # The declared total is also the read budget, even if many files
            # understate their actual size. Oversized inputs stay unresolved.
            receipt, raw = read_fingerprint(path, retain=retain, limit=entry["bytes"])
            check.update(observed_bytes=receipt["bytes"], observed_sha256=receipt["sha256"])
            if receipt["bytes"] != entry["bytes"] or receipt["sha256"] != entry["sha256"]:
                check["status"] = "different_bytes"
            else:
                check["status"] = "matching_declared_bytes"
                if raw is not None:
                    raw_manifests[entry["role"]] = raw
        except (OSError, ValueError) as error:
            check["error_type"] = type(error).__name__
        checks.append(check)

    relations = []
    for role, kind, input_role in (("fixture_manifest", "synthetic_fixture_inputs", "fixture_input"),
                                   ("oracle_manifest", "independent_oracle_inputs", "oracle_input")):
        relation = {"role": role, "status": "unresolved", "declared_input_count": None}
        try:
            require(role in raw_manifests, "Input manifest bytes were not established")
            declared = input_declarations(raw_manifests[role], kind)
            expected_paths = {entry["path"] for entry in entries.values() if entry["role"] == input_role}
            require({entry["path"] for entry in declared} == expected_paths, "Input manifest and candidate file lists differ")
            for entry in declared:
                known = entries[entry["path"].casefold()]
                require(entry["bytes"] == known["bytes"] and entry["sha256"] == known["sha256"] and known["role"] == input_role,
                        "An input manifest reference disagrees with the declared artifact")
            relation.update(status="declared_relations_consistent", declared_input_count=len(declared))
        except (ValueError, UnicodeError, RecursionError) as error:
            relation["error_type"] = type(error).__name__
        relations.append(relation)

    complete = all(check["status"] == "matching_declared_bytes" for check in checks) and all(
        relation["status"] == "declared_relations_consistent" for relation in relations)
    return {"schema": 1, "kind": "development_reference_integrity_not_final_admission",
            "status": "empty_no_records" if not rows else "complete_reference_check" if complete else "unresolved_references",
            "run_id": manifest["run_id"], "variant": manifest["variant"], "bindings": manifest["bindings"],
            "record_count": len(rows), "unrun_record_count": sum(row["system_decision"] == "not_run" for row in rows),
            "byte_references_established": complete, "file_observations": checks, "input_relations": relations,
            "final_evaluation_admitted": False, "experiments_executed": False, "accuracy_computed": False,
            "limits": ["Matching bytes do not establish actual execution, correct labels, independent review or pre-run chronology.",
                       "The implementation commit and source-to-runner relationship remain declared, not authenticated.",
                       "The file lists establish declared closure, not completeness of every actual engine/font/runtime input.",
                       "One explicit controlled root is used; link/identity checks are not a hostile-filesystem security sandbox.",
                       "No candidates or commands from manifests are executed, no sources saved and no freeze record changed."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rows", type=Path)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--development", action="store_true", required=True)
    args = parser.parse_args()
    inputs = {}
    try:
        rows, inputs["rows"] = read_json(args.rows)
        manifest, inputs["manifest"] = read_json(args.manifest)
        result = admit(rows, manifest, args.root)
    except (OSError, ValueError, UnicodeError, RecursionError) as error:
        result = {"schema": 1, "kind": "development_reference_integrity_not_final_admission",
                  "status": "rejected_input", "error_type": type(error).__name__,
                  "final_evaluation_admitted": False, "experiments_executed": False, "accuracy_computed": False}
    result.update(recorded_utc=datetime.now(timezone.utc).isoformat(), input_identities=inputs,
                  checker_files={name: hashlib.sha256(Path(__file__).with_name(name).read_bytes()).hexdigest()
                                 for name in ("admit_records.py", "study_records.py", "evidence_io.py")})
    write_json(args.output, result)
    print("Development reference report written. No application correctness or final admission established.")
    return 0 if result["status"] == "complete_reference_check" else 1


if __name__ == "__main__":
    raise SystemExit(main())
