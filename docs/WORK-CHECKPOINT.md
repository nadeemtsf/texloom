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
