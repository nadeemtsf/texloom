# Validation record

Updated 2 November 2026. Individual checks, not release certification. Approved scope: docs/APPROVED-IMPLEMENTATION-PLAN.md.

## P4/P5 working checkpoints

P5 explicit-gap adapter checkpoint: 64 core, 19 native service, 67 frontend and 17 build-infrastructure checks passed locally (167 total). Existing numeric line-break lengths can change without rewriting their unit or surrounding bytes; unsupported expressions stay read-only. All six literal/spacing cases and the existing movement/selection corpus passed at 3900ac5 in workflow 41490191030. Raw outcomes: docs/evidence/spacing-verified-run.json. Frontend and Windows x64 production builds passed; executable SHA-256: df0b9080389585d46685fcd532845b9eaa6533405e9f7edd88ec84cdc5d54a37. Native interaction remains unverified after desktop-control access to texloom-p5 was explicitly denied. See docs/PROTECTED-TEXT.md.

P4 source ab9eedc3b3e58a64a7e89446d4fb8ca61eabc5a3 passed all 33 declared selection outcomes: 26 positive selections and seven explicit refusals. This includes a measured pdfLaTeX mixed-format appearance refusal; strict runtime pixels remain required. Alternate samples resolve included-file mapping under pdfLaTeX/XeLaTeX. Raw report: docs/evidence/selection-verified-run.json. Engine workflow: 41400437651; exact Windows packaging results are tracked in PR #4.

P5 protected literal editing: 58 core, 17 native and 58 frontend/PDF tests passed locally. The unchanged 17 build-infrastructure tests passed at P3 (150 combined tests). Frontend and Windows x64 production builds passed. Local executable SHA-256: b928107f04a29211fe69211f5fbfe4d2b1fd10c4c2b308c44bc469269b62a2e9. Three-engine protected edits passed at 6eb2b5a in workflow 41402245293; the movement and selection suites passed at the same source. Raw edit report: docs/evidence/protected-text-verified-run.json. Exact Windows CI is recorded in PR #5. See docs/PROTECTED-TEXT.md for exact behavior and remaining spacing/native gates.

Native P2 input, complete P4/P5 mouse/edit/reflow acceptance and P6-P9 gates remain open. No new release is certified.

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

P6 gesture boundary checkpoint: six tests execute the real controller with a DOM/event boundary; the frontend suite passes 64 tests (156 combined local checks). Pointermove makes no edit dispatch; release dispatches one; Escape/cancel and stale/read modes dispatch none. Native Windows interaction and independent runtime verification remain open. See docs/GESTURE-VERIFICATION.md.

P6 Rust comparison checkpoint: 64 core, 29 build/comparison, 19 native service and 67 frontend tests passed locally (179 total). Full-raster identity over a real PDF passed; the Rust checker rejects the retained unsafe direct-literal PDF for wrong geometry. Twelve comparison tests include duplicate ink in the vacated target region. The new three-engine differential corpus is pending CI. No packaged runtime inspector, native commit authorization or complete native signature acceptance is certified. See docs/DISPLACEMENT-POLICY.md.

P6 comparison follow-up at 07c803a passed the complete three-engine differential corpus (24 positives/six refusals), build corpus and Windows packaging: workflows 41493768700, 41493768662 and 41493768634. The corrected identity geometry tolerance remains bounded at 0.02 bp and pixel equality remains exact; the maximum measured bounds error was 0.01092529296875 bp. Failed and successful raw reports are retained in docs/evidence/displacement-policy-*.json.

The next native inspection experiment passes 185 local checks: 64 core, 35 build/comparison/worker, 19 native service and 67 frontend. This includes three explicitly invoked normally-ignored headless worker tests. Rust inspection exactly matches the pinned reference on a real compiled PDF; synthetic marker/movement cases pass and ambiguous/nested/duplicate/non-text cases are refused. See docs/PDFIUM-WORKER.md. Worker CI is pending; parent-side native commit binding, native UI acceptance and release gates remain open.
