"""Development-only summary of supplied correctness annotations, not new trials.

The script checks denominators/provenance strata. It does not independently
establish that interactions happened or that their oracle labels are correct.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path

GROUP_FIELDS = ("study_arm", "contract_family", "variant", "engine_version", "language", "evidence_origin")
DECISIONS = {"accepted", "refused", "compile_failure", "timeout", "crash", "not_run"}


def rate(numerator, denominator):
    return {"numerator": numerator, "denominator": denominator,
            "fraction": numerator/denominator if denominator else None}


def validate(row):
    for name in (*GROUP_FIELDS, "case_id", "document_id"):
        if not isinstance(row.get(name), str) or not row[name]:
            raise ValueError(f"Missing annotation field: {name}")
    if row["study_arm"] not in {"natural_request", "injected_candidate_diagnostic"}:
        raise ValueError("Unknown study arm")
    if row["contract_family"] not in {"displacement", "reflow", "semantic", "legacy"}:
        raise ValueError("Unknown operation contract")
    if row.get("system_decision") not in DECISIONS or row.get("oracle_truth") not in {"valid", "unsafe", "unresolved"}:
        raise ValueError("Unknown decision or oracle label")
    if not isinstance(row.get("declared_in_scope"), bool):
        raise ValueError("In-scope label is not established")
    for name in ("adjudicated_correct_if_accepted", "source_outside_patch_exact", "unrelated_rendered_content_preserved"):
        value = row.get(name)
        if value is not None and not isinstance(value, bool):
            raise ValueError(f"Expected a boolean or unresolved annotation: {name}")
    adjudication = row.get("adjudicated_correct_if_accepted")
    if row["system_decision"] != "accepted" and adjudication is not None:
        raise ValueError("A nonaccepted request cannot have accepted-edit adjudication")
    if adjudication is True and (row["oracle_truth"] != "valid" or any(
            row.get(name) is False for name in ("source_outside_patch_exact", "unrelated_rendered_content_preserved"))):
        raise ValueError("Correct-acceptance label conflicts with oracle/preservation evidence")


def summarize(rows):
    groups, identities = defaultdict(list), set()
    for row in rows:
        validate(row)
        group = tuple(row[name] for name in GROUP_FIELDS)
        identity = (*group, row["case_id"])
        if identity in identities:
            raise ValueError("Duplicate correctness case; timing repetitions are not new trials")
        identities.add(identity);groups[group].append(row)
    output = {"schema": 1, "kind": "development_analysis_of_supplied_annotations",
              "final_evaluation_executed_by_this_tool": False,
              "status": "empty_no_results" if not rows else "development_only",
              "latency_metrics": "Not computed from correctness rows; separate instrumented repetitions required.",
              "limits": "Annotations/interactions are supplied, not independently established by this script; no strata are pooled.",
              "groups": []}
    for key, data in sorted(groups.items()):
        decisions = Counter(row["system_decision"] for row in data)
        accepted = [row for row in data if row["system_decision"] == "accepted"]
        valid = [row for row in data if row["oracle_truth"] == "valid"]
        unsafe = [row for row in data if row["oracle_truth"] == "unsafe"]
        in_scope = [row for row in valid if row["declared_in_scope"]]
        def incorrect(row):
            return row["oracle_truth"] == "unsafe" or row.get("adjudicated_correct_if_accepted") is False or any(
                row.get(name) is False for name in ("source_outside_patch_exact", "unrelated_rendered_content_preserved"))
        def correct(row):
            return (row["system_decision"] == "accepted" and row["oracle_truth"] == "valid"
                    and row.get("adjudicated_correct_if_accepted") is True
                    and row.get("source_outside_patch_exact") is True
                    and row.get("unrelated_rendered_content_preserved") is True)
        bad = sum(incorrect(row) for row in accepted)
        unknown = sum(not incorrect(row) and not correct(row) for row in accepted)
        oracle_unknown = sum(row["oracle_truth"] == "unresolved" for row in data)
        record = {**dict(zip(GROUP_FIELDS, key)), "case_count": len(data),
                  "document_count": len({row["document_id"] for row in data}),
                  "decisions": dict(decisions), "known_incorrect_acceptances": bad,
                  "accepted_unresolved": unknown, "oracle_unresolved": oracle_unknown,
                  "metrics": None}
        if decisions["not_run"]:
            record["safety_gate"] = "fail" if bad else "incomplete"
        else:
            error_rate = rate(bad, len(accepted))
            error_rate.update(lower_bound=error_rate["fraction"],
                              upper_bound=(bad+unknown)/len(accepted) if accepted else None)
            if unknown:
                error_rate["fraction"] = None
            record["metrics"] = {
                "incorrect_acceptance_fraction": error_rate,
                "correct_valid_acceptance": rate(sum(correct(row) for row in valid), len(valid)),
                "valid_refusal": rate(sum(row["system_decision"] == "refused" for row in valid), len(valid)),
                "correct_in_scope_acceptance": rate(sum(correct(row) for row in in_scope), len(in_scope)),
                "unsafe_requests_accepted": rate(sum(row["system_decision"] == "accepted" for row in unsafe), len(unsafe)),
                "unsafe_requests_refused": rate(sum(row["system_decision"] == "refused" for row in unsafe), len(unsafe)),
                "accepted_source_preservation_failures": sum(row.get("source_outside_patch_exact") is False for row in accepted),
                "accepted_render_preservation_failures": sum(row.get("unrelated_rendered_content_preserved") is False for row in accepted)}
            record["safety_gate"] = "fail" if bad else ("unresolved" if unknown or oracle_unknown else "no_observed_incorrect_acceptances")
        output["groups"].append(record)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rows", type=Path, help="JSON list; do not pass the unfilled row-specification example")
    parser.add_argument("output", type=Path)
    parser.add_argument("--development", action="store_true", required=True,
                        help="Required: this prototype does not produce final study claims")
    args = parser.parse_args()
    rows = json.loads(args.rows.read_text(encoding="utf-8"))
    if not isinstance(rows, list):
        raise ValueError("Expected a list of actual annotation rows")
    result = summarize(rows)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2);stream.write("\n")
    print("Development annotation summary written; no experiments executed.")


if __name__ == "__main__":
    main()
