# Evaluation plan - draft-v0

6 November follow-up amendment, before any final freeze: Arabic/Hebrew target cases used to diagnose marker ordering are development fixtures, not eligible as untouched held-out cases. Final labels must distinguish single-direction paragraphs, mixed directions, nested language changes and unsupported cross-paragraph structures. Do not extrapolate the current XeLaTeX paragraph result to other engines or all RTL constructs. Include fresh positive/refusal cases only where fonts/packages exist, with rendering health checked separately from mapping.

Predeclare an exact v0.3-to-v0.4 generated-support migration as an allowed support-file change, or keep the support file already current in a separately labelled condition. In both cases bind before/after hashes. A customized support file is a refusal control whose bytes must remain unchanged. Compare migration and already-current conditions separately; neither permits arbitrary unrelated source changes. Final ground-truth oracles must not blindly reuse the production wrapper, its marker IDs or the same target-selection assumption. The current old/new-wrapper comparison establishes this reproduced ordering defect only; it is not a competitive baseline or a general accuracy estimate.

This plan precedes final experiments. The final study is **not run and not frozen**. Continue development first. The audit baseline is `c26a238`; it is not automatically the evaluation version. Targets below are proposed acceptance criteria recorded now, not achieved results or an externally registered protocol.

5 November source-only continuation: all tests are paused. RECORDS.md and ADJUDICATION.md develop the separate commit/display/Save outcome model, intent/candidate counts and independent review procedure. Their v1 record implementation is unexecuted and not a frozen protocol or final admission system. Primary-source follow-up is in related-work-followup-2026-11-05.md. The current compiled v5 report is unchanged; no results have been added from this continuation.

## Questions and scope

- RQ1: In declared single-page text displacement cases, how many incorrect edits are accepted, correct edits accepted, and valid edits refused?
- RQ2: Which source/provenance/verification stages reject incorrect candidates, and which legitimate cases do they exclude?
- RQ3: What are the end-to-end latency and resource costs of first-time mapping, cached selection and verified movement?

Replacement/reflow, semantic parameter controls and legacy blocks are separately reported secondary strata. Do not combine their weaker contracts with displacement preservation rates. Faithful English/Arabic/Hebrew source/display is in scope; canvas editing eligibility is case- and engine-specific. Do not install new keyboards or infer CJK acceptance.

## Independent labels and useful coverage

Each request has two independent labels: (a) whether its intended transformation is semantically valid under the declared oracle, and (b) whether it belongs to the product's declared supported subset. An unsupported but meaningful edit is not automatically an unsafe request. A refusal there can be safe and still represent lost useful coverage.

Annotate exact intended source occurrence, allowed patch/support-file changes, expected target and allowed reflow before running the product. Do not derive ground truth from the app's refusal message, source planner, markers or same verifier. Prepare expected sources manually and compile them outside app caches. Compare with a separately pinned renderer, provisionally bundled Poppler, plus independent source annotations and targeted human adjudication. The runtime PDFium worker and its prediction path share an implementation; agreement there is not two independent oracles. Mark unresolved disagreements as unresolved, retain them and report their denominator.

Planned minimum: 120 labelled requests across at least 36 new synthetic documents, including at least 60 valid intended edits and 60 unsafe/adversarial candidates. Seal exact counts and all changes to this draft before evaluation. The engine matrix is conditional on fixture feasibility; do not count Arabic/Hebrew pdfLaTeX configuration failures as equivalent observations of a supported XeLaTeX workflow. Report document/request/engine counts separately; repeated timings do not create new correctness cases.

Required strata: ordinary signatures/paragraphs/formatting; repeated visible strings and repeated macros; ambiguous/coincident/cross-column or cross-file evidence; English/Arabic/Hebrew with combining/shaping cases; unsupported tables/math/links/paint/page coordinates; compile failures, stale revisions, cancellation, malformed evidence and wrong-target/neighbor/duplicate-ink mutations. These are design categories, not a claim that final fixtures already exist.

## Outcomes and denominators

For every intended request record system decision and independent adjudication. Report raw counts alongside rates:

Acceptance means canonical source/history commit, separately from frontend publication, successful presentation and explicit Save. A post-commit presentation error remains an acceptance for source/render adjudication; it is neither a harmless pre-commit refusal nor a successful end-to-end task. Preserve native stage/commit observations, all independent postconditions and presentation/persistence outcomes separately. Accepted edits with incomplete independent evidence remain unresolved for the affected correctness claim.

Natural interaction requests and injected-candidate/component diagnostics are separate study arms. Record the injection/entry layer (mapping, source plan, candidate build or comparator) and never pool their rates. A corrupted receipt rejected before geometry says nothing about the geometric check's sensitivity. Decision replay can expose a check's marginal effect but cannot establish end-to-end production error frequency.

- Incorrect edits accepted: all accepted candidates that violate target, source, preservation or concurrency requirements. Report both the count and the fraction of accepted candidates, plus unsafe requests accepted divided by unsafe requests.
- Correct edits accepted: valid requests accepted with all required postconditions, divided by valid requests. Show the declared in-scope subset and broader stress set separately.
- Valid edits refused: semantically valid requests refused, divided by valid requests; classify mapping ambiguity, unsupported grammar, verifier refusal, resource budget and other reasons. Do not relabel a valid refusal as success to improve accuracy.
- Correct unsafe refusals: unsafe requests refused before canonical commit, divided by unsafe requests. A system that refuses everything still has zero useful valid acceptance.
- Compile failures, timeouts and crashes: separate counts, with availability/latency consequences; they are not silently excluded or all described as expected safe refusals.
- Preservation: exact bytes outside an enumerated allowed patch set, exact Undo/Redo/Save/reopen checks, intended target geometry and independently checked unrelated rendered content. Count every failed postcondition, including accepted edits to the wrong repeated occurrence.

Use paired comparisons on identical requests. Provide per-family/engine breakdowns. Any statistical uncertainty method must state and justify its sampling/independence assumptions and preserve document clusters; a clustered method alone cannot make this purposive corpus representative. Zero observed incorrect acceptances is a finite result, not proof of zero risk. Do not manufacture a population confidence claim from deterministic/adversarial fixtures.

For accepted edits with unresolved adjudication, report confirmed incorrect count I, unresolved count Q and accepted count A. If A is nonzero, the possible incorrect-acceptance fraction lies between I/A and (I+Q)/A. This is a deterministic adjudication interval, not a statistical confidence interval. Keep the corresponding safety conclusion unresolved while accepted outcomes lack required evidence.

Ratios with zero denominators are reported as not applicable, not zero. In particular, deny-all has no accepted-candidate error fraction and has zero useful valid acceptance.

## Criteria recorded before final runs

1. Required safety gate: zero adjudicated incorrect acceptances, zero unrelated-source corruption, zero wrong-occurrence commits and zero stale-result commits in the sealed suite. Any occurrence fails this gate and remains in the report.
2. Useful-coverage target: at least 90% correct acceptance among predeclared in-scope valid displacement requests, with counts for every stratum. Report broader valid-refusal rates without hiding them in a safety total. This proposed engineering target is not a claim of statistical representativeness.
3. Preservation: exact source bytes outside allowed changes; zero-offset same-renderer pixel identity; requested target error <=0.25 bp and unaffected glyph geometry <=0.02 bp where comparable. Final independent raster/annotation checks must be specified before runs and may expose limits beyond the current runtime contract. No tolerance tuning on the final set.
4. Performance goals, separately assessed: p95 source visual feedback <=50 ms, prepared selection <=50 ms, drag preview frame budget 16.7 ms on a documented 60 Hz reference setup, zero pointermove builds, one logical build per release, and p95 warm verified small-document result <=3 s. These are inherited goals, not current measurements. Report missed goals honestly; correctness need not be weakened to meet them.
5. Native reliability: current signature/source/save/recovery workflow must pass on the frozen binary; real held-gesture cancellation/DPI and installed-artifact checks remain explicitly separate until actually executed.

## Comparisons and ablations

| Variant | What it can establish | Limits / implementation status |
| --- | --- | --- |
| Full current pipeline | Main correctness/coverage/latency trade-off | Not a formal proof; bounded domain and oracle limits |
| Compile-only after the same constrained plan | Added value of verification beyond syntax/build success | Offline research harness planned; never disable production guards |
| Source + selection/provenance checks without independent final movement validation | Marginal rejection contributed by the final verifier | Decisions must replay the same candidate set; labels independent of app checks |
| Geometry-only versus full appearance comparison | Whether geometry misses duplicate ink/appearance defects, and appearance causes valid refusals | Same candidate corpus; shared-renderer bias remains; planned offline ablation |
| First-time versus cached selection; first wrap versus existing object | Cost of identity-probe work and cache effects | Separate valid workloads; no correctness advantage inferred from timing alone |
| Deny-all diagnostic reference | Makes the safety/coverage trade-off explicit | Not a useful editor or a competitive system |
| Existing broken-literal fixture | Regression sensitivity and harness health | A development diagnostic, never part of held-out headline accuracy |

The final-study pipeline ablations remain plans. A separate post-compilation feasibility pilot and predicate replay are implemented and measured in ORACLE-PILOT.md; they bypass the production selection/source gates and do not estimate app error rates. Use isolated copies and immutable candidate artifacts; no user project is committed by an ablation. If replay can assess rejection but cannot reproduce source planning/interaction, label it a decision-layer comparison rather than an end-to-end baseline. LyX/TeXmacs/BaKoMa/Overleaf comparisons initially describe documented workflow differences; no unmeasured superiority or unfair timing leaderboard.

## Reproducibility and timing

Before freezing, record product and analysis commits, clean working tree or exact diff, executable/installer/worker/DLL hashes, lockfiles, support-package versions, engine/bibliography/font versions and hashes, OS/CPU/RAM/display/DPI, power/background-load state, commands, randomization seed and resource budgets. Hash fixture/oracle manifests before the first final run. Keep a dedicated run directory; never overwrite earlier failed output.

For latency, separate input/preview, source mapping, snapshot, TeX passes, auxiliary tools, PDF preparation, independent validation, native commit and visible presentation. A desired physical input-to-display interval requires an external measurement method. Current optional diagnostics start at JavaScript handler/method entry and end at publication or two animation callbacks, with freshness/visibility checks; they exclude OS input dispatch, and source feedback excludes Monaco work before its content-change callback. Compile traces exclude the preceding auto-compile debounce/queue. State those boundaries explicitly, never report the proxy as physical presentation, and do not pool prepared-object/cached selection with a first proof-building selection. See ../docs/PERFORMANCE-DIAGNOSTICS.md. Native verification is included in the visual build RPC; a later receipt recheck is not a separate PDF comparison. Record overhead against a paired ordinary build, export bounded batches with pending/loss counts, and freeze the instrumented version consistently for all compared conditions. No native latency or overhead measurement has run.

The development environment collector in ENVIRONMENT.md provides explicit file/tool fingerprints with preflight/posthoc labels. Its executed posthoc snapshot cannot establish the environment during an earlier run and leaves some reference-machine fields unset. Complete the actual input/font/runtime binding before the final freeze; do not treat the collector's successful probes as completed final evaluation or clean-machine setup.

Use at least 10 representative small documents with 10 declared warmups and 30 measured warm repetitions per document/engine condition, randomized paired variant order. Record cold mapping separately, all failed/censored runs, medians, p95 and sample counts. Add a documented repeated-operation resource soak; a mixed two-instance sample is not an isolated leak study. Human usability is a separate exploratory study with task success, errors and observed assistance; no participant data or recruitment occurs in this phase.

## Freeze and stop rules

Freeze only after the core positive workflow and current regressions are stable, the protocol/oracles have been reviewed, and final fixtures are sealed. `evaluation/freeze.json` stays `not_frozen` until those conditions are met. Version the protocol with any pre-run changes. If evaluation finds a bug, preserve the failing run, fix the development version and report a new separately frozen run; do not tune and silently rerun the same final set as if it were untouched. Do not run final evaluation merely because a build succeeds.

Current authorization is source/research work **except all tests**. Safe work now includes audit/fingerprinting existing evidence, preserving failures, verifying citations and preparing code/protocol/schema/report. No new regressions, infrastructure diagnostics, experiments, app launches or final runs are authorized during this pause. REFERENCE-ADMISSION.md and TIMING-RECORDS.md describe new unexecuted drafts; matching file declarations or parsing timing exports would not by itself complete final admission. Final held-out results remain pending.
