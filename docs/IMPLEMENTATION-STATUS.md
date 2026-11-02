# Implementation status

Updated 2 November 2026. Owner-approved scope: APPROVED-IMPLEMENTATION-PLAN.md.
Primary repository: https://github.com/nadeemtsf/texloom.

| Stage | Status | Evidence / remaining gate |
| --- | --- | --- |
| P0 baseline | Reconciled | Local checkout matched verified remote commit 1ee59e3 before P2 |
| P1 mapping/movement | Movement experiment passed; mapping decisions open | 24 positive cases over three engines, six oracle refusals; provenance/parser/extended coordinates pending |
| P2 source/history | Core and desktop integration implemented; native input acceptance open | 51 core, 5 native service, 31 frontend/PDF tests; production build/sample compile passed; native input blocked by foreground capture |
| P3 compilation | Build service implemented; automated acceptance passed | 120 local tests; all 24 expected three-engine corpus outcomes, Windows packaging and movement proof passed at 1ae8c75; exact CI recorded in PR #3; limited direct recipe, native P3 UI unverified |
| P4 selection | Probe and three-engine corpus passed; native mouse acceptance open | 26 positive selections and seven explicit refusals across three engines at ab9eedc; bounded alternate hints resolve included sources; one measured pdfLaTeX mixed-format raster refusal; see LAYOUT-SELECTION.md |
| P5 text/spacing | Protected literal editor implemented; spacing adapters and native acceptance open | Guarded run edits preserve command scaffolds, colours, macros, comments, Unicode and line endings; LaTeX fallback retains edits; real-engine compile coverage running |
| P6 displacement | Pending | Independent runtime verification and complete native signature workflow |
| P7 hardening | Pending | Recovery, external conflicts, performance and compatibility |
| P8 release | Pending | Exact installer and clean-machine acceptance |
| P9 richer adapters | Pending | Separate fixtures and acceptance for images, tables and typography |

P2 owns source bytes and transactional history independently of unfinished P1 provenance. It does not interpret TeX or map arbitrary PDF glyphs. See DOCUMENT-SESSION.md and ../VALIDATION.md.

The disconnected checkout was safely reconciled before this work. No new release is certified; the previous downloadable ZIP is older.

The owner deferred manual P2 input acceptance and explicitly authorized P3. See BUILD-SERVICE.md; that authorization does not mark the P2 checks passed.

P5 protected editing is tracked in PROTECTED-TEXT.md; unfinished gates remain explicit.
