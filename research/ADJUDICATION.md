# Independent intent and preservation annotation - proposed procedure

Prepared 5 November 2026; status reconciled 13 November. This is a protocol draft, not an executed adjudication, a sealed final set or independent review evidence. Synthetic helper tests exercise record/admission/timing rules; actual app interaction and independent final review remain separate. This document develops the evaluation plan and `RECORDS.md`; it creates no final fixture documents. The latest compiled technical report is v15 (33 pages, 25 references), with no page cap; final-study results remain pending.

## 1. Record intention before asking the system

For each natural visual request, declare the intended source occurrence and target operation without relying on Texloom's mapping, generated ID or refusal. Bind a document hash/file set, source byte span or independently identified existing object, physical page and target region, plus displacement or replacement parameters. A repeated spelling is insufficient identity. Describe whether the request is meaningful and separately whether it belongs to the predeclared supported subset.

Use unique `case_id` and `intent_id` values. Natural requests do not become new correctness samples through repeated timings. Injected corruptions get distinct case IDs but retain their parent intent ID, document and corruption type. Keep natural interaction, candidate-entry diagnostics and comparator replay separate. A source-only harness does not supply a PDF-click observation.

The proposed minimum remains 120 cases over at least 36 new synthetic documents. One possible allocation is 60 natural valid requests plus 60 deliberately corrupted candidate cases linked to a declared set of intentions. This would not be 120 independent natural requests. Exact counts, support labels, document clusters, engine variants and any additional unsafe natural requests must be reviewed and sealed before final execution; this allocation is not frozen.

## 2. State what valid, unsafe and unresolved mean

For a natural request, `valid` means its intention and preservation obligations are meaningful under the declared contract; unsupported structure can still make it a valid request. A product refusal then represents lost useful coverage, with support recorded separately. Do not redefine support after a failure.

For a diagnostic candidate, `valid`/`unsafe` describe the proposed transformation under the parent intention. A packet that moves the first repeated phrase when the independently declared target is the second is unsafe even if it compiles and follows a different internally consistent operation plan. An edited support file outside the permitted set is also unsafe even if the PDF looks identical. Do not call the parent user intention unsafe merely because a corrupted candidate was supplied.

If intention, source identity or the independent reference cannot be resolved, retain `unresolved`. A product refusal does not establish an unsafe request, and an acceptance does not establish validity. Configuration failures, missing fonts/packages, timeouts, crashes and uncertain commits retain their own outcomes. They must not be hidden by changing the semantic label or excluding the attempt.

## 3. Prepare independent source and output obligations

Enumerate the entire relevant source/support-file set. Seal baseline bytes including BOM, CRLF/LF, comments and auxiliary source files. Declare exact allowed patches or a restricted fresh-ID template before inspecting a candidate. If several representations are legitimate, declare the allowed relation or alternatives; do not reject a correct edit merely because it differs from one conveniently generated gold file. Never use the runtime planner to generate its own expected-source answer.

Prepare expected sources outside app caches. Reference construction must not simply invoke the production movement macro. The development pilot's plain TeX box/phantom approach is one bounded alternative, not a universally independent specification. Record preparer and reviewer separately, leaving unknown roles unresolved; do not infer independence from different labels or a Git author field.

For displacement, declare target identity, allowed physical-page bounds, requested translation and unrelated-output obligations. The draft independent word-geometry tolerance is 0.02 bp for the declared gold relation, with raster settings and exact-versus-tolerant criteria fixed in advance. Runtime target tolerance 0.25 bp is a different contract; it must not silently replace an independent reference criterion. Reflow/semantic/legacy families need their own obligations, without claiming displacement's unchanged-layout guarantee.

Record exact Undo, Redo and Save/reopen obligations when part of the case. Save is separate from native source/history acceptance. A successful Save message is not independently verified byte persistence. Keep direct source/render correctness and full task completion distinct.

## 4. Review reference health before product results

Compile the declared baseline and gold only when experiments are authorized again. Check the zero reference against its independent baseline, then check that the moved reference changes the intended target and retains unrelated material under its declared policy. Inspect non-text content as well as word bounds where required. Manually inspect difficult cases and bind the review/version to the protocol before running the app.

The first native-entry pilot's manually authored gold inserted an unintended blank line. Its later health check detected a specific class of word-geometry drift; it did not prove arbitrary TeX semantics, rendered non-text preservation or correctness of every reference. Do not reuse a development health result as blanket validation of new golds. Keep the invalid protocol/results, diagnosis and corrected version separate.

## 5. Handle correspondence ambiguity explicitly

| Situation | Annotation/adjudication rule |
| --- | --- |
| Repeated identical words in distinct known source occurrences | Bind intended occurrence independently; check source bytes and output context. Text equality alone cannot select the occurrence. |
| Coincident or visually indistinguishable occurrences | Preserve the intended source distinction if independently specified. If rendered correspondence cannot be resolved, leave that output postcondition unknown instead of pairing by extraction index. |
| Multiple columns or changed extraction order | Use physical geometry plus declared source/context evidence and review. Reordering a token list to make comparison pass is not independent adjudication. |
| A macro emits multiple outputs or several source spans generate one output | Declare the permitted relation explicitly or classify the reference as unresolved. Product marker identity alone cannot supply independent intent. |
| Arabic/Hebrew shaping, combining marks or text-layer order differs | Check declared language/engine/font context and visual/source intent separately. A word-box mismatch can reflect extraction behavior; do not automatically label it a wrong source edit or silently normalize it away. |
| Gold and actual differ but each seems plausible | Preserve both. Review target, allowed patches, fonts/build inputs and contract. Keep unresolved outcomes visible until a dated adjudication resolves them. |

Do not add a greedy correspondence rule merely to complete the denominator. A documented ambiguity/refusal policy and its useful-coverage cost are legitimate findings. Stable keys, positional matching and operation-based alignment have prior foundations; see the primary-source follow-up, especially Matching Lenses. The proposed annotation process is not a new general alignment algorithm.

## 6. Compare stages that were actually reached

Record the injection layer and observed refusal stage. The existing native pilot's 25 corrupt source packets fail before compilation; a later renderer never sees them. They establish that source rederivation rejected those packets, not the independent renderer's marginal benefit.

Before an ablation, state which candidate population reaches each compared check. A compiler-only comparison after the same constrained plan may have no differential cases if the plan already rejects every supplied corruption. Report that limitation rather than attributing all rejections to final verification. Separate controlled comparator replays can study PDF sensitivity, but cannot establish full-app error frequency.

Particularly informative future cases include a wrong intention/occurrence that is internally consistent with the supplied operation plan, first-wrap versus existing-object paths, and output changes introduced despite an apparently allowed source patch. These are investigation categories, not known defects or prepared final fixtures. Source guards, independent output checks and an independent intent oracle can answer different questions.

## 7. Retain decisions and disagreements

After execution is authorized, record native commit or uncertainty, displayed result, Save outcome and each required postcondition independently. A confirmed commit followed by a presentation error remains accepted. A lost/unusable acknowledgement remains uncertain unless separate evidence resolves it. Do not retry or relabel an uncertain case until the original state and evidence are retained.

Unreviewed positive labels cannot become established successes. Any unrun case remains unrun; no final cells are filled from regression totals or component pilots. Publish raw counts by entry layer, basis, engine, family and document cluster with unknown/failed/cancelled outcomes. Zero accepted cases means no accepted-error fraction, not a zero-error editor. Keep deterministic adjudication bounds separate from statistical uncertainty.

The v1 record code has bounded synthetic regression evidence and still needs review against actual experiment records before final use. No fixture freeze, independent reviewer, performance sample or usability result is established by writing this procedure or passing those helper tests.
