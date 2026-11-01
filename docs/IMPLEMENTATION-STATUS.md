# Implementation status

Updated 1 November 2026. Owner-approved scope: APPROVED-IMPLEMENTATION-PLAN.md.
Primary repository: https://github.com/nadeemtsf/texloom.

| Stage | Status | Evidence / remaining gate |
| --- | --- | --- |
| P0 baseline | Reconciled | Local checkout matched verified remote commit 1ee59e3 before P2 |
| P1 mapping/movement | Movement experiment passed; mapping decisions open | 24 positive cases over three engines, six oracle refusals; provenance/parser/extended coordinates pending |
| P2 source/history | Core and desktop integration implemented; native input acceptance open | 51 core, 5 native service, 30 frontend/PDF tests; production build/sample compile passed; native input blocked by foreground capture |
| P3 compilation | Pending | Dependency manifests, convergence, cancellation, diagnostics |
| P4 selection | Pending native acceptance | Correct-or-refused mapping and complete mouse group selection |
| P5 text/spacing | Pending | Protected text editing and spacing adapters |
| P6 displacement | Pending | Independent runtime verification and complete native signature workflow |
| P7 hardening | Pending | Recovery, external conflicts, performance and compatibility |
| P8 release | Pending | Exact installer and clean-machine acceptance |

P2 owns source bytes and transactional history independently of unfinished P1 provenance. It does not interpret TeX or map arbitrary PDF glyphs. See DOCUMENT-SESSION.md and ../VALIDATION.md.

The disconnected checkout was safely reconciled before this work. No new release is certified; the previous downloadable ZIP is older.
