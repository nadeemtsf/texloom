# Validation record

Updated 1 November 2026. Individual checks, not release certification. Approved scope: docs/APPROVED-IMPLEMENTATION-PLAN.md.

## P2 source/history

Branch feature/document-session-core, based on verified P1 commit 1ee59e354c30128851286f5efb2d50524169717a.

| Check | Observed result |
| --- | --- |
| cargo test --offline --locked -p texloom-core | 33 existing + 18 document integration tests passed |
| cargo test --offline --locked -p texloom-desktop --lib | 5 real service tests passed: Serde/UTF-16, BOM/mixed EOL/emoji, simultaneous edits and cursors, guarded candidates, minimal diffs, explicit save/reopen |
| npm test | 30 frontend/PDF tests passed; transaction tests use an explicit IPC double |
| npm run build | Passed; existing large-bundle warning |
| Windows production build | Passed with cargo build --offline --locked --release --target x86_64-pc-windows-msvc -p texloom-desktop --features tauri/custom-protocol |
| Native startup | New executable opened; accessibility tree reported one PDF page, pdflatex, 1.79 seconds |
| Native input | Unverified: foreground ChatGPT capture persisted after selecting/activating Texloom |
| Build-Desktop.ps1 | PowerShell syntax parser passed; script and Windows CI now include native service and document integration tests |

Tested executable SHA-256: 4ed55207ff0890ec4005a68f09f80fc4967cf5d23588cd0caa320cf15a231a5d.

The first save test hit a replacement restriction in this session's redirected temporary folder. A separate filesystem probe reproduced it. The same test passed with TEMP/TMP set to writable .tools/test-tmp. Production save behavior was not weakened.

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

P1 provenance/parser/coordinate decisions and P3-P8 compilation, selection, editing, verification, recovery and packaging gates remain. No complete signature mouse workflow or new release is certified. The downloadable baseline is older.
