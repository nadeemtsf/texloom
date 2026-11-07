# Study records v1 - development implementation draft

Prepared 5 November 2026 during the earlier test pause. Updated 7 November: synthetic infrastructure checks now exercise the v1 record code, including preservation of known commits with presentation failures, rejection of unknown-commit relabelling, unrelated-change failures and deny-all denominators. The combined study-record module has 13 passing tests within the 40-helper suite. This is bounded regression evidence, not a final evaluation, real app observation or migration of old results. The template remains an unfilled specification: its null identifiers and empty criteria are intentionally not a valid measured record.

The v5 PDF remains the last compiled report. This follow-up implements its proposed separation of native acceptance, display and durability; it supplies no new results. The existing `summarize_annotations.py` and original v0 row specification remain unchanged for their older diagnostics. Their recorded five annotation checks do not validate this new implementation. Do not silently translate the native pilots' old raw schemas into final v1 observations.

## Boundaries and outcome rules

`decision_basis` is either `canonical_commit` or `component_predicate`. A component replay can accept a PDF without ever committing source. It therefore has no canonical commit, presentation or Save observation; those statuses are `not_applicable`. Natural visual requests must enter at `pdf_selection` with a canonical-commit basis. Injected diagnostics explicitly name their entry layer: `pdf_selection`, `source_mapping`, `source_plan`, `candidate_build` or `comparator`.

Canonical `system_decision: accepted` requires `commit_status: committed`. If display subsequently fails or the process crashes, acceptance remains recorded and `presentation_status`/`terminal_event` retain the failure. If commit is uncertain, both decision and status remain `commit_unknown`/`unknown`; do not infer a refusal from an exception. There is no automatic retry or final-evaluation admission in this tool.

`case_id` identifies a natural request or an injected candidate/predicate case. `intent_id` links multiple corruptions to their shared intended edit. The summary reports case, intent and document counts separately. In a natural arm `oracle_truth` labels the intended request; in an injected arm it labels the supplied candidate under that intention. These denominators and meanings must not be pooled. A meaningful unsupported request can still be `valid`; support is the separate predeclared boolean.

| Field | Allowed observations |
| --- | --- |
| `system_decision` | `accepted`, `refused`, `compile_failure`, `timeout`, `crash`, `cancelled`, `commit_unknown`, `not_run` |
| `commit_status` | `committed`, `not_committed`, `unknown`, `not_attempted`, `not_applicable` |
| `presentation_status` | `published`, `failed`, `unknown`, `not_attempted`, `not_applicable` |
| `save_status` | `saved`, `failed`, `unknown`, `not_attempted`, `not_applicable` |
| `terminal_event` | `completed`, `refused`, `compile_failure`, `presentation_failure`, `timeout`, `crash`, `cancelled`, `unknown`, `not_run` |

Display and Save refer to the requested candidate, not the retained baseline. `saved` records a Save outcome; exact saved/reopened bytes still require the separate `save_reopen_exact` postcondition. A source commit is not an original-file write. A component predicate is not a full-app task.

A refusal needs `refusal_stage` (`mapping`, `identity`, `source_plan`, `native_validation`, `frontend_validation`, `commit`, `component_predicate` or `unknown`) and `refusal_reason` (`ambiguous_mapping`, `unsupported_structure`, `source_plan_mismatch`, `preservation_mismatch`, `resource_budget`, `stale`, `other` or `unknown`). Other decisions leave both null. Compile failures remain compile failures. Do not attribute a packet rejected during source planning to renderer sensitivity. Valid-refusal reasons and observed refusal stages are retained in separate counts.

Each record explicitly declares its required postconditions before execution. Canonical correctness requires at least `target_correct`, `source_outside_patch_exact` and `unrelated_rendered_content_preserved`. Exact Undo, Redo and Save/reopen can be additional requirements. Every named postcondition is boolean or null; null means unresolved/unobserved, not a pass. A positive overall accepted-edit adjudication requires a valid intention and every required postcondition true. Every unrun postcondition remains null.

`annotation` distinguishes preparer, reviewer and review status. Role labels are supplied metadata; differing strings do not prove independent people or independent preparation. Unknown roles stay null. Self-review is reported separately and cannot be presented as independent review. Nothing here establishes individual project contributions.

## Binding and denominator design

Executed rows require a recorded implementation commit, runner/protocol/fixture/oracle manifest hashes and relative raw-evidence references with SHA-256 values. These are **declared bindings**, not verified files or sealed final manifests. The tool validates record structure. The separate `admit_records.py` development checker, exercised with synthetic inputs, checks declared file bytes and fixture/oracle-manifest relations under an explicit controlled root; it cannot establish actual execution, correct labels, pre-run sealing or source-to-binary provenance. See [reference admission](REFERENCE-ADMISSION.md). Neither route runs compilers or opens the app.

Groups separate run, arm, decision basis, entry layer, contract, variant, fixture family, engine, language and evidence origin. Binding changes inside a stratum are refused. Case IDs remain unique within a run/arm/basis/layer/variant/engine/origin, even if a family or contract label changes. Repeated natural intents within that identity are also refused. Engine variants and reused cases in distinct runs are not pooled as independent requests. Timing repetitions belong in separate records, not duplicated correctness rows.

The planned summary retains counts of accepted, refused, failed, cancelled, unknown and unrun cases. Correct-acceptance numerators require reviewed supplied adjudications and established postconditions; unresolved or draft-only labels cannot become successes. A reviewed status is not itself proof of independent preparation. Valid refusals exclude compile failures/crashes. Any unrun case suppresses group rates. An observed wrong acceptance still fails the finite safety gate even when other cases are unrun. Unknown accepted decisions, uncertain commits, unresolved intentions and missing label review prevent a resolved safety conclusion.

For accepted cases, confirmed incorrect I, unresolved Q and total A produce the deterministic interval I/A to (I+Q)/A. A zero denominator is not applicable. This interval covers unresolved accepted adjudications only; uncertain commit requests are separately counted and cannot justify a safe conclusion. It is not a population confidence interval. Deny-all has no accepted-error fraction and zero useful valid acceptance. No population estimate, physical presentation or latency statistic is computed.

For canonical records only, preview/Save failures after acceptance are counted separately. The complete-workflow rate uses only valid requests whose predeclared required set includes all six postconditions; success also requires candidate publication and Save. Component groups have no end-to-end workflow metric.

## Deferred use and remaining work

When tests are authorized again, review this implementation and its boundary/conflict cases before using it on observations. Its eventual development-only interface is:

```text
python research/tools/study_records.py ANNOTATED-ROWS.json NEW-SUMMARY.json --development
```

This command has **not** been run. Output creation is exclusive. Final manifest admission, independent label review, timing batches and statistical analysis remain separate work. `evaluation/freeze.json` stays `not_frozen`; no final fixtures have been created or evaluated by this change. Never compile `.text` files or treat commands in records as instructions to execute them.
