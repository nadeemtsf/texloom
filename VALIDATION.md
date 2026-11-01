# Validation record

Updated 1 November 2026. Individual checks, not release certification. Approved scope: docs/APPROVED-IMPLEMENTATION-PLAN.md.

## P3 compile-service checkpoint

The owner explicitly authorized P3 before manual P2 input acceptance. Native P2 interaction remains open.

- 51 core, 17 Windows build-infrastructure, 10 native service and 37 frontend/PDF tests passed locally: 115 tests.
- Real Windows subprocess tests verify argv, full output draining, cancellation/timeout and descendant termination. A cold-start timing failure was corrected by waiting for the child's ready signal before measuring cancellation; the timeout/heartbeat checks remain.
- Native tests also cover revision/cancellation-gated atomic export, protected source and cache ownership/cleanup.
- The three-engine corpus is explicitly ignored in the local default run. Its 24 scenarios must pass in CI; the first CI run passed 23/24 scenarios. Cold-start LuaLaTeX refused a standard fontconfig path outside the declared roots. That exact root is now declared and still hashed; the rerun must pass. Raw outcome: docs/evidence/build-first-run.json.
- Frontend build passed; the Windows x64 production build passed. Executable SHA-256: aad46552bd2219df00b3aba906deadca46350d2e01368664d7e5041f3cf4e3a4. Native P3 UI interaction is not verified.
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

P1 provenance/parser/coordinate decisions and P3-P8 compilation, selection, editing, verification, recovery and packaging gates remain. No complete signature mouse workflow or new release is certified. The downloadable baseline is older.

The second corpus run also passed 23/24. It exposed a versioned Lua font cache created during the first pass after root discovery. Root discovery now runs after each pass, and discovered files remain hashed and checked before acceptance. Raw report: docs/evidence/build-second-run.json. The next fresh-run corpus remains the gate.
