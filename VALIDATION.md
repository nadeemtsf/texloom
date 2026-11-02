# Validation record

Updated 2 November 2026. Individual checks, not release certification. Approved scope: docs/APPROVED-IMPLEMENTATION-PLAN.md.

## P3 compile-service checkpoint

The owner explicitly authorized P3 before manual P2 input acceptance. Native P2 interaction remains open.

- 51 core, 17 Windows build-infrastructure, 14 native service and 38 frontend/PDF tests passed locally: 120 tests.
- Real Windows subprocess tests verify argv, full output draining, cancellation/timeout and descendant termination. A cold-start timing failure was corrected by waiting for the child's ready signal before measuring cancellation; the timeout/heartbeat checks remain.
- Native tests also cover revision/cancellation-gated atomic export, protected source, closed-project pin refusal, no-op job preservation, cache ownership/cleanup and retention of interrupted sessions.
- The three-engine corpus is explicitly ignored in the local default run. Fresh CI run 41356053455 at source 3e40b997881cbbb3366c78775898f93a4bf33f1e passed all 24 expected outcomes: 12 eligible builds, six readable/ineligible PDFs and six expected failures. Engines: pdfTeX 1.40.25, LuaHBTeX 1.17.0 and XeTeX 0.999995, TeX Live 2023/Debian. Raw outcomes and exact versions: docs/evidence/build-verified-run.json.
- Frontend build passed. Final Windows x64 production build passed with cargo build --offline --locked --release --target x86_64-pc-windows-msvc -p texloom-desktop --features tauri/custom-protocol. Executable SHA-256: c6db652427410bd049ca518202be2eb06f949f47a9b001866349e8fe65e45e89. Native P3 UI interaction is not verified. Windows CI run 41356053441 and movement proof run 41356053490 passed at source 3e40b99; final revision CI is recorded in PR #3.
- See docs/BUILD-SERVICE.md for the exact recipe, input manifest, limits and remaining scope.

## P2 source/history

Branch feature/document-session-core, based on verified P1 commit 1ee59e354c30128851286f5efb2d50524169717a.

| Check | Observed result |
| --- | --- |
| cargo test --offline --locked -p texloom-core | 33 existing + 18 document integration tests passed |
| cargo test --offline --locked -p texloom-desktop --lib | 5 real service tests passed: Serde/UTF-16, BOM/mixed EOL/emoji, simultaneous edits and cursors, guarded candidates, minimal diffs, explicit save/reopen |
| npm test | 31 frontend/PDF tests passed; transaction tests use an explicit IPC double |
| npm run build | Passed; existing large-bundle warning |
| Windows production build | Passed with cargo build --offline --locked --release --target x86_64-pc-windows-msvc -p texloom-desktop --features tauri/custom-protocol |
| Native startup | New executable opened; accessibility tree reported one PDF page, pdflatex, 1.55 seconds |
| Native input | Unverified: foreground ChatGPT capture persisted after selecting/activating Texloom |
| Build-Desktop.ps1 | PowerShell syntax parser passed; script and Windows CI now include native service and document integration tests |

Tested executable SHA-256: 4a2a0fe4ad5904b93c29de5c892db7ec2ab698bfee036160c9309eb7ce1e8348.

The first save test hit a replacement restriction in this session's redirected temporary folder. A separate filesystem probe reproduced it. The same test passed with TEMP/TMP set to writable .tools/test-tmp. Production save behavior was not weakened.

P2 source commit b8b1c49c792559261f14a1af67deb88ccb208241 passed Windows CI run 41348692350 and movement proof run 41348692351. Manual typing/IME/mouse acceptance is deferred by the owner.

## Verified P1 experiment

Commit 1ee59e354c30128851286f5efb2d50524169717a, draft PR https://github.com/nadeemtsf/texloom/pull/1.

- Proof run 41328375751 passed: 24 positive movement cases across pdfLaTeX/LuaLaTeX/XeLaTeX; six expected cross-page/overlap refusals by the fixture oracle.
- TeX Live 2023/Debian; eleven fixtures, ten exact wrap/unwrap round trips and one linked-text planner refusal.
- Zero-offset pixel identity, translation/appearance, unchanged neighbors/non-text/pixels checked. PDF.js and independent PDFium reject the retained unsafe direct-literal PDF.
- Windows CI run 41328375757 succeeded at the same commit, confirmed after reconnecting.
- Full successful report remains permanently in PR #1's description; earlier failed reports remain under docs/evidence.
- The checkout was safely reconciled to that exact verified remote tip before P2.

This proves a bounded fixture experiment. Provenance, arbitrary PDF glyph mapping, extended page coordinates and runtime independent verification remain open.

## Remaining acceptance

P2 needs actual native typing/IME/multi-cursor/context-menu undo and interactive save/reopen checks. Strict TypeScript/generated IPC types, unsupported-encoding viewing and recovery are unfinished. String-buffer performance is unmeasured. See docs/DOCUMENT-SESSION.md.

P1 provenance/parser/coordinate decisions, native P2/P3 acceptance and P4-P8 selection, editing, verification, recovery and packaging gates remain. No complete signature mouse workflow or new release is certified. The downloadable baseline is older.

The first two corpus runs each passed 23/24. They exposed standard fontconfig input and a versioned Lua font cache created during the first pass. Exact standard roots are declared, discovery runs after each pass and discovered files remain hashed and checked before acceptance. Reports are retained in docs/evidence/build-first-run.json and build-second-run.json; the third fresh run passed 24/24. This corpus does not establish arbitrary package support or full hermeticity.
