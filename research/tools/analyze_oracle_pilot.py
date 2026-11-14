"""Replay diagnostic predicates on a completed pilot; no app edits or reruns."""
import argparse
import hashlib
import json
from pathlib import Path


def analyze(report, protocol):
    if report.get("status") not in {"pass", "fail"}:
        raise ValueError("Pilot is unfinished; do not silently analyze a prefix")
    if report.get("kind") != "development_oracle_feasibility_not_final_evaluation":
        raise ValueError("Expected a development feasibility report")
    key = lambda condition: (condition["engine"], condition["multilingual_neighbors"])
    conditions = [key(c) for c in report["conditions"]]
    if len(set(conditions)) != len(conditions) or set(conditions) != {key(c) for c in protocol["conditions"]}:
        raise ValueError("Pilot conditions do not match the pre-run protocol")
    expected_variants = protocol["expected_control_decisions"]
    variants = {name: {"admitted": 0, "matching_reference_controls_admitted": 0,
                        "known_fault_controls_admitted": 0, "correct_controls_not_admitted": 0,
                        "compile_failures_not_admitted": 0, "unresolved": 0}
                for name in ["compile_only", "text_and_geometry", "full_reference"]}
    controls = 0
    for condition in report["conditions"]:
        names = [c["variant"] for c in condition["controls"]]
        if len(set(names)) != len(names) or set(names) != set(expected_variants):
            raise ValueError("Missing or duplicate control rows; do not analyze a prefix")
        for control in condition["controls"]:
            if control["expected"] != expected_variants[control["variant"]]:
                raise ValueError("Control label differs from the pre-run protocol")
            controls += 1
            comparison = control.get("comparison", {})
            compiled = control.get("build", {}).get("ok")
            if (control["actual"] not in {"matches_reference", "differs", "compile_failure"} or not condition["reference"]["ok"]
                    or not isinstance(compiled, bool)):
                decisions = {key: None for key in variants}
            else:
                checks = [comparison.get(k) for k in ["page_contract_equal", "text_multiplicities_equal", "word_geometry_matches"]]
                geometry = False if any(x is False for x in checks) else (True if all(x is True for x in checks) else None)
                decisions = {"compile_only": compiled, "text_and_geometry": False if not compiled else geometry,
                             "full_reference": control["actual"] == "matches_reference"}
            for name, admitted in decisions.items():
                totals = variants[name]
                if admitted is None:
                    totals["unresolved"] += 1
                    continue
                totals["admitted"] += int(admitted)
                if control["expected"] == "matches_reference":
                    totals["matching_reference_controls_admitted"] += int(admitted)
                    totals["correct_controls_not_admitted"] += int(not admitted)
                elif control["expected"] == "compile_failure":
                    totals["compile_failures_not_admitted"] += int(not admitted)
                else:
                    totals["known_fault_controls_admitted"] += int(admitted)
    return {"schema": 1, "kind": "offline_diagnostic_predicate_replay_not_app_acceptance_rates",
            "final_evaluation_executed": False, "conditions": len(report["conditions"]),
            "control_rows": controls, "variants": variants,
            "limits": ["Purposeful controls, not a representative sample of user requests.",
                       "Replay uses the same candidate/reference PDFs and extraction/raster results.",
                       "Neither the native source-plan/selection gates nor full app decisions are replayed.",
                       "Counts are not final correctness, usability or end-to-end performance evidence."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    raw = args.report.read_bytes()
    protocol_raw = args.report.with_name("protocol.json").read_bytes()
    result = analyze(json.loads(raw), json.loads(protocol_raw))
    result["input_report_sha256"] = hashlib.sha256(raw).hexdigest()
    result["pre_run_protocol_sha256"] = hashlib.sha256(protocol_raw).hexdigest()
    result["analysis_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2);stream.write("\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
