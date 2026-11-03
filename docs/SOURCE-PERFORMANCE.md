# Native source-service performance baseline

Typing IPC now calls Backend::document_edit_delta and returns a tagged source_delta acknowledgement containing only the changed canonical file. It includes the exact base/new revision, history flags, profile and selection. A no-op contains no source files. The frontend validates the base, single-file identity, revision increment, source type and metadata before merging it into its acknowledged replica. Invalid replies stop dependent source events while retaining the last acknowledgement. Opening, undo/redo and visual commits still return full snapshots because those operations can change the file set.

Both response paths call the same native edit implementation: projection, preconditions, candidate commit, shared history, compile cancellation and stale-candidate invalidation are unchanged. The complete-snapshot API remains a native test/benchmark reference. Canonical strings remain owned by Rust. The frontend uses shallow copies of immutable source strings and exact saved-buffer equality, avoiding whole-project structured cloning and JSON allocation during typing acknowledgements/status checks.

The original local example did not compile because path-importing service.rs changed child-module lookup. The corrected example imports the existing library through a benchmark feature that exports only Rust harness types. It adds no IPC commands or desktop-control hooks. Cargo requires explicit feature opt-in, and the example is not packaged.

## Run

After frontend assets/verifier preparation or an ordinary production build:

    cargo run --locked --release -p texloom-desktop --features benchmark --example source_benchmark --target x86_64-pc-windows-msvc -- artifacts/source-benchmark

The parent directory must exist and the output directory must be new. The harness refuses to overwrite existing output. All generated projects stay inside that supplied directory. It does not invoke a compiler, send UI input or save the edits to the original generated files.

Five synthetic sizes range from one 16 KiB file to 128 files totaling 32 MiB. Schema 2 measures both complete-snapshot reference and changed-file production replies in the same run. Each uses BOM, CRLF and supplementary Unicode, 10 warmup edits and 100 measured edits. A fixed source typing group is undone/redone exactly. Original disk fingerprints must remain unchanged. The report separates native edit/reply time from JSON encoding and records response size, median, p95 and maximum.

The Windows workflow runs the release benchmark after building and uploading the desktop artifact. Its JSON is separately preserved. CI numbers characterize the runner and cannot establish laptop or editor-paint targets. WebView transport, JavaScript parsing, response destruction, paint, active compiler/cache effects, recovery journaling and disk Save are outside the interval.

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
