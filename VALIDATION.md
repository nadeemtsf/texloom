# Validation record

Updated 1 November 2026. This records observed checks, not a release certification.
Approved implementation: docs/APPROVED-IMPLEMENTATION-PLAN.md.
Working experiment: spike/visual-editing-proof, draft PR https://github.com/nadeemtsf/texloom/pull/1.

## Reproduced locally

- 33 Rust core tests passed with cargo test --offline --locked -p texloom-core.
- 16 frontend/PDF/oracle tests passed with npm test.
- npm run build passed; the existing Vite large-bundle warning remains.
- Windows x64 production compilation passed with cargo build --offline --locked --release --target x86_64-pc-windows-msvc -p texloom-desktop --features tauri/custom-protocol.
- The Rust fixture CLI built successfully.
- Nine source wrap/unwrap round trips preserved every original byte; linked text was refused.
- PDF.js and independent PDFium both rejected the known unsafe direct-literal PDF.

## Reproduced remotely

The initial proof workflow on commit 3d8558076509b92d6885d8ed48c8b806f996ff83 passed source tests, 16 JavaScript tests, source round trips, and both existing-PDF regressions. It compiled all 27 initial fixture/engine combinations with the TeX Live 2023/Debian profile. Three cross-page cases were correctly identified; remaining comparisons failed at the initial oracle or overlap checks. Their raw outcomes are retained in docs/evidence/proof-first-run.json.

The first comparison did not establish the corrected movement macro's acceptance. The revised fixture/oracle experiment needs its own result. No whole-project test pass is inferred from a successful compile.

## Environment limits

MiKTeX command-line invocation could not access configuration registry keys. The built-in compiler failed to start. The Windows execution environment then disconnected. GitHub preserves the experiment branch; local and remote changes must be reconciled safely after reconnecting.

## Outstanding release checks

Native click/group/drag/edit/undo/save/reopen; correct-engine and PDF appearance verification; runtime independent validation; external-editor conflicts; crash recovery; actual installer and clean-machine setup; signing; current dependency audit; Store certification.

The existing application and downloadable baseline remain development prototypes. No new release or installer certification is claimed.
