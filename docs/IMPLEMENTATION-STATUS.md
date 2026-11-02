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
| P5 text/spacing | Protected literal editor and explicit line-gap adapter implemented; native acceptance open | Six literal/spacing cases and existing corpus passed in three-engine CI at 3900ac5; numeric-only existing gaps preserve units/source; broader spacing adapters open; see PROTECTED-TEXT.md |
| P6 displacement | Rust comparison corpus passed; standalone native worker tested; application gate open | 24 positive/six refused engine comparisons; 13 comparison tests; five headless worker checks including independent marker movement; runtime integration and native signature workflow pending; see DISPLACEMENT-POLICY.md and PDFIUM-WORKER.md |
| P7 hardening | Pending | Recovery, external conflicts, performance and compatibility |
| P8 release | Pending | Exact installer and clean-machine acceptance |
| P9 richer adapters | Pending | Separate fixtures and acceptance for images, tables and typography |

P2 owns source bytes and transactional history independently of unfinished P1 provenance. It does not interpret TeX or map arbitrary PDF glyphs. See DOCUMENT-SESSION.md and ../VALIDATION.md.

The disconnected checkout was safely reconciled before this work. No new release is certified; the previous downloadable ZIP is older.

The owner deferred manual P2 input acceptance and explicitly authorized P3. See BUILD-SERVICE.md; that authorization does not mark the P2 checks passed.

P5 protected editing is tracked in PROTECTED-TEXT.md; unfinished gates remain explicit.

Latest local spacing checkpoint: 64 core, 19 native service and 67 frontend tests passed. Desktop control explicitly denied the test executable, so native typing/IME/mouse checks cannot be signed off. The account usage endpoint also fails; unavailable allowance is not treated as unused allowance.

The native inspection checkpoint passes 185 local checks including three explicitly executed worker tests that are ignored by default. Read WORK-CHECKPOINT.md before resuming. No phase is certified by headless tests alone.
