# Independent-reference feasibility pilot

This is a **development diagnostic**, not the final Texloom evaluation. Criteria were written to the run's protocol before compilation. The final held-out directory and evaluation freeze remain unset.

Question: can separately authored TeX box positioning provide a reference for the shipped movement wrapper, with a PDF comparison independent of Texloom's source planner, PDF.js and PDFium?

The reference positions the second of two identical Latin lines using an explicitly sized TeX box. Candidates use the portable movement package. Poppler extracts word bounds and renders full pages; the comparison requires matching text multiplicities, word geometry within 0.02 bp and **zero changed pixels** at 144 dpi. No runtime marker IDs select the reference target. The comparator does not verify source intent or authorize an application commit.

Five conditions ran: Latin fixtures under pdfLaTeX, XeLaTeX and LuaLaTeX, plus XeLaTeX/LuaLaTeX fixtures with Arabic and Hebrew neighbors. Every condition had one intended move, four purposeful faults (wrong occurrence, changed unrelated text, color-only change and duplicate ink), and one deliberate compile failure. These are 30 controls from five simple fixture conditions, not 30 independent user tasks.

All five intended moves matched their references. All 20 faulty PDFs differed, and all five deliberately invalid sources failed compilation. Replaying predicates on the same artifacts gave:

| Diagnostic predicates | Reference-matching controls admitted | Known-fault controls admitted | Compile failures stopped before comparison |
| --- | ---: | ---: | ---: |
| Compiler success only | 5 | 20 | 5 |
| Compiler, text and word geometry | 5 | 5 | 5 |
| Compiler plus full reference comparison | 5 | 0 | 5 |

The five color-only controls retained text and geometry; full-page pixels distinguished them. This supports the feasibility of that diagnostic comparison on these inputs. It is **not** a measured ablation of the production source/selection gates, an app incorrect-acceptance rate, or evidence of universal preservation. The five compile failures are not successes of a renderer that never ran on them.

Limitations: one configured Windows host, a shared reference/control preparation workflow without independent adjudication, Latin targets, simple single-page reading order, and no sealed font/machine image. Arabic/Hebrew neighbors test preservation in those fixtures, not unrestricted shaped-text editing. Independent source-intent annotations and ambiguous-layout adjudication remain necessary. The pilot deliberately uses no final held-out documents.

## Reproduce or inspect

Research-only dependencies are listed in `requirements.txt`; Python is not an app runtime dependency. Use the existing configured tools; this workflow does not install Windows input languages, open Texloom or compile `.text` files.

From the repository root, with the research Python environment and Poppler utilities available:

```text
python -m unittest discover -s research/tests -v
python research/tools/oracle_feasibility.py artifacts/NEW-RUN --pdftotext PATH-TO-PDFTOTEXT --pdftoppm PATH-TO-PDFTOPPM
python research/tools/analyze_oracle_pilot.py artifacts/NEW-RUN/report.json artifacts/NEW-RUN/predicate-replay.json
```

Every run directory must be new. Existing output is refused. Commands, engine versions, executable/support-package hashes, protocol, source/PDF/raster artifacts, logs and individual outcomes are retained. The analysis refuses missing/duplicate conditions or controls and reports missing or unexecuted evidence as unresolved. Nine research-tool diagnostics passed separately; those checks are not added to an app accuracy denominator. The initial and strengthened analysis sources/replays are retained separately; their measured counts agree.

Current raw run: ignored `artifacts/oracle-feasibility-20261104-first/`. The normalized receipt is `docs/evidence/oracle-feasibility-2026-11-04.json`. The observed Poppler text extractor and rasterizer are separately versioned/hashes recorded; sharing a code lineage still limits oracle independence. No final evaluation or publication occurred.
