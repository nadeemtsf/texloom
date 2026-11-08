# Read-only source foundation: development evidence and study implications

8 November 2026. Technical-report working note, not a final evaluation result. Product source: `171830a1854929b1ca35133b8d2013d4e54b3a15`. The engineering receipt is [`foundation-validation-2026-11-08.json`](../docs/architecture/foundation-validation-2026-11-08.json); the contract is [foundation contract v1](../docs/architecture/FOUNDATION-CONTRACT.md).

## What changed in the implementation

The core now exposes a borrowed, exact-byte source partition with a conservative literal-paragraph subset and opaque unsupported regions. References bind file, exact occurrence, expected bytes, source revision and in-process session identity. Resolving them is a source-identity check, not an operation, source-to-PDF correspondence proof or independent output check. The existing visual UI and legacy movement handlers do not use this new model yet.

The provisional research question still describes the intended end-to-end investigation. This increment addresses only an early source-identification prerequisite. It does not demonstrate source-native visual authoring, a bidirectional transformation system, general TeX parsing, novelty or publication readiness. No final-result section should be filled from it.

## Evidence that can be reused

Seventeen synthetic domain tests cover lossless partitioning, repeated occurrences, changed-source and foreign-session rejection, opaque syntax, included fragments without context, unavailable operations and bounded fallback. Two added negative controls first failed for math/control-flow leakage; the failing log is preserved and both pass after conservative changes. The final default core run passed 70 unit, 18 prior integration and 17 new tests. A separate recovery-serde run and the 291-test frontend run have explicitly narrower source-snapshot timing in the receipt. These are overlapping suites/configurations, not independent document samples.

Exact preservation of English, Arabic and Hebrew UTF-8 bytes does not establish bidi selection, grapheme editing, font shaping, rendering or actual Windows input. No UI, engine, PDF mapping, performance or usability experiment was run for this increment. Raw logs remain local ignored artifacts; the tracked receipt fingerprints them but does not make them available in a fresh checkout.

## Coverage cost to investigate next

The scanner deliberately declines unknown commands/definitions and unresolved context. A normal document containing macro definitions, including the current application template, can therefore be entirely opaque in this new projection. A passing lossless/refusal test is not evidence of useful editable coverage. Existing editing behavior remains on its prior path.

Before expanding recognition, measure development-only eligibility by document family: recognized region count, eligible intended targets and opaque reason. Report document-level and target-level denominators separately; do not call an ineligible target an accepted incorrect edit. Include literal prose, repeated paragraphs, known formatting, included fragments, macro-defined text, comments, incomplete source and mixed English/Arabic/Hebrew source. Engine-dependent semantics still require a later compiled-document experiment.

## Bounded diagnostic comparisons, not yet executed

1. Replay source-only reference resolution with the current strict revision/session guard and, in an isolated analysis harness, a text-only occurrence lookup. Predeclare repeated-text, inserted-prefix, changed-source and identical-foreign-session cases. Measure correct occurrence resolution, incorrect resolution and refusal separately. The comparison asks what strict identity contributes; it does not estimate PDF mapping or editor error rates. Never weaken production guards for the comparison.
2. Compare conservative whole-document fallback with a proposed context-aware projection only after independent intended-region labels are recorded. Measure incorrect eligibility and useful target coverage, with unresolved labels retained. Neither result counts as edit acceptance until an actual operation and its postconditions are exercised.

For every executed diagnostic, retain exact code/fixture hashes and expected source ranges before running it. These proposed comparisons do not justify creating a new final set now: implementation fixtures are development data, and final documents/oracles must be separately recorded once the chosen core workflow is stable. The existing final freeze remains `not_frozen`; no app binary or report PDF was rebuilt here.

## Report integration boundary

Use this note in a later methods revision to distinguish source eligibility, identity, native planning, compilation and independent verification. Describe excessive conservatism as a coverage limitation and preserve the negative-control failure history. The existing v10 PDF/export remains an immutable earlier snapshot; this note does not silently revise its results or page count. No new literature or novelty claim is introduced.
