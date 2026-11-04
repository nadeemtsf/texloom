"""Development pilot of an independent reference; not the final Texloom study."""
import argparse
from datetime import datetime, timezone
from importlib import metadata
import json
from pathlib import Path
import shutil
import subprocess
import time

from reference_oracle import compare_pdfs, digest, executable, tool_receipt

ROOT = Path(__file__).resolve().parents[2]
MULTILINGUAL = r"""\usepackage{fontspec}
\usepackage{polyglossia}
\setdefaultlanguage{english}
\setotherlanguages{arabic,hebrew}
\setmainfont{Arial}
\newfontfamily\arabicfont[Script=Arabic]{Arial}
\newfontfamily\hebrewfont[Script=Hebrew]{Arial}
"""
NEIGHBORS = """\n\\begin{Arabic}
مرحبا بالعالم. هذا نص عربي للتجربة.
\\end{Arabic}
\\begin{hebrew}
שלום עולם. זהו טקסט בעברית לבדיקה.
\\end{hebrew}
"""
VARIANTS = {"correct": "matches_reference", "wrong_first_occurrence": "differs",
            "unrelated_text_changed": "differs", "appearance_changed": "differs",
            "duplicate_ink": "differs", "compile_failure": "compile_failure"}


def source(variant, multilingual):
    # The gold update is manually specified TeX box positioning. Candidate
    # controls use the shipped movement package, never the runtime source planner.
    first = r"\usebox{\pilotline}"
    second = {
        "reference": r"\leavevmode\hbox to \wd\pilotline{\hskip12bp\usebox{\pilotline}\hss}",
        "correct": r"\TexloomObject{pilot-target}{12}{0}{Repeated target}",
        "wrong_first_occurrence": first,
        "unrelated_text_changed": r"\TexloomObject{pilot-target}{12}{0}{Repeated target}",
        "appearance_changed": r"\TexloomObject{pilot-target}{12}{0}{\textcolor{red}{Repeated target}}",
        "duplicate_ink": r"\TexloomObject{pilot-target}{12}{0}{\rlap{Repeated target}Repeated target}",
        "compile_failure": r"\PilotUndefinedControl",
    }[variant]
    if variant == "wrong_first_occurrence":
        first = r"\TexloomObject{pilot-target}{12}{0}{Repeated target}"
    neighbor = "Changed unrelated neighbor." if variant == "unrelated_text_changed" else "Unchanged unrelated neighbor."
    return (r"\documentclass{article}"+"\n"+r"\usepackage[margin=25mm]{geometry}"+"\n"
            +r"\usepackage{xcolor}"+"\n"+r"\usepackage{texloom-objects}"+"\n"
            +(MULTILINGUAL if multilingual else "")
            +r"\pagestyle{empty}\setlength{\parindent}{0pt}"+"\n"
            +r"\newsavebox{\pilotline}"+"\n"+r"\begin{document}"+"\n"
            +r"\sbox{\pilotline}{Repeated target}"+"\n"
            +"Independent reference pilot.\\par\n"+first+r"\par\vspace{20bp}"+"\n"
            +second+r"\par\vspace{20bp}"+"\n"+neighbor+"\\par\n"
            +(NEIGHBORS if multilingual else "")+r"\end{document}"+"\n")


def build(engine, text, output):
    output.mkdir(parents=True)
    main = output/"main.tex"
    main.write_text(text, encoding="utf-8")
    # All inputs are synthetic; no .text enumeration or user project compilation.
    assert main.suffix == ".tex"
    shutil.copyfile(ROOT/"resources/texloom-objects.sty", output/"texloom-objects.sty")
    version = subprocess.run([str(engine), "--version"], capture_output=True, timeout=10)
    miktex = b"MiKTeX" in version.stdout + version.stderr
    command = [str(engine), *(["--disable-installer"] if miktex else []), "-no-shell-escape",
               "-interaction=nonstopmode", "-halt-on-error", "-file-line-error", "main.tex"]
    started = time.perf_counter()
    codes = []
    for number in range(2):
        with (output/f"pass-{number+1}.log").open("wb") as log:
            result = subprocess.run(command, cwd=output, stdout=log, stderr=subprocess.STDOUT, timeout=45)
        codes.append(result.returncode)
        if result.returncode:
            break
    return {"ok": codes == [0, 0] and (output/"main.pdf").is_file(), "exit_codes": codes,
            "compile_seconds": time.perf_counter()-started, "source_sha256": digest(main),
            "command": command, "engine_version": (version.stdout+version.stderr).decode("utf-8", errors="replace")[:2000]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="Fresh evidence directory")
    parser.add_argument("--pdftotext")
    parser.add_argument("--pdftoppm")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    tools = {"text": executable(args.pdftotext, "pdftotext"),
             "raster": executable(args.pdftoppm, "pdftoppm")}
    protocol = {"schema": 1, "kind": "development_oracle_feasibility_not_final_evaluation",
                "recorded_before_compilation_utc": datetime.now(timezone.utc).isoformat(),
                "conditions": [{"engine": e, "multilingual_neighbors": m}
                               for e in ["pdflatex", "xelatex", "lualatex"]
                               for m in ([False] if e == "pdflatex" else [False, True])],
                "expected_control_decisions": VARIANTS, "word_tolerance_bp": 0.02,
                "raster_tolerance": "zero changed pixels at 144 dpi",
                "question": "Can an independently authored box-position reference match the portable wrapper and distinguish wrong-occurrence, unrelated-text, appearance and duplicate-ink controls?",
                "limits": ["Post-compilation diagnostic controls, not application interaction/acceptance rates.",
                           "Same author specifies reference and controls; independence is from runtime planner/renderer, not blinded adjudication.",
                           "The pilot tests Latin repeated targets with English/Arabic/Hebrew neighbors; it does not certify shaped-text editing.",
                           "Source-intent correctness and ambiguous reading order still need separate annotations/adjudication."],
                "tools": {k: tool_receipt(v) for k, v in tools.items()},
                "packages": {n: metadata.version(n) for n in ["Pillow", "pypdf"]},
                "support_package_sha256": digest(ROOT/"resources/texloom-objects.sty"),
                "runner_sha256": digest(__file__), "oracle_sha256": digest(Path(__file__).with_name("reference_oracle.py"))}
    # These criteria are on disk before any engine/fixture run.
    (output/"protocol.json").write_text(json.dumps(protocol, indent=2)+"\n", encoding="utf-8")
    report = {"schema": 1, "kind": protocol["kind"], "final_evaluation_executed": False,
              "status": "running", "conditions": []}
    for condition in protocol["conditions"]:
        name = condition["engine"]+("-multilingual" if condition["multilingual_neighbors"] else "-latin")
        directory = output/name
        engine = executable(None, condition["engine"])
        reference = build(engine, source("reference", condition["multilingual_neighbors"]), directory/"reference")
        row = {**condition, "reference": reference, "controls": []}
        for variant, expected in VARIANTS.items():
            control = {"variant": variant, "expected": expected}
            try:
                candidate = build(engine, source(variant, condition["multilingual_neighbors"]), directory/variant)
                control["build"] = candidate
                if not candidate["ok"]:
                    control["actual"] = "compile_failure"
                elif not reference["ok"]:
                    control.update(actual="unresolved", reason="Reference did not compile")
                else:
                    comparison = compare_pdfs(directory/"reference/main.pdf", directory/variant/"main.pdf",
                                              directory/(variant+"-comparison"), tools)
                    control.update(actual=comparison["decision"], comparison=comparison)
                control["matches_predeclared_control"] = control["actual"] == expected
            except (OSError, ValueError, subprocess.SubprocessError) as error:
                control.update(actual="unresolved", reason=str(error), matches_predeclared_control=False)
            row["controls"].append(control)
            print(json.dumps({"condition": name, "variant": variant, "actual": control["actual"],
                              "matches_control": control["matches_predeclared_control"]}), flush=True)
        report["conditions"].append(row)
        (output/"report.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    report["status"] = "pass" if all(c["reference"]["ok"] and all(
        v["matches_predeclared_control"] for v in c["controls"]) for c in report["conditions"]) else "fail"
    (output/"report.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print(f"Pilot status: {report['status']}; final evaluation not run.")
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
