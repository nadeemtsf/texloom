# Texloom research workspace

Status: **technical report draft and evaluation design, not a paper or a completed evaluation**. Started 4 November 2026. Application development remains the priority. Nothing in this directory authorizes publication or submission.

The audit baseline is product commit `c26a23851155f7e4e542eef80fae92e35c51f707`. It is a development baseline, **not the final evaluation freeze**. Its three CI workflows were observed successful. Current working-tree/document changes and executable receipts must accompany any later claim.

Later local product commit `05761985a7deef312426c268b953d4d01eccdaca` adds focus-loss cancellation for active gestures; 239 frontend and 88 default core checks and a Windows build passed. Its native/installation tests are pending at the owner's request. A separate independent-reference feasibility pilot and diagnostic predicate replay have run; they are described in [ORACLE-PILOT.md](ORACLE-PILOT.md), with raw receipts. No final held-out evaluation has run.

5 November continuation: `5e59e77282fd5e4b4f70440437b8f841ccea1364` adds setup help and optional local handler/publication/frame-proxy timing. 252 frontend and 88 default core checks and the ordinary Windows build pass. Native timing, overhead, setup and installation checks remain deferred while the owner is away. [Timing boundaries](../docs/PERFORMANCE-DIAGNOSTICS.md) and the [partial posthoc environment snapshot](ENVIRONMENT.md) prepare later measurement; neither supplies final results.

Proposed scope: investigate the correctness/coverage/latency trade-off of provenance-constrained source patches and verification after compilation for bounded, single-page text displacement. Text reflow and semantic parameter controls have weaker, separate contracts. Do not describe Texloom as a general inverse of TeX, a formally verified editor, or a novel visual-editing paradigm.

Files:

- [Implementation audit](implementation-audit.md): actual pipeline, code anchors and policy boundaries.
- [Evidence inventory](evidence-inventory.md): admissible claims, historical failures and missing evidence. A generated JSON catalog fingerprints existing evidence; parsing a receipt is not reproducing its experiment.
- [Related work](related-work.md): primary sources, what was checked and how each narrows the claim.
- [Evaluation plan](evaluation-plan.md): outcomes, denominators, success criteria, independent oracles, comparisons and freeze rules.
- [Report outline](report-outline.md): intended 5-8-page structure.
- [Contribution notes](contributions.md): avoid inferring individual work from a Git author field.
- [Editable report](report/main.tex), with [bibliography](report/references.bib).
- [Independent-reference pilot](ORACLE-PILOT.md) and [prototype byte-level source oracle](SOURCE-CONTRACT.md): executed development diagnostics, separate from final study results.
- [Development annotation analysis](ANALYSIS.md): tested denominators, unresolved/unrun handling and separate evidence strata; no final data summarized.
- [Environment records](ENVIRONMENT.md): explicit tool/font/input hashes, posthoc versus preflight limits and remaining final-freeze requirements.
- [Native candidate-boundary pilot](NATIVE-REFERENCE-PILOT.md): real native source-plan/build/approval/commit/Undo plus separate source/Poppler references. The first invalid-reference run is retained; a corrected protocol passes five intended cases and refuses 25 source-corrupt packets. No mouse-mapping, app accuracy rate or final evaluation is implied.
- [Freeze record](evaluation/freeze.json): deliberately unset until the core workflow and protocol are stable.

Keep development fixtures in their existing directories. Do not populate or run the final held-out set yet. Future final inputs, labels, executable, dependencies, engine/font versions, machine description and analysis code must be sealed together. Any product fix during final evaluation requires a new version and a separately labelled run; failures remain preserved.

Compile only the explicitly named `.tex` report, with shell escape disabled. Never compile `.text` files. Generated PDFs/logs/renders stay outside tracked research source, under ignored `artifacts/research-report/`; a review copy may be placed in the chat outputs. The report's results section remains pending.
