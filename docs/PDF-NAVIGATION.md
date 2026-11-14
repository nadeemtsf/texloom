# PDF preview scrolling and zoom

12 November 2026. The r9 owner check failed: zoom felt very choppy and did not stay under the pointer. The complaint is qualitative, not a measured five-frames-per-second result. Historical build/test passes remain recorded and do not override that failure. Issue #69 stays open for acceptance of the corrected build.

## Corrected implementation

The viewer now uses the existing pinned PDF.js 6.2.108 public `PDFPageView`, its event bus and matching viewer stylesheet. The short and long-document paths share that renderer. It uses PDF.js's existing `update({scale,drawingDelay})` cached-canvas CSS scaling instead of rebuilding the PDF DOM or drawing every page for each wheel pause. After 400 ms of inactivity, a bounded queue paints nearby pages sharply. Offscreen bitmaps remain scaled until approached. This is implementation behavior, not a measured physical presentation latency.

Pointer zoom captures the actual PDF point using `getPagePoint` and restores it through viewport conversion. Fit/buttons use the viewport center; the browser clamps scroll at document edges. PDF points, CSS pixels and device pixels remain distinct. The SDK uses pixel rounding at fractional display scales and downsamples canvases when its raster budgets require it.

Ordinary wheel/two-finger input scrolls, preserving fractional two-axis deltas and line/page units. Ctrl-wheel/pinch events zoom only the PDF (20%–250%), including suppressing app zoom at limits. Horizontal trackpad and Shift-wheel scroll wide pages. Navigation cancels a held drag/marquee. Source controls and hits are fenced while zoom refines; a reused page does not accumulate hit buttons or event listeners. No navigation operation compiles source, saves files or changes undo history.

Candidate views still prepare their complete supported short PDF before publication. Long views remain reading/source-editing only; independent visual proof budgets are unchanged. PDF scripting and automatic external-link activation are not enabled. The loaded document remains owned by the app/candidate; disposing a view releases its renderer work without destroying the shared loading task.

## Evidence and limits

303 frontend tests pass. The real headless Edge renderer passes 21 development checks on deterministic six- and 25-page English PDFs at display scales 1, 1.5 and 2. They check cached-canvas reuse/no immediate paint, pointer coordinates, nearby sharp drawing, rapid Ctrl-event coalescing, subsequent scroll, text-layer alignment, Fit/edge clamping and replacement of a document during redraw. A new 139-check core run and Windows/NSIS build pass. Exact r10 source is `663b7e2`; EXE SHA-256 starts `12056133f162`. All 15 portable members and executable/verifier binding pass independent checks. The setup file is hashed; its embedded payload was not separately extracted. [Artifact receipt](evidence/pdf-navigation-r10-artifacts-2026-11-12.json).

Receipts: [corrected viewer](evidence/pdf-navigation-correction-2026-11-12.json), [r9 including owner failure](evidence/pdf-navigation-2026-11-10.json). Retained development failures distinguish missing VM exports and corrected browser rounding assertions from product observations. Screenshot review confirms the synthetic text rendering; it does not establish physical input quality.

The opt-in reproduction entry is `scripts/check-pdf-navigation.mjs`. Set `PLAYWRIGHT_MODULE` to an installed Playwright module when it is not a project dependency; `PDF_TEST_BROWSER` defaults to `msedge`, and `PDF_TEST_OUTPUT_DIR` records JSON and a screenshot. It runs a separate headless fixture, never the Texloom executable or real projects. No extra package was added for the viewer.

The final owner check must use the newly named r10 build, not the installed shortcut or r9: scroll; zoom over a specific word in both directions; check immediate motion and pointer anchoring except at clamped edges; release Ctrl and scroll; Fit; horizontal/Shift-wheel; Read text and reselect/edit after zoom. Confirm source and undo history remain unchanged by navigation. Mouse/precision-trackpad feel, actual Windows DPI/monitor behavior and release acceptance remain unrun until the owner reports them.

## Primary implementation references

- [Mozilla PDF.js page view, pinned v6.2.108](https://github.com/mozilla/pdf.js/blob/v6.2.108/web/pdf_page_view.js): existing cached-canvas CSS update, delayed drawing and text-layer behavior.
- [Mozilla PDF.js viewer, pinned v6.2.108](https://github.com/mozilla/pdf.js/blob/v6.2.108/web/pdf_viewer.js): zoom origin, page-location and viewport scaling patterns.

No Overleaf application code is copied. Texloom uses its already included PDF.js dependency, with Mozilla notices retained in the distribution.


## Owner r10 follow-up and targeted correction

The owner reports ordinary scrolling is good enough, but Select objects fails with either preview-scale or normal-document-flow errors, and laptop touchpad zoom fails with or without Ctrl. Other checklist outcomes are not assumed. The r10 feature is not accepted. Its source/artifact remains immutable.

The first error is reproduced: nearby-only zoom refinement left offscreen canvases at old resolutions, but selection identity compares the entire short PDF against a freshly rendered probe. Before a new identity proof, the shared renderer now paints all bounded short-view pages at the captured scale/current DPR. Exact pixels, glyphs, markers, page structure, budgets and wrong-occurrence checks are retained. The canonical PDFPageView canvas is used, zero-size canvases refuse, and a changed scale/DPR/source during proof preparation invalidates it. Explicit existing raster budgets replace the SDK's changing screen-area heuristic; retained/in-progress old bitmaps are additional transient memory, not an advertised memory cap.

The second error is reproduced before a page break: the unit resolver swallowed standalone page-flow commands. Exact newpage/clearpage/cleardoublepage/pagebreak/nopagebreak commands now bound the paragraph without entering its wrapper. Inline structural content and grouping across the command still refuse.

For laptop pinch, pinned Tauri/wry disabled native zoom input by default. The configuration now enables that delivery path. Browser-default wheel zoom outside the PDF and browser zoom keys are canceled, while the PDF's Ctrl-wheel handler consumes its own events. This is SDK configuration and existing input routing, not a new gesture recognizer. [Microsoft documents the separate pinch setting](https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2settings.ispinchzoomenabled). Actual hardware behavior requires owner confirmation.

306 frontend,147 core and148 recovery-serde checks pass. The expanded headless suite passes33 cases, including initial/post-zoom selection, emulated density change, outside-pane zoom suppression and rejection of offscreen non-text changes. A separate six-pair corpus compiles nine baseline/probe PDFs under all three engines and passes strict PDF.js identity for text immediately before/after page breaks after zoom. It bypasses native SyncTeX/UI interaction and does not establish physical gestures. The dated owner-follow-up receipt preserves the original failures and test-harness corrections. The exact next Windows artifact and owner acceptance are recorded separately.

Owner scope: finish this feature and exposed selection regressions only; wait for owner tests and fix failures until they pass, then stop. Other implementation/research work is on hold. The branch/push question is deferred until this feature passes; no push is authorized.


The exact r11 build is now source `e215bb6`, EXE SHA-256 `a2479afe19f846245b87c131b521e0b0689f6add77afa0863822b09de276470c`. Windows/NSIS build, all 15 portable member CRCs/bytes/hashes, executable/verifier binding and AMD64 header pass. The installer payload is not separately extracted. The portable app and fresh six-/25-page fixtures/checklist are in the chat outputs. No agent GUI launch/control/install occurred. See `docs/evidence/pdf-navigation-r11-artifacts-2026-11-12.json`. The feature is awaiting owner physical selection/pinch/scroll/Read text/gesture-cancellation observations; it is not complete from these automated checks.


## Owner r11 acceptance and final r12 amendments

The owner reports that every item in the exact r11 checklist works, including physical laptop pinch, selection before/after zoom, Read text, held-drag cancellation and long-document navigation. This is an owner-reported named-build acceptance, not an instrumented latency or universal hardware result. The earlier pending statements above retain their historical scope.

The owner requests two final changes: 25% more gesture response and automatic compilation when an existing project is opened through Projects. R12 increases the shared pinch/Ctrl-wheel coefficient from .0015 to .001875. Valid path, Browse and recent opens request exactly one preview compile after successful adoption and unlocking, even when the Auto typing checkbox is off. Invalid paths retain the current project. Source-only/recovery-pending/non-.tex inputs and create/import/conversion retain their existing review behavior. The standalone Open project toolbar route is unchanged.

315 frontend,147 default-core and33 real headless navigation checks pass. The stronger gesture exposed an unchanged 96 MiB selection-limit refusal when combined with the highest emulated DPR; the retained failure is recorded, and the density-only test now sets moderate zoom before changing DPR. No proof budgets or exact identity checks were relaxed. The new app-shell test deliberately fails its mock compiler to check that auto-open reaches compilation after unlock and surfaces the compile error. It does not claim a real native compile. R11 engine/recovery evidence is reused only for its unchanged-code scope. The r12 Windows/NSIS build and all 15 portable member CRCs/bytes/hashes, desktop/verifier binding and AMD64 header now pass. Source is `c93799b5cce542db3f7fdc04882d424f797c07f5`; EXE SHA-256 is `941799c70c2fddd8450d94bdc57497e61073e0b618c9a92fe4789e1dc289963e`. The installer payload was not separately extracted. Portable app, fresh synthetic fixtures and a focused checklist are exported. Owner r12 acceptance remains pending. See [r12 artifacts](evidence/pdf-navigation-r12-artifacts-2026-11-12.json). See [r12 follow-up](evidence/pdf-navigation-r12-followup-2026-11-12.json).

After the owner accepts these final amendments, record the result, answer the deferred branch/push question and stop; no other development or research is authorized in this slice.
