"""Predeclared development pilot at native candidate entry, not PDF mouse selection."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess

from oracle_feasibility import MULTILINGUAL, NEIGHBORS
from reference_oracle import compare_pdfs, digest, executable, tool_receipt, page_contract, words_from_xml
from source_contract import verify_sources

ROOT = Path(__file__).resolve().parents[2]
CONDITIONS = [(engine, language) for engine, languages in (
    ("pdflatex", ["latin"]), ("xelatex", ["latin", "multilingual"]),
    ("lualatex", ["latin", "multilingual"])) for language in languages]
FIRST = r"\TexloomObject{native-first}{0}{0}{Repeated target}"
TARGET = r"\TexloomObject{native-target}{0}{0}{Repeated target}"
MOVED = r"\TexloomObject{native-target}{12}{0}{Repeated target}"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def document(engine, language, mode):
    # The gold uses ordinary TeX boxes/phantom, never the native planner or its
    # displacement operation. The zero reference also omits both object macros.
    first = FIRST if mode == "baseline" else "Repeated target"
    second = (TARGET if mode == "baseline" else "Repeated target" if mode == "zero"
              else r"\leavevmode\hbox to 0pt{\kern12bp Repeated target\hss}\phantom{Repeated target}")
    text = (f"% !TeX program = {engine}\n" + r"\documentclass{article}" + "\n"
            + r"\usepackage[margin=25mm]{geometry}\usepackage{xcolor}\usepackage{texloom-objects}" + "\n"
            + (MULTILINGUAL if language == "multilingual" else "")
            + r"\pagestyle{empty}\setlength{\parindent}{0pt}" + "\n"
            + r"\begin{document}" + "\nIndependent native reference pilot.\\par\n"
            + first + "\\par\n\\vspace{20bp}\n" + second + "\\par\n\\vspace{20bp}\n"
            + "Unchanged unrelated neighbor.\\par\n"
            + (NEIGHBORS if language == "multilingual" else "")
            + r"\end{document}" + "\n")
    return "\ufeff" + text.replace("\r\n", "\n").replace("\n", "\r\n")


def condition(engine, language, style, target_style=None):
    baseline = {"main.tex": document(engine, language, "baseline"),
                "texloom-objects.sty": style, "notes.sty": "% Unchanged support file. مرحبا שלום\r\n"}
    original = baseline["main.tex"]
    assert original.count(TARGET) == original.count(FIRST) == 1
    correct = {**baseline, "main.tex": original.replace(TARGET, MOVED),
               "texloom-objects.sty": style if target_style is None else target_style}
    variants = {
        "wrong_first_occurrence": {**correct, "main.tex": original.replace(FIRST, FIRST.replace("{0}{0}", "{12}{0}"))},
        "unrelated_text_changed": {**correct, "main.tex": correct["main.tex"].replace("Unchanged unrelated neighbor.", "Changed unrelated neighbor.")},
        "appearance_changed": {**correct, "main.tex": correct["main.tex"].replace(MOVED, MOVED.replace("{Repeated target}", r"{\textcolor{red}{Repeated target}}"))},
        "duplicate_ink": {**correct, "main.tex": correct["main.tex"].replace(MOVED, MOVED.replace("{Repeated target}", r"{\rlap{Repeated target}Repeated target}"))},
        "support_file_changed": {**correct, "notes.sty": baseline["notes.sty"] + "% Unrequested change\r\n"},
        "correct": correct,
    }
    contract = {"schema": 1, "files": {name: {"before_sha256": sha(text.encode("utf-8")),
                "after_sha256": sha(correct[name].encode("utf-8"))} for name, text in baseline.items()}}
    return {"id": engine + "-" + language, "engine": engine, "language": language,
            "baseline": baseline, "request": {"main": "main.tex", "file": "main.tex", "action": "move",
                "id": "native-target", "expected": "Repeated target", "dx": 12, "dy": 0},
            "candidates": [{"variant": name, "sources": sources} for name, sources in variants.items()],
            "source_contract": contract,
            "gold_zero": {**baseline, "main.tex": document(engine, language, "zero")},
            "gold_moved": {**baseline, "main.tex": document(engine, language, "moved")}}


def save_json(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def prepare(root, support_style="current"):
    root.mkdir(parents=True, exist_ok=False)
    # Preserve the actual distribution bytes, including line endings.
    target_style = (ROOT/"resources/texloom-objects.sty").read_bytes().decode("utf-8")
    style = ((ROOT/"resources/legacy/texloom-objects-v03.sty").read_bytes().decode("utf-8")
             if support_style == "legacy" else target_style)
    protocol = {"schema": 1, "kind": "native_candidate_boundary_development_v1", "protocol_revision": 3,
        "support_style": support_style,
        "support_style_contract": "Exact declared v0.3-to-current migration" if support_style == "legacy" else "Already-current support bytes unchanged",
        "recorded_before_execution_utc": datetime.now(timezone.utc).isoformat(),
        "repository_head_before_preparation": subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),
        "entry_layer": "proposed_sources_plus_move_request_at_native_prepare",
        "final_evaluation_executed": False,
        "criteria": {"intended_move": "native commit, exact declared source, Poppler reference match, exact Undo, no disk write",
            "tampered_packet": "native prepare refuses with canonical source/history unchanged; do not compile it",
            "baseline_oracle_health": "native zero-offset baseline equals independent plain-text reference",
            "moved_reference_health": "Only second Repeated/target words move 12 bp horizontally; every other word bound stays within 0.02 bp",
            "geometry_tolerance_bp": 0.02, "raster_dpi": 144, "raster_match": "exact full-page pixels"},
        "limits": ["Development cases, same preparer, no independent human adjudication",
            "Existing wrapped objects; no PDF.js input/mapping/frontend selection probe or physical UI",
            "Source-plan rejection is not evidence that the independent renderer saw a fault",
            "No latency benchmark, general acceptance rate or final held-out claim"],
        "conditions": [condition(engine, language, style, target_style) for engine, language in CONDITIONS]}
    code_paths=[Path(__file__),Path(__file__).with_name("reference_oracle.py"),Path(__file__).with_name("source_contract.py"),
                Path(__file__).with_name("oracle_feasibility.py"),ROOT/"src-tauri/examples/native_reference_development.rs"]
    protocol["implementation_files_sha256"]={path.relative_to(ROOT).as_posix():digest(path) for path in code_paths}
    save_json(root/"protocol.json", protocol)
    for name in ["native_reference_pilot.py", "reference_oracle.py", "source_contract.py", "oracle_feasibility.py"]:
        shutil.copyfile(Path(__file__).with_name(name), root/("executed-"+name))
    shutil.copyfile(ROOT/"src-tauri/examples/native_reference_development.rs",root/"executed-native_reference_development.rs")
    return protocol


def gold_build(engine, sources, output):
    output.mkdir()
    for name, text in sources.items():
        if name not in {"main.tex", "notes.sty", "texloom-objects.sty"}:
            raise ValueError("Only explicitly named .tex/.sty inputs are permitted")
        (output/name).write_bytes(text.encode("utf-8"))
    version = subprocess.run([str(engine), "--version"], capture_output=True, timeout=10)
    command = [str(engine), *(["--disable-installer"] if b"MiKTeX" in version.stdout+version.stderr else []),
               "-no-shell-escape", "-interaction=nonstopmode", "-halt-on-error", "-file-line-error", "-recorder", "main.tex"]
    codes = []
    for number in range(2):
        with (output/f"pass-{number+1}.log").open("wb") as log:
            result = subprocess.run(command, cwd=output, stdout=log, stderr=subprocess.STDOUT, timeout=60)
        codes.append(result.returncode)
        if result.returncode:
            break
    save_json(output/"build.json", {"command": command, "exit_codes": codes,
              "engine_sha256": digest(engine), "engine_version": (version.stdout+version.stderr).decode("utf-8", errors="replace")})
    return codes == [0, 0] and (output/"main.pdf").is_file()


def reference_word_health(zero, moved, dx=12, tolerance=0.02):
    """A bounded, declared occurrence check for this isolated-line reference only."""
    result={"decision":"unresolved","target_word_keys":[[1,"Repeated",1],[1,"target",1]],
            "expected_dx_bp":dx,"tolerance_bp":tolerance}
    if not math.isfinite(dx) or not math.isfinite(tolerance) or tolerance<0:
        return {**result,"reason":"Invalid health-check displacement/tolerance"}
    if len(zero)!=len(moved) or any((a["page"],a["text"])!=(b["page"],b["text"]) for a,b in zip(zero,moved)):
        return {**result,"reason":"Missing, changed or reordered words require adjudication"}
    counts={};target_errors=[];other_errors=[]
    targets={(1,"Repeated",1),(1,"target",1)}
    for a,b in zip(zero,moved):
        key=(a["page"],a["text"]);ordinal=counts.get(key,0);counts[key]=ordinal+1
        target=(*key,ordinal) in targets
        if not all(math.isfinite(v) for v in a["bounds"]+b["bounds"]):
            return {**result,"reason":"Invalid word geometry"}
        expected=[dx,0,dx,0] if target else [0,0,0,0]
        error=max(abs((new-old)-delta) for old,new,delta in zip(a["bounds"],b["bounds"],expected))
        (target_errors if target else other_errors).append(error)
    if len(target_errors)!=2 or counts.get((1,"Repeated"))!=2 or counts.get((1,"target"))!=2:
        return {**result,"reason":"The declared two-occurrence target was not uniquely resolved"}
    result.update(target_word_count=2,maximum_target_error_bp=max(target_errors),maximum_non_target_error_bp=max(other_errors,default=0))
    result["decision"]="matches_declared_translation" if max(target_errors+other_errors,default=0)<=tolerance else "differs"
    return result


def reference_health(zero, moved, output, tools):
    output.mkdir()
    if page_contract(zero)!=page_contract(moved):
        result={"decision":"differs","reason":"Reference page count or geometry changed"}
    else:
        words=[]
        for label,pdf in [("zero",zero),("moved",moved)]:
            xml=output/(label+".xml")
            with (output/(label+"-console.log")).open("xb") as log:
                run=subprocess.run([str(tools["text"]),"-bbox-layout","-enc","UTF-8",str(pdf),str(xml)],
                                   stdout=log,stderr=subprocess.STDOUT,timeout=45)
            if run.returncode:raise ValueError("Reference-health text extraction failed")
            words.append(words_from_xml(xml))
        result=reference_word_health(*words)
    save_json(output/"health.json",result)
    return result


def evaluate(root, runner, tools):
    protocol_path = root/"protocol.json"
    before = digest(protocol_path)
    protocol = json.loads(protocol_path.read_text(encoding="utf-8"))
    runner_before=digest(runner)
    for name,expected in protocol["implementation_files_sha256"].items():
        if digest(ROOT/name)!=expected or digest(root/("executed-"+Path(name).name))!=expected:
            raise ValueError("Implementation changed after protocol preparation; preserve this run and prepare a new one")
    if (root/"native-output").exists() or (root/"analysis").exists():
        raise ValueError("Earlier runs must not be overwritten; prepare a fresh run")
    with (root/"native-console.log").open("xb") as log:
        result = subprocess.run([str(runner), str(protocol_path), str(root/"native-output")],
                                stdout=log, stderr=subprocess.STDOUT, timeout=900)
    if result.returncode:
        raise ValueError("Native exporter failed; retained native-console.log and partial output")
    native = json.loads((root/"native-output/native-report.json").read_text(encoding="utf-8"))
    rows = []; analysis = root/"analysis"; analysis.mkdir()
    for case in protocol["conditions"]:
        item = next((x for x in native["conditions"] if x["id"] == case["id"]), {})
        row = {"id": case["id"], "engine": case["engine"], "language": case["language"], "native": item,
               "status": "unresolved", "native_source_contract": None, "baseline_comparison": None, "candidate_comparison": None,"reference_health":None}
        rows.append(row); work=analysis/case["id"]; work.mkdir()
        try:
            engine = executable(None, case["engine"])
            for name in ("gold_zero", "gold_moved"):
                if not gold_build(engine, case[name], work/name):
                    raise ValueError(name + " compilation failed")
            native_root=root/"native-output"/case["id"]
            row["baseline_comparison"] = compare_pdfs(work/"gold_zero/main.pdf", native_root/"baseline.pdf", work/"baseline-comparison", tools)
            row["reference_health"]=reference_health(work/"gold_zero/main.pdf",work/"gold_moved/main.pdf",work/"reference-health",tools)
            if row["reference_health"]["decision"]!="matches_declared_translation":
                row["status"]="invalid_or_unresolved_reference"
                continue
            if item.get("intended_move", {}).get("status") != "committed":
                row["status"] = "native_postcommit_failure" if item.get("intended_move",{}).get("commit_performed") else "native_failed_or_refused"
                continue
            committed={name: (native_root/"committed-sources"/name).read_bytes() for name in case["baseline"]}
            row["native_source_contract"]=verify_sources({name:text.encode("utf-8") for name,text in case["baseline"].items()},committed,case["source_contract"])
            row["candidate_comparison"]=compare_pdfs(work/"gold_moved/main.pdf",native_root/"native-candidate.pdf",work/"candidate-comparison",tools)
            valid=item["intended_move"]
            row["status"] = "matches_declared_postconditions" if (
                row["baseline_comparison"]["decision"]==row["candidate_comparison"]["decision"]=="matches_reference"
                and row["native_source_contract"]["decision"]=="matches_contract"
                and valid.get("undo_exact") and valid.get("disk_unchanged") and item.get("disk_unchanged_after_run")) else "postcondition_failure_or_unresolved"
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            row["reason"]=str(error)
        finally:
            save_json(work/"outcome.json", row)
    if digest(protocol_path) != before or digest(runner)!=runner_before:
        raise ValueError("Protocol or native runner changed during execution")
    report={"schema":1,"kind":"native_candidate_boundary_development_not_full_app_evaluation",
            "protocol_sha256":before,"native_runner_sha256":digest(runner),"conditions":rows,
            "counts":{"conditions":len(rows),"intended_moves_matching_all_declared_postconditions":sum(r["status"]=="matches_declared_postconditions" for r in rows),
                "fault_packets_refused_before_compilation":sum(not f["prepare_accepted"] and f["canonical_source_history_unchanged"] for r in rows for f in r["native"].get("fault_packets",[])),
                "unexpected_tampered_prepare_accepts":sum(f["prepare_accepted"] for r in rows for f in r["native"].get("fault_packets",[]))},
            "tools":{key:tool_receipt(value) for key,value in tools.items()},"final_evaluation_executed":False,
            "limits":protocol["limits"]}
    save_json(root/"report.json",report)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action",choices=("prepare","evaluate"));parser.add_argument("output",type=Path)
    parser.add_argument("--native-runner",type=Path);parser.add_argument("--pdftotext");parser.add_argument("--pdftoppm")
    parser.add_argument("--support-style", choices=("current", "legacy"), default="current",
                        help="Preparation only: declare already-current or exact legacy support migration")
    args=parser.parse_args();root=args.output.resolve()
    if args.action=="prepare":
        protocol=prepare(root,args.support_style);print(f"Protocol written before execution: {len(protocol['conditions'])} conditions; support={args.support_style}; no compilation or app launch.")
    else:
        if args.native_runner is None:parser.error("--native-runner is required")
        report=evaluate(root,args.native_runner.resolve(),{"text":executable(args.pdftotext,"pdftotext"),"raster":executable(args.pdftoppm,"pdftoppm")})
        print(json.dumps(report["counts"]))


if __name__=="__main__":main()
