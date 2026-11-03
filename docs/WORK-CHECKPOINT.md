# Work checkpoint - 3 November 2026

Primary project: nadeemtsf/texloom, Rust/Tauri Windows. Approved scope is APPROVED-IMPLEMENTATION-PLAN.md. The owner authorizes continuing all phases and finishing earlier gaps, without the previous usage cutoff. Ordinary-PDF expansion is outside this scope. Do not equate implemented code with passed native acceptance.

## Current checkout and environment

Local commands work. Exact remote history through source-acknowledgements at eb2108d9dbcd9a4e3e380e444942450482f4b6c4 was reconciled with matching Git trees/commit objects and connectivity checks. Earlier backups and the original untracked work-in-progress/source_benchmark.rs are preserved. Do not reset or discard user work.

Current development branch: feature/portable-package-isolation, based on ffab26183eaa4b5565caaf1ab843986bd299fb89 (draft PR #20). Main remains the older verified baseline; development uses the existing draft stack in the primary repository. Command-line Git authentication fails in its Windows credential helper; the connected GitHub service works. App-control tests remain explicitly deferred.

## Verified implementation checkpoints

- P5 explicit line-break gaps: 3900ac5, PR #7. All six literal/spacing engine cases passed. See PROTECTED-TEXT.md.
- P6 Rust comparison: 07c803a, PR #8; isolated worker: 10357ff, PR #9; checked parent receipts: b47fc0d, PR #10. See DISPLACEMENT-POLICY.md and PDFIUM-WORKER.md.
- P6 native source/layout gate: 2786504e7ee9f02b522d91bb6f7779cfde06d5a8, draft PR #11. Windows 41527609301, proof 41527609539 and build corpus 41527609160 passed. Native mark attribution passed 24 positives and three expected overlap refusals. Both earlier failures are retained; inherited boxes use the pinned effective-box API. See NATIVE-LAYOUT-GATE.md and evidence/native-marker-verified-run.json.
- P7 guarded Save/copy: 803f2b4; interrupted-Save recovery/discard: 426d052e7001883ac4e1bfc3610aeed24579ce07, draft PR #12. Windows 41532942547, proof 41532942542 and build corpus 41532942522 passed: 156 Rust plus 77 frontend checks, 233 total. Executable and portable packaging passed. Documentation-only e1caf7 records these results. See PROJECT-DURABILITY.md and evidence/project-durability-ci.json.

## Resumed local checks and startup correction

On 3 November the reconciled source passed 149 default Rust checks, 77 frontend checks, five explicitly invoked isolated PDFium checks and two explicitly invoked native document-gate checks: 233 combined. A fresh Windows desktop executable built.

Launching the existing .tools/p5-ui-test/Texloom-P5.exe succeeded. This pre-fix executable has SHA-256 374ca5ffe3c26b0a85a4b453486871653f16b33e2180752087af86658d3d6cce. Activation eventually produced the actual Texloom screenshot and accessibility tree, which remained at Starting with an empty source editor. An attempted input did not complete; a later observation was rejected with "Computer Use was not approved to use texloom-p5." No typing, mouse selection, IME, undo or signature workflow passed. Do not rename, indirectly launch or substitute UI automation to bypass the denial. No further native-control retries under the current denial.

Source inspection found a startup failure: app.js assigns a handler and state to cancel-compile, but the HTML lacks that element. Initialization throws before openProject runs. The correction adds the visible, initially disabled Cancel compile control. Initial project-open errors now receive a persistent explanation and leave Open project available.

Four new tests execute the complete production shell against IDs from shipped HTML, retaining real DocumentClient and transaction code with mock native/Monaco/DOM boundaries. They cover source opening, reproducing the missing-element failure, wired cancellation and failed initial open. They are shell integration tests, not native input tests. All 81 frontend and required 64 core checks passed after the correction; the Windows executable rebuilt successfully. Combined unchanged-Rust and updated-frontend coverage is 237 checks. Exact source/artifact hashes: evidence/desktop-startup-local.json. Startup source c773a90 passed Windows 41583936566 and proof 41583936685, including Windows mark inspection: 156 Rust and 81 frontend checks. Raw CI evidence is in evidence/desktop-startup-ci.json; draft PR #13.

The running P5 test app predates the startup correction. The target/release executable contains that correction but has not been interactively tested. The compiler follow-up uses the explicit target/x86_64-pc-windows-msvc/release executable and the portable artifact; exact hashes belong to its separate evidence. Never label an older executable or ZIP as the current build.

## Compiler preflight follow-up

All three installed MiKTeX engines were explicitly exercised through the real service. All 24 local outcomes failed their version preflight with access-denied errors in registry/configuration access; no document was compiled. Evidence: evidence/windows-engine-first-local.json, with raw report/log hashes. Do not call these expected passes or change configuration/permissions to bypass the restriction.

The follow-up returns short structured initialization diagnostics while retaining captured traces, refuses failed/timed-out/truncated version evidence, rechecks inputs and emits no PDF/output receipts for a failed preflight. Tests include a nonzero version check that leaves an untrusted PDF and a zero-exit version check with truncated output. The frontend rejects these results before PDF preparation. Validation passed: 152 default Rust, five explicit worker, two explicit native-gate and 83 frontend checks (242 combined), including the final untrusted-PDF guard. The Windows executable and portable package built; ZIP CRC and all packaged executable/verifier receipts were checked. Exact hashes and logs: evidence/compiler-preflight-local.json. New remote CI is tracked separately. Native input and local installed-engine acceptance remain open.

## Open gates and next work

- P1 parser/provenance decisions and extended coordinates.
- P2 native typing/IME/multicursor/context-menu undo and interactive Save/reopen; strict/generated IPC and unsupported-encoding support.
- P3 native acceptance and broader recipe orchestration. The current direct-engine recipe is not bibliography-complete.
- P4 mouse/group-selection acceptance and broader mandatory mapping fixtures.
- P5 broader spacing/reflow adapters and native editing acceptance. Existing numeric line-break spacing is implemented and fixture-tested.
- P6 fractional displacement, fuller paint/coordinate inspection and the full native signature scenario. Do not interpolate/mask pixels or weaken exact appearance/neighbor checks. Any fractional experiment needs a separately rendered expected translation.
- P7 durable typing recovery is implemented in the current follow-up (UNSAVED-RECOVERY.md); native crash/relaunch, retention/performance/long-document/usability acceptance remain open. Explicit-Save recovery is independently implemented.
- P8 exact installed-artifact/clean-machine/signing gates and dependency security/license review. The development PDFium pin is not production approval.
- P9 separate richer LaTeX adapters with their own fixtures. No ordinary-PDF expansion.

Continue authorized coding, preserve failures and successes, and keep blocked native gates explicit. The phase plan and release are not complete.

## Owner-directed test sequencing and current execution availability

The latest owner instruction on 3 November defers native app-control/typing/IME/mouse checks until implementation through the remaining phases is complete. Continue automated source/build/headless worker/engine checks. Deferred native cases and completion language are in FINAL-NATIVE-ACCEPTANCE.md; do not retry app control while implementing the remaining work.

The desktop executor disconnected during resumption and then reported no usable execution environment. The local checkout was read successfully first: feature/source-benchmark at 2f3d39 with an untracked work-in-progress/source_benchmark.rs draft. That initial draft failed to compile because its service path import resolved child modules incorrectly; no timings exist. Preserve the local changes. Work now continues remotely on feature/source-benchmark through GitHub and Windows CI. Do not reset the disconnected local tree to a newer remote head without inspecting it.

Compiler source 2f3d393a0350cc2c0d626e319486f6b621b12bcc passed Windows 41586639323, visual proof/native inspection 41586639242 and compile corpus 41586639296. The benchmark follow-up adds an explicit opt-in library harness and CI measurement; its own build/results remain pending until observed. See SOURCE-PERFORMANCE.md. Unsaved-typing recovery and other phase implementation items remain open.

## Reconnected implementation and measured typing acknowledgements

The local checkout was reconciled to c83dda1ece80758bbfdaddf89a78021857bae7cf, feature/recovery-checkpoints, by matching exact remote Git trees/commit objects and checking connectivity. The original untracked work-in-progress/source_benchmark.rs bytes remain intact. No reset/discard was used. Current follow-up branch: feature/source-acknowledgements.

Benchmark source b649034 passed Windows CI 41593287808. Its real native API returned roughly 34 MB for a one-character edit in a 32 MiB source project. Recovery checkpoint source c83dda1 passed Windows 41595141062, proof 41595141053 and build corpus 41595141129. This validates the opaque core checkpoint/optional codec, including retained undo/redo and fresh ownership; it does not implement durable typing recovery by itself.

Typing IPC now returns a tagged, revision-checked changed-file reply through the same native authority as the full-snapshot reference. The frontend retains untouched files, refuses malformed/mismatched acknowledgements, and avoids full-project structured cloning/JSON comparisons in acknowledgement/status handling. Opening, visual commits and history retain complete snapshots. Same-host release measurements reduced the 32 MiB reply from 34,452,005 to 269,362 bytes and p95 native edit plus encoding from 79.17 to 19.76 ms. No editor-paint target is inferred. SOURCE-PERFORMANCE.md and evidence/source-ack-local.json retain exact measurements and limits.

Local checks passed: 69 core default, 30 native default and 100 frontend checks, plus frontend and Windows x64 release builds. The two normally ignored native-renderer checks were not repeated for this response-only change; existing CI evidence is separate. The freshly built executable was not launched and no new installer/portable release is claimed. All native input/IME/mouse checks remain owner-deferred. Next implementation work remains the bounded, durable unsaved-typing journal, guarded recovery adoption/status/UI, then other explicit phase gaps. P7 and the overall release are not complete.

## Durable unsaved typing and recovery adoption

The current follow-up implements a framed, hash-chained and flushed journal for canonical source/profile/history/selection/approved-visual changes. It preserves original saved disk fingerprints, has bounded checkpoint rotation, active-instance locks, expected-content-only flat cleanup, partial-tail warnings and safe recovery after storage failure. Source acceptance and recovery protection are distinct; a successful explicit Save is not reported failed because its separate recovery checkpoint failed.

Recover work lists verified closed drafts across projects, opens a fresh native session without old visual authority, retains the earlier crash draft, and keeps restored work explicitly unsaved. Missing-folder recovery opens source/history and permits a new source-only copy without recreating the original path. Close/discard is revision guarded; uncertain cleanup retains data and returns a warning. The new controller and native APIs use blocking worker tasks rather than disk I/O in synchronous WebView commands. Details and remaining limits: UNSAVED-RECOVERY.md.

Local checks passed: 70 core default (71 with the optional codec), 71 build-infrastructure, 36 native default, five explicitly invoked headless PDFium and two explicitly invoked headless native approval checks, and 113 frontend checks. This is 297 across default plus explicit suites, excluding repeated codec cases. Frontend, verifier and Windows x64 release builds passed. Exact source/log/resource/executable receipts and earlier integration failures are in evidence/typing-recovery-local.json. The executable was not launched; no new portable ZIP/installer was produced.

The actual journal-enabled release benchmark passed ten cases with protection assertions, exact undo/redo and unchanged project originals. At 32 MiB the production changed-file path measured 56.39 ms p95 through encoding with a 269,459-byte reply. This includes durable source append/flush, excludes cursor-event flushes/compaction/transport/paint, and does not establish the 50 ms paint target. Schema-2 destruction timing was clarified without changing historical raw measurements. Canonical full-project string copies remain a measured performance concern and are the next P7 optimization to investigate.

PR #17 source eb2108d9dbcd9a4e3e380e444942450482f4b6c4 passed Windows 41599564604 and proof 41599564562. The compile-corpus workflow was not returned for that response-only commit. The new recovery follow-up requires its own remote checks. Native UI/crash/IME/mouse/signature and local installed-engine gates remain unverified. Continue remaining implementation without retrying app control; do not declare P7, P8/P9 or the release complete.

## Owner-requested five-point visual bridge audit

Recovery source d67c5af37717091e64a973964c3322b5b55e7ad2 passed Windows 41607398490, proof 41607398601 and compile corpus 41607398503. A local source-performance branch was created but no optimization code was written before the owner supplied five architecture questions. Work switched to feature/visual-bridge-audit to inspect and correct those paths first. The original WIP draft remains unchanged.

Actual implementation: JavaScript/Monaco, CSS-only pointer movement and release-triggered transactions; real SyncTeX edit/view CLI; -synctex=1 already present; pending candidates preserve canonical source/PDF until validated commit; constrained balanced scanning plus guarded byte patches, not a full TeX AST. Exact excerpts and all five answers are in VISUAL-BRIDGE-AUDIT.md.

Two concrete gaps were corrected: forward mapping's guessed/inflated/clipped geometry and ignored process failure status; legacy drag/resize outlines remaining at unaccepted preview bounds after a rejected transaction. Queries now require complete/unambiguous valid output, successful nontruncated execution, a sealed trace and matching current source identity. Artifact reads use their own lifetime so a valid baseline remains queryable after a private probe supersedes its compiler lease. Legacy preview cleanup now restores accepted bounds on refusal/cancellation/capture loss.

Local audit checks passed: 70 core, 41 native default, 118 frontend, frontend build and Windows x64 release build. The two normally ignored native renderer tests were not repeated for this bridge-only follow-up. Receipts: evidence/visual-bridge-audit-local.json. No executable launch or new portable/installer was performed. Remote audit CI remains separate until observed. Strict/generated IPC typing, broader syntax/provenance, fractional movement, source-copy/selection performance and all deferred native/release gates remain open. Resume earlier-phase and remaining implementation after answering the audit; no React rewrite or AST-wide source regeneration was authorized by those questions.

## Single-file typing plans and shared history

Audit source 30cf0f9 passed Windows 41669476879 and proof 41669476764. The compile-corpus workflow was not returned for that bridge-only commit. Current source-edit-plans removes full-project source copies during typing without changing source authority or the visual transaction gate. Opaque single-existing-file plans retain owner/base/expected-byte/UTF-8/size checks and share the exact history recorder with generic candidates. Undo/redo now consumes one private project draft across coalesced steps. Source-only WAL patches revalidate through the same fast planner; profile/create/remove/visual records retain generic preparation.

Eight new core tests include allocation identity, atomic history failure and an 800-step Unicode differential comparison with the generic planner and optional checkpoint codec. Local checks passed: 78 core default (79 with optional codec), 71 build, 41 native default, five explicitly invoked headless worker, two explicitly invoked headless native approval and 118 frontend: 315 without repeated codec cases. Frontend, verifier and Windows x64 release builds passed. Exact working-source/log/resource/executable receipts and raw benchmark: evidence/source-plans-local.json. The executable was not launched and no new portable ZIP/installer was produced.

The fifteen-case schema-4 release benchmark isolates old project-snapshot planning versus single-file planning with identical changed-file replies and durable source recovery in the same run. At 32 MiB, p95 native edit through encoding measured 23.74 ms versus 16.09 ms, with the same 269,459-byte reply. Every acknowledgement was protected; coalesced undo/redo and original disk fingerprints were exact. These synthetic backend results exclude cursor flushes, checkpoint rotation/replay, IPC/paint and native typing. SOURCE-PERFORMANCE.md preserves all limitations and earlier raw measurements.

Next work: finish remaining earlier-phase typing/IPC and retention/long-document items, repair portable packaging so old staging contents cannot enter a new archive, then continue gated adapter/release implementation. Strict/generated IPC, unsupported encodings, bibliography recipes, parser/provenance, fractional movement, richer adapters and native/installed release gates remain open. Preserve the original WIP draft. Native app-control checks stay deferred; no phase or release is declared complete.

## Portable package isolation and receipts

Source-plan implementation ffab26183eaa4b5565caaf1ab843986bd299fb89 is published as draft PR #20. Its compile corpus 41674162284 and proof 41674162254 passed; Windows 41674162228 was still running at last observation and is not yet marked passed. Local history was reconciled exactly with the published tree/commit, preserving the original WIP bytes.

The packager no longer reuses the fixed staging directory or recursively zips its contents. Every run uses fresh staging, fixed input paths, source/copy checks, per-file receipts, exclusive CreateNew output and exact ZIP reinspection. Previous output is preserved under its original name; new output gets a unique name when needed. Build-Desktop and CI use the returned archive path. There are no recursive move/delete operations.

Local npm test passed 123 cases with zero failures and one explicit skip: this host cannot create the junction fixture (Win32 AccessDenied), so link-refusal execution remains unverified here. Five new filesystem cases passed, and a real thirteen-file package passed independent Python ZIP CRC/exact-entry/hash checks. Windows PowerShell 5 -File execution was denied by AuthorizationManager; PowerShell 7, matching CI, passed without changing policy. Earlier failures remain in local logs, not relabeled as passes. Exact final artifact/source/log receipts are in evidence/portable-packaging-local.json.

This builds a portable development package of the already-tested single-file source-plan executable; it does not launch it or close native input, installer/clean-machine, signing or PDFium dependency review gates. Source/history/renderer checks from the unchanged ffab261 implementation remain the baseline. Continue the open earlier-phase strict/generated IPC, unsupported-encoding, parser/provenance, bibliography, fractional movement, longer-document/performance and richer-adapter work; all native GUI cases remain deferred through implementation. No all-phase or release completion is claimed.
