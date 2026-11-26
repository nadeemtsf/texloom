"""Compare profiled build outputs and summarize native timing stages."""

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


def output_signature(directory, pages=1, scaling=False):
    pdf = directory / ".out/preview.pdf"
    info = subprocess.check_output(["pdfinfo", str(pdf)], timeout=20)
    if not any(line.split() == [b"Pages:", str(pages).encode()] for line in info.splitlines()):
        raise ValueError(f"Expected exactly {pages} pages in {pdf}")
    text = subprocess.check_output(["pdftotext", "-layout", str(pdf), "-"], timeout=20)
    raster_args = ["-r", "36"] if scaling else ["-r", "96", "-singlefile"]
    raster = subprocess.check_output(["pdftoppm", *raster_args, str(pdf)], timeout=60)
    if not text.strip() or not raster.startswith(b"P6\n"):
        raise ValueError(f"Missing extracted text or page raster for {pdf}")
    if scaling:
        if raster.count(b"P6\n") != pages:
            raise ValueError(f"Missing rendered pages in {pdf}")
        page_text = text.split(b"\f")
        for page in range(1, pages + 1):
            if f"Page {page:03}.".encode() not in page_text[page - 1]:
                raise ValueError(f"Unexpected text on page {page} in {pdf}")
    manifest = json.loads((directory / "texloom-build-manifest.json").read_text(encoding="utf-8"))
    return {
        "text_sha256": hashlib.sha256(text).hexdigest(),
        "raster_sha256": hashlib.sha256(raster).hexdigest(),
        "arguments": manifest["arguments"],
        "passes": manifest["passes"],
        "used_project_inputs": manifest["used_project_inputs"],
        "engine_version": manifest["engine_version"],
    }


def compare_pairs(root, scaling=False):
    summaries = []
    engines = ("pdflatex",) if scaling else ENGINES
    cases = ("pages-1", "pages-20", "pages-100") if scaling else CASES
    for engine in engines:
        directory = root / f"paired-{engine}"
        path = root / f"paired-{engine}.jsonl"
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines()]
        if len(rows) != 36 or any(row["engine"] != engine for row in rows):
            raise ValueError(f"Unexpected paired samples in {path}")
        samples = {(row["case"], row["trial"], row["profile"]): row for row in rows}
        expected = {(case, trial, arm) for case in cases for trial in range(6) for arm in ("fresh", "warm")}
        if samples.keys() != expected or any(row["warmup"] != (row["trial"] == 0) for row in rows):
            raise ValueError(f"Missing, duplicated, or mislabeled trials in {path}")
        for case in cases:
            pages = int(case.removeprefix("pages-")) if scaling else 1
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
                if output_signature(prefix.with_name(prefix.name + "-fresh"), pages, scaling) != output_signature(
                    prefix.with_name(prefix.name + "-warm"), pages, scaling
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
            if scaling:
                summaries[-1]["pages"] = pages
                summaries[-1]["warm_phase_medians_seconds"] = {
                    stage: median(samples[case, trial, "warm"]["timings"][stage] for trial in range(1, 6))
                    for stage in ("snapshot_seconds", "engine_seconds", "verify_seconds",
                                  "recorder_seconds", "finalize_seconds")
                }
    return {"matched_pairs": sum(row["count"] for row in summaries), "summaries": summaries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runs", type=Path)
    parser.add_argument("output", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--paired", action="store_true")
    mode.add_argument("--pages", action="store_true")
    args = parser.parse_args()
    if args.paired or args.pages:
        result = compare_pairs(args.runs, args.pages)
        with args.output.open("x", encoding="utf-8", newline="\n") as output:
            json.dump(result, output, indent=2)
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
