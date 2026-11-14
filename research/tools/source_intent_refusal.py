"""Predeclared native refusal diagnostic; development source outcomes only."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
from source_intent_transport import Bridge, digest, write_json


def condition(bridge, record, paragraph, replacement, escaped):
    prefix = "\ufeff\\documentclass{article}\r\n\\begin{document}\r\n"
    gap = "\r\n\r\n"
    suffix = "\r\n\r\nTail remains unchanged.\r\n\\end{document}\r\n"
    original = {"main.tex": prefix + paragraph + gap + paragraph + suffix,
                "notes.tex": "% Unchanged auxiliary source\r\n"}
    replaced = {**original, "main.tex": prefix + paragraph + gap + escaped + suffix}
    deleted = {**original, "main.tex": prefix + paragraph + gap + suffix}
    record["gold_sources"] = {"original": original, "replace": replaced, "delete": deleted}

    def check(name, observed, expected):
        passed = observed == expected
        record["assertions"].append({"name": name, "passed": passed,
                                      "observed": observed, "expected": expected})
        if not passed:
            raise AssertionError(name)

    def op(number):
        return f"{number:032x}"

    def refusal(reply, requested, observed, reason, label):
        check(label + "-outcome", reply["outcome"], {
            "status": "source_draft_refused", "requested_revision": requested,
            "observed_revision": observed, "reason": reason})

    opened = bridge.ok("open", sources=original)
    before = opened["snapshot"]
    check("open-exact-source", before["sources"], original)
    targets = bridge.ok("targets", revision=0, file="main.tex")["targets"]
    repeated = [t for t in targets if t["text"] == paragraph]
    check("two-declared-occurrences", len(repeated), 2)
    target = repeated[1]["handle"]
    requests = []
    for number, (literal, reason) in enumerate([
            ("", "empty_literal"), ("Two\nparagraphs", "paragraph_break"),
            ("Bad\x01control", "control_character"), ("x" * 256_001, "literal_too_large")], 1):
        request = {"revision": 0, "operationId": op(number), "intent": {
            "kind": "replace_literal_paragraph", "target": target, "text": literal}}
        requests.append(request)
        if number == 2:
            # Deliberately discard the returned acknowledgement at the caller.
            # The transcript keeps it for audit; this is not a real IPC outage.
            bridge.ask("intent", **request)
            reply = bridge.ok("status", operationId=op(number))
        else:
            reply = bridge.ok("intent", **request)
        refusal(reply, 0, 0, reason, reason)
        check(reason + "-unchanged-canonical-state", reply["snapshot"], before)
        check(reason + "-lookup", bridge.ok("status", operationId=op(number)), reply)
        check(reason + "-duplicate", bridge.ok("intent", **request), reply)
    check("payload-id-conflict", bridge.ask("intent", **{
        **requests[0], "intent": {**requests[0]["intent"], "text": "Different payload"}})["ok"], False)
    changed = bridge.ok("intent", revision=0, operationId=op(5), intent={
        "kind": "replace_literal_paragraph", "target": target, "text": replacement})
    check("valid-edit-after-refusals-source", changed["snapshot"]["sources"], replaced)
    check("valid-edit-after-refusals-revision", changed["snapshot"]["revision"], 1)
    check("still-no-render-approval", changed["outcome"]["render_verification"], "unresolved")
    replay = bridge.ok("intent", **requests[0])
    refusal(replay, 0, 0, "empty_literal", "old-refusal-after-change")
    check("old-refusal-fresh-state", replay["snapshot"], changed["snapshot"])
    fresh = bridge.ok("targets", revision=1, file="main.tex")["targets"]
    replacement_target = [t for t in fresh if t["text"] == escaped][0]["handle"]
    for number, requested, handle, observed, reason in [
            (6, 1, target, 1, "target_unavailable"),
            (7, 1, "0" * 32, 1, "target_unavailable"),
            (8, 0, replacement_target, 1, "revision_changed"),
            (9, 5, replacement_target, 1, "revision_changed")]:
        reply = bridge.ok("intent", revision=requested, operationId=op(number), intent={
            "kind": "delete_paragraph_content", "target": handle})
        refusal(reply, requested, observed, reason, f"guard-{number}")
        check(f"guard-{number}-source-preserved", reply["snapshot"]["sources"], replaced)
    removed = bridge.ok("intent", revision=1, operationId=op(10), intent={
        "kind": "delete_paragraph_content", "target": replacement_target})
    check("second-delete-after-refusals", removed["snapshot"]["sources"], deleted)
    check("exact-delete-undo", bridge.ok("history", revision=2, redo=False)["sources"], replaced)
    redone = bridge.ok("history", revision=3, redo=True)
    check("exact-delete-redo", redone["sources"], deleted)
    replay = bridge.ok("status", operationId=op(1))
    refusal(replay, 0, 0, "empty_literal", "old-refusal-after-history")
    check("old-refusal-after-history-current", replay["snapshot"], redone)
    restored = bridge.ok("restart_recovery")
    check("protected-recovery-source", restored["snapshot"]["sources"], deleted)
    check("reopened-receipt-unknown", bridge.ok("status", operationId=op(1))["outcome"], {"status": "unknown"})
    reply = bridge.ok("intent", revision=4, operationId=op(11), intent={
        "kind": "delete_paragraph_content", "target": replacement_target})
    refusal(reply, 4, 4, "target_unavailable", "old-session-handle")
    check("old-session-source-preserved", reply["snapshot"]["sources"], deleted)
    remaining = bridge.ok("targets", revision=4, file="main.tex")["targets"]
    foreign = remaining[0]["handle"]
    bridge.ok("open", sources=original)
    reply = bridge.ok("intent", revision=0, operationId=op(12), intent={
        "kind": "delete_paragraph_content", "target": foreign})
    refusal(reply, 0, 0, "target_unavailable", "foreign-session-handle")
    check("foreign-session-original-source", reply["snapshot"]["sources"], original)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output.resolve()
    if not output.is_relative_to(root / "artifacts"):
        raise ValueError("Output must be a fresh directory under repository artifacts")
    output.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve(strict=True)
    inputs = [Path(__file__), root / "research/tools/source_intent_transport.py",
              root / "research/SOURCE-INTENT-REFUSAL-PLAN.md", root / "Cargo.lock",
              root / "src-tauri/Cargo.toml", root / "src-tauri/examples/source_intent_transport_development.rs"]
    inputs += sorted((root / "src-tauri/src").rglob("*.rs"))
    inputs += sorted((root / "crates/texloom-core/src").rglob("*.rs"))
    identities = {p.relative_to(root).as_posix(): digest(p.read_bytes()) for p in inputs}
    write_json(output / "pre-run.json", {"recorded_utc": datetime.now(timezone.utc).isoformat(),
        "binary_sha256": digest(binary.read_bytes()), "source_sha256": identities,
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "scope": "Uncommitted source bound by hashes, not a final evaluation freeze"})
    runs, failure = [], None
    conditions = [("english", "Repeated paragraph.", "Updated & 50%.", "Updated \\& 50\\%."),
                  ("arabic", "مرحبا بالعالم.", "مرحبا & 50%.", "مرحبا \\& 50\\%."),
                  ("hebrew", "שלום עולם.", "שלום & 50%.", "שלום \\& 50\\%.")]
    for language, paragraph, replacement, escaped in conditions:
        record = {"language": language, "assertions": []}
        runs.append(record)
        bridge = Bridge(binary, output / language)
        try:
            condition(bridge, record, paragraph, replacement, escaped)
            original = record["gold_sources"]["original"]
            for name, expected in original.items():
                actual = (output / language / "project-0" / name).read_bytes()
                assert actual == expected.encode(), "Original file was written"
            record["original_files_unchanged"] = True
        except Exception as error:
            failure = {"language": language, "error": str(error), "type": type(error).__name__}
        finally:
            code = bridge.close()
            write_json(output / (language + "-transcript.json"), bridge.transcript)
            if code and failure is None:
                failure = {"language": language, "error": f"Bridge exit {code}"}
        if failure:
            break
    changed = [name for name, sha in identities.items() if digest((root/name).read_bytes()) != sha]
    result = {"schema": 1, "kind": "native_source_draft_refusal_development_diagnostic",
        "entry": "headless_native_service_not_tauri_dispatch_or_pdf_selection",
        "status": "passed" if not failure and not changed else "failed", "runs": runs,
        "failure": failure, "source_changed_during_run": changed, "final_evaluation": False,
        "render_proof": "unresolved", "gui_opened": False, "tex_compilation": False,
        "service_test_only": ["receipt_capacity", "injected_recovery_failure", "pending_build_and_selection_preservation"]}
    write_json(output / "results.json", result)
    print(json.dumps({"status": result["status"], "conditions": len(runs),
        "assertions": sum(len(r["assertions"]) for r in runs), "failure": failure}))
    if result["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
