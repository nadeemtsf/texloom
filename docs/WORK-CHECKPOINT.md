# Work checkpoint - 3 November 2026

Primary project: nadeemtsf/texloom, Rust/Tauri Windows. Approved scope is APPROVED-IMPLEMENTATION-PLAN.md. The owner authorizes continuing all phases and finishing earlier gaps, without the previous usage cutoff. Ordinary-PDF expansion is outside this scope. Do not equate implemented code with passed native acceptance.

## Current checkout and environment

Local commands work again. Exact remote history was reconciled to feature/project-durability at e1caf7ce9649a6e7e9aa6c19d890480ae0ecc5e0 with Git object/ancestry/connectivity checks. All 151 working file byte snapshots were preserved; ignored backups are in .tools/reconcile-git. Do not reset or discard user work.

Current development branch: feature/compiler-preflight, based on startup source c773a9045868d10e843134693bd9f64c582c8cd3. Main remains the older verified baseline. PR #11 (native-layout-gate) and PR #12 (project-durability) are drafts in the primary repository. Command-line Git authentication fails in its Windows credential helper; the connected GitHub service works.

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
- P7 recovery for unsaved typing independent of explicit Save, retention/performance/long-document/usability acceptance. Explicit-Save recovery is implemented; never-saved typing recovery is not.
- P8 exact installed-artifact/clean-machine/signing gates and dependency security/license review. The development PDFium pin is not production approval.
- P9 separate richer LaTeX adapters with their own fixtures. No ordinary-PDF expansion.

Continue authorized coding, preserve failures and successes, and keep blocked native gates explicit. The phase plan and release are not complete.
