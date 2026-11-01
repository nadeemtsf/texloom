# P0/P1 checkpoint — 1 November 2026

The owner approved docs/APPROVED-IMPLEMENTATION-PLAN.md and authorized implementation.

## Observed locally before the Windows workspace disconnected

- Baseline e7b6d1eb334c879a4aa28c4f22b984fb3c17532b; main initially clean.
- Work started on spike/visual-editing-proof.
- 33 Rust core tests passed.
- 16 frontend/PDF/oracle tests passed (13 existing and 3 new).
- npm run build passed with the existing large-chunk warning.
- Windows x64 Tauri production build passed with tauri/custom-protocol.
- The fixture CLI built using the real texloom-core functions.
- Nine fixture source selections wrapped/unwrapped byte-identically.
- Linked text was refused by the Rust planner.
- PDF.js rejected the known direct-literal regression: unselected record 4 moved.
- Independent PDFium 126.0.6462.0 through pypdfium2 4.30.0 rejected it too: unselected record 109 moved.
- Baseline/zero/moved source snapshots were prepared.

The Python/PDFium inspector is a development-only dependency, not shipped with the application.

## Not established

Fresh geometry/raster equivalence for the corrected movement package; the complete three-engine matrix; Lua attribute/source provenance; parser-library choice; a final architecture ADR; native mouse group drag/edit/undo/save/reopen; installation; a new release package.

MiKTeX command-line access failed with RegOpenKeyExW/access-denied errors. The built-in compiler failed twice to start. The attached Windows environment then became unavailable.

## Checkpoint integrity

This branch preserves the fixture experiment and approved plan through GitHub after local execution disappeared. It was reconstructed from the changes/tool results visible in the session. Diagnostic-message construction and the cross-page runner path received small review corrections during checkpoint preparation; remote checks must establish the resulting committed file set, rather than assuming byte identity with the earlier local run.

Main and user projects are unchanged. On reconnect, compare local uncommitted changes with this branch before integrating; do not reset or overwrite that working tree.

## Reproduction

Run npm ci, cargo test --locked -p texloom-core, and cargo build --locked -p texloom-core --example proof_candidate.
Install the development oracle with python -m pip install -r scripts/proof/requirements.txt.
Run npm test.
Run node scripts/movement-proof.mjs --prepare for source identity and existing PDF regression checks.
Run node scripts/movement-proof.mjs for the real-engine experiment.

PYTHON selects the Python executable; PROOF_CANDIDATE_BIN selects the fixture CLI. A local isolated install under .tools/proof-python is also supported.

Generated evidence goes under artifacts/proof, ignored by Git. Exit 0 means passed/prepared, 1 failed, 2 blocked. Prepared is not a movement-verification pass.

The engine experiment may legitimately fail: its purpose is to find adapter failures, not certify the current macro. The first inspector refuses rotated/nonzero-crop pages. The fixed two-pass recipe is only for these bibliography-free fixtures, not the production compile scheduler. The Linux CI distribution packages are an exploratory compatibility profile; capture actual versions from the report, and do not call this a hermetic release environment.

## First remote run

Run 41327359752 reproduced source/regression checks and reached all three real engines. Its initial layout oracle failed on overlaps and extraction-order assumptions. The raw report is retained in evidence/proof-first-run.json. A follow-up revision adds position-based correspondence, clear-space fixtures, and an explicit overlap-refusal case. The new test file now contains one additional reordering regression; the last locally executed count remains 16.
