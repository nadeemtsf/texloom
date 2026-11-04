# Current validation status - 5 November 2026

Local product source `5e59e77282fd5e4b4f70440437b8f841ccea1364` adds read-only setup guidance and build-opt-in local timing diagnostics. Required checks passed: 252 frontend (zero skips), 88 default core (70 unit and 18 integration), an ordinary Windows release/NSIS build with tracing disabled, and a separate tracing-enabled Windows release build without an installer. Five existing Rust warnings remain. These checks do not measure native latency, tracing overhead or clean-account setup. See docs/SETUP-HELP.md and docs/PERFORMANCE-DIAGNOSTICS.md for exact scope and boundaries.

No new executable has been opened or installed. The owner remains away and all tests requiring approval or Texloom to open are deferred. Current local work is unpushed; no new remote CI or final research evaluation ran. Earlier results below retain their own artifact identities.

## Preceding checkpoint - 4 November 2026

The current local product source is `05761985a7deef312426c268b953d4d01eccdaca`, followed by local documentation/research work. Active canvas drags and marquees now cancel on window blur. Required checks passed: 239 frontend (zero skips), 88 default core, and a full Windows release/NSIS build. The new binary has not been launched or installed; the owner explicitly deferred all approval-dependent and Texloom UI tests while away. No remote CI was run for this unpushed local commit.

The preceding c26a238 modal-feedback correction passed its own 237 frontend/88 core/build checks and all three workflows: Windows 41804977453, compiler corpus 41804977541 and visual proof 41804977450. Earlier optional-codec, build54, native50, explicit worker/approval and three-engine receipts remain separate evidence; they are not added to an accuracy denominator or represented as newly rerun local suites.

Native signature workflows passed under all three engines on the recorded preceding candidates. The exact approved `916356f...` installer was installed and tested on the existing developer account: verified nudge, exact Save/Undo/reopen, Letter/CV templates, resources and shutdown during real compilation passed. Approved interrupted-Save rollback/discard completed with exact original and recovered-copy hashes. The owner subsequently approved the `ae856732...` r2 setup: its existing-account reinstall, matching resources, verified nudge, exact Save/Undo, clean close and relaunch passed. Portable r2 EXE `20520866...` and installed r2 EXE `a3c4225b...` differ only in Tauri's three-byte bundle marker. These are not clean-machine installation results. Uninstaller launch was blocked by the desktop tool and no uninstall ran.

The project remains unfinished at held-gesture cancellation, Windows DPI/multiple-monitor, clean-machine dependency setup/uninstall, reference end-to-end performance and human usability gates. See docs/FINAL-NATIVE-ACCEPTANCE.md and the dated evidence receipts. The local `research/` directory contains an initial technical report, an audit and an evaluation design; final evaluation is not frozen or run. Text below records historical checkpoints; older deferral/unlaunched statements do not describe the current candidate.

A separate headless development pilot ran 30 post-compilation controls in five engine/language conditions. Five intended controls matched an independently authored reference, 20 purposeful faults differed, and five deliberate compile failures remained failures. Predicate replay shows the limits of compiler-only and text/geometry checks on those artifacts. It bypasses the app selection/source-plan gates and supplies no production acceptance rate. Nine PDF/replay and five source-contract diagnostics pass; a separate CLI smoke check preserved BOM/CRLF multilingual bytes and detected a supporting-file change. See research/ORACLE-PILOT.md, research/SOURCE-CONTRACT.md and their receipts. Bounded runtime pin/provenance review is in docs/DEPENDENCY-REVIEW.md; it is not security certification.

---

# Validation record

Updated 3 November 2026. Individual checks, not release certification. Approved scope: docs/APPROVED-IMPLEMENTATION-PLAN.md.

## Compiler/mapping protocol and PDF renderer continuation

Local validation passed 78 core, 71 build-infrastructure, 42 native with typegen, two explicitly invoked headless native approvals and 188 frontend/filesystem checks: 381 executed checks, excluding repeated earlier runs. The frontend run has zero failures and zero skips, including the junction fixture on this host. Thirteen compile-time negative contracts pass. The default build suite's five optional standalone worker checks and two engine corpora were not rerun or counted. Native approval used the actual pinned isolated worker. PR #22's three remote workflows were observed successful before this continuation; this new source requires its own CI.

Compiler and mapping replies now cross generated Rust contracts and strict runtime checks before rendering, source planning or navigation. Candidate compilation is bound to its prepared source/revision; malformed replies and forged unit acknowledgements cannot publish a PDF or authorize a visual commit. New shell cases preserve accepted source/history/PDF, reject wrong-source selection before any probe, and keep later source typing available. See docs/BUILD-PROTOCOL.md.

PDF.js is pinned to 6.2.108. The first updated run exposed the removed proxy destroy method in two real-PDF tests; the application and proof tools now own loading-task disposal through the shared loader. Tests include real PDF painting/text extraction and failure/idempotent cleanup. The final npm audit reports zero known vulnerabilities. This is not a complete security or license review; the PDFium pin remains a release-review item. See docs/PDF-RENDERER.md.

Frontend, verifier and Windows x64 release builds passed. A final frontend/native rebuild followed the lifecycle correction; an intermediate build is not delivered. Portable validation checks ZIP CRC, unique exact entries, per-file hashes, matching native resources and the AMD64 executable header. Exact working-source/log/artifact/package receipts are in docs/evidence/build-protocol-local.json after those checks. The executable/package were not launched. Native mouse/typing/IME/crash/signature, installed-artifact, remaining IPC migration, wider adapters and all-phase/release gates remain open.

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

Native worker source 10357ff subsequently passed Windows CI 41496866570, engine proof 41496866516 and build corpus 41496866661. The parent receipt checkpoint passes 194 local checks: 64 core, 44 build/comparison/worker, 19 native service and 67 frontend. Four normally-ignored worker checks were explicitly run. Protocol 2 binds exact request inputs and checks results/raster receipts, paths, cancellation and bounded cleanup; parent CI is pending. Native document authorization and application interaction gates remain open.

Parent source b47fc0d passed Windows 41500732419, engine proof 41500732666 and build corpus 41500732377. The native transaction checkpoint now passes 203 local checks (64 core, 44 build/comparison/worker, 26 native service including two explicitly invoked worker checks, 69 frontend). Rust owns typed source plans, zero-offset probes, operation-specific inspection and one-shot commit authority. Probe-only candidates cannot commit. Tests caught and corrected a long Windows working-directory failure; a Windows PowerShell build-hook authorization failure was resolved by using the existing Node build runtime without changing policy settings. Frontend and Windows production builds passed. Exact executable, resource and package receipts are recorded in docs/evidence/native-layout-local.json. The newly added real-engine Windows mark-attribution job is pending CI. Native UI launch was rejected by automatic approval review because of the existing desktop-access denial; typing/IME/mouse and complete signature acceptance remain unverified. See docs/NATIVE-LAYOUT-GATE.md. No new release is certified.

P6 inherited-page-box source 2786504 passed all 27 Windows real-engine outcomes: 24 positives and three overlap refusals, workflow 41527609539. Windows packaging 41527609301 and build corpus 41527609160 also passed. Both earlier failures and the successful marker report are retained.

P7 explicit save/recovery checkpoint 426d052 passed Windows 41532942547, proof 41532942542 and build corpus 41532942522. Windows ran 156 Rust and 77 frontend checks (233 combined), including actual worker/native approval, external-change refusal, save rollback/recovery/copy, revision fences and recovery controller boundaries. Executable and portable packaging passed. Exact report: docs/evidence/project-durability-ci.json. Local command execution is blocked by invalid Windows permission URI file:///workspace; native launch is rejected by automatic approval review under the prior access denial. No native input/signature acceptance, fractional displacement, unsaved typing recovery, P8/P9 completion or release is claimed.

## 3 November local execution and desktop startup correction

Local commands work again and exact published Git history was reconciled without changing working file bytes. The existing 149 default Rust, five explicitly invoked isolated-worker, two explicitly invoked native-gate and 77 frontend checks passed locally (233 total). A fresh executable launched and could be observed; input access was subsequently rejected with "Computer Use was not approved to use texloom-p5." No native input gate passed.

The app stayed at Starting because app.js bound cancel-compile while the element was absent from index.html. The correction supplies that initially disabled control and a persistent initial-open error. Four new tests execute the complete shell with IDs from shipped HTML: source opening, the missing-element regression, actual cancellation wiring and initial-open failure. All 81 frontend and 64 required core checks passed, and the corrected Windows executable built successfully. Combined existing Rust and current frontend coverage is 237 checks. Exact source and executable hashes: docs/evidence/desktop-startup-local.json. Shell tests use mock native/Monaco/DOM boundaries and do not establish mouse, keyboard, IME, installed-artifact or signature acceptance. Remote CI and native retesting remain separate gates.

Startup source c773a9045868d10e843134693bd9f64c582c8cd3 subsequently passed Windows CI 41583936566 and proof 41583936685, including Windows native inspection job 125746092381. CI ran 156 Rust and 81 frontend checks. Raw native mark-corpus results are retained in docs/evidence/desktop-startup-ci.json. This does not close native input gates.

## Local installed-engine preflight and failure diagnostics

The explicit Windows/MiKTeX run at c773a90 failed all 24 cases before document compilation: every engine returned access-denied errors during its version check. Evidence: docs/evidence/windows-engine-first-local.json, including exact raw-report/log hashes. It remains a blocked/failed engine acceptance result. The application and test harness did not change configuration or permissions.

The follow-up returns bounded structured initialization diagnostics with retained captured logs, refuses truncated version evidence even after a successful exit, and issues no output receipts or PDF for a failed preflight. Real subprocess tests include an untrusted PDF left by a failed version check. Existing source/publication barriers remain required. Final local checks/artifact receipts are recorded in docs/evidence/compiler-preflight-local.json when available; no native UI, local installed-engine, all-phase or release gate is inferred from these tests.

Final local follow-up validation passed 152 default Rust, five explicit isolated-worker, two explicit native-gate and 83 frontend checks (242 combined). The Windows executable and portable package built, ZIP CRC passed, and packaged executable/worker/DLL/notice hashes matched their receipts. This includes the final guard against sealing an untrusted PDF emitted by a failed version check. Exact source, artifact and log hashes: docs/evidence/compiler-preflight-local.json. Remote CI is recorded separately; local MiKTeX and native input gates remain open.

## Source checkpoint and acknowledgement follow-ups

Recovery checkpoint source c83dda1 passed Windows 41595141062, proof 41595141053 and build corpus 41595141129. The optional codec preserves/restores canonical sources, retained history/redo, profile and selections after validating the entire timeline. It contains no compiler/visual authority. Durable typing journaling and recovery adoption remain separate implementation work.

The changed-file typing reply follow-up passed 69 core, 30 native default and 100 frontend checks locally (199 across those suites), frontend build and Windows x64 release build. Two normally ignored native renderer checks were not repeated locally for this response-only change. A schema-2 release benchmark measured both real native response paths on the same Windows host and required exact undo/redo and unchanged original disk bytes. For 32 MiB the reply fell from 34,452,005 to 269,362 bytes and p95 native edit plus encoding from 79.17 to 19.76 ms. Raw reports/source/executable/log receipts are in docs/evidence/source-ack-local.json. No WebView/editor-paint, active-journal, native interaction, installed-artifact or release acceptance is inferred. Owner-deferred native tests remain in docs/FINAL-NATIVE-ACCEPTANCE.md.

## Durable typing recovery follow-up

Local validation passed 70 core default (71 with the optional codec), 71 build-infrastructure, 36 native default, five explicit headless PDFium, two explicit headless native approval and 113 frontend checks: 297 excluding repeated codec cases. Frontend/verifier/Windows x64 release builds passed. The native service tests reconstruct a new backend, recover exact source/history, reject old visual tokens, retain external Save conflicts, tolerate recovery I/O failure after accepted typing/Save, guard close/discard and salvage source when its folder is missing. Frame tests cover every final-record truncation, corruption/reordering, locking, rotation and content-checked cleanup. Controller tests remain explicit native/DOM doubles.

The journal-enabled schema-3 release benchmark passed ten cases with protected-revision assertions and exact undo/redo/original-disk checks. At 32 MiB the changed-file path measured 56.39 ms p95 through encoding and a 269,459-byte reply. Single-run results exclude per-edit cursor flushes, compaction, transport and paint; they do not pass the 50 ms paint target. Schema 3 captures the endpoint before native reply destruction; the prior schema-2 combined interval included that destruction despite its generic exclusion note. Historical raw evidence is retained.

Receipts and limitations: docs/evidence/typing-recovery-local.json and docs/UNSAVED-RECOVERY.md. The executable was built but not launched. No new portable ZIP/installer, native window-crash/input/signature, clean-machine, local MiKTeX or all-phase/release acceptance is claimed. Those gates remain explicit, with native interaction owner-deferred.

## Visual bridge audit

The owner-requested audit found and corrected invented forward-mapping geometry/ignored SyncTeX failures and legacy rejected-drag preview cleanup. Mapping now checks complete/unambiguous values, process failure/truncation/cancellation, sealed trace integrity and current source identity before/after a bounded CLI query. An accepted artifact remains readable after its old compiler lease is superseded. Tests cover actual nonzero/truncated child processes, metadata/trace guards and legacy prepare/compile/cancellation rollback.

Local checks passed: 70 core, 41 native default and 118 frontend (229 across these suites), frontend build and Windows x64 release build. Normally ignored renderer checks were not repeated for this bridge-only change; unchanged prior evidence is separate. Receipts: docs/evidence/visual-bridge-audit-local.json. Answers/exact excerpts and limits: docs/VISUAL-BRIDGE-AUDIT.md. Recovery source d67c5af separately passed Windows 41607398490, proof 41607398601 and build corpus 41607398503. The audit has its own remote checks; no native app-control/input, new portable/installer or release gate is inferred.

## Single-file source-plan checkpoint

Local validation passed 78 core default (79 optional recovery-codec), 71 build-infrastructure, 41 native default, five explicitly invoked headless worker, two explicitly invoked headless native-approval and 118 frontend checks: 315 excluding repeated codec cases. Frontend, verifier and Windows x64 release builds passed. The 800-step Unicode differential test checks exact source/history/checkpoint equivalence against full preparation. Mixed source/visual/settings history, stale/foreign plans, source limits, no-op redo and failure-atomic coalesced history are covered. The fifteen-case active-journal benchmark requires protected acknowledgements, exact undo/redo and unchanged originals; raw timings and source/log/executable receipts are in docs/evidence/source-plans-local.json. No native GUI, local installed-engine, installer, all-phase or release acceptance is inferred. The freshly built executable was not launched.

## Portable package isolation

The portable packager now uses fresh staging and an explicit per-file list. It preserves existing archives, checks source/copy receipts, refuses links/unknown staging files, writes with CreateNew, and reopens the completed ZIP to check entry identity, lengths and SHA-256. CI uploads the returned archive path. Five new filesystem cases passed locally: previous-folder isolation/exact UTF-8 bytes, hidden private-file refusal, changed-content refusal, existing-archive preservation, and invalid/duplicate/missing receipts. The sixth case could not create its junction because this host returns Win32 AccessDenied and is explicitly skipped, not passed. The complete frontend run reports 123 passed, zero failed, one skipped. Earlier 78 core, 71 build, 41 native default and seven explicit headless renderer/approval checks remain the unchanged-source baseline; they were not repeated for packaging scripts.

A real development portable archive was produced and independently checked with Python ZIP CRC/exact-entry/per-file hash validation; exact final receipts are in docs/evidence/portable-packaging-local.json. The packaged executable is the single-file source-plan build, has not been launched here, and does not close native UI, installed-artifact, signing, dependency review or release gates. Windows PowerShell 5 file execution was denied by this host's authorization manager; PowerShell 7 is the tested packaging runtime. No policy was changed.

## Strict document IPC checkpoint

The ordered source/history client and reply validator now compile from strict TypeScript, with request/reply DTOs and Serde fixtures generated from actual Rust handler records. Full, changed-file, metadata and prepared replies have distinct runtime guards before replica acceptance. Malformed successful commits/Save replies stop dependent mutations; ordinary backend conflicts/refusals remain nonfatal. Project-open validation precedes replacing the current editor/PDF. Generated runtime freshness and invalid TypeScript checks fail without rewriting files.

Local checks passed: 78 core, 42 native with optional typegen, two explicitly invoked headless native approval and 149 frontend/filesystem, 271 total. Six compile-time negative contracts also passed. One local junction case remains skipped; the same packaging case passed in predecessor Windows CI 41677327977 (124 frontend/filesystem cases, zero skips). Build/worker suites unchanged from earlier verified source are not counted again here. Frontend/verifier/Windows x64 release builds and independent portable ZIP entry/CRC/hash checks passed. Receipts: docs/evidence/typed-protocol-local.json. The executable was not launched. Full frontend/IPC migration, unsupported encoding, native UI, installed-artifact and other phase/release gates remain open.


## Remaining desktop features and native acceptance — 4 November 2026

The current working-tree candidate adds guarded project ZIP import/export, recents, explicit encoding conversion into a new copy, persistent root/engine profiles, editable templates, declared BibTeX/Biber recipes, source layout/image controls, PDF search and bounded long-document painting. The independent renderer is now pinned to PDFium 156.0.8076.0 with protocol 3 and full pinned notices. Strict workspace IPC joins the existing document/build contracts.

Local automated checks passed: 89 core (71 unit plus 18 integration), 54 build-library, 50 native default, 231 frontend, two explicitly invoked headless native approvals and five explicitly invoked standalone worker checks. Negative TypeScript contracts pass. Real suites pass all 24 expected compiler outcomes, 27 adapters, 19 template/class cases, both bibliography recipes, nine fractional moves, 24 movement positives plus six expected refusals, 28 selections plus six expected refusals, six protected text/spacing cases and the actual three-engine Windows mark-identity corpus. Earlier failed reports are preserved. The LuaTeX minipage regression now compiles and rejects wrong bounded candidates before the selected name is accepted; complete glyph/layout/pixel checks remain strict.

Actual Windows interaction has begun and is recorded separately in FINAL-NATIVE-ACCEPTANCE.md: the signature, exact Undo/Redo/Save/reopen, PDF word copy, installed English/Arabic/Hebrew keys, multi-cursor editing, abrupt-crash source/history restore, external Save conflict, and saved project-copy paths passed on the specified candidate. A saved project independently rebuilt under all three engines outside the app cache preserves the 12 bp group displacement and unchanged neighbors; its pdfLaTeX full-page raster equals the native exported PDF. The rebuilt candidate also passes clean active-draft catalog and missing-folder source salvage. This is not a complete native release matrix.

The seventh unsigned executable SHA-256 is 4b188a1f0606a6d7b0af0e32aa1c2465758e29d83856bec76ce787a14faa464c; the NSIS SHA-256 is 63a5be14d6650d2c83dc63304514a2b12ed0051c3f1984545b88c3dd8d8776f7. Native path-fix repetition is tracked separately. Exact 15-entry portable receipts, test report hashes, advisory results and limits are recorded in docs/evidence/remaining-features-local.json. Both package forms include the locked dependency inventory, MPL source-availability URLs and notices. Cargo audit found zero vulnerabilities; glib/proc-macro-error warnings are not in the Windows dependency graph. npm audit found zero known vulnerabilities.

The final 15-case release-mode source benchmark retains exact undo/redo and untouched originals. At 32 MiB the production changed-file path measured 19.48 ms p95 through JSON encoding with a 269,459-byte reply. This excludes WebView transport/paint and cursor journal flushes and was measured under uncontrolled background load. Clean-machine installation/uninstall, human usability and the remaining interactive/coordinate/performance cases are still explicit release gates. English, Arabic and Hebrew are the current input scope; CJK IME acceptance was deferred by the owner, and no Windows languages were added. No .text file was compiled.
