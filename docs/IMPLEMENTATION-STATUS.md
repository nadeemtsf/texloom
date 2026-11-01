# Implementation status

Owner approval: 1 November 2026. The active plan is APPROVED-IMPLEMENTATION-PLAN.md.
The primary repository is https://github.com/nadeemtsf/texloom. Publication is complete; a release is not.

| Stage | Status | Evidence |
| --- | --- | --- |
| P0 baseline | Source checks and native build reproduced | 33 Rust tests, 16 frontend/PDF tests, frontend build and local Windows x64 production build passed |
| P1 mapping/movement | In progress | 11 fixtures; both PDF.js and PDFium reject the known broken PDF; real-engine experiment running in CI |
| P2 source/history | Pending | Existing protections are retained; Rust-owned session/history not yet implemented |
| P3 compilation | Pending | Production still uses fixed two passes; experiment uses a limited fixture recipe |
| P4 selection | Pending native acceptance | Complete group-selection mouse workflow unverified |
| P5 text/spacing | Pending | No new visual editor claimed |
| P6 displacement | Pending | Corrected adapter still needs full acceptance and runtime validation |
| P7 hardening | Pending | Recovery, conflicts, performance and compatibility |
| P8 release | Pending | Installer, clean-machine verification and release artifact |

Read PROOF-CHECKPOINT.md before resuming local work: the Windows environment disconnected with uncommitted changes. Compare it with the experiment branch before integrating. The remote checkpoint includes review corrections that are not yet in that local tree.

The first real-engine run's raw outcome is in evidence/proof-first-run.json. It exposed overlaps in the initial fixture positions and an oracle assumption that PDF extraction order is stable. The revised oracle checks unique expected-position correspondence and keeps an explicit overlap-refusal case. These changes must be validated; they do not lower movement tolerances or authorize ambiguous matches.
