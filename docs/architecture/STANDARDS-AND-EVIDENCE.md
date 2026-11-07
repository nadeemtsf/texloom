# Standards, established patterns and current-source evidence

Reviewed 7 November 2026. This is a design basis, not a certification or exhaustive standards survey. No single generally adopted architecture standard prescribing a Word-like editor for arbitrary LaTeX was found in this review. Component standards and proven editor patterns still provide useful requirements.

Public ISO abstracts were inspected, not the purchased full normative texts. W3C draft status is recorded explicitly. Borrowed editor patterns do not imply a dependency or framework replacement. Source/tool retrieval failures are not evidence of absence.

## Standards and guidance

| ID / primary source | Status and inspected scope | Application to Texloom | Limit |
| --- | --- | --- | --- |
| S01 [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) | Published architecture-description standard; public abstract inspected | Record stakeholders, concerns, views, decisions and their relationships | Describes architecture descriptions; does not prescribe this product architecture or certify this document |
| S02 [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) | Published product-quality model; public abstract inspected | Set measurable functional, reliability, interaction, performance, compatibility and maintainability goals | Quality-model use is not conformance, security clearance or passing evidence |
| S03 [ISO 9241-210:2019](https://www.iso.org/standard/77520.html) | Published human-centred-design standard; public overview inspected | Validate actual author tasks and iterate on observed usability | Does not supply a complete interaction design; full clauses were not audited |
| S04 [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | W3C Recommendation, 12 December 2024 | Target applicable AA keyboard, focus, status and dragging-alternative criteria | This is a future tested target for web content; Windows/assistive-technology behavior still needs actual evaluation |
| S05 [ARIA APG keyboard guidance](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/) | Implementation guidance | Distinguish focus from selection and define predictable navigation/focus return | ARIA roles do not provide keyboard behavior automatically |
| S06 [Input Events Level 2](https://www.w3.org/TR/input-events-2/) | Working Draft, 15 October 2026 | Intent-oriented input, replacement/deletion/paste and composition lifecycle | Draft behavior and cancelability require WebView2 compatibility checks |
| S07 [Selection API](https://www.w3.org/TR/selection-api/) | Working Draft, 11 June 2026 | Anchor/focus/direction inform the input adapter | DOM selection is not canonical source identity or a complete PDF caret solution |
| S08 [EditContext](https://www.w3.org/TR/edit-context/) | Working Draft, 10 September 2026 | Candidate input-context/geometry integration for a custom visual surface | Feasibility spike only; no baseline dependency or runtime-support claim |
| S09 [Clipboard API and events](https://www.w3.org/TR/clipboard-apis/) | Working Draft, 24 June 2026 | Explicit clipboard intent, focus/permission and failure handling | Cut must not delete source after a failed clipboard write; runtime behavior must be checked |
| S10 [UAX #29 revision 49](https://www.unicode.org/reports/tr29/tr29-49.html) | Stable Unicode 18.0 text segmentation document inspected | Separate bytes/UTF-16/code points/graphemes and declare word-boundary policy | Record the actual implemented runtime/library version; a recent document does not upgrade WebView2 or Rust automatically |
| S11 [UAX #9 revision 52](https://www.unicode.org/reports/tr9/tr9-52.html) | Stable Unicode 18.0 bidi document inspected | Logical order, visual order and caret affinity need separate representations | Bidi rules alone do not solve TeX shaping, ligatures or source mapping |

For S04, explicitly consider SC 2.1.1, 2.1.2, 2.4.7, 2.4.11, 2.5.7 and 4.1.3. A keyboard shortcut alone does not meet the separate single-pointer alternative-to-dragging requirement. Accessible authoring also needs a usable editing/input representation, not only a canvas description.

## Established patterns, not mandatory standards

| ID / primary source | Useful pattern | Boundary |
| --- | --- | --- |
| P01 [ProseMirror transformations](https://github.com/ProseMirror/website/blob/master/markdown/guide/transform.md) and [state](https://github.com/ProseMirror/website/blob/master/markdown/guide/state.md) | Explicit transactions, selection state, mappings and history metadata | Use principles; do not force arbitrary LaTeX into its document schema |
| P02 [CodeMirror change model](https://github.com/codemirror/state/blob/main/src/change.ts) and [selection model](https://github.com/codemirror/state/blob/main/src/selection.ts) | Change sets, mapped ranges and insertion affinity | Numeric mapping is not semantic identity; replacing Monaco is unnecessary |
| P03 [Roslyn syntax model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-syntax) | Preserve complete source/trivia and immutable syntax snapshots | C# is an analogy. TeX needs a supported structural projection plus opaque regions, not a claimed complete universal semantic AST |
| P04 [LaTeX spacing implementation](https://github.com/latex3/latex2e/blob/main/base/ltspace.dtx) and [boxing implementation](https://github.com/latex3/latex2e/blob/main/base/ltboxes.dtx) | Native spacing/boxing has explicit semantics to respect | Standard commands do not automatically preserve current displacement behavior; `raisebox` also boxes its content |

Direct editor-guide retrieval returned 403 in parts of the research. Official indexed guide content and official repository sources were used where available. The design does not depend on uninspected competitor internals. None of these sources establishes Texloom novelty or a universal source-to-layout inverse.

## Current-source audit and gap classification

Anchors below refer to application baseline `6f55baca3100ef8af48d0a3cadb419855c125339`. Later edits can move line numbers; verify before implementing.

| Finding | Evidence | Assessment / planned task |
| --- | --- | --- |
| Exact canonical sources and inverse patches exist | `crates/texloom-core/src/document.rs:47`, `:54`, `:511` | Retain; extend semantic operations around them, ARCH-04/06 |
| Native draft source editing is separate from visual approval | `crates/texloom-core/src/document/source_edit.rs:1`, `:37`, `:85` | Retain two lifecycles; do not compile-gate each typed character, ARCH-05 |
| BOM/newline/editor-source offset conversion exists | `crates/texloom-core/src/document/projection.rs:1`, `:49` | Extend to typed anchors and grapheme/visual positions, ARCH-03 |
| Selection stores offsets but does not establish semantic rebasing | `crates/texloom-core/src/document.rs:110`, `:267`, `:397` | Add change maps, affinity and invalidation, ARCH-03 |
| Editable units are bounded source ranges; wrapped objects have macro IDs | `crates/texloom-core/src/objects.rs:6`, `:17`, `:29`, `:169` | Add partial entity/capability projection, ARCH-02 |
| Visual operation family and text actions are movement-centric | `src-tauri/src/service/layout_gate.rs:16`; `service/visual.rs:137`, `:269` | Typed editing intents and independent source-native route, ARCH-04/10 |
| Text planner always ensures custom support/package | `src-tauri/src/service/visual.rs:393` | Isolate compatibility; new source-native operations need a different plan path, ARCH-10 |
| Native approval, receipts and cancellation are strong reusable boundaries | `src-tauri/src/service/layout_gate.rs:50`, `:60`; `crates/texloom-build/src/snapshot.rs:183`, `:307`; `scheduler.rs:37` | Preserve native authority and artifact binding |
| Semantic policy permits reflow and validates readable inspection | `src-tauri/src/service/layout_gate.rs:374` | It is not a general edit-preservation proof; ARCH-09 |
| Current normal selection begins with complete PDF lines | `web/core/canvas-controller.js:20`; `web/app.js:396` | Add text-range/caret/node selection family, ARCH-03/07 and TYPE-02 |
| Stale-source and replaced-PDF DOM cannot support a persistent caret directly | `web/app.js:131`, `:157`, `:453`; `web/core/pdf-surface.js:74` | Logical state must survive rendering; ARCH-05 and TYPE-02 |
| Positioned-block insertion and deletion already exist | `web/app.js:782`, `:927`, `:930` | Preserve accurate scope; this is not general in-flow typing or normal-text Delete |
| Normal text editing is modal and frame keyboard handling is limited | `web/app.js:626`; `web/core/canvas-controller.js:98` | CORE-01/04 and TYPE-02/03 |
| Frontend already distinguishes uncertain commit and failed presentation | `web/core/transactions.js:35`, `:40`; `web/app.js:518` | Preserve distinctions during ARCH-07 extraction |
| Existing composition listeners belong to source-mode Monaco | `web/app.js:841`, `:846` | Reuse lessons; visual input still needs its own adapter, TYPE-03 |
| Native spacing parameters can be edited directly | `crates/texloom-core/src/adapters.rs:200`, `:612` | Reuse for CORE-03; canvas intent/verification are additional work |
| Asset import writes a real project asset outside text-only history | `crates/texloom-core/src/document.rs:54`; `src-tauri/src/service/project_workflows.rs:473`, `:514` | Existing behavior is not a demonstrated data-loss bug; future media transactions need ARCH-08 |
| Unwrap removes the displacement while retaining content | `crates/texloom-core/src/objects.rs:257` | Not appearance-preserving migration; CORE-05 |
| Long-document reading and editing limits differ | `web/core/long-pdf.js:18`; `web/core/pdf-surface.js:20` | Preserve declared budgets and independent projection state; QUALITY-02 |

The source audit itself was read-only. After the owner additionally requested private-project cleanup, the existing 291 frontend and 70+18 core checks were rerun successfully for the metadata changes. No proposed feature, real-engine or native-input acceptance was executed. See [validation scope](VALIDATION.md). Existing recorded results retain their version/fixture scopes. Planned capability does not imply implementation, and implemented source does not imply current Windows acceptance.
