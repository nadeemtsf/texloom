# Implementation status

Updated 2 November 2026. Owner-approved scope: APPROVED-IMPLEMENTATION-PLAN.md.
Primary repository: https://github.com/nadeemtsf/texloom.

| Stage | Status | Evidence / remaining gate |
| --- | --- | --- |
| P0 baseline | Reconciled | Local checkout matched verified remote commit 1ee59e3 before P2 |
| P1 mapping/movement | Movement experiment passed; mapping decisions open | 24 positive cases over three engines, six oracle refusals; provenance/parser/extended coordinates pending |
| P2 source/history | Core and desktop integration implemented; native input acceptance open | 51 core, 5 native service, 31 frontend/PDF tests; production build/sample compile passed; native input blocked by foreground capture |
| P3 compilation | Build service implemented; automated acceptance passed | 120 local tests; all 24 expected three-engine corpus outcomes, Windows packaging and movement proof passed at 1ae8c75; exact CI recorded in PR #3; limited direct recipe, native P3 UI unverified |
| P4 selection | Private zero-offset probe implemented; real-engine and native acceptance open | 51 core, 15 native and 52 frontend tests passed; source/receipt guards, actual glyph frame, repeated-instance refusal and whole-page identity checks; see LAYOUT-SELECTION.md |
| P5 text/spacing | Pending | Protected text editing and spacing adapters |
| P6 displacement | Pending | Independent runtime verification and complete native signature workflow |
| P7 hardening | Pending | Recovery, external conflicts, performance and compatibility |
| P8 release | Pending | Exact installer and clean-machine acceptance |

P2 owns source bytes and transactional history independently of unfinished P1 provenance. It does not interpret TeX or map arbitrary PDF glyphs. See DOCUMENT-SESSION.md and ../VALIDATION.md.

The disconnected checkout was safely reconciled before this work. No new release is certified; the previous downloadable ZIP is older.

The owner deferred manual P2 input acceptance and explicitly authorized P3. See BUILD-SERVICE.md; that authorization does not mark the P2 checks passed.
