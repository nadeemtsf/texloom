# Native source-service performance baseline

The production Backend::document_edit currently returns a complete DocumentSnapshot, including every source file. The benchmark measures that real method and serialization of the real response type, rather than a copied implementation or a generic string microbenchmark.

The original local example did not compile because path-importing service.rs changed child-module lookup. The corrected example imports the existing library through a benchmark feature that exports only Rust harness types. It adds no IPC commands or desktop-control hooks. Cargo requires explicit feature opt-in, and the example is not packaged.

## Run

After frontend assets/verifier preparation or an ordinary production build:

    cargo run --locked --release -p texloom-desktop --features benchmark --example source_benchmark --target x86_64-pc-windows-msvc -- artifacts/source-benchmark

The parent directory must exist and the output directory must be new. The harness refuses to overwrite existing output. All generated projects stay inside that supplied directory. It does not invoke a compiler, send UI input or save the edits to the original generated files.

Five synthetic sizes range from one 16 KiB file to 128 files totaling 32 MiB. Each uses BOM, CRLF and supplementary Unicode, 10 warmup edits and 100 measured edits. A fixed source typing group is undone/redone exactly. Original disk fingerprints must remain unchanged. The report separates native edit/snapshot time from JSON encoding and records response size, median, p95 and maximum.

The Windows workflow runs the release benchmark after building and uploading the desktop artifact. Its JSON is separately preserved. CI numbers characterize the runner and cannot establish laptop or editor-paint targets. WebView transport, JavaScript parsing, response destruction, paint, active compiler/cache effects, recovery journaling and disk Save are outside the interval.

No performance result is claimed until the harness compiles and its report is inspected. Native interaction remains in FINAL-NATIVE-ACCEPTANCE.md. The local disconnected checkout's work-in-progress draft must be preserved and reconciled carefully when it reconnects.
