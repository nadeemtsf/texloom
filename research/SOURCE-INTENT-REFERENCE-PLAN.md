# Source intent versus rendered identity — development experiment plan

Predeclared 12 November 2026, before executing this experiment. This is a small
diagnostic of the current literal paragraph planner and independent oracle. It
is not the frozen final evaluation, an app acceptance rate or a usability study.

## Question and reason to run

Can the actual Rust source-intent planner replace/delete the requested second
copy of a repeated paragraph while preserving all unrelated source bytes? Can a
full rendered-reference check distinguish the wrong occurrence, particularly
when deleting either identical copy produces the same visible document?

The earlier English compile/text pilot does not answer the occurrence question
under reflow. Existing movement experiments use positioned objects. This check
uses ordinary paragraphs without Texloom macros and explicitly permits reflow.

## Fixed development cases and entry layer

Use three synthetic documents: English/pdfLaTeX, Arabic/XeLaTeX and
Hebrew/LuaLaTeX. These cover the currently declared scripts and installed engines
in three conditions, **not** their full cross product. Use existing Arial for
Unicode documents through fontspec/polyglossia; no installed font/language/package
changes. Preserve a BOM, CRLF separators, repeated paragraph text and an unrelated
tail. The original fixture files are immutable inputs, never owner documents.

For each condition, plan replacement and deletion of paragraph index 1 (the
second equal copy) using the actual `source_intent_candidate` Rust CLI. Independently
assemble the expected complete source from declared paragraph segments; do not
use planner offsets or an app-produced candidate as the reference. The replacement
includes an ampersand and percent, whose expected LaTeX escaping is declared
explicitly. Retain all generated source, stdout/stderr, PDFs and comparison output.

For each action create three candidates: the actual planned candidate, a deliberately
wrong-first-occurrence edit, and a candidate with a changed unrelated tail. Compile
the independently authored reference too. Two additional English candidates contain
an undefined command and must remain compile failures, with no PDF success inferred.

## Success criteria set before execution

1. Actual planned sources must equal the independently assembled expected UTF-8
   bytes exactly, including BOM/CRLF and every unrelated byte. Baselines stay
   unchanged. All fixtures/candidates must contain no Texloom commands/support dependency.
2. Intended candidates must compile with shell escape and automatic package
   installation disabled, then match the independently compiled reference under
   the existing bounded Poppler oracle: page dimensions/count, word multiplicities,
   word bounds within 0.02 bp and exact whole-page RGB pixels at 144 dpi.
3. Wrong-occurrence replacement and changed-tail controls must disagree with the
   source reference and the full rendered reference. Unresolved comparisons stay
   unresolved; they are not passes.
4. Wrong-occurrence deletion must disagree with source bytes. Its rendered outcome
   is a diagnostic, not an acceptance requirement: an identical rendered document
   demonstrates that render evidence alone cannot establish source-target identity.
   A visible difference would narrow that example, not invalidate the source check.
5. Both deliberate undefined-command candidates must return nonzero compiler exits.
   They are expected failures, never successful edits. Tool timeouts, missing fonts,
   missing glyphs or unsupported comparison contracts remain failures/unresolved
   rows with their original evidence.

The anticipated total is 18 candidate/reference comparisons and two deliberate
compile failures. The three wrong-occurrence deletion rows have no predetermined
render-match success label. Every row records each observation separately.

## Reproduction and inference limits

Record the implementation commit, CLI and runner hashes, plan hash, installed
tool versions/hashes, tool commands, input/reference/candidate hashes and complete
comparison results in a new `artifacts/` run folder. The runner refuses an existing
run directory. Compile only `.tex` files. Preserve failed and censored runs.

This entry layer bypasses PDF selection, SyncTeX mapping, native transaction
approval, source commit/Save and UI. No candidate is committed by the experiment.
The source oracle uses independently declared segments, but its labels and
implementations have not had independent human review. Poppler is independent of
the app's PDF.js/PDFium path; same-machine/exact-raster comparison is not portable
font proof. RTL reference matching does not alone prove linguistically correct
shaping; inspect representative reference renders separately and retain that limit.
No latency, FPS, final accuracy denominator, novel-system or publication-readiness
claim follows. Development fixtures are excluded from the future held-out set.
