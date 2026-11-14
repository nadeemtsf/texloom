"""Run and independently summarize the predeclared headless service cost diagnostic."""
import argparse
from collections import defaultdict
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import platform
import statistics
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_new(path, data):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(data, stream, indent=2)
        stream.write("\n")


def capture(args):
    p = subprocess.run(args, capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=30,
                       creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    return {"command": args, "exit": p.returncode, "stdout": p.stdout.strip(), "stderr": p.stderr.strip()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--build-log", type=Path, required=True)
    parser.add_argument("--expected-arms", type=int, choices=[2,3], default=3)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output.resolve()
    if not output.is_relative_to(root / "artifacts") or output.exists():
        raise ValueError("Use a fresh output directory under repository artifacts")
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = args.binary.resolve(strict=True)
    inputs = [Path(__file__), root / "research/SOURCE-INTENT-COST-PLAN.md", root / "Cargo.lock",
              root / "src-tauri/Cargo.toml", root / "src-tauri/examples/source_intent_cost_development.rs"]
    inputs += sorted((root / "src-tauri/src").rglob("*.rs"))
    inputs += sorted((root / "crates/texloom-core/src").rglob("*.rs"))
    inputs += sorted((root / "crates/texloom-build/src").rglob("*.rs"))
    followup = root / "research/TRUSTED-PREFLIGHT-FORK-PLAN.md"
    if followup.exists():
        inputs.append(followup)
    identities = {p.relative_to(root).as_posix(): sha(p) for p in inputs}
    hardware = capture(["pwsh", "-NoProfile", "-Command",
        "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);"
        "[ordered]@{cpu=(Get-CimInstance Win32_Processor|Select-Object -First 1).Name;"
        "ram_bytes=(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory;"
        "os=(Get-CimInstance Win32_OperatingSystem).Caption;"
        "os_build=(Get-CimInstance Win32_OperatingSystem).BuildNumber;"
        "active_power_scheme=(powercfg /getactivescheme)}|ConvertTo-Json -Compress"])
    pre = {"recorded_utc": datetime.now(timezone.utc).isoformat(), "binary_sha256": sha(binary),
           "source_sha256": identities, "build_log_sha256": sha(args.build_log),
           "head": capture(["git", "rev-parse", "HEAD"])["stdout"],
           "rustc": capture(["rustc", "-Vv"]), "hardware": hardware,
           "python_platform": platform.platform(), "background_load_controlled": False,
           "scope": "Working source hashes and controlled build invocation; not a final freeze or binary attestation"}
    pre_path = output.with_name(output.name + "-pre-run.json")
    write_new(pre_path, pre)
    log_path = output.with_name(output.name + "-stdout.log")
    with log_path.open("xb") as log:
        proc = subprocess.run([str(binary), str(output)], stdout=log, stderr=subprocess.STDOUT,
                              timeout=600, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    if proc.returncode:
        raise RuntimeError(f"Native diagnostic failed with exit {proc.returncode}; see retained log")
    result = json.loads((output / "results.json").read_text(encoding="utf-8"))
    rows = [json.loads(line) for line in (output / "samples.jsonl").read_text(encoding="utf-8").splitlines()]
    assert result["status"] == "passed" and len(rows) == args.expected_arms * 3 * 35
    groups = defaultdict(list)
    for row in rows:
        assert row["exact_full_source_and_one_step_undo"] and row["protected_acknowledgement"]
        if not row["warmup"]:
            groups[(row["project_source_bytes"], row["mode"])].append(row)
    assert len(groups) == args.expected_arms * 3
    for key in groups:
        assert sum(1 for row in rows if (row["project_source_bytes"],row["mode"])==key and row["warmup"]) == 5
    summaries = []
    for group, samples in sorted(groups.items()):
        assert len(samples) == 30
        derived = {"project_source_bytes": group[0], "mode": group[1], "samples": len(samples)}
        native = next(s for s in result["summaries"] if (s["project_source_bytes"], s["mode"]) == group)
        for field in ["native_service_and_reply_ms", "reply_json_encoding_ms", "native_service_through_encoding_ms", "response_bytes"]:
            values = sorted(float(r[field]) for r in samples)
            assert all(math.isfinite(v) and v >= 0 for v in values)
            d = {"median": statistics.median(values), "p95": values[math.ceil(.95 * len(values)) - 1], "max": max(values)}
            for stat, value in d.items():
                assert math.isclose(native[field][stat], value, rel_tol=1e-12, abs_tol=1e-12)
            derived[field] = d
        summaries.append(derived)
    changed = [name for name, value in identities.items() if sha(root/name) != value]
    assert not changed, f"Source changed during measurement: {changed}"
    originals = {p.relative_to(output).as_posix(): {"bytes": p.stat().st_size, "sha256": sha(p)}
                 for p in sorted(output.glob("source-bytes-*/*/project/*.tex"))}
    analysis = {"schema": 1, "status": "passed", "independent_summary_matches": True,
                "source_changed_during_run": changed, "summaries": summaries, "original_files": originals,
                "pre_run": str(pre_path), "limits": ["Different protection/reply contracts", "One run with correlated repetitions", "No controlled background load, physical input, transport or display", "No TeX or rendered acceptance", "No final performance study"]}
    write_new(output / "analysis.json", analysis)
    print(json.dumps({"status": "passed", "groups": len(summaries), "raw_rows": len(rows), "independent_summary_matches": True}))
    for s in summaries:
        print(json.dumps({"MiB": s["project_source_bytes"] / 1048576, "mode": s["mode"],
                          "p95_service_ms": s["native_service_and_reply_ms"]["p95"],
                          "p95_encoding_ms": s["reply_json_encoding_ms"]["p95"],
                          "p95_combined_ms": s["native_service_through_encoding_ms"]["p95"],
                          "response_bytes": s["response_bytes"]["median"]}))


if __name__ == "__main__":
    main()
