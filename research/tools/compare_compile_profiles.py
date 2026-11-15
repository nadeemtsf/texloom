"""Compare the protocol's one-page outputs and summarize native timing stages."""

import argparse
import hashlib
import json
from pathlib import Path
from statistics import median
import subprocess


ENGINES = ("pdflatex", "xelatex", "lualatex")
CASES = ("plain", "references", "large-project")
MODES = ("baseline", "verified-fresh", "verified-warm")


def read_rows(root, mode, engine):
    label = f"{mode}-{engine}"
    path = root / f"{label}.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines()]
    if len(rows) != 18 or any(r["engine"] != engine for r in rows):
        raise ValueError(f"Unexpected sample count or engine in {label}")
    for case in CASES:
        selected = [r for r in rows if r["case"] == case]
        if sorted(r["trial"] for r in selected) != list(range(6)):
            raise ValueError(f"Missing trials in {label}/{case}")
        for row in selected:
            warmup = row["trial"] == 0
            reused = mode == "verified-warm" and not warmup
            if row["warmup"] != warmup or row["timings"].get("version_reused", False) != reused:
                raise ValueError(f"Unexpected warm-up or version probe state in {label}/{case}")
    return {(r["case"], r["trial"]): r for r in rows if not r["warmup"]}


def output_signature(directory):
    pdf = directory / ".out/preview.pdf"
    info = subprocess.check_output(["pdfinfo", str(pdf)], timeout=20)
    if not any(line.split() == [b"Pages:", b"1"] for line in info.splitlines()):
        raise ValueError(f"Expected exactly one page in {pdf}")
    text = subprocess.check_output(["pdftotext", "-layout", str(pdf), "-"], timeout=20)
    raster = subprocess.check_output(["pdftoppm", "-r", "96", "-singlefile", str(pdf)], timeout=20)
    if not text.strip() or not raster.startswith(b"P6\n"):
        raise ValueError(f"Missing extracted text or page raster for {pdf}")
    manifest = json.loads((directory / "texloom-build-manifest.json").read_text(encoding="utf-8"))
    return {
        "text_sha256": hashlib.sha256(text).hexdigest(),
        "raster_sha256": hashlib.sha256(raster).hexdigest(),
        "arguments": manifest["arguments"],
        "passes": manifest["passes"],
        "used_project_inputs": manifest["used_project_inputs"],
        "engine_version": manifest["engine_version"],
    }


def compare_pairs(root):
    summaries = []
    for engine in ENGINES:
        directory = root / f"paired-{engine}"
        path = root / f"paired-{engine}.jsonl"
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines()]
        if len(rows) != 36 or any(row["engine"] != engine for row in rows):
            raise ValueError(f"Unexpected paired samples in {path}")
        samples = {(row["case"], row["trial"], row["profile"]): row for row in rows}
        for case in CASES:
            differences = []
            first_pass = []
            version = []
            fresh_seconds = []
            warm_seconds = []
            for trial in range(1, 6):
                prefix = directory / case / f"run-{trial:02}"
                fresh = samples[case, trial, "fresh"]
                warm = samples[case, trial, "warm"]
                if fresh["timings"]["version_reused"] or not warm["timings"]["version_reused"]:
                    raise ValueError(f"Wrong profile result in {prefix}")
                if output_signature(prefix.with_name(prefix.name + "-fresh")) != output_signature(
                    prefix.with_name(prefix.name + "-warm")
                ):
                    raise ValueError(f"Output or manifest mismatch in {prefix}")
                differences.append(fresh["seconds"] - warm["seconds"])
                fresh_seconds.append(fresh["seconds"])
                warm_seconds.append(warm["seconds"])
                first_pass.append(warm["timings"]["first_pass_seconds"])
                version.append(fresh["timings"]["version_seconds"])
            summaries.append({
                "engine": engine,
                "case": case,
                "count": len(differences),
                "median_fresh_seconds": median(fresh_seconds),
                "median_warm_seconds": median(warm_seconds),
                "median_paired_saving_seconds": median(differences),
                "median_warm_first_pass_seconds": median(first_pass),
                "median_fresh_probe_seconds": median(version),
                "individual_paired_saving_seconds": differences,
            })
    return {"matched_pairs": sum(row["count"] for row in summaries), "summaries": summaries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runs", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--paired", action="store_true")
    args = parser.parse_args()
    if args.paired:
        with args.output.open("x", encoding="utf-8", newline="\n") as output:
            json.dump(compare_pairs(args.runs), output, indent=2)
            output.write("\n")
        return
    summaries = []
    matched = 0
    for engine in ENGINES:
        reference = {}
        for mode in MODES:
            rows = read_rows(args.runs, mode, engine)
            for (case, trial), row in rows.items():
                directory = args.runs / f"{mode}-{engine}" / case / f"run-{trial:02}"
                signature = output_signature(directory)
                if row["passes"] != signature["passes"]:
                    raise ValueError(f"Pass count mismatch in {directory}")
                if mode == "baseline":
                    reference[case, trial] = signature
                elif reference[case, trial] != signature:
                    raise ValueError(f"Output or manifest mismatch in {directory}")
                else:
                    matched += 1
            for case in CASES:
                samples = [v for (c, _), v in rows.items() if c == case]
                summaries.append({
                    "engine": engine,
                    "mode": mode,
                    "case": case,
                    "count": len(samples),
                    "median_seconds": median(x["seconds"] for x in samples),
                    "median_timings": {
                        key: median(x["timings"][key] for x in samples)
                        for key in samples[0]["timings"] if key.endswith("_seconds")
                    },
                })
    with args.output.open("x", encoding="utf-8", newline="\n") as output:
        json.dump({"matched_outputs": matched, "summaries": summaries}, output, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
