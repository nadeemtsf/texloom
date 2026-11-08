# Execution backlog for the visual editor roadmap

Planning baseline: 7 November 2026, application source `6f55baca3100ef8af48d0a3cadb419855c125339`. The 8 November continuation began ARCH-01/02 in local commit `171830a`; all issues remain open. Other tasks remain planned. See the [foundation contract](FOUNDATION-CONTRACT.md) and [executed engineering receipt](foundation-validation-2026-11-08.json) for the bounded implemented slice; neither establishes product acceptance. P0/P1/P2 indicate relative priority, not severity or promised dates. Dependencies establish execution order; existing GH issues retain their named-version scope. See [the roadmap](ROADMAP.md) and [the architecture](ARCHITECTURE.md).

## ARCH-01: Define the visual editor capability and saved-source contract

GitHub: [#45](https://github.com/nadeemtsf/texloom/issues/45) — open/in progress. Contract and matrix written; owner design review remains pending.

Phase: foundation; priority: P0; type: design.

Depends on: no new task.

The earlier bounded movement scope does not define the expanded Word-like editing product or the distinction between no custom macros and no standard boxing constructs.

Review target entity families and Replace/Delete/Reorder/Spacing/Format/Insert semantics. Define source-native output, allowed reflow, opaque-source preservation, legacy compatibility, current languages and unsupported cases. Record decisions D01-D08 without implementing future features.

Acceptance:

- [ ] A reviewed capability matrix distinguishes implemented, tested, planned and unsupported behavior for each entity/operation.
- [ ] Delete content, remove positioning and physical asset deletion are distinct.
- [ ] New saved source-native edits avoid Texloom-specific commands/packages; normal project dependencies and any permitted standard constructs are explicit.
- [ ] Reorder and spacing are explicit intents; no new free-floating mode or universal Word/TeX promise is introduced.

Validation to execute after implementation is authorized:

- Owner design review against paragraph/signature, repeated-text, multilingual and legacy examples.
- Trace requirements to verification obligations and follow-up issues; no runtime pass inferred from the design.

Current source anchors: `docs/APPROVED-IMPLEMENTATION-PLAN.md:5`, `src-tauri/src/service/layout_gate.rs:16`.

## ARCH-02: Add a partial lossless source projection and capability registry

GitHub: [#46](https://github.com/nadeemtsf/texloom/issues/46) — open/in progress. Read-only literal-paragraph projection and strict snapshot references implemented; richer context/entities and command integration remain pending.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-01`.

Current source units and macro-derived object IDs do not form a general editable entity model.

Derive supported paragraphs, literal spans, groups and future media/table entities from authoritative source; preserve comments, trivia, incomplete and unknown TeX as opaque regions. Give each operation its own capability and refusal reason.

Acceptance:

- [ ] Opening and no-op round trips preserve all original bytes, including unsupported and incomplete syntax.
- [ ] Entity ranges carry file, revision, context/scope and dependency evidence.
- [ ] A replaceable fragment is not automatically reorderable or deletable; macro definitions/shared invocations are not confused.
- [ ] No mandatory normalized HTML/JSON document replaces the original LaTeX.

Validation to execute after implementation is authorized:

- Synthetic lossless round trips, scopes/comments/literal regions and included-file cases.
- Explicit refusals for unknown or structurally ambiguous boundaries.

Current source anchors: `crates/texloom-core/src/objects.rs:17`, `crates/texloom-core/src/objects.rs:169`.

## ARCH-03: Introduce typed anchors, insertion positions and change mappings

GitHub: [#47](https://github.com/nadeemtsf/texloom/issues/47) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-01`, `ARCH-02`.

Valid raw offsets and current object rectangles do not establish stable caret/selection identity through edits or reflow.

Specify byte/UTF-16/grapheme/logical/PDF position types, anchor/head direction, insertion affinity, change maps, inverse mapping and deterministic invalidation. Keep build geometry separate from editable source identity.

Acceptance:

- [ ] Insert/delete/reorder before a selection maps it correctly; destructive edits invalidate ambiguous identity.
- [ ] Repeated text, included files, source edits and Undo/Redo cannot silently switch the target occurrence.
- [ ] BOM/CRLF, combining marks, emoji and English/Arabic/Hebrew offsets remain distinct and correct.
- [ ] Logical before/after and visual left/right have explicit bidi behavior and recorded algorithm/runtime versions.

Validation to execute after implementation is authorized:

- Synthetic mapping/invalidation and inverse-change properties with targeted wrong-occurrence controls.
- Later actual caret/navigation tests remain separate from domain checks.

Current source anchors: `crates/texloom-core/src/document.rs:110`, `crates/texloom-core/src/document.rs:267`, `crates/texloom-core/src/document/projection.rs:49`.

## ARCH-04: Generalize typed native editing intents and minimal patch planning

GitHub: [#48](https://github.com/nadeemtsf/texloom/issues/48) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-02`, `ARCH-03`.

Current visual families and string action fields are tied to object movement and optional-field combinations.

Define typed Replace/Delete/Reorder/Adjust/Format and future Insert/Split/Join requests, native rederivation, read/write sets, effect contracts, operation IDs and transport outcomes. Reuse canonical state and exact inverse patches.

Acceptance:

- [ ] Invalid payload/target combinations refuse before compilation and cannot manufacture source or visual authority.
- [ ] Mouse, keyboard, menu and later input-host intents use the same native planner.
- [ ] Schema/type generation and runtime checks agree; requests bind expected source/revision/context.
- [ ] Unknown acknowledgements have a resolution/idempotency design before automatic retry; future Insert does not require a second transaction subsystem.

Validation to execute after implementation is authorized:

- Protocol-negative, stale/context/dependency and exact patch-set tests.
- One request/action produces one documented source outcome under acknowledgement failure.

Current source anchors: `src-tauri/src/service/layout_gate.rs:16`, `src-tauri/src/service/visual.rs:137`, `crates/texloom-core/src/document.rs:511`.

## ARCH-05: Model draft input and verified rendered publication as separate lifecycles

GitHub: [#49](https://github.com/nadeemtsf/texloom/issues/49) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-03`, `ARCH-04`.

The current freshness gate is appropriate for geometric operations but would block visual typing after its first source change if reused indiscriminately.

Represent canonical draft, input/composition session, prepared visual transaction, last verified render and pending build/publication identities. Preserve guarded draft editing and stronger visual approval within shared history.

Acceptance:

- [ ] Typing can preserve incomplete source without a complete compile per character.
- [ ] Stale/failed builds do not replace newer typing, cancel composition or grant stale geometry capabilities.
- [ ] Provisional feedback and verified PDF state are identifiable; last-good PDF and draft recovery survive failure.
- [ ] Known source commit with failed presentation remains distinct from pre-commit refusal and unknown commit status.

Validation to execute after implementation is authorized:

- State-transition and race tests for burst input, compile failure, cancellation, project switching and late publication.
- Actual responsiveness and input behavior require later owner-run packaged-app checks.

Current source anchors: `crates/texloom-core/src/document/source_edit.rs:37`, `web/app.js:131`, `web/app.js:157`, `web/app.js:518`.

## ARCH-06: Extend shared history with semantic grouping and selection restoration

GitHub: [#50](https://github.com/nadeemtsf/texloom/issues/50) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-03`, `ARCH-04`, `ARCH-05`.

Future caret/input hosts, composition and object operations need coherent grouping without adding a separate undo owner.

Specify command/typing/composition undo units, source and selection inverses, recovery metadata and the relationship to staged asset references.

Acceptance:

- [ ] One completed delete/reorder/spacing action has one coherent Undo/Redo outcome.
- [ ] A composition or declared typing group is not fragmented by background rendering.
- [ ] Undo/Redo restores exact source plus appropriate logical selection/caret or an explicit safe invalidation.
- [ ] Save/reopen/recovery does not rely on stale PDF DOM or an unrelated browser undo stack.

Validation to execute after implementation is authorized:

- Mixed source/visual/history sequences, refusal and presentation-failure controls.
- Crash/recovery and owner interaction remain separately recorded gates.

Current source anchors: `crates/texloom-core/src/document.rs:174`, `crates/texloom-core/src/document.rs:344`, `web/app.js:841`.

## ARCH-07: Separate frontend session, selection, input and preview orchestration

GitHub: [#51](https://github.com/nadeemtsf/texloom/issues/51) — open/backlog.

Phase: foundation; priority: P1; type: design.

Depends on: `ARCH-04`, `ARCH-05`.

Growing shared app.js state and object-specific controllers do not provide clear ownership for future caret/input and accessible editing.

Incrementally extract project/session, command execution, selection/input sessions, preview lifecycle and inspector/history/recovery integration. Preserve pure geometry/protocol and native authority. Define keyboard/focus and non-drag command interfaces now.

Acceptance:

- [ ] Controllers have explicit ownership/disposal and injected interfaces rather than duplicate document/compiler/history state.
- [ ] Closed/switched projects cannot republish stale work or retain actionable stale selection.
- [ ] Keyboard, pointer and menu paths resolve the same intent/capability.
- [ ] Focus restoration and command availability are represented independently of object-frame DOM replacement.

Validation to execute after implementation is authorized:

- Lifecycle/cancellation and equivalent-intent integration checks.
- Preserve existing gesture cancellation and commit/presentation failure distinctions.

Current source anchors: `web/app.js:53`, `web/app.js:453`, `web/app.js:518`, `web/core/canvas-controller.js:5`.

## ARCH-08: Design staged asset references and recoverable object transactions

GitHub: [#52](https://github.com/nadeemtsf/texloom/issues/52) — open/backlog.

Phase: foundation; priority: P1; type: design.

Depends on: `ARCH-04`, `ARCH-06`.

Current text history and direct image import do not define the future source/asset lifecycle for insertion, Delete, Undo and recovery.

Define staged imports, owned content identity, source references, cancellation/retention, Save/recovery and conservative cleanup. Existing behavior is not labelled a demonstrated data-loss bug.

Acceptance:

- [ ] Default visual Delete removes the source reference, not a potentially shared file.
- [ ] Cancelled insertion and Undo/Redo retain required original/owned bytes.
- [ ] Shared or unknown macro references block guessed automatic garbage collection.
- [ ] Source/asset recovery and export bind exact required bytes; external changes remain protected.

Validation to execute after implementation is authorized:

- Synthetic shared-asset, failed import, cancellation, external modification and interrupted-save cases.
- No real project or physical asset deletion used as a test fixture.

Current source anchors: `crates/texloom-core/src/document.rs:54`, `src-tauri/src/service/project_workflows.rs:473`, `crates/texloom-build/src/persistence.rs:158`.

## ARCH-09: Define operation-specific source and rendered-output verification

GitHub: [#53](https://github.com/nadeemtsf/texloom/issues/53) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-01`, `ARCH-02`, `ARCH-04`.

Existing displacement, reflow and Semantic policies cannot certify all future editing operations under one preservation claim.

Specify Replace/Delete/Reorder/Spacing/Format/Insert/media source effects, allowed geometry changes and bounded dependent/generated-output effects such as counters,references and contents. Use policy versions and independent development oracles; keep strict displacement separate.

Acceptance:

- [ ] Wrong occurrence/order, duplicates/omissions, collateral source/render changes and stale candidates are explicit negative controls.
- [ ] A readable PDF or text multiset alone cannot establish intended order, formatting or correct identity.
- [ ] Allowed pagination/reflow and dependent numbering/reference/contents changes are declared before execution; unbounded effects refuse.
- [ ] Runtime acceptance and independent final evaluation have separate identities and limitations.

Validation to execute after implementation is authorized:

- Predeclared synthetic positive/refusal/failure cases for each enabled operation.
- Retain failed references and unresolved oracles; do not turn a design into a passing result.

Current source anchors: `src-tauri/src/service/layout_gate.rs:374`, `src-tauri/src/service/layout_gate.rs:382`, `crates/texloom-build/src/verification.rs:349`.

## ARCH-10: Decouple source-native selection identity from persisted object markers

GitHub: [#54](https://github.com/nadeemtsf/texloom/issues/54) — open/backlog.

Phase: foundation; priority: P0; type: design.

Depends on: `ARCH-02`, `ARCH-03`, `ARCH-04`, `ARCH-09`.

Current selection/approval and normal move planning depend on object markers and automatic custom package preparation.

Add a source-native operation path using proved source anchors/entities. Evaluate retained private instrumentation versus alternatives; prove that any probe leaves uninstrumented output equivalent and never leaks into canonical Save. Isolate legacy adapters.

Acceptance:

- [ ] New supported source-native operations add no Texloom macro/package to canonical saved source.
- [ ] Repeated text and macro expansion cannot authorize a guessed target; ambiguous identity refuses.
- [ ] Any temporary proof is bound to current source/build and uninstrumented candidate identity.
- [ ] Legacy marker-based behavior remains explicitly scoped rather than silently granting new authority.

Validation to execute after implementation is authorized:

- Real-engine mapping/probe identity, correct/wrong occurrence and byte-preservation controls.
- Record refusal/latency trade-offs; do not promise universal wrapper-free exact displacement.

Current source anchors: `web/core/selection-proof.js:20`, `web/app.js:396`, `src-tauri/src/service/visual.rs:393`, `src-tauri/src/service/layout_gate.rs:382`.

## CORE-01: Deliver generalized visual deletion for supported source entities

GitHub: [#55](https://github.com/nadeemtsf/texloom/issues/55) — open/backlog.

Phase: source_native; priority: P0; type: feature.

Depends on: `ARCH-02`, `ARCH-03`, `ARCH-04`, `ARCH-06`, `ARCH-09`, `ARCH-10`.

Visual Delete currently covers legacy blocks rather than ordinary proved text/entities.

Add Delete for declared supported text ranges/paragraphs/groups with correct source boundaries and operation policy. Later media deletion uses ARCH-08. Keep removal of positioning distinct.

Acceptance:

- [ ] Only the selected intended occurrence is removed; structure and unrelated source remain correct.
- [ ] Keyboard/menu/non-drag pointer routes agree and unsupported targets explain refusal.
- [ ] One Undo restores exact source and suitable selection; failed/stale operations retain accepted state.
- [ ] Deletion does not introduce custom wrappers or silently unlink physical assets.

Validation to execute after implementation is authorized:

- Repeated text, comments/scopes, signature groups, multilingual/grapheme boundaries, compile failure and history cases.
- Later owner-run exact-build deletion/Save/reopen acceptance.

Current source anchors: `web/app.js:930`, `web/core/canvas-controller.js:98`.

## CORE-02: Implement source-native flow reordering with explicit destinations

GitHub: [#56](https://github.com/nadeemtsf/texloom/issues/56) — open/backlog.

Phase: source_native; priority: P0; type: feature.

Depends on: `ARCH-02`, `ARCH-03`, `ARCH-04`, `ARCH-06`, `ARCH-09`, `ARCH-10`.

Drawing displacement is not a source-order operation and a soft-wrapped PDF line is not necessarily an independent source unit.

Start with supported paragraphs/balanced groups and visible legal insertion targets. Atomically remove and insert the same source fragment; expand to literal spans only after scope/format/identity rules are proved.

Acceptance:

- [ ] Content moves exactly once to the declared legal destination without changing its source bytes unexpectedly.
- [ ] Formatting, comments and associated structural context are preserved or the operation refuses.
- [ ] Allowed reflow/pagination and intended reading order are independently checked.
- [ ] Repeated moves and Undo/Redo do not accumulate wrappers or choose a repeated occurrence incorrectly.

Validation to execute after implementation is authorized:

- Before/after offsets, multi-patch boundaries, included files, repeated text, RTL/logical-order and scope controls.
- Owner sees reorder destination and can use keyboard/non-drag pointer alternatives.

Current source anchors: `crates/texloom-core/src/objects.rs:169`, `web/core/canvas-controller.js:39`.

## CORE-03: Connect visual spacing adjustments to ordinary LaTeX parameters

GitHub: [#57](https://github.com/nadeemtsf/texloom/issues/57) — open/backlog.

Phase: source_native; priority: P0; type: feature.

Depends on: `ARCH-01`, `ARCH-04`, `ARCH-09`, `ARCH-10`.

Existing native parameter controls are not a general verified drag-to-source-spacing route; spacing can move downstream content.

Prefer modifying proved existing local gap/indent/position parameters. Add standard commands only in explicitly supported contexts. Keep source-order and drawing displacement semantics separate.

Acceptance:

- [ ] Repeated adjustments update the same recognized parameter rather than stack duplicate commands.
- [ ] Units, scope and local/global effects are explicit; no blind pixel-to-TeX-point substitution or global rewrite for one object.
- [ ] The signature/following-paragraph example either meets its preservation contract or discloses/rejects the different flow effect.
- [ ] New source-native output has no Texloom package dependency; unsupported numeric expressions remain protected.

Validation to execute after implementation is authorized:

- Native unit/spacing and three-engine effect cases with unchanged unrelated source.
- Owner acceptance of permitted reflow versus position-only behavior.

Current source anchors: `crates/texloom-core/src/adapters.rs:200`, `crates/texloom-core/src/adapters.rs:612`.

## CORE-04: Move visual replacement and formatting onto shared source-native commands

GitHub: [#58](https://github.com/nadeemtsf/texloom/issues/58) — open/backlog.

Phase: source_native; priority: P0; type: feature.

Depends on: `ARCH-04`, `ARCH-06`, `ARCH-09`, `ARCH-10`.

Current bounded replacement/formatting exists, but normal text-object workflows remain coupled to wrapper preparation.

Reuse protected literal/style adapters through the new command/capability model. Specify plain-text escaping versus explicit raw source, formatting scopes and declared reflow.

Acceptance:

- [ ] Supported replacement/formatting edits only the intended literal/style scope and retains opaque commands.
- [ ] English/Arabic/Hebrew, repeated text, special characters and existing formatting are handled under declared rules.
- [ ] Native approval, shared history and uncertain/presentation outcomes retain their meanings.
- [ ] No new Texloom-specific saved wrapper/package is required by the source-native route.

Validation to execute after implementation is authorized:

- Existing bounded regressions reused only where source/scope justify it; new route gets affected engine/oracle checks.
- Owner-run supported replacement/format/Save/reopen acceptance.

Current source anchors: `web/app.js:626`, `src-tauri/src/service/visual.rs:269`, `crates/texloom-core/src/adapters.rs:394`.

## CORE-05: Provide explicit legacy-project compatibility and safe source-native conversion

GitHub: [#59](https://github.com/nadeemtsf/texloom/issues/59) — open/backlog.

Phase: source_native; priority: P1; type: feature.

Depends on: `CORE-01`, `CORE-02`, `CORE-03`, `CORE-04`, `ARCH-08`, `ARCH-09`.

Unwrapping currently removes displacement and retains package preparation; deleting the support file is not appearance-preserving migration.

Load old projects without rewriting on open; isolate compatibility controls; offer conversion on a protected copy/reviewed transaction with equivalence or an explicit changed-layout decision. Preserve normal external compilation dependencies.

Acceptance:

- [ ] Legacy/current/customized support files are distinguished and protected.
- [ ] Failed/partial conversion leaves original files and placements intact.
- [ ] Package removal occurs only after accounting for remaining commands/dependencies.
- [ ] Undo/Save/reopen and independent external compilation confirm the selected conversion outcome.

Validation to execute after implementation is authorized:

- Synthetic old/custom/shared-use and interrupted conversion cases; no actual project used as a disposable fixture.
- Portability acceptance extends existing #31 without claiming its old pass validates the new path.

Current source anchors: `crates/texloom-core/src/objects.rs:257`, `src-tauri/src/service/visual.rs:393`.

## TYPE-01: De-risk visual input host and provisional caret/layout behavior

GitHub: [#60](https://github.com/nadeemtsf/texloom/issues/60) — open/backlog.

Phase: authoring; priority: P0; type: spike.

Depends on: `ARCH-03`, `ARCH-05`, `ARCH-07`.

A stale compiled PDF and its text-layer spans do not directly supply a responsive Word-like input/caret system.

After implementation authorization, run a bounded synthetic feasibility study comparing controlled editing-host and EditContext/fallback approaches on actual WebView2. Evaluate provisional echo, surrounding text/geometry, reflow, composition, accessibility and build catch-up before committing to an API.

Acceptance:

- [ ] A decision record binds actual runtime, method, observed behavior, failures and performance endpoints.
- [ ] No full TeX compile is required before each character is retained in the protected draft.
- [ ] Logical grapheme boundaries, shaped/PDF clusters and caret stops have a defensible mapping; ambiguous/unavailable stops are explicit,never estimated by equal division of a run width.
- [ ] Provisional caret/input is understandable during stale PDF, compile failure and reflow.
- [ ] An unsuitable approach causes a design revision rather than weaker source/identity guarantees; no prototype pass becomes a product pass.

Validation to execute after implementation is authorized:

- Synthetic helper checks then owner-run packaged runtime/input/accessibility observations.
- No new Windows languages installed and no physical-display latency inferred from animation callbacks.

Current source anchors: `web/app.js:131`, `web/app.js:846`, `web/core/pdf-surface.js:74`.

## TYPE-02: Add logical visual caret, text ranges and insertion-point navigation

GitHub: [#61](https://github.com/nadeemtsf/texloom/issues/61) — open/backlog.

Phase: authoring; priority: P0; type: feature.

Depends on: `ARCH-03`, `ARCH-07`, `ARCH-10`, `TYPE-01`.

Whole-line/object selection and PDF DOM geometry cannot represent persistent logical caret/ranges through recompile and virtualization.

Implement anchor/head/affinity-based text selection, node selection and insertion points with revision-bound geometry and accessible focus. Reproject after verified publication; refuse ambiguous blank-space clicks.

Acceptance:

- [ ] Pointer and keyboard navigation handle supported grapheme/word/paragraph boundaries and mixed-direction text with cluster-level caret geometry evidence.
- [ ] Reflow, zoom, page unmount, source edits, Undo/Redo and failed builds cannot switch occurrence or reuse stale authority.
- [ ] Logical order remains distinct from visual bidi placement; unavailable mappings have a clear fallback.
- [ ] Caret/selection is not an authoritative DOM node or remembered PDF rectangle.

Validation to execute after implementation is authorized:

- Synthetic range/bidi/grapheme/projection cases and owner-run focus/navigation/recompile observations.
- Declared document editing budgets remain enforced.

Current source anchors: `web/core/canvas-controller.js:20`, `web/core/pdf-surface.js:74`, `web/app.js:453`.

## TYPE-03: Implement visual typing and composition through the shared draft lane

GitHub: [#62](https://github.com/nadeemtsf/texloom/issues/62) — open/backlog.

Phase: authoring; priority: P0; type: feature.

Depends on: `TYPE-02`, `ARCH-05`, `ARCH-06`, `ARCH-09`.

Modal visual replacement and source-mode composition are not ordinary in-flow visual typing.

Translate actual input/composition/replacement/deletion intentions into guarded contextual source edits; protect drafts, group history and coalesce background compilation. Never serialize PDF spans as LaTeX.

Acceptance:

- [ ] English/Arabic/Hebrew input, combining sequences and selection replacement preserve correct source and escaping through a validated bounded visible-text/source projection.
- [ ] Noncancelable host composition updates,cancellation/focus transitions and compile failure are reconciled without losing draft input.
- [ ] One composition/declared typing group has coherent Undo; geometry-dependent actions retain appropriate gates.
- [ ] Verified PDF catch-up cannot move the current caret using stale evidence or overwrite newer text.

Validation to execute after implementation is authorized:

- Input-protocol, state/race and source-preservation controls; later owner-run actual WebView2 typing/composition checks.
- Latency, correctness, collector boundaries and physical presentation are reported separately.

Current source anchors: `crates/texloom-core/src/document/source_edit.rs:37`, `web/app.js:846`, `web/app.js:626`.

## TYPE-04: Add source-safe visual cut, copy, paste and selection replacement

GitHub: [#63](https://github.com/nadeemtsf/texloom/issues/63) — open/backlog.

Phase: authoring; priority: P1; type: feature.

Depends on: `TYPE-03`, `CORE-01`, `ARCH-09`.

Reading/copying PDF text and modal replacement do not define editable clipboard transactions or Cut failure behavior.

Implement plain-text clipboard operations first through shared selection/operation contracts. Distinguish explicit raw-LaTeX input; rich clipboard conversion is a later separately scoped capability.

Acceptance:

- [ ] Cut deletes source only after successful clipboard write and revalidation of the same revision/selection; copy success plus deletion refusal reports copy-only completion and retains source.
- [ ] Paste/replacement escapes according to insertion context and does not execute clipboard HTML or hidden source commands.
- [ ] Mixed-direction and multiline text preserves intended logical order under declared policies.
- [ ] One completed operation has coherent Undo/Redo and draft protection; permission/focus failure retains source and uncertain deletion is resolved before retry.

Validation to execute after implementation is authorized:

- Synthetic clipboard failure, stale selection, special-character and wrong-context cases.
- Owner-run packaged clipboard/focus/paste checks; no assumption of universal browser API permission behavior.

Current source anchors: `web/app.js:626`, `web/core/pdf-search.js:3`.

## TYPE-05: Support paragraph split/join, headings, lists and bounded formatting

GitHub: [#64](https://github.com/nadeemtsf/texloom/issues/64) — open/backlog.

Phase: authoring; priority: P1; type: feature.

Depends on: `TYPE-03`, `CORE-02`, `CORE-04`, `ARCH-09`.

Word-like Enter/Backspace and structural formatting need semantic operations rather than raw newline insertion into arbitrary TeX contexts.

Add supported paragraph split/join, heading/list item operations and declared character/paragraph formatting using ordinary LaTeX. Preserve references/macros/opaque regions or refuse.

Acceptance:

- [ ] Enter/Backspace behavior follows the supported structural context and does not corrupt environments or command arguments.
- [ ] Heading/list changes preserve associated source/reference obligations and declare legitimate output changes.
- [ ] Navigation, selection, history and reflow use the shared contracts.
- [ ] Unsupported tables/math/macros remain intact and accurately unavailable.

Validation to execute after implementation is authorized:

- Balanced-context, comments/references, multilingual and split/join inverse cases.
- Owner-run complete edit/format/navigation/Save/reopen tasks.

Current source anchors: `crates/texloom-core/src/objects.rs:169`, `crates/texloom-core/src/adapters.rs:394`.

## OBJECT-01: Complete supported image and caption authoring with asset-safe history

GitHub: [#65](https://github.com/nadeemtsf/texloom/issues/65) — open/backlog.

Phase: authoring; priority: P1; type: feature.

Depends on: `ARCH-08`, `TYPE-02`, `CORE-01`, `CORE-03`, `ARCH-09`.

Current image import/parameter controls do not constitute complete visual insert/edit/delete/reorder/history across source and assets.

Deliver declared image-reference insertion, resizing/parameters, captions, supported placement/reordering and reference deletion. Respect floating-environment semantics and shared files.

Acceptance:

- [ ] Source and asset references participate in the staged transaction/recovery design.
- [ ] Deleting one visual use retains shared/original media bytes.
- [ ] Placement semantics are explained; float position is not promised as arbitrary canvas coordinates.
- [ ] Undo/Redo, cancellation, Save/reopen/export and external compilation preserve required bytes and captions.

Validation to execute after implementation is authorized:

- Synthetic shared/duplicate images, caption/label context, failed insertion and external-file change cases.
- Owner-run supported image workflow with accessible command alternatives.

Current source anchors: `src-tauri/src/service/project_workflows.rs:473`, `crates/texloom-core/src/adapters.rs:192`.

## OBJECT-02: Add bounded visual table-cell and structural editing adapters

GitHub: [#66](https://github.com/nadeemtsf/texloom/issues/66) — open/backlog.

Phase: authoring; priority: P2; type: feature.

Depends on: `TYPE-05`, `CORE-01`, `CORE-02`, `ARCH-09`.

Current table parameter controls and ordinary-text refusal do not support arbitrary cell/row/column authoring.

Declare a finite supported tabular/table subset with cell replacement, row/column actions, captions and navigation. Keep multirow/custom expansion/complex math or unsupported structures opaque until separately designed.

Acceptance:

- [ ] Cell identity and source separators/scopes remain correct under replace/insert/delete/reorder.
- [ ] Rules, column modifiers, captions and labels survive supported transformations.
- [ ] Ambiguous expansion or unsupported cell content refuses without partial mutation.
- [ ] History/recovery and external compilation confirm the declared table workflow.

Validation to execute after implementation is authorized:

- Synthetic escaped separators, repeated cells, formatting, malformed structure and negative-control cases.
- Owner-run keyboard/pointer/table navigation and editing acceptance.

Current source anchors: `crates/texloom-core/src/adapters.rs:218`, `crates/texloom-core/src/objects.rs:93`.

## QUALITY-01: Accept keyboard, accessibility and non-drag pointer authoring workflows

GitHub: [#67](https://github.com/nadeemtsf/texloom/issues/67) — open/backlog.

Phase: product; priority: P0; type: quality.

Depends on: `ARCH-07`, `TYPE-02`, `TYPE-03`, `TYPE-04`, `TYPE-05`.

Canvas/frame controls and accessible labels alone do not establish an accessible visual authoring experience.

Design throughout P1 and validate the completed supported authoring workflows against applicable WCAG2.2AA targets plus actual Windows assistive-technology/input behavior.

Acceptance:

- [ ] Supported actions have keyboard and applicable single-pointer alternatives without dragging; keyboard alone does not satisfy the latter requirement.
- [ ] Focus, selection, caret, command availability and status are exposed and restored predictably through recompile/dialogs/history.
- [ ] No keyboard traps or hidden focus; visual mode has usable semantics beyond source-mode fallback.
- [ ] Record actual screen-reader/native findings and limitations; do not claim certification from ARIA or unit tests.

Validation to execute after implementation is authorized:

- Automated semantics/focus checks plus owner-run keyboard, pointer, screen-reader and input tasks.
- Include source/visual transitions, long-document virtualization and refusal/error announcements.

Current source anchors: `web/core/canvas-controller.js:22`, `web/core/canvas-controller.js:98`, `web/app.js:953`.

## QUALITY-02: Accept the declared Word-like product scope on a recorded release candidate

GitHub: [#68](https://github.com/nadeemtsf/texloom/issues/68) — open/backlog.

Phase: product; priority: P0; type: quality.

Depends on: `CORE-01`, `CORE-02`, `CORE-03`, `CORE-04`, `CORE-05`, `TYPE-02`, `TYPE-03`, `TYPE-04`, `TYPE-05`, `OBJECT-01`, `OBJECT-02`, `QUALITY-01`, [#31](https://github.com/nadeemtsf/texloom/issues/31), [#32](https://github.com/nadeemtsf/texloom/issues/32), [#33](https://github.com/nadeemtsf/texloom/issues/33), [#34](https://github.com/nadeemtsf/texloom/issues/34), [#35](https://github.com/nadeemtsf/texloom/issues/35), [#36](https://github.com/nadeemtsf/texloom/issues/36).

Current bounded-core/build passes do not demonstrate the expanded visual authoring product, its scale, usability or deployment readiness.

Review and execute the final supported capability matrix, measured performance/scale, migration, portability, Windows reliability and human workflows. Reuse existing issue gates instead of duplicating their scope. Coordinate versioned research through #37-#39.

Acceptance:

- [ ] A named source/binary/environment candidate completes the declared edit/delete/move/insert/format/object workflows with exact history, Save and recovery.
- [ ] Input feedback, mapping, compile/inspection/publication, tracing overhead and memory/long-session budgets have scoped measured evidence; reading and editing limits remain distinct.
- [ ] Existing install/DPI/clean-environment/dependency/usability gates are met for the claimed release or remain explicit blockers; no missing-equipment pass is invented.
- [ ] Source-native output compiles externally with normal dependencies; no unresolved blocking data-loss/correctness defect remains.
- [ ] Delivery remains private with no public distribution or Store work; report publication/submission is not implied and no universal Word/TeX/security guarantee is inferred.

Validation to execute after implementation is authorized:

- Predeclared synthetic functional/negative/failure corpus, real-engine independent checks and owner-run exact-packaged Windows workflows.
- A fixed-version research study may cover a narrower stable scope earlier; each later expansion gets separately versioned evidence.

Current source anchors: `docs/CORE-COMPLETION-CHECKLIST.md`, `web/core/long-pdf.js:18`, `research/evaluation-plan.md`.
