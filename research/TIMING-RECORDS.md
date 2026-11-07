# Timing records - development analysis protocol

Prepared 5 November 2026 during the earlier test pause. Updated 7 November: the analysis helpers are exercised by `tests/test_study_records_v1.py`, including three timing checks within its 13 passing synthetic tests. These are a subset of the already passing 40-helper suite, not additional independent experiments. They check separate endpoints, suppression for a missing trial and overlapping-export rejection. No actual Windows latency, collector-boundary or tracing-overhead measurement is established. Final evaluation remains `not_frozen`.

## Collector identity and loss accounting

The collector exports schema 2. An enabled diagnostic collector requests a local random UUID v4 once per launch; an ordinary disabled collector requests neither randomness nor clock values. The identifier is unrelated to projects and is not sent anywhere. If UUID creation fails, it stays null; the analysis draft declines that export instead of inventing an identity. An evaluator must still bind the actual diagnostic executable, flags and environment separately.

Record IDs increase within a launch. `epoch` increases on `.clear()`; `first_id`/`last_id` describe IDs issued in that epoch. Completed and active rows have distinct storage. Per-epoch counters retain evictions, capacity drops, explicit discards and invalid-clock starts. Cumulative `clear_history` counts completed/pending rows erased by clear. These counters describe tracing, not failed or successful user edits. Clear does not save a batch or prove that erased rows were exported.

An issued record should be retained, pending, evicted or explicitly discarded. The analysis draft checks that accounting, IDs, monotonic offsets, allowlisted categories and separate operation/frame fields. It rejects legacy schema-1 exports for this route; old artifacts are preserved and must not have synthetic collector IDs retrofitted. Existing legacy analysis remains separate.

## Batch specification

`evaluation/timing-batches-template-v1.json` is an empty specification, not data. A nonempty batch contains exactly `context`, `snapshot` and `trials`:

- `context`: `run_id`, `variant`, a 40-character `source_commit`, the 64-character hashes `executable_sha256`, `protocol_sha256`, `environment_sha256`, and `workload_id`, `document_id`, `engine`, `language`, `cache_state`, `primary_kind`. Hashes are lowercase hexadecimal. Cache state is `cold`, `warm`, `mixed` or `unknown`; a condition label is not proof of a cold cache. Primary kinds are the eight categories in PERFORMANCE-DIAGNOSTICS.md.
- `snapshot`: a portable relative `path`, positive `bytes` and lowercase SHA-256. The file is resolved only under an explicit controlled evidence root. Its observed bytes must match before its records can contribute.
- `trials`: planned `{trial_id, record_id, role}` entries. Role is `warmup` or `measured`, fixed before collection. Record ID is a positive integer, or null when the planned action produced no identified trace. Null/missing traces remain unobserved, never zero latency. Trial IDs are unique within a run/variant. A record can serve only one trial. Additional main-category records must be declared rather than silently discarded; other categories are counted as auxiliary traces.

Export small, disjoint batches. Do not submit multiple cumulative snapshots containing the same `(collector_id, epoch, record_id)`. They are rejected rather than counted twice. One collector cannot be relabelled with conflicting run, source, executable, protocol or environment bindings. Separate actual source/executable versions use separate variants and collectors. Warmups and repeated timing trials are not new independent correctness requests.

The checker uses bounded, read-only local inputs, strict JSON, relative-path checks and exclusive output creation from `evidence_io.py`. It does not run commands, load the app, compile documents, collect trials or alter the final freeze record. Controlled directories are required; this is not an operating-system security boundary against a hostile filesystem. Byte identity does not authenticate supplied context, pre-run planning, execution or source-to-executable provenance.

Each snapshot's declared byte length is its read ceiling as well as an expected identity. Oversized/changed inputs remain unresolved; declaring many tiny files cannot authorize reading a full maximum-sized file at every path. The synthetic infrastructure checks cover a bounded subset of the helper rules; they are not exhaustive validation of every budget or filesystem condition.

## Analysis rules fixed before collection

Group by the entire context above; do not pool engines, languages, documents, workloads, cache states, builds or environments into one favourable number. The output retains every planned trial, warmup role, missing trace, pending operation, outcome, frame state and loss counter. Refusals, stale/cancelled work, compilation failures, uncertain commits and committed presentation failures are distinct outcomes. A `published` trace is not an independent correctness judgement.

For current epochs with any loss, pending operation, unclaimed main record, truncated selected trace or missing planned sample, suppress distributions and retain counts. This conservative rule applies to the current exported epochs only. Cumulative clear counts remain visible; they do not establish complete coverage of an entire launch or prior epochs. Stopped unfinished operations also make coverage incomplete. A completed operation whose frame estimate is censored can retain an observed operation endpoint.

Within complete current epochs, report measured-trial distributions separately by outcome and endpoint. `operation_ms` spans handler/method entry to its operation boundary. `frame_proxy_ms` spans the same start to two animation callbacks after publication while current/visible. Missing/censored frame estimates are counted by state, not assigned zero or timeout duration. Distributions are explicitly conditional on observed endpoints; observation counts accompany them. A conditional p95 is not a p95 for all requested actions when endpoints are censored. Sample median and nearest-rank p95 (`ceil(.95*n)`) describe the collected sample, without confidence intervals or population generalization.

The module computes no correctness, human usability, hardware input-to-display latency, sustained gesture frame rate or tracing overhead. In particular, diagnostic-only offsets cannot compare ordinary and diagnostic build overhead. That comparison still requires a predeclared external method and paired ordinary/diagnostic workloads on recorded executables. No timing target can pass while its endpoint and actual samples remain unmeasured.

## Deferred use

Before app trials, review the actual diagnostic collector and its analysis on a named build with controlled synthetic projects. Helper-test results do not complete Windows measurement admission. A future development-only invocation is:

```powershell
python research/tools/timing_records.py <batch-manifest.json> <new-report.json> --root <controlled-evidence-root> --development
```

This command has **not run**. Its exit status would describe input processing, not a performance or product pass. Any collector/analysis failure must be preserved before changes; no old observations are silently migrated or converted into final results.
