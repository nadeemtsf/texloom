# Native candidate-boundary development pilot

Protocol v1, written before execution. This is development integration, not the final evaluation or an actual Texloom window test. The original post-compilation pilot in ORACLE-PILOT.md remains unchanged.

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

At protocol authoring, execution/results are **pending**. Final held-out fixtures remain empty, `evaluation/freeze.json` remains `not_frozen`, and the owner's deferral of all tests requiring Texloom to open or approval still applies.
