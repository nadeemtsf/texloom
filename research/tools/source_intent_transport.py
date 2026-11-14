"""Development diagnostic of real native draft endpoints; no UI, TeX or Save."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import queue
import subprocess
import threading


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def write_json(path, data):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


class Bridge:
    def __init__(self, binary, root):
        self.stderr = (root.parent / (root.name + "-stderr.log")).open("xb")
        self.process = subprocess.Popen([str(binary), str(root)], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=self.stderr)
        self.lines = queue.Queue()
        self.transcript = []
        def read():
            for line in self.process.stdout:
                self.lines.put(line)
            self.lines.put(None)
        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def ask(self, command, **fields):
        request = {"command": command, **fields}
        self.process.stdin.write((json.dumps(request, ensure_ascii=False) + "\n").encode())
        self.process.stdin.flush()
        try:
            raw = self.lines.get(timeout=30)
        except queue.Empty as error:
            raise RuntimeError("Native bridge response timed out") from error
        if raw is None:
            raise RuntimeError("Native bridge ended before its response")
        reply = json.loads(raw)
        self.transcript.append({"request": request, "reply": reply})
        return reply

    def ok(self, command, **fields):
        reply = self.ask(command, **fields)
        if not reply.get("ok"):
            raise AssertionError(f"Unexpected native refusal: {reply}")
        return reply["result"]

    def close(self):
        self.process.stdin.close()
        try:
            code = self.process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.process.kill()
            code = self.process.wait(timeout=10)
        self.stderr.close()
        return code


def diagnostic(bridge, language, paragraph, replacement, escaped):
    # Gold states are built from declared source segments, not planner offsets.
    prefix = "\ufeff\\documentclass{article}\r\n\\begin{document}\r\n"
    separator = "\r\n\r\n"
    suffix = "\r\n\r\nTail remains unchanged.\r\n\\end{document}\r\n"
    original = prefix + paragraph + separator + paragraph + suffix
    sources = {"main.tex": original, "notes.tex": "% Unrelated opaque bytes\r\n"}
    replaced = {**sources, "main.tex": prefix + paragraph + separator + escaped + suffix}
    deleted = {**sources, "main.tex": prefix + paragraph + separator + suffix}
    rows = []

    def check(case, actual, expected):
        passed = actual == expected
        rows.append({"case": case, "passed": passed, "expected": expected, "observed": actual})
        if not passed:
            raise AssertionError(f"{language}: {case}")

    def op(n):
        return f"{n:032x}"

    opened = bridge.ok("open", sources=sources)
    check("opened-source", opened["snapshot"]["sources"], sources)
    project = opened["projectId"]
    targets = bridge.ok("targets", revision=0, file="main.tex")["targets"]
    equal = [t for t in targets if t["text"] == paragraph]
    check("two-declared-occurrences", len(equal), 2)
    target = equal[1]["handle"]
    request = {"revision": 0, "operationId": op(1), "intent": {
        "kind": "replace_literal_paragraph", "target": target, "text": replacement}}
    check("unknown-operation", bridge.ok("status", operationId=op(1))["outcome"]["status"], "unknown")
    applied = bridge.ok("intent", **request)
    check("second-replacement-source", applied["snapshot"]["sources"], replaced)
    check("draft-only-proof", applied["outcome"]["render_verification"], "unresolved")
    check("protected-before-reply", applied["snapshot"]["recovery"]["protected_revision"], 1)
    check("lookup-after-discarded-reply", bridge.ok("status", operationId=op(1)), applied)
    check("duplicate-no-second-mutation", bridge.ok("intent", **request), applied)
    conflict = {**request, "intent": {**request["intent"], "text": "Different payload"}}
    check("operation-payload-conflict", bridge.ask("intent", **conflict)["ok"], False)
    check("conflict-source-preserved", bridge.ok("snapshot")["sources"], replaced)
    undo = bridge.ok("history", revision=1, redo=False)
    check("replace-exact-undo", undo["sources"], sources)
    retried = bridge.ok("intent", **request)
    check("retry-after-undo-original-receipt", retried["outcome"], applied["outcome"])
    check("retry-after-undo-current-snapshot", retried["snapshot"], undo)
    stale = {"revision": 2, "operationId": op(2), "intent": {
        "kind": "delete_paragraph_content", "target": target}}
    check("same-bytes-stale-handle", bridge.ask("intent", **stale)["ok"], False)
    fresh = bridge.ok("targets", revision=2, file="main.tex")["targets"]
    target = [t for t in fresh if t["text"] == paragraph][1]["handle"]
    noop = bridge.ok("intent", revision=2, operationId=op(3), intent={
        "kind": "replace_literal_paragraph", "target": target, "text": paragraph})
    check("noop-revision", noop["snapshot"]["revision"], 2)
    check("noop-retains-redo", noop["snapshot"]["can_redo"], True)
    deletion = bridge.ok("intent", revision=2, operationId=op(4), intent={
        "kind": "delete_paragraph_content", "target": target})
    check("second-deletion-exact-source", deletion["snapshot"]["sources"], deleted)
    check("delete-exact-undo", bridge.ok("history", revision=3, redo=False)["sources"], sources)
    redo = bridge.ok("history", revision=4, redo=True)
    check("delete-exact-redo", redo["sources"], deleted)
    check("unchanged-main-profile", [redo["main"], redo["engine"]],
          [opened["snapshot"]["main"], opened["snapshot"]["engine"]])
    recovered = bridge.ok("restart_recovery")
    check("restart-protected-source", recovered["snapshot"]["sources"], deleted)
    check("restart-operation-unresolved", bridge.ok("status", operationId=op(4))["outcome"]["status"], "unknown")
    check("reopened-old-target-refuses", bridge.ask("intent", revision=recovered["snapshot"]["revision"],
          operationId=op(5), intent={"kind":"delete_paragraph_content", "target":target})["ok"], False)
    recovered_targets = bridge.ok("targets", revision=recovered["snapshot"]["revision"], file="main.tex")["targets"]
    foreign = recovered_targets[0]["handle"]
    bridge.ok("open", sources=sources)
    check("foreign-session-target-refuses", bridge.ask("intent", revision=0, operationId=op(6),
          intent={"kind":"delete_paragraph_content", "target":foreign})["ok"], False)
    check("foreign-refusal-source-preserved", bridge.ok("snapshot")["sources"], sources)
    # The original main.tex was never saved by any draft/history/recovery command.
    return {"language": language, "project_id": project, "rows": rows,
            "expected_sources": {"original": sources, "replace": replaced, "delete": deleted}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output.resolve()
    if not output.is_relative_to(root / "artifacts"):
        raise ValueError("Use a fresh output directory below repository artifacts")
    output.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve(strict=True)
    inputs = [Path(__file__), root / "research/SOURCE-INTENT-TRANSPORT-PLAN.md",
              root / "src-tauri/examples/source_intent_transport_development.rs"]
    inputs += sorted((root / "src-tauri/src").rglob("*.rs"))
    inputs += sorted((root / "crates/texloom-core/src").rglob("*.rs"))
    identities = {p.relative_to(root).as_posix(): digest(p.read_bytes()) for p in inputs}
    write_json(output / "pre-run.json", {"recorded_utc": datetime.now(timezone.utc).isoformat(),
               "binary_sha256": digest(binary.read_bytes()), "source_sha256": identities,
               "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()})
    conditions = [("english", "Repeated paragraph.", "Changed & 50%.", "Changed \\& 50\\%."),
                  ("arabic", "مرحبا بالعالم.", "مرحبا & 50%.", "مرحبا \\& 50\\%."),
                  ("hebrew", "שלום עולם.", "שלום & 50%.", "שלום \\& 50\\%.")]
    runs = []
    failure = None
    for language, paragraph, replacement, escaped in conditions:
        bridge = Bridge(binary, output / language)
        try:
            result = diagnostic(bridge, language, paragraph, replacement, escaped)
            # Independent disk check: no command has original-file Save authority.
            first = output / language / "project-0" / "main.tex"
            assert first.read_bytes() == result["expected_sources"]["original"]["main.tex"].encode()
            runs.append(result)
        except Exception as error:
            failure = {"language": language, "error": str(error), "type": type(error).__name__}
        finally:
            code = bridge.close()
            write_json(output / (language + "-transcript.json"), bridge.transcript)
            if code and failure is None:
                failure = {"language": language, "error": f"Bridge exit {code}"}
        if failure:
            break
    changed = [name for name, expected in identities.items() if digest((root/name).read_bytes()) != expected]
    result = {"schema": 1, "kind": "native_source_draft_transport_development_diagnostic",
              "entry": "headless_native_service_not_tauri_dispatch_or_pdf_selection",
              "status": "passed" if not failure and not changed else "failed",
              "runs": runs, "failure": failure, "source_changed_during_run": changed,
              "final_evaluation": False, "render_proof": "unresolved", "gui_opened": False,
              "separate_service_test_conditions": ["ledger_capacity", "protection_failure", "pending_build_cancellation", "strict_request_dtos"]}
    write_json(output / "results.json", result)
    print(json.dumps({"status":result["status"], "conditions":len(runs),
                      "assertions":sum(len(r["rows"]) for r in runs), "failure":failure}))
    if result["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
