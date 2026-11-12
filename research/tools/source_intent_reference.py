"""Predeclared source/render occurrence diagnostic; never opens or commits an app.

See research/SOURCE-INTENT-REFERENCE-PLAN.md. Existing tools and synthetic .tex
only. Writes a new artifacts directory, retaining every failure and comparison.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess

from reference_oracle import compare_pdfs, digest, executable, tool_receipt

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    {"id": "english-pdftex", "engine": "pdflatex", "language": None,
     "same": "Repeated paragraph.", "tail": "Tail unchanged.",
     "changed_tail": "Tail was corrupted.", "literal": "Revised & confirmed: 50%.",
     "escaped": r"Revised \& confirmed: 50\%."},
    {"id": "arabic-xetex", "engine": "xelatex", "language": "arabic",
     "same": "مرحبا بالعالم.", "tail": "هذا نص ثابت.",
     "changed_tail": "هذا نص مختلف.", "literal": "نص جديد & تجربة 50%.",
     "escaped": r"نص جديد \& تجربة 50\%."},
    {"id": "hebrew-xetex", "engine": "xelatex", "language": "hebrew",
     "same": "שלום עולם.", "tail": "זהו טקסט קבוע.",
     "changed_tail": "זהו טקסט שונה.", "literal": "טקסט חדש & בדיקה 50%.",
     "escaped": r"טקסט חדש \& בדיקה 50\%."},
)
CASES = (*CASES, {**CASES[0], "id": "english-luatex", "engine": "lualatex"})
EXPECTED_ROWS = len(CASES) * 6



def document(case: dict, first: str, second: str, tail: str) -> bytes:
    # Independent declarative source oracle: no planner offsets or escaping call.
    lines = [r"\documentclass{article}", r"\pagestyle{empty}", r"\setlength{\parindent}{0pt}"]
    if case["language"]:
        language = case["language"]
        lines += [r"\usepackage{fontspec}", r"\usepackage{polyglossia}",
                  "\\setdefaultlanguage{" + language + "}", r"\setmainfont{Arial}",
                  "\\newfontfamily\\" + language + "font[Script=" + language.title() + "]{Arial}"]
    return ("\ufeff" + "\r\n".join(lines) + "\r\n\\begin{document}\r\n" +
            first + "\r\n\r\n" + second + "\r\n\r\n" + tail +
            "\r\n\\end{document}\r\n").encode("utf-8")


def invoke(argv: list[str], cwd: Path, prefix: str) -> dict:
    try:
        done = subprocess.run(argv, cwd=cwd, capture_output=True, timeout=60, check=False)
        stdout, stderr, code, error = done.stdout, done.stderr, done.returncode, None
    except subprocess.TimeoutExpired as exc:
        stdout, stderr, code, error = exc.stdout or b"", exc.stderr or b"", None, "timeout after 60 seconds"
    (cwd / (prefix + ".stdout")).write_bytes(stdout)
    (cwd / (prefix + ".stderr")).write_bytes(stderr)
    return {"argv": argv, "exit": code, "error": error,
            "stdout": str(cwd / (prefix + ".stdout")), "stderr": str(cwd / (prefix + ".stderr"))}


def compile_tex(source: bytes, folder: Path, engine: Path) -> dict:
    folder.mkdir(parents=True, exist_ok=False)
    tex = folder / "main.tex"
    assert tex.suffix == ".tex"
    tex.write_bytes(source)
    result = invoke([str(engine), "--disable-installer", "-no-shell-escape", "-interaction=nonstopmode",
                     "-halt-on-error", "-file-line-error", tex.name], folder, "compiler")
    result["source_sha256"] = digest(tex)
    result["texloom_dependency_absent"] = b"texloom" not in source.lower()
    log = (folder / "main.log").read_bytes() if (folder / "main.log").exists() else b""
    result["missing_character_warning"] = b"Missing character:" in log
    pdf = folder / "main.pdf"
    result["pdf_sha256"] = digest(pdf) if pdf.is_file() else None
    result["renderable"] = result["exit"] == 0 and pdf.is_file() and not result["missing_character_warning"]
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    binary, output = args.binary.resolve(), args.output.resolve()
    output.relative_to((ROOT / "artifacts").resolve())
    if not binary.is_file():
        raise ValueError("Build the actual core fixture CLI first")
    engines = {name: executable(None, name) for name in ("pdflatex", "xelatex", "lualatex")}
    tools = {"text": executable(None, "pdftotext"), "raster": executable(None, "pdftoppm")}
    output.mkdir(parents=True, exist_ok=False)
    plan = ROOT / "research/SOURCE-INTENT-REFERENCE-PLAN.md"
    record = {"schema": 1, "protocol_version": 2, "kind": "development_source_intent_occurrence_reference_not_app_acceptance",
              "recorded_utc": datetime.now(timezone.utc).isoformat(),
              "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "binary_sha256": digest(binary), "runner_sha256": digest(Path(__file__)),
              "plan_sha256": digest(plan), "conditions": list(CASES), "tools": {}, "rows": [], "compile_failure_controls": [],
              "originals_unchanged": False, "complete": False, "app_acceptance_executed": False,
              "limitations": ["Four language/engine conditions, not their full cross product; one Windows host; corrected protocol reuses development cases.",
                              "Read-only planner/CLI entry: no PDF selection, native approval, source/history/Save or UI.",
                              "No held-out data, final accuracy/FPS/latency or universal language/rendering claim.",
                              "Expected source/labels have no independent human review; exact raster equality is host/tool specific.",
                              "Wrong-occurrence deletion render match is diagnostic, not a success/failure label."]}
    originals = []
    try:
        for name, path in {**engines, **tools}.items():
            info = tool_receipt(path)
            if name in engines:
                version = subprocess.run([str(path), "--version"], capture_output=True, timeout=10)
                info["version"] = (version.stdout + version.stderr).decode("utf-8", errors="replace")[:2000]
            record["tools"][name] = info
        font = Path(r"C:\Windows\Fonts\arial.ttf")
        record["arial_font"] = {"path": str(font), "sha256": digest(font)} if font.is_file() else {"status": "not_found_at_expected_path"}
        for case in CASES:
            folder = output / case["id"]
            folder.mkdir()
            original_bytes = document(case, case["same"], case["same"], case["tail"])
            original = folder / "original.tex"
            original.write_bytes(original_bytes)
            originals.append((original, original_bytes))
            for action in ("replace", "delete"):
                action_folder = folder / action
                action_folder.mkdir()
                expected_second = case["escaped"] if action == "replace" else ""
                expected_bytes = document(case, case["same"], expected_second, case["tail"])
                reference = compile_tex(expected_bytes, action_folder / "reference", engines[case["engine"]])
                command = [str(binary), str(original), "1", action] + ([case["literal"]] if action == "replace" else [])
                planning = invoke(command, action_folder, "planner")
                variants = {
                    "planned": Path(planning["stdout"]).read_bytes() if planning["exit"] == 0 else None,
                    "wrong_occurrence": document(case, expected_second, case["same"], case["tail"]),
                    "collateral_tail": document(case, case["same"], expected_second, case["changed_tail"]),
                }
                for variant, source in variants.items():
                    row = {"condition": case["id"], "engine": case["engine"], "action": action,
                           "variant": variant, "planning": planning if variant == "planned" else None,
                           "reference": reference, "source_matches_reference": source == expected_bytes,
                           "render_comparison": "unexecuted", "expectation_met": False}
                    record["rows"].append(row)
                    if source is None:
                        row["reason"] = "planner failed or timed out"
                        continue
                    compiled = compile_tex(source, action_folder / variant, engines[case["engine"]])
                    row["candidate"] = compiled
                    if compiled["renderable"] and reference["renderable"]:
                        compared = compare_pdfs(action_folder / "reference/main.pdf", action_folder / variant / "main.pdf",
                                                action_folder / (variant + "-comparison"), tools)
                        row["render_comparison"] = compared["decision"]
                        row["comparison"] = compared
                    if variant == "planned":
                        row["expectation_met"] = source == expected_bytes and row["render_comparison"] == "matches_reference"
                    elif variant == "wrong_occurrence" and action == "delete":
                        row["expectation_met"] = source != expected_bytes and row["render_comparison"] in ("matches_reference", "differs")
                        row["interpretation"] = "Source is wrong regardless of whether its rendered output is distinguishable."
                    else:
                        row["expectation_met"] = source != expected_bytes and row["render_comparison"] == "differs"
                    row["expectation_met"] &= compiled["texloom_dependency_absent"]
                if case["id"] == "english-pdftex":
                    broken = expected_bytes.replace(b"\\end{document}", b"\\diagnosticUndefinedCommand\r\n\\end{document}")
                    failed = compile_tex(broken, action_folder / "deliberate-compile-failure", engines[case["engine"]])
                    record["compile_failure_controls"].append({"action": action, "compiler": failed,
                        "expectation_met": failed["exit"] is not None and failed["exit"] != 0})
        record["originals_unchanged"] = all(p.read_bytes() == before for p, before in originals)
    except Exception as error:
        record["interruption"] = f"{type(error).__name__}: {error}"
    finally:
        record["expectations_met"] = sum(bool(row["expectation_met"]) for row in record["rows"])
        record["complete"] = (len(record["rows"]) == EXPECTED_ROWS and record["expectations_met"] == EXPECTED_ROWS and
            len(record["compile_failure_controls"]) == 2 and all(row["expectation_met"] for row in record["compile_failure_controls"]) and
            record["originals_unchanged"] and "interruption" not in record)
        (output / "receipt.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"receipt": str(output / "receipt.json"), "rows": len(record["rows"]),
                      "expectations_met": record["expectations_met"], "complete": record["complete"]}))
    return 0 if record["complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
