# Shared editing foundation contract v1

8 November 2026. Implements the ARCH-01 contract deliverable and the first read-only ARCH-02 source-model slice. This defines behavior for later commands; it does not enable new visual Delete/reorder/typing or certify a rendered edit. The implementation is `crates/texloom-core/src/document/structure.rs` with integration checks in `crates/texloom-core/tests/structure.rs`.

## Adopted source/output and interaction policy

The owner's recorded goal is Word-like visual authoring while retaining ordinary LaTeX. "Object" means a source-backed selectable entity, not necessarily a generated macro. New source-native saved operations must not introduce Texloom-specific commands/support packages. Standard LaTeX formatting/spacing/boxing is permitted where a declared operation can preserve its contract; forbidding every standard wrapping construct would be a different, narrower requirement. Existing normal project dependencies remain.

Reorder changes logical source order; spacing adjustment changes a supported parameter or declared flow effect. A pointer gesture must identify which command is intended. Delete content removes its selected source; remove positioning keeps the content; physical asset deletion is a separate lifecycle action. No new free-floating mode is required.

The current marker-based handlers remain compatibility paths. Private probes may continue in existing code; the source-native proof strategy remains ARCH-10 work. Input-host choice and caret geometry remain the TYPE-01 feasibility study. This contract does not choose an experimental browser API or claim its native behavior is validated.

## Capability and evidence matrix

"Candidate" below means lexical source eligibility requiring an operation planner, context/dependency validation, source/PDF identity and the appropriate verification. It never grants permission to mutate a document or publish a PDF.

| Entity/operation | Current application evidence | New foundation status | Remaining work |
| --- | --- | --- | --- |
| Plain top-level paragraph | Existing bounded wrapper movement/replacement has recorded tests; r8 owner acceptance remains open | Read-only lexical entity with exact range, canonical revision and session identity | Source-native Replace/Delete/Reorder/Format/Insert/Split/Join planners and render policies |
| Literal word/span/caret | Protected literal editing exists through the old modal flow | No substring/caret capability created by this slice | Grapheme/cluster mapping, anchors, text projection and insertion-point proof |
| Formatted paragraph/signature group | Existing balanced-unit/protected editing paths remain | Markup/groups/environments are retained as opaque source | Structural context and per-operation capabilities; do not imply all signatures are projected |
| Existing native spacing | Numeric parameter adapters already exist | Plain paragraph does not acquire a spacing parameter merely because it is text | Native adapter/entity connection and declared reflow policy |
| Positioned/wrapped legacy content | Existing movement, unmake and narrow insertion/deletion keep their old scope | No automatic conversion or new marker grant | Explicit migration/equivalence; preserve customized styles |
| Images/tables/captions | Existing import/parameter controls have bounded evidence | Opaque here; physical DeleteAsset never becomes a text operation | Asset transactions and dedicated structural adapters |
| Unknown/dynamic/incomplete source | Source editing remains available | Exact source retained; no entity-reference grant for opaque regions | Add supported context incrementally; unknown expansion is not guessed |
| English/Arabic/Hebrew source | Prior app evidence retains its named-version scope | Exact UTF-8 source/ranges, BOM and CRLF preservation tested | No grapheme editing, font/render/shaping or bidi caret pass inferred |

The existing visual UI is not wired to this model yet. Therefore current user-visible selection, editing and movement behavior is not being replaced by this conservative first subset.

## Source model API and limits

`Session::source_projection(file)` borrows the canonical source and returns an ordered partition of exact byte ranges. Regions are plain paragraphs, trivia or opaque source with a reason. Concatenating all region text reproduces the entire input exactly, including the preamble, comments, unsupported constructs and incomplete source. It does not serialize a normalized document or rewrite files.

The initial subset requires a single complete document envelope in the selected file. Included fragments without a proven document context remain opaque. Only literal top-level prose and escaped literal symbols can become paragraph candidates. Soft-wrapped source lines stay one paragraph. Formatting arguments, nested environments, comments and unsupported syntax are not converted into editable prose.

Definitions, conditionals, unknown control sequences, TeX character-preprocessing syntax and incomplete structural context conservatively decline projection. Multiline math cannot expose apparently plain intervening paragraphs. Even recognized lexical candidates have unresolved engine/package/render context: their readiness explicitly requires a future plan and proof. This is not a full TeX parser, source mapping oracle or security boundary.

Bounded work uses at most 200,000 outer scanner iterations, 128 structural scopes and 10,000 output regions, alongside the existing source-file size limit. Budget exhaustion returns the original source as opaque, not a partial collection of actionable candidates. Argument/literal helpers remain bounded by the existing source size; the scanner-iteration count is not a measurement of total CPU work.

`entity_reference(index)` exists only for a recognized paragraph. The reference has private fields, exact file/range/expected content, revision and an unforgeable in-process session identity. It is deliberately not serialized or accepted as an IPC capability. `resolve_entity_reference` rejects a different session, stale revision, missing file or mismatched bytes. Repeated text retains the particular range selected; no nearest-text search occurs.

A changed source revision invalidates these references even after Undo restores the same bytes. A no-op edit that does not change revision retains a current reference. These are strict snapshot references, not yet the change mappings, affinity, logical text anchors or caret positions planned in ARCH-03. They grant no compiler/visual approval.

## Required contracts for the later commands

- Replace/Insert: validated literal/context and correct escaping; distinguish plain text from explicit raw LaTeX.
- Delete: exact intended content removed once, compatible source boundaries, unrelated source preserved; retain shared physical assets by default.
- Reorder: move the same source fragment once to a legal destination, preserving intended content and checking its new order.
- Spacing/Format: affect only the declared local parameter/style scope; disclose allowed reflow instead of promising strict displacement.
- Split/Join/structural edits: preserve syntax/scopes and declare legitimate dependent counter/reference/contents effects.
- All commands: native rederivation, current source/selection identity, shared history/recovery, explicit source/commit/publication/Save outcomes and operation-specific verification.

None of those new command handlers is implemented by the readiness enum. Draft typing remains distinct from a verified geometric edit. Do not route future operations through the generic readable-PDF check and call them preservation-verified.

## Executed evidence and remaining gates

The first 15 integration checks passed. Two added negative controls then reproduced excessive candidate recognition for multiline math and unknown control flow. That failing run is retained under ignored `artifacts/foundation-20261108/structure-negative-before-fix.log`; the parser was tightened and the expanded 17-check suite passed. Additional cases cover character preprocessing and the final-region budget boundary.

Checks cover exact Unicode/BOM/CRLF partitions, opaque markup/environments/literals, incomplete/duplicate envelopes, dynamic source, repeated occurrences, stale and foreign references, no-op edits, no source/history mutation, unavailable spacing/asset actions and bounded fallback. Required existing core/frontend checks and exact snapshot limits are recorded in the [engineering receipt](foundation-validation-2026-11-08.json). These are headless domain regressions, not new real-engine, mapping, native interaction, latency or final-evaluation results. ARCH-01 remains open for owner design review; writing the contract is not that review.

ARCH-02 remains in progress: richer entities, included-file context, dependency/scope evidence and integration with native intent planning remain future work. ARCH-03 change maps, ARCH-04 operation execution, source-native mapping and visual input are not completed by this first slice. The owner performs all actual app-opening/input/installation acceptance; no new app was opened for this work.
