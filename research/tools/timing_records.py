"""Unexecuted development timing analysis; never starts Texloom or an experiment.

Only schema-2 collector exports are accepted. Handler/operation and two-frame
proxy offsets are separate; neither is physical input-to-display latency.
"""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import math
from pathlib import Path
import re
import statistics

if __package__:
    from .evidence_io import (MAX_JSON_BYTES, contained_file, read_fingerprint,
                             read_json, relative_name, require, strict_json, write_json)
else:
    from evidence_io import (MAX_JSON_BYTES, contained_file, read_fingerprint,
                            read_json, relative_name, require, strict_json, write_json)


KINDS = {"source-feedback", "source-selection", "object-selection", "drag-preview",
         "text-move", "text-edit", "compile-manual", "compile-auto"}
OUTCOMES = {"feedback", "selected", "published", "committed-ui-failed", "commit-unknown",
            "refused", "failed", "compile-failed", "stale", "cancelled", "superseded"}
FRAME_STATES = {"observed", "hidden", "superseded", "timeout", "unavailable", "invalid-clock",
                "stopped", "not-requested", "pending"}
PHASES = {"source-queued", "feedback-updated", "mapping-resolved", "planned", "native-prepared",
          "build-returned", "pdf-prepared", "frontend-validated", "build-rechecked", "native-committed",
          "published", "selection-ready", "preview-updated", "sync-flushed"}
CONTEXT = ("run_id", "variant", "source_commit", "executable_sha256", "protocol_sha256",
           "environment_sha256", "workload_id", "document_id", "engine", "language", "cache_state", "primary_kind")
SNAPSHOT_FIELDS = {"schema", "clock", "endpoint", "collector_id", "epoch", "first_id", "last_id",
                   "clear_history", "enabled", "capacity", "evicted", "dropped", "discarded", "invalid_starts",
                   "active", "records", "pending"}
RECORD_FIELDS = {"id", "kind", "started_ms", "phases", "truncated_phases", "outcome",
                 "operation_ms", "frame_proxy_ms", "frame_status"}


def count(value):
    return type(value) is int and 0 <= value <= 2 ** 53 - 1


def duration(value):
    return type(value) in {int, float} and math.isfinite(value) and value >= 0


def digest(value, length=64):
    return isinstance(value, str) and re.fullmatch(f"[0-9a-f]{{{length}}}", value) is not None


def label(value):
    return isinstance(value, str) and 0 < len(value) <= 256 and not any(ord(c) < 32 for c in value)


def validate_record(row, *, pending):
    require(isinstance(row, dict) and set(row) == RECORD_FIELDS, "Invalid collector record fields")
    require(count(row["id"]) and row["id"] > 0 and isinstance(row["kind"], str) and row["kind"] in KINDS,
            "Invalid collector record identity/category")
    require(duration(row["started_ms"]) and count(row["truncated_phases"]), "Invalid collector offset/count")
    require(isinstance(row["frame_status"], str) and row["frame_status"] in FRAME_STATES, "Unknown frame state")
    require(row["outcome"] is None or isinstance(row["outcome"], str) and row["outcome"] in OUTCOMES, "Unknown outcome")
    require(all(row[key] is None or duration(row[key]) for key in ("operation_ms", "frame_proxy_ms")), "Invalid duration")
    require(isinstance(row["phases"], list) and len(row["phases"]) <= 64, "Invalid phase list")
    last = 0
    for phase in row["phases"]:
        require(isinstance(phase, dict) and set(phase) == {"name", "offset_ms"} and
                isinstance(phase["name"], str) and phase["name"] in PHASES and
                duration(phase["offset_ms"]) and phase["offset_ms"] >= last, "Invalid or backwards phase")
        last = phase["offset_ms"]
    require(row["operation_ms"] is None or row["operation_ms"] >= last, "Operation precedes a phase")
    if row["outcome"] is None:
        require(row["operation_ms"] is None and row["frame_proxy_ms"] is None and
                row["frame_status"] == ("not-requested" if pending else "stopped"), "Invalid unfinished operation")
    if row["frame_status"] == "observed":
        require(not pending and row["operation_ms"] is not None and row["frame_proxy_ms"] is not None and
                row["frame_proxy_ms"] >= row["operation_ms"], "Invalid observed frame proxy")
    else:
        require(row["frame_proxy_ms"] is None, "Unobserved frame has a fabricated duration")
    require((pending and row["frame_status"] in {"not-requested", "pending"}) or
            (not pending and row["frame_status"] != "pending"), "Pending/completed state disagrees")


def validate_snapshot(value):
    require(isinstance(value, dict) and set(value) == SNAPSHOT_FIELDS, "Wrong snapshot fields; legacy schema 1 needs separate handling")
    require(type(value["schema"]) is int and value["schema"] == 2 and value["clock"] == "performance.now" and
            value["endpoint"] == "two-animation-frames-proxy", "Unknown measurement boundary")
    require(isinstance(value["collector_id"], str) and re.fullmatch(
        r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}", value["collector_id"]) is not None,
        "Missing per-launch collector identity")
    require(type(value["enabled"]) is bool and all(count(value[key]) for key in
        ("epoch", "first_id", "last_id", "capacity", "evicted", "dropped", "discarded", "invalid_starts", "active")), "Invalid collector counters")
    require(1 <= value["capacity"] <= 4096 and value["active"] <= 64 and value["first_id"] >= 1, "Collector budget exceeded")
    require(isinstance(value["clear_history"], dict) and set(value["clear_history"]) == {"completed", "pending"} and
            all(count(n) for n in value["clear_history"].values()), "Invalid clear history")
    require(isinstance(value["records"], list) and len(value["records"]) <= value["capacity"] and
            isinstance(value["pending"], list) and len(value["pending"]) == value["active"], "Invalid record storage")
    rows = {}
    for is_pending, records in ((False, value["records"]), (True, value["pending"])):
        for row in records:
            validate_record(row, pending=is_pending)
            require(value["first_id"] <= row["id"] <= value["last_id"] and row["id"] not in rows, "Duplicate/out-of-epoch record")
            rows[row["id"]] = (row, is_pending)
    require(value["last_id"] - value["first_id"] + 1 == len(rows) + value["evicted"] + value["discarded"],
            "Issued record count disagrees with retained/lost records")
    return rows


def validate_manifest(value):
    require(isinstance(value, dict) and set(value) == {"schema_version", "kind", "batches"} and
            type(value["schema_version"]) is int and value["schema_version"] == 1 and
            value["kind"] == "development_timing_batches_not_final", "Wrong timing manifest kind")
    require(isinstance(value["batches"], list) and len(value["batches"]) <= 1000, "Invalid batch count")
    trial_ids, paths, total_trials, total_bytes = set(), set(), 0, 0
    for batch in value["batches"]:
        require(isinstance(batch, dict) and set(batch) == {"context", "snapshot", "trials"}, "Invalid batch fields")
        context, reference = batch["context"], batch["snapshot"]
        require(isinstance(context, dict) and set(context) == set(CONTEXT) and all(label(v) for v in context.values()), "Incomplete timing context")
        require(digest(context["source_commit"], 40) and all(digest(context[key]) for key in
                ("executable_sha256", "protocol_sha256", "environment_sha256")) and context["primary_kind"] in KINDS,
                "Missing fixed source/executable/protocol/environment binding")
        require(context["cache_state"] in {"cold", "warm", "mixed", "unknown"}, "Unknown cache condition")
        require(isinstance(reference, dict) and set(reference) == {"path", "bytes", "sha256"}, "Invalid snapshot reference")
        relative_name(reference["path"])
        require(reference["path"].casefold() not in paths and count(reference["bytes"]) and
                0 < reference["bytes"] <= MAX_JSON_BYTES and digest(reference["sha256"]), "Duplicate/invalid snapshot reference")
        paths.add(reference["path"].casefold())
        total_bytes += reference["bytes"]
        require(total_bytes <= 2 * 1024 ** 3, "Timing input byte budget exceeded")
        require(isinstance(batch["trials"], list) and 0 < len(batch["trials"]) <= 10000, "Missing planned trials")
        record_ids = set()
        for trial in batch["trials"]:
            require(isinstance(trial, dict) and set(trial) == {"trial_id", "record_id", "role"} and
                    label(trial["trial_id"]) and isinstance(trial["role"], str) and
                    trial["role"] in {"warmup", "measured"}, "Invalid planned trial")
            key = (context["run_id"], context["variant"], trial["trial_id"])
            require(key not in trial_ids, "Duplicate planned timing trial")
            trial_ids.add(key)
            if trial["record_id"] is not None:
                require(count(trial["record_id"]) and trial["record_id"] > 0 and trial["record_id"] not in record_ids,
                        "Duplicate/invalid trial-to-record association")
                record_ids.add(trial["record_id"])
        total_trials += len(batch["trials"])
        require(total_trials <= 10000, "Timing trial budget exceeded")
    return value["batches"]


def distribution(values):
    if not values:
        return {"n": 0, "median_ms": None, "p95_ms": None, "max_ms": None}
    ordered = sorted(values)
    return {"n": len(values), "median_ms": statistics.median(ordered),
            "p95_ms": ordered[math.ceil(.95 * len(ordered)) - 1], "max_ms": ordered[-1]}


def summarize(manifest, root):
    batches = validate_manifest(manifest)
    groups, observations, seen, collector_bindings = defaultdict(list), [], set(), {}
    for batch in batches:
        context, reference = batch["context"], batch["snapshot"]
        observation = {"context": context, "snapshot_reference": reference, "status": "unresolved_snapshot",
                       "coverage_complete_current_epoch": False, "trials": []}
        try:
            path = contained_file(root, reference["path"])
            receipt, raw = read_fingerprint(path, retain=True, limit=reference["bytes"])
            require(receipt == {key: reference[key] for key in ("bytes", "sha256")}, "Snapshot bytes differ from declaration")
            snapshot = strict_json(raw)
            records = validate_snapshot(snapshot)
        except (OSError, ValueError, UnicodeError, RecursionError) as error:
            observation["error_type"] = type(error).__name__
            snapshot, records = None, {}
        if snapshot is not None:
            collector = snapshot["collector_id"]
            binding = tuple(context[key] for key in ("run_id", "variant", "source_commit", "executable_sha256", "protocol_sha256", "environment_sha256"))
            require(collector not in collector_bindings or collector_bindings[collector] == binding, "One collector has conflicting run/artifact bindings")
            collector_bindings[collector] = binding
            keys = {(collector, snapshot["epoch"], record_id) for record_id in records}
            require(not keys & seen, "Overlapping snapshots would repeat retained records")
            seen.update(keys)
            claimed = {trial["record_id"] for trial in batch["trials"]}
            unclaimed = sum(row["kind"] == context["primary_kind"] and record_id not in claimed
                            for record_id, (row, _) in records.items())
            observation.update(status="snapshot_bytes_and_structure_established", collector_id=collector,
                epoch=snapshot["epoch"], clear_history=snapshot["clear_history"],
                counters={key: snapshot[key] for key in ("evicted", "dropped", "discarded", "invalid_starts", "active")},
                auxiliary_records=sum(row["kind"] != context["primary_kind"] for row, _ in records.values()),
                unclaimed_primary_records=unclaimed)
            observation["coverage_complete_current_epoch"] = not unclaimed and not any(observation["counters"].values())
        for trial in batch["trials"]:
            sample = {**trial, "status": "not_observed", "outcome": None, "frame_status": None,
                      "operation_ms": None, "frame_proxy_ms": None}
            if trial["record_id"] in records:
                row, pending = records[trial["record_id"]]
                require(row["kind"] == context["primary_kind"], "Trial category differs from declared workload")
                sample.update(status="pending" if pending else "retained", outcome=row["outcome"],
                    frame_status=row["frame_status"], operation_ms=row["operation_ms"], frame_proxy_ms=row["frame_proxy_ms"],
                    truncated_phases=row["truncated_phases"])
                if pending or row["truncated_phases"] or row["outcome"] is None:
                    observation["coverage_complete_current_epoch"] = False
            else:
                observation["coverage_complete_current_epoch"] = False
            observation["trials"].append(sample)
        groups[tuple(context[key] for key in CONTEXT)].append(observation)
        observations.append(observation)
    summaries = []
    for key, data in sorted(groups.items()):
        coverage = all(item["coverage_complete_current_epoch"] for item in data)
        samples = [trial for item in data for trial in item["trials"]]
        measured = [trial for trial in samples if trial["role"] == "measured"]
        outcomes = {}
        for outcome in sorted({trial["outcome"] for trial in measured if trial["outcome"] is not None}):
            rows = [trial for trial in measured if trial["outcome"] == outcome]
            operations = [row["operation_ms"] for row in rows if row["operation_ms"] is not None]
            frames = [row["frame_proxy_ms"] for row in rows if row["frame_status"] == "observed"]
            outcomes[outcome] = {"n": len(rows), "frame_states": dict(Counter(row["frame_status"] for row in rows)),
                "operation_observed": len(operations), "frame_proxy_observed": len(frames),
                "conditional_operation_distribution": distribution(operations) if coverage else None,
                "conditional_frame_proxy_distribution": distribution(frames) if coverage else None}
        summaries.append({**dict(zip(CONTEXT, key)), "batch_count": len(data), "planned_trials": len(samples),
            "warmups": sum(row["role"] == "warmup" for row in samples), "measured_trials": len(measured),
            "not_observed": sum(row["status"] == "not_observed" for row in measured),
            "pending": sum(row["status"] == "pending" for row in measured),
            "coverage_complete_current_epochs": coverage, "outcomes": outcomes})
    return {"schema": 1, "kind": "development_timing_summary_not_final_evaluation",
            "status": "empty_no_results" if not batches else "development_only", "groups": summaries, "batches": observations,
            "experiments_executed": False, "final_evaluation_executed": False,
            "correctness_computed": False, "physical_latency_computed": False,
            "limits": ["Trial planning, artifact relationships and execution chronology remain supplied metadata, not authenticated facts.",
                       "Quantiles are conditional on observed endpoints; missing/censored frames are counted, never zero-filled.",
                       "Incomplete current epochs suppress distributions; clear history is reported, not proof that prior epochs were exported.",
                       "Median uses the sample median; p95 uses nearest rank ceil(.95*n); neither is a confidence interval.",
                       "No pooling across run, variant, artifact, environment, workload, document, engine, language, cache state or category.",
                       "Operation outcomes are not independent correctness labels; frame callbacks do not establish physical presentation.",
                       "No ordinary-build tracing overhead, sustained frame rate, human usability or population performance is estimated."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--development", action="store_true", required=True)
    args = parser.parse_args()
    inputs = {}
    try:
        manifest, inputs["manifest"] = read_json(args.manifest)
        result = summarize(manifest, args.root)
    except (OSError, ValueError, UnicodeError, RecursionError) as error:
        result = {"schema": 1, "kind": "development_timing_summary_not_final_evaluation", "status": "rejected_input",
                  "error_type": type(error).__name__, "experiments_executed": False, "final_evaluation_executed": False}
    result.update(recorded_utc=datetime.now(timezone.utc).isoformat(), input_identities=inputs,
                  analysis_files={name: hashlib.sha256(Path(__file__).with_name(name).read_bytes()).hexdigest()
                                  for name in ("timing_records.py", "evidence_io.py")})
    write_json(args.output, result)
    print("Development timing record report written. No experiment or physical latency measurement performed.")
    return 1 if result["status"] == "rejected_input" else 0


if __name__ == "__main__":
    raise SystemExit(main())
