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
