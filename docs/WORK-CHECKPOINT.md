# Work checkpoint - 2 November 2026

Primary project: nadeemtsf/texloom, Rust/Tauri Windows. Preserve the active checkout and user projects. Approved scope is APPROVED-IMPLEMENTATION-PLAN.md. Latest owner instruction authorizes continuing all phases and finishing earlier gaps; it does not turn unverified gates into passes. Ordinary-PDF expansion is not part of these phases.

Completed checkpoints:

- P5 explicit line-break gap controls: source 3900ac5, draft PR #7. All six literal/spacing engine cases, movement/selection corpus and Windows packaging passed. Exact spacing executable checksum is in PROTECTED-TEXT.md.
- P6 Rust comparison: source 07c803a, draft PR #8. All 24 positive movement cases and six expected refusals passed in workflow 41493768700; build corpus passed 41493768662; Windows packaging passed 41493768634. Maximum observed bounds error 0.01092529296875 bp. Identity raster equality remains exact. First failed threshold experiment is retained alongside the passing raw report.
- P6 native worker experiment: source 10357ff, draft PR #9. Windows, engine proof and build corpus passed (41496866570, 41496866516, 41496866661). Standalone Rust/Windows inspection, pinned DLL loading, bounded outputs, marker attribution and headless integration tests; see PDFIUM-WORKER.md.
- P6 parent receipts: source b47fc0d, draft PR #10. All three CI workflows passed (41500732419, 41500732666, 41500732377). Protocol 2 checks exact request and artifact identities, private paths, cancellation and flat cleanup.
- P6 native transaction gate: current feature/native-layout-gate work. Native source replanning, private zero probes, typed movement/reflow/detach policies and one-shot receipt-bound commit authority are implemented. Verifier resources/notices are packaged and the desktop embeds its worker hash. See NATIVE-LAYOUT-GATE.md. Local validation and artifact receipts are in docs/evidence/native-layout-local.json; exact new CI remains pending.

Open gates remain explicit:

- P1 parser/provenance decisions and extended coordinate support.
- P2 actual typing/IME/multicursor/context-menu undo and interactive save/reopen. Desktop control rejected the test app: "Computer Use was not approved to use texloom-p5." The latest launch retry was also rejected by automatic approval review because the recorded access denial was still in force. Do not retry, rename, indirectly launch or substitute UI automation to bypass this rejection. Native tests remain unverified.
- P3 native UI acceptance; the current recipe is a documented direct-engine recipe without bibliography-tool orchestration.
- P4 actual mouse/group-selection acceptance and broader mandatory mapping fixtures.
- P5 broader spacing/reflow adapters and native editing acceptance.
- P6 real-engine native mark-attribution corpus is newly wired into Windows CI but not yet passed; its test is ignored in the local default run. Complete native signature scenario, fractional-pixel verification and fuller paint/coordinate inspection remain open. Source-plan binding is implemented and tested headlessly, not a substitute for those gates.
- P7 recovery/conflicts/performance/usability, P8 exact installed-artifact acceptance and P9 separate richer adapters remain pending. Do not declare earlier stages complete to advance the table.

The owner explicitly removed the usage cutoff on 2 November and authorized continuing the full phase plan. Do not stop because the usage endpoint remains unavailable. No replacement restart automation was created.

Continue coding/tests within the authorized scope, preserve passing and failing evidence, and only close gates when their actual acceptance checks pass. Native UI access is still required to complete the plan; headless subprocess tests do not replace it.

## Active durability work and environment interruption

P6 native gate is saved at 4b31c5a (PR #11). Its native-engine test first failed during candidate attribution; the failed result is retained in native-marker-first-run.json. Follow-up f9afe4d recognizes only the predeclared overlap refusal and adds fixture context; fresh CI remains pending.

Standalone save/recovery infrastructure is on remote feature/project-durability (draft PR #12). Initial source 3c2a40b hit a Rust closure-result type error, corrected at 13707f8. Component tests and native integration are not yet passed. Application Save still uses its earlier implementation until the component passes and is integrated.

The supplied Windows workspace permission configuration now contains invalid URI file:///workspace. exec_command and apply_patch reject requests before starting. Direct file access within this workspace and GitHub connector/CI still work. No local command checks after this interruption are claimed. Local branch feature/project-durability still has HEAD 4b31c5a with preserved uncommitted component and P6 test follow-up files; remote heads are newer. Reconcile carefully when the command runner works again; do not reset or drop these files.

Next: finish the pending CI checks, retain exact engine results, integrate checked disk state into revision-guarded Save/conflict/recovery flows, then typing recovery/performance and remaining phase code. Native GUI access remains denied; no alternate launch or automation may bypass it. P6/P7 acceptance and P8/P9 are not complete.

## Latest verified implementation checkpoint

Native layout source 2786504e7ee9f02b522d91bb6f7779cfde06d5a8 passed Windows 41527609301, proof 41527609539 and build corpus 41527609160. The Windows mark-handle corpus passed 24 positive cases and three expected overlap refusals. Both failed experiments are retained; inherited page boxes now use the pinned effective-box API. Native input and fractional-pixel displacement remain open.

Save/conflict/copy source 803f2b47adf0b867262a63c46033edb55ac94e7e passed Windows 41529949033, proof 41529948802 and build corpus 41529948752. Explicit interrupted-Save recovery/discard source 426d052e7001883ac4e1bfc3610aeed24579ce07 passed Windows 41532942547, proof 41532942542 and build corpus 41532942522. Exact Windows checks: 156 Rust plus 77 frontend, 233 total. The executable and portable package also built. Evidence: docs/evidence/project-durability-ci.json and native-marker-verified-run.json.

Current remote development branch is feature/project-durability at 426d052 (draft PR #12, based on native-layout-gate PR #11). Native Save now guards loaded disk fingerprints and source revision, stages/flushes recoverable file replacements, preserves external changes, offers a separate editor-version project copy, and provides explicit finish/rollback/discard choices for interrupted saves. Cleanup checks contents as well as filenames and never recurses into unknown data.

Local Git remains at 4b31c5a on feature/project-durability because exec_command/apply_patch reject requests before starting: invalid Windows permission URI file:///workspace. All newer code is preserved in this existing working tree through direct file access and is published remotely. Do not reset/drop it. Reconcile Git metadata carefully when local commands work again; remote history includes a merge bringing 2786504 into the durability branch. The last locally built executable predates the inherited-box and durability changes; do not identify it as the latest CI build.

Computer Use still rejected launching the existing native test app. Do not retry, rename the app or use indirect UI/process automation to bypass that denial. Codex app attachment requests also failed through the app bridge. No native typing/IME/mouse/signature workflow is certified.

Next work remains: P6 ordinary fractional displacement (use a separately rendered expected translation, never interpolated/masked pixels as proof); P7 recovery for unsaved typing independent of explicit Save, retention/performance/usability; earlier parser/provenance/coordinate and native interaction gates; P8 installer/clean-machine acceptance; P9 richer adapters. Ordinary-PDF expansion is still outside this phase scope. Do not mark all previous phases complete or call this a release.
