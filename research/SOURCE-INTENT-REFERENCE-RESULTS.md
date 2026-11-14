# Source intent and rendered identity: development findings

12 November 2026. Technical report development evidence, not final evaluation or app acceptance. Exact source `897c121`, predeclared/amended plan `SOURCE-INTENT-REFERENCE-PLAN.md`, receipt `docs/evidence/source-intent-reference-2026-11-12.json`.

| Corrective candidate family | Rows | Exact source matches | Full rendered-reference matches |
|---|---:|---:|---:|
| Intended Replace/Delete | 8 | 8 | 8 |
| Delete first instead of second equal paragraph | 4 | 0 | 4 |
| Replace first instead of second equal paragraph | 4 | 0 | 0 |
| Change unrelated tail | 8 | 0 | 0 |

Conditions: English/pdfLaTeX, English/LuaLaTeX, Arabic/XeLaTeX and Hebrew/XeLaTeX on one host. This is not the full language/engine cross product. Source comparison uses independently assembled expected segments, including exact BOM/CRLF and escaping; render comparison uses Poppler page contracts, word bounds/multiplicities and exact 144-dpi RGB reference pages. Eight intended candidates satisfy both. Two deliberately undefined commands retain compiler failures. Original fixtures are unchanged and no Texloom support dependency is introduced.

Deleting either adjacent equal paragraph leaves the same visible document. Exact full-page render agreement therefore cannot establish the intended source occurrence in these examples. Source provenance/minimal patch checking and render checking answer separate questions. Offline compiler-only replay would admit all 24 compiled rows, render-only 12 (including four wrong deletions), and the conjunction 8. These are targeted predicate diagnostics, not production decisions or estimated false-acceptance rates. Native app selection and approval were bypassed; no candidate was committed or saved.

The initial run is preserved at `artifacts/continuation-20261112/source-intent-reference/`: 18 rows/8 declared expectations met, with all six actual planning calls refused by unknown preamble syntax, plus the missing `unibidi-lua.sty` Hebrew/LuaTeX setup. The corrected protocol follows a narrow preamble extension and an amendment recorded before rerunning. It reuses development cases and is not independent replication. The missing Hebrew/LuaTeX setup remains unresolved; substituting XeLaTeX does not pass that gate. Nothing was installed.

Arabic and Hebrew replacement-reference pages were visually inspected for visible text/layout and the requested ampersand/percent. There were no missing-character log warnings. This is not independent linguistic/shaping adjudication, a caret/IME test or universal multilingual support. Source/reference labels still need independent human review. Exact raster equality is host/tool/font-specific. Final evaluation remains unfrozen/unrun, with these fixtures excluded from its held-out set.

The finding is a concrete instance of the established non-injective-view/alignment problem discussed in the related-work notes. It does not establish novelty or publication readiness. The useful next implementation step is to retain exact source-target authority when integrating typed plans into the native verifier/IPC; render agreement must not replace it.
