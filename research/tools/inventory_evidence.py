"""Read and fingerprint development receipts. This does not run experiments."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess


def catalog(root: Path) -> dict:
    entries = []
    for path in sorted((root / "docs/evidence").glob("*.json")):
        raw = path.read_bytes()
        entry = {
            "path": path.relative_to(root).as_posix(),
            "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest(),
            "audit_action": "read_parse_fingerprint_only",
            "experiment_reproduced_by_this_audit": False,
        }
        try:
            data = json.loads(raw.decode("utf-8-sig"))
            if not isinstance(data, dict):
                raise ValueError("Receipt root is not an object")
            for key in ("schema", "status", "source_commit", "sourceCommit",
                        "product_source_commit", "implementation_base", "recorded_utc",
                        "createdAt", "date", "run", "release_ready"):
                if key in data:
                    entry[key] = data[key]
            cases = data.get("cases")
            if isinstance(cases, list):
                entry["recorded_case_rows"] = len(cases)
                entry["case_status_counts_as_recorded"] = dict(Counter(
                    str(case.get("status", "not_recorded"))
                    for case in cases if isinstance(case, dict)
                ))
            entry["parse_status"] = "ok"
        except (UnicodeError, ValueError) as error:
            entry["parse_status"] = "failed"
            entry["parse_error"] = str(error)
        entries.append(entry)
    return {
        "schema": 1,
        "kind": "development_evidence_inventory_not_evaluation_results",
        "recorded_utc": datetime.now(timezone.utc).isoformat(),
        "audited_head": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "working_tree_note": "Uncommitted report/evidence changes may be present; see the session checkpoint.",
        "final_evaluation_executed": False,
        "entries": entries,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output or root / "research/evidence-catalog.json"
    result = catalog(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    failures = sum(entry["parse_status"] != "ok" for entry in result["entries"])
    print(f"Cataloged {len(result['entries'])} evidence files; {failures} parse failures; no experiments executed.")
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
