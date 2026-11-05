# Original feature scope: source coverage, not acceptance

Updated 5 November 2026 after the owner asked to keep implementing while tests stay paused. This maps the approved initial workflow to actual source, rather than treating an earlier build or a working code editor as completion. New UI/failure-handling source is unbuilt and untested. A listed implementation is not a claim that its current Windows behavior passed.

| Approved behavior | Source implementing it | Evidence boundary |
| --- | --- | --- |
| Open normal multi-file projects, choose main/engine, edit/find source | `web/app.js`, `core/project-controls.js`, `core/document-client.js`; native project/document service | Earlier versioned native workflows exist; current source needs regression/native acceptance |
| Compile immutable snapshots; keep reading access after failures | `crates/texloom-build`, `core/transactions.js`, `core/pdf-surface.js` | Prior engine/reading evidence; latest cleanup changes untested |
| Read/copy/search/zoom long PDFs separately from layout selection | `core/pdf-search.js`, `core/long-pdf.js`, `core/pdf-surface.js` | Prior native and renderer receipts, not current resource/latency certification |
| Select a supported line/group/whole paragraph, with visible scope and hover outline | `core/canvas-controller.js`, `core/selection-proof.js`, native selection resolver, CSS | Earlier signatures/repeated-text/refusal evidence; unsupported mappings still refuse |
| Highlight corresponding source when selecting PDF text, without stealing keyboard focus | New `core/source-highlight.js`, guarded file activation in `app.js` | Previously required explicit Show source; new automatic link is source-only and untested |
| Drag/nudge with preview, then one verified source/history operation | Canvas controller, native operation planning/build approval/commit, transactions | Earlier real-engine/native cases; held-gesture/DPI/current-source repeats pending |
| Reset displacement without removing the movable text; remove positioning without undoing wording edits | New Reset position action reuses existing checked move-to-zero; separate existing detach operation now labelled Remove positioning | Dedicated reset action and clearer distinction are new, untested source |
| Edit literal text while retaining supported formatting and protecting commands; raw-fragment fallback | Existing native literal/plain/spacing planners; new formatting/direction hints and protected-command presentation in `core/text-input-appearance.js` | Transformation mechanisms have earlier evidence; new presentation and modal behavior untested |
| Open the text editor near the selected unit and keep keyboard/modal access | New `core/dialog-anchor.js` plus existing dialog controls | Places alongside visible target only if it fits; otherwise retains centered modal; new resize/placement behavior untested |
| One source/visual Undo/Redo, explicit Save/export and local recovery | Rust document/session/persistence/recovery, document client and app wiring | Earlier exact-byte/recovery receipts remain versioned; new UI/uncertain-commit paths need acceptance |
| Images/aspect ratio, bounded table/font/spacing controls, templates, project/encoding transfer | Native adapters/project tools and `core/project-controls.js` | Existing bounded grammars/cases, not arbitrary LaTeX editing or exhaustive package support |
| Setup explanations and local optional timing collection | `core/setup-help.js`, `core/performance-diagnostics.js` | Source/build evidence at earlier versions; no clean-machine success or measured end-to-end latency |

## Details of the newly completed source gaps

Automatic source highlighting uses the already validated BOM-free, LF-normalized UTF-16 ranges returned by the native protocol. It checks file/range/current-source identity, uses Monaco decorations rather than a source edit, and preserves PDF-frame focus. File activation restores the previous editor/file pairing if display setup fails; if restoration cannot be completed, the existing synchronization-error state stops further writes. Ordinary source typing, file changes, selection clearing and Read text mode clear the decoration. Explicit Show source remains a separate keyboard-focus/navigation action. No string-search guess or source transformation is added by highlighting.

Reset position requests zero horizontal/vertical displacement through the existing checked movement path. It preserves the current text and movement capability. Remove positioning uses the existing detach operation and preserves current content. Neither is Undo, and clicking reset must not restore an older name. Reset is unavailable for an unwrapped source selection or an already-zero object.

Literal-run inputs use only a small allowlist of native style labels for visual hints: bold, italic/slanted, small caps, underline and generic font families. Plain/literal text uses automatic direction; raw LaTeX stays left-to-right. Protected command names are inserted as text, deduplicated and visibly bounded; they are shown only in the protected literal-edit mode. The advanced raw-fragment option hides that guarantee. This is not exact TeX font/color/shaping reproduction; the compiled PDF remains the typeset view.

The modal retains native dialog semantics. Placement uses the selected page's current viewport rectangle, tries adjacent positions inside the viewport and falls back to centering when there is insufficient room or usable target geometry. Content/window resize triggers repositioning; maximum height and scrolling keep the close/apply controls reachable. These are intended properties of unexecuted code, not observed DPI/keyboard/accessibility results.

## What still prevents calling the app complete

All tests are paused, so new source has neither automated/type tests nor native acceptance. The normal build includes protocol type tests; no alternate path was added to bypass them. Exact current build/installation, held-gesture cancellation, display-scale transitions, setup/removal, timing/overhead/resources and unassisted use remain open. Further reproducible failures may still require implementation fixes.

Arbitrary math/TikZ/table-cell editing, cross-page manipulation and additional languages are later scope. The optional Lua sidecar/resident-engine research directions are alternatives to investigate if justified, not missing mandatory features to add without evidence. Public distribution, signing and updates remain later owner decisions. Current work does not authorize tests, app launch, installation, publication or submission. Never compile `.text` files.
