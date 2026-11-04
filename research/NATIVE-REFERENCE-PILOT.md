# Native candidate-boundary development pilot

Native entry API schema 1; reference protocol revisions 1 and 2 were each written before their execution. This is development integration, not the final evaluation or an actual Texloom window test. The original post-compilation pilot in ORACLE-PILOT.md remains unchanged.

## Question and entry layer

Can a predeclared move of an existing tagged second occurrence pass the real native prepare/build/independent-approval/commit path and an external reference/source check? Do deliberately corrupted proposed-source packets, paired with the unchanged intended operation, fail before compilation?

Entry is **proposed sources plus a move request at native prepare**. The runner uses the same `Backend` methods as the app, through the existing opt-in `benchmark` harness feature. It starts no Tauri window, calls no Save, writes no original project and removes no evidence. It does not run PDF.js mapping, a frontend zero-offset selection probe, drag input or the normal frontend planner invocation. Native prepare does rederive its source plan. No full-app correctness or latency rate follows.

## Predeclared cases and criteria

Five conditions: Latin targets under pdfLaTeX/XeLaTeX/LuaLaTeX, plus Latin targets with Arabic/Hebrew neighbors under XeLaTeX/LuaLaTeX. Every baseline has two identical visible phrases with distinct existing object identifiers; the requested second object moves 12 bp right. All sources include a BOM/CRLF main file and an unchanged supporting file. This does not test moving shaped Arabic/Hebrew target text or prove all multilingual content editable.

For each condition, five deliberately corrupted packets precede the correct packet while the same baseline/revision remains current: move the first occurrence instead; modify a neighbor; change target color; duplicate target ink; alter only an unrelated supporting file. Keep the intended native operation constant. A refused packet must leave canonical source/history unchanged; it is not compiled. A packet unexpectedly prepared is recorded and discarded, without compilation/commit. Preparation is not final edit acceptance.

The intended move succeeds only if native commit completes, exact predeclared source hashes match, the compiled result matches an independently authored TeX-box reference in Poppler word bounds (0.02 bp) and full-page 144 dpi pixels, Undo restores every source byte, and disk files are unchanged. First check the native zero-offset baseline against a plain-text reference. Reference placement uses ordinary TeX boxes/phantom rather than the movement macro or native planner. Failed/unresolved references or post-commit failures cannot become passing refusals.

There are five intended moves and 25 deliberately corrupt packets. These are component diagnostics with an explicit entry layer, not a representative dataset or statistical safety estimate. Source-gate refusals do not demonstrate that a renderer inspected those faulty packets. Reference/control preparation has no independent human adjudication; shared fonts/page setup and one host limit inference.

## Reproduction and evidence

`tools/native_reference_pilot.py prepare NEW_RUN_DIRECTORY` writes an exclusive protocol and snapshots the executed helper/native source before any experiment compilation. It records criteria, inputs, source contracts, code fingerprints and the current Git baseline. Build the opt-in `native_reference_development` example from the recorded source with locked dependencies, then use `evaluate` with that exact binary and explicit Poppler tools. The evaluator refuses changed implementation snapshots, an existing run/output, or a changed protocol/runner.

Native output retains source copies, baseline/candidate PDFs and logs, per-packet reasons, commit status, exact Undo copies and a separate cache of compiler/worker receipts. Gold builds use only explicitly named `.tex` inputs with shell escape and automatic MiKTeX installation disabled; recorder output is kept. The source oracle and Poppler comparisons remain separate from the native gate. Every failure is preserved in a new run directory; do not overwrite it or pool a tuned rerun.

## First executed attempt and reference correction

The first attempt at `artifacts/native-reference-development-20261105-first` retains protocol SHA-256 `8cd000d559011a5e14e7accb5dc1ab67967763dd936cb648ea079b625d5d6a84`. Its recorded result is **zero of five intended moves matching every declared postcondition**. All five native commits/source contracts/Undo checks completed and 25 modified packets were refused with `The candidate differs from the native operation plan`; each left source/history unchanged and was not compiled.

The failed comparisons exposed an invalid reference: a zero-width `hbox` began in vertical mode and its following `phantom` started another blank line. Both target words moved the intended 12 bp, but the gold's following neighbor moved down about 11.955 bp (Latin) or 11.956 bp (multilingual). Native candidate geometry retained the neighbor. Gold-only comparisons of the two reference PDFs reproduce this error without consulting native output. A pair of rendered pages was also inspected. The original zero/five result, sources and output are unchanged; no first-run case is retroactively labelled a full correctness pass. Posthoc diagnosis is separately retained under `artifacts/native-reference-diagnosis-20261105`.

Before a second attempt, protocol revision 2 adds `leavevmode` before the independent reference box and a gold-only health condition: the two declared second-occurrence words must translate exactly, and every other word bound must remain within 0.02 bp. This is a bounded word-geometry health check, not proof of all reference semantics/appearance. Missing or reordered/ambiguous target words remain unresolved. The native executable and acceptance implementation are unchanged. Two new diagnostics check the observed neighbor drift and unresolved-target handling; 27 research helper checks pass. A fresh protocol/run is required; this is a corrected development rerun on reused cases, not new holdout evidence.

## Corrected run: measured development results

`artifacts/native-reference-development-20261105-reference-v2` completed with **five of five intended moves matching all declared postconditions** and **25 of 25 corrupted packets refused before compilation**. All ten baseline/candidate Poppler full-page comparisons have zero changed pixels; all five gold-only geometry health checks and exact byte-source contracts pass. Exact Undo and unchanged original source files hold in all five conditions. The native refusal reason is the source-plan mismatch, not an independently observed renderer rejection of those packets.

Native executable SHA-256 `fcb46dae8fcdbc86e415209cc9f9caf62892ff4315009658185468768422c2a3` is identical across both runs. Baseline sources, candidate packets, requests, source contracts and zero references also match between attempts. Only the moved-reference construction and its health analysis changed. These are the same five intended cases and 25 corrupted packets reused in a corrected development rerun; do not pool the two attempts as ten/fifty distinct trials or call them independent replication. Receipts identify the code/protocol/outputs and the retained failure: `../docs/evidence/native-reference-development-2026-11-05.json`.

Final held-out fixtures remain empty, `evaluation/freeze.json` remains `not_frozen`, and the owner's deferral of all tests requiring Texloom to open or approval still applies. No end-to-end latency, actual mouse interaction or clean-machine result was produced.
