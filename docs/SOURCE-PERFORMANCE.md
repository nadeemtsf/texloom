# Native source-service performance baseline

Typing IPC now calls Backend::document_edit_delta and returns a tagged source_delta acknowledgement containing only the changed canonical file. It includes the exact base/new revision, history flags, profile and selection. A no-op contains no source files. The frontend validates the base, single-file identity, revision increment, source type and metadata before merging it into its acknowledged replica. Invalid replies stop dependent source events while retaining the last acknowledgement. Opening, undo/redo and visual commits still return full snapshots because those operations can change the file set.

Both response paths call the same native edit implementation: projection, preconditions, candidate commit, shared history, compile cancellation and stale-candidate invalidation are shared. Typing now prepares only the affected existing source file; visual/settings candidates retain full immutable snapshots. The complete-snapshot API remains a native test/benchmark reference. Canonical strings remain owned by Rust. The frontend uses shallow copies of immutable source strings and exact saved-buffer equality, avoiding whole-project structured cloning and JSON allocation during typing acknowledgements/status checks.

The original local example did not compile because path-importing service.rs changed child-module lookup. The corrected example imports the existing library through a benchmark feature that exports only Rust harness types. It adds no IPC commands or desktop-control hooks. Cargo requires explicit feature opt-in, and the example is not packaged.

## Run

After frontend assets/verifier preparation or an ordinary production build:

    cargo run --locked --release -p texloom-desktop --features benchmark --example source_benchmark --target x86_64-pc-windows-msvc -- artifacts/source-benchmark

The parent directory must exist and the output directory must be new. The harness refuses to overwrite existing output. All generated projects stay inside that supplied directory. It does not invoke a compiler, send UI input or save the edits to the original generated files.

Five synthetic sizes range from one 16 KiB file to 128 files totaling 32 MiB. Historical schema 2 measures complete-snapshot reference and changed-file production replies in the same run; schema 4 additionally isolates project-snapshot versus single-file planning with identical changed-file replies and durable journals. Each uses BOM, CRLF and supplementary Unicode, 10 warmup edits and 100 measured edits. A fixed source typing group is undone/redone exactly. Original disk fingerprints must remain unchanged. The report separates native edit/reply time from JSON encoding and records response size, median, p95 and maximum.

The Windows workflow runs the release benchmark after building and uploading the desktop artifact. Its JSON is separately preserved. CI numbers characterize the runner and cannot establish laptop or editor-paint targets. WebView transport, JavaScript parsing, response destruction, paint, active compiler/cache effects and disk Save are outside the interval. Schemas 3 and 4 include source recovery append/flush; schema 2 predates the active journal.

## Observed results

On 3 November 2026 the full-snapshot benchmark at b649034 passed Windows CI 41593287808. Its 32 MiB case returned 34,452,005 bytes and measured 73.03 ms p95 through response encoding on the two-core CI runner. This established the response-size problem; it is not a laptop measurement.

The local schema-2 release run compared both paths on the same eight-logical-processor Windows host. Raw timings and limitations are preserved in evidence/source-ack-local.json. These are single runs with uncontrolled background load, not statistically controlled latency guarantees.

| Project | Full response bytes | Changed-file response bytes | Full p95 through encoding | Changed-file p95 through encoding |
| --- | ---: | ---: | ---: | ---: |
| 1 x 16 KiB | 17,004 | 17,046 | 0.58 ms | 0.51 ms |
| 1 x 256 KiB | 269,320 | 269,362 | 10.10 ms | 9.25 ms |
| 16 x 64 KiB | 1,077,043 | 67,510 | 5.01 ms | 2.23 ms |
| 64 x 256 KiB | 17,226,085 | 269,362 | 72.06 ms | 17.92 ms |
| 128 x 256 KiB | 34,452,005 | 269,362 | 79.17 ms | 19.76 ms |

A single-file edit still returns that file, and native preparation still clones the canonical project map/strings. Further storage/projection optimizations require their own measurements. The frontend changes have functional tests but no native paint measurement. This does not establish the 50 ms editor-paint target, typing/IME acceptance, active-journal performance or a release gate. Native interaction remains in FINAL-NATIVE-ACCEPTANCE.md.

The reconnected checkout was reconciled to c83dda1 without discarding the earlier untracked work-in-progress/source_benchmark.rs draft. That historical draft remains preserved and is not used by the working benchmark.

## Journal-enabled measurement

Schema 3 runs the production durable journal, asserts that every measured acknowledgement reports its revision protected, and includes append/flush work in native edit timing. It captures the combined endpoint before dropping the native reply, consistently excluding reply/payload destruction and transport. The earlier schema-2 combined endpoint occurred after branch-local reply destruction; its generic destruction-exclusion text was too broad. Its raw measurements are preserved and should not be treated as a controlled fsync-overhead comparison with schema 3.

The local active-journal release run passed; its raw schema-3 report is in evidence/typing-recovery-local.json. Each of the ten cases includes 10 warmup and 100 measured edits, protected-revision assertions, exact coalesced undo/redo and unchanged original disk fingerprints.

| Project | Full p95 through encoding | Changed-file p95 through encoding | Changed-file reply bytes |
| --- | ---: | ---: | ---: |
| 1 x 16 KiB | 6.02 ms | 6.99 ms | 17,143 |
| 1 x 256 KiB | 24.40 ms | 19.74 ms | 269,459 |
| 16 x 64 KiB | 13.43 ms | 8.23 ms | 67,607 |
| 64 x 256 KiB | 155.56 ms | 55.17 ms | 269,459 |
| 128 x 256 KiB | 257.85 ms | 56.39 ms | 269,459 |

This is one uncontrolled run, not a controlled comparison of journaling overhead against earlier runs. Large-project timings remain a performance concern; canonical preparation still copies untouched source strings. No 50 ms editor-paint target is inferred. Measured edits do not include per-edit cursor acknowledgements or force checkpoint compaction. Startup/recovery replay, editor paint and native typing remain separate measurements.

## Guarded single-file preparation

The canonical typing path uses an opaque PreparedSourceEdit bound to its creating Session and base revision. It validates the same exact UTF-8 replacements and inverse as full preparation, checks per-file/project byte limits, and replaces only the existing affected String on acceptance. It cannot create/remove files, change the build profile or carry compiler/visual approval. Untouched source allocations remain identical during typing. Both plan types use one shared history recorder with the same source coalescing, redo invalidation, byte/entry bounds and selection rules. Visual/settings candidates and compiler inputs remain immutable full snapshots.

Coalesced undo/redo now copies the project once into a private draft and consumes that draft for successive steps. A failed later step discards the draft without changing canonical buffers, revision, selection or the history cursor. Verified source-only journal records use the same single-file planner during replay; other journal mutations retain generic preparation. Checkpoint validation still checks the complete retained timeline and is outside this typing measurement.

Eight new core checks include unchanged allocation identity, foreign/stale plans, no-op after undo, invalid boundaries/expected bytes/overlap, revision exhaustion, source byte limits, mixed source/visual/profile history, atomic failed history replay, and an 800-step deterministic Unicode differential sequence against the full-snapshot planner. With the optional codec enabled, every differential step compares complete checkpoint data as well. Existing native recovery tests cross from generic recorded patches into the new replay planner.

Schema 4 runs the actual reference planner behind the opt-in benchmark feature and shares production projection, commit/history, cache/cancellation, journal and acknowledgement code. It adds no IPC command or UI control. All fifteen cases require protected acknowledgements, exact coalesced undo/redo and unchanged original project fingerprints. The third mode retains a full-response measurement but uses the new planner; compare the first two modes to assess planning alone.

| Project | Project plan, changed-file reply p95 | Single-file plan, same reply p95 | Reply bytes in both |
| --- | ---: | ---: | ---: |
| 1 x 16 KiB | 3.81 ms | 3.59 ms | 17,143 |
| 1 x 256 KiB | 10.95 ms | 10.60 ms | 269,459 |
| 16 x 64 KiB | 5.52 ms | 4.86 ms | 67,607 |
| 64 x 256 KiB | 18.02 ms | 10.83 ms | 269,459 |
| 128 x 256 KiB | 23.74 ms | 16.09 ms | 269,459 |

This local same-run comparison supports removing the untouched-source copies. It is one synthetic run with uncontrolled background load, not an end-to-end typing guarantee. Do not compare it to earlier separate runs as a controlled recovery-overhead experiment. Source-file projection and reply encoding still process the affected file. Per-edit cursor flushes, checkpoint compaction/startup replay, WebView transport, editor paint, native typing/IME and long-document usability remain separate work. Raw schema-4 measurements and exact tested source/log/executable receipts are in evidence/source-plans-local.json.
