"""Development-only compile/text checks for actual Rust-generated candidates.

No Texloom app, native acceptance path, final fixture, glyph/geometry oracle or
end-to-end latency measurement is exercised. Writes only a new artifacts folder.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[2]
ORIGINAL = r"""\documentclass{article}
\pagestyle{empty}
\begin{document}
Before unchanged.

Original target paragraph.

After unchanged.
\end{document}
"""
REQUESTED = "Revised & confirmed: 50%."

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def call(argv: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(argv, cwd=cwd, capture_output=True, timeout=45, check=False)

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    output, binary = args.output.resolve(), args.binary.resolve()
    output.relative_to((ROOT / "artifacts").resolve())
    if not binary.is_file():
        raise ValueError("The compiled fixture bridge does not exist")
    paths = {name: shutil.which(name) for name in ("pdflatex", "xelatex", "lualatex", "pdftotext")}
    if not all(paths.values()):
        raise ValueError("All three engines and pdftotext must already be installed")
    output.mkdir(parents=True, exist_ok=False)
    fixture = output / "original.tex"
    fixture.write_text(ORIGINAL, encoding="utf-8", newline="")
    record = {"schema": 1, "recorded_utc": datetime.now(timezone.utc).isoformat(),
        "kind": "development_source_intent_compile_text_portability_not_final_evaluation",
        "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "binary_sha256": digest(binary), "runner_sha256": digest(Path(__file__)),
        "requested_literal": REQUESTED, "tools": {}, "rows": [],
        "limitations": ["One synthetic English document on one Windows machine; reused across engines.",
            "Compilation and exact normalized text checks only; no glyph/geometry preservation proof.",
            "No UI selection, native visual acceptance, source commit/Save, RTL rendering or usability trial.",
            "No latency benchmark, final fixture freeze or general correctness/portability rate."]}
    receipt = output / "receipt.json"
    try:
        for name, path in paths.items():
            version = call([path, "-v" if name == "pdftotext" else "--version"], output)
            record["tools"][name] = {"path": path, "sha256": digest(Path(path)),
                "version_exit": version.returncode,
                "version": (version.stdout + version.stderr).decode("utf-8", errors="replace").splitlines()[:2]}
        variants = {"baseline": ORIGINAL.encode("utf-8")}
        for action in ("replace", "delete"):
            argv = [str(binary), str(fixture), "1", action] + ([REQUESTED] if action == "replace" else [])
            generated = call(argv, output)
            (output / f"{action}-planner.stderr").write_bytes(generated.stderr)
            if generated.returncode:
                raise RuntimeError(f"Planner failed: {action}")
            variants[action] = generated.stdout
        for engine in ("pdflatex", "xelatex", "lualatex"):
            for variant, source in variants.items():
                folder = output / engine / variant
                folder.mkdir(parents=True)
                tex = folder / "main.tex"
                tex.write_bytes(source)
                argv = [paths[engine], "--disable-installer", "-no-shell-escape", "-interaction=nonstopmode",
                    "-halt-on-error", "-file-line-error", "main.tex"]
                compiled = call(argv, folder)
                (folder / "compiler.stdout").write_bytes(compiled.stdout)
                (folder / "compiler.stderr").write_bytes(compiled.stderr)
                row = {"engine": engine, "variant": variant, "source_sha256": digest(tex),
                    "argv": argv, "compile_exit": compiled.returncode, "result": "failed"}
                record["rows"].append(row)
                pdf = folder / "main.pdf"
                if compiled.returncode or not pdf.is_file():
                    continue
                row["pdf_sha256"] = digest(pdf)
                extracted = call([paths["pdftotext"], "-layout", "main.pdf", "extracted.txt"], folder)
                (folder / "extractor.stderr").write_bytes(extracted.stderr)
                row["extraction_exit"] = extracted.returncode
                if extracted.returncode:
                    continue
                text = " ".join((folder / "extracted.txt").read_text(encoding="utf-8").split())
                expected = "Before unchanged. " + {"baseline": "Original target paragraph. ",
                    "replace": REQUESTED + " ", "delete": ""}[variant] + "After unchanged."
                row["extracted_text"] = text
                row["expected_text"] = expected
                row["result"] = "passed" if text == expected else "failed"
        record["original_fixture_unchanged"] = fixture.read_bytes() == ORIGINAL.encode("utf-8")
    except Exception as error:
        record["interruption"] = f"{type(error).__name__}: {error}"
    finally:
        record["passed"] = sum(row["result"] == "passed" for row in record["rows"])
        record["complete"] = len(record["rows"]) == 9 and record["passed"] == 9 and record.get("original_fixture_unchanged", False)
        receipt.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"receipt": str(receipt), "passed": record["passed"], "complete": record["complete"]}))
    return 0 if record["complete"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
