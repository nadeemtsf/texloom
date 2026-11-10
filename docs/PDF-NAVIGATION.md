# PDF preview scrolling and zoom

10 November 2026. The persistent compiled-PDF viewport now owns wheel/trackpad navigation independently of source editing and compilation.

- Mouse wheel and ordinary two-finger trackpad input scroll the PDF vertically. Horizontal trackpad deltas remain horizontal; Shift plus a vertical wheel scrolls horizontally when the page is wider than the viewport.
- Ctrl plus wheel zooms the PDF, with the existing 20%-250% limits. Trackpad pinch events represented as Ctrl-wheel use the same route. The event is prevented from zooming the whole application shell, including at the limits or when no PDF is available.
- Rapid zoom input updates the requested scale immediately and coalesces PDF repaint after 80 ms of inactivity. It reuses the loaded PDF and the existing page/fraction-at-viewport-center preservation path. This is not an observed latency measurement.
- Fit and the zoom buttons cancel a pending wheel repaint. Scroll retains fractional device deltas, normalizes line/page units and clamps to the viewport bounds; it does not apply a second native scroll.
- A navigation that changes the viewport cancels an in-progress object drag/marquee preview. It does not apply a source transformation. No-op input at an edge does not cancel a gesture unnecessarily.

`web/core/pdf-scroll.js` owns the input adapter, scheduling and cleanup. The app only supplies current scale, visual updates, repaint and gesture cancellation. There is no source/compiler/history API in the adapter. The CSS explicitly permits the viewport to shrink inside its flex layout and contains scroll chaining.

## Evidence and limits

The r9 Windows release build is source `5a506aa1923fcdcc980a98c102f5eac99f1b8266`, with executable SHA-256 `7ec277eff23ba3ad4d1075891c0232957f71ea0a15a8d18c4b30cb8e4232b6c7`. The portable ZIP has 15 independently checked members/CRCs; executable/verifier pairing agrees. Final frontend checks: 300 passed; default/recovery core: 128/129 passed. The owner packet includes independently preflighted six- and 25-page synthetic documents and exact-build instructions. These do not establish native input or installation acceptance. See `docs/evidence/pdf-navigation-2026-11-10.json`.

The first full frontend run retained in `artifacts/anchors-20261110/frontend-scroll.log` failed because two VM-based app tests strip imports and had not injected the new module. The harnesses now use the real adapter. This was a test integration failure, not a reproduced Windows input observation. Separate adapter/controller tests cover fractional two-axis input, mouse line/page units, boundaries, zoom coalescing/limits, teardown and absence of source/history/compile effects. Final counts and build/artifact identities are recorded separately after checks finish.

Actual mouse-wheel, precision-trackpad/pinch feel, source/selection preservation and held-drag cancellation must still be observed by the owner in the exact rebuilt app. A headless test or successful web/native build cannot complete that gate. The agent does not open/control/install Texloom.

## Owner check on the new build

1. Open a disposable multi-page document, compile and hover over a PDF page or its selectable text. Wheel/two-finger scroll should move through pages; the source and Undo history should stay unchanged.
2. Hold Ctrl and wheel both directions. Only the PDF should resize, the app toolbars should stay the same size and the same reading page should remain near the center. Release Ctrl and verify ordinary scrolling resumes. Try Fit afterward.
3. At a zoom wide enough to overflow, use a horizontal trackpad gesture or Shift-wheel. Check the edges and a rapid series of zoom events. If available, test a pinch.
4. Begin a disposable drag/marquee and then scroll or zoom. The preview should cancel and leave the original object/source unchanged. This remains an owner-run check.

The earlier r8 executable does not include this implementation. Use only the newly recorded binary for these observations. No results are assumed from the written instructions.
