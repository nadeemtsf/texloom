# Work checkpoint - 2 November 2026

Primary project: nadeemtsf/texloom, Rust/Tauri Windows. Preserve the active checkout and user projects. Approved scope is APPROVED-IMPLEMENTATION-PLAN.md. Latest owner instruction authorizes continuing all phases and finishing earlier gaps; it does not turn unverified gates into passes. Ordinary-PDF expansion is not part of these phases.

Completed checkpoints:

- P5 explicit line-break gap controls: source 3900ac5, draft PR #7. All six literal/spacing engine cases, movement/selection corpus and Windows packaging passed. Exact spacing executable checksum is in PROTECTED-TEXT.md.
- P6 Rust comparison: source 07c803a, draft PR #8. All 24 positive movement cases and six expected refusals passed in workflow 41493768700; build corpus passed 41493768662; Windows packaging passed 41493768634. Maximum observed bounds error 0.01092529296875 bp. Identity raster equality remains exact. First failed threshold experiment is retained alongside the passing raw report.
- P6 native worker experiment: current feature/pdfium-worker work. Standalone Rust/Windows inspection, pinned DLL loading, bounded outputs, marker attribution and headless integration tests; see PDFIUM-WORKER.md. It is not wired to native document commits.

Open gates remain explicit:

- P1 parser/provenance decisions and extended coordinate support.
- P2 actual typing/IME/multicursor/context-menu undo and interactive save/reopen. Desktop control rejected the test app: "Computer Use was not approved to use texloom-p5." Do not bypass this denial by renaming the app or substituting UI automation.
- P3 native UI acceptance; the current recipe is a documented direct-engine recipe without bibliography-tool orchestration.
- P4 actual mouse/group-selection acceptance and broader mandatory mapping fixtures.
- P5 broader spacing/reflow adapters and native editing acceptance.
- P6 parent-side worker lifecycle/receipts, independent source-plan binding and complete native signature scenario, including undo/save/reopen and an independent compile. Fractional-pixel verification and fuller paint/coordinate inspection remain open.
- P7 recovery/conflicts/performance/usability, P8 exact installed-artifact acceptance and P9 separate richer adapters remain pending. Do not declare earlier stages complete to advance the table.

The account usage endpoint repeatedly returns `Codex app tool request failed`. Remaining allowance is unavailable, not zero usage. The owner's 1%-remaining stop threshold cannot currently be monitored reliably. No replacement restart automation was created.

Continue coding/tests within the authorized scope, preserve passing and failing evidence, and only close gates when their actual acceptance checks pass. Native UI access is still required to complete the plan; headless subprocess tests do not replace it.
