# Optional local timing diagnostics

Implemented 4 November 2026; native measurements and overhead evaluation are pending. Ordinary builds disable tracing. A separate build with `VITE_TEXLOOM_PERFORMANCE=1` opts in and exposes `window.texloomPerformance.snapshot()`, `.stop()` and `.clear()` for a controlled evaluation harness. Record that build's flag, complete source identity and executable hash. Do not describe it as byte-identical to the ordinary build.

5 November source-only continuation adds `committed-ui-failed` and `commit-unknown` outcomes. A later UI error does not erase a native commit or an already recorded publication milestone; an unusable commit response is not proof of refusal. These new categories and their handling are **untested** because the owner paused every test run. Earlier diagnostic build/check receipts apply to the preceding implementation, not these additions. See COMMIT-PRESENTATION.md.

Later source changes add automatic source highlighting/file activation before the controller's `selection-ready` boundary. Its cost is therefore inside future selection traces on that source version; it was absent from earlier build receipts. Reset position uses the `text-move` category and must be identified by the declared workload condition, not inferred from an unlabelled timing record. No latency or overhead measurement accompanies either change.

The collector records static event categories, sequence numbers and `performance.now()` offsets. It does not record source text, document names, paths, selection content, error messages or identifiers from projects. No trace is automatically saved or transmitted. A snapshot is a detached copy; the evaluator must export it and record its hash before resetting the collector.

## Measurement boundaries

| Category | Start | Operation boundary | Frame proxy |
| --- | --- | --- | --- |
| `source-feedback` | Monaco content-change callback after the editor has changed | App source-change feedback updated and native edit queued | Two animation callbacks, if the same client/revision remains current |
| `source-selection` | Eligible `selectHits` call | Mapping/probe completes and the selection frame is attached | Two callbacks while that selection remains current |
| `object-selection` | Eligible existing-object selection call | Frame attached and focus restored | Same selection freshness check |
| `drag-preview` | First meaningful pointer-move callback in a drag | Outline translation updated | Same gesture still active; one sample per drag |
| `text-move` | Pointer-release handler, or keyboard/inspector move entry | Native commit, source/PDF publication and reselection | Same accepted view/revision remains current |
| `text-edit` | Text operation entry | Same transaction/publication path | Same accepted view/revision remains current |
| `compile-manual` / `compile-auto` | An active compile call, before native source synchronization | Prepared PDF published | Same current, non-lazy view |

Starts are JavaScript handler/method entry, **not physical input arrival**. `source-feedback` excludes OS dispatch and Monaco work before its content-change notification. Compile traces exclude the preceding automatic-compile debounce and time queued behind another active job. Long-document lazy views receive no publication-to-frame latency because publication alone does not establish that the visible page finished painting.

Milestones include native source preparation, build return, PDF.js preparation, frontend geometry validation, build receipt recheck, native commit and publication. A visual `compile_document` reply already includes native independent layout approval; therefore `build-returned` includes the recipe, IPC and native verification work. `build-rechecked` is `verify_build`'s receipt/dependency recheck, **not a separate PDF comparison**. These coarse offsets do not isolate TeX from worker costs. Repeated mapping attempts may repeat milestones.

The endpoint is deliberately named `frame_proxy_ms`: two animation callbacks after DOM publication, with a visible document and an unchanged operation identity. Browser rendering/presentation can differ; occlusion, scanout and the user's perception are not measured. `operation_ms` and `frame_proxy_ms` must be reported separately. Do not call either hardware input-to-display latency. One outline sample per drag does not prove a sustained 60 Hz frame rate.

## Failure accounting and limits

Keep every record, including refusal, compilation failure, stale, cancelled and failed work. These are execution outcomes, not independent correctness labels. `hidden`, `superseded`, `timeout`, `unavailable`, `invalid-clock`, `stopped` and `not-requested` frame states have no observed frame latency. A 2-second presentation timeout does not cancel compilation; it only withholds a frame estimate. An interrupted trace remains incomplete on `.stop()`.

Storage holds 256 completed records, at most 16 active operations, and 64 milestones per operation. Evictions, dropped starts and truncated milestones are counted. Export small batches; a final timing study must reject or explicitly account for nonzero losses/pending records. The collector does not infer missing events as successes. `.clear()` intentionally clears both pending/completed diagnostics and loss counters; use it only between recorded batches.

## Required evaluation before performance claims

1. Build and hash ordinary and diagnostic variants from the same recorded source/dependency state. Keep the same release optimizations and machine configuration.
2. Check trace boundaries against actual Windows actions and external observation after the owner returns. Confirm publication, selection and cancellation on the exact diagnostic executable.
3. Seal a timing workload and an external measurement method. Randomize paired ordinary/diagnostic order, including warmups, to estimate tracing overhead. The ordinary build cannot supply its own trace-based overhead measurement.
4. Collect declared warm/cold, failed/censored and successful samples. Record the machine, WebView2, engine, font, dependency and display configuration. Report distributions by category and condition, with counts and losses.
5. Compare with the predeclared targets only for the boundaries actually measured. Leave physical presentation, sustained gesture-frame timing and usability claims pending unless separately measured.

Headless tests establish bounded storage, milestone plumbing, failure/pending handling and absence of an API/clock sampling in disabled mode. They do not establish real app latency, acceptable overhead or native usability. No Texloom window was opened for these tests.

Primary API basis: [W3C High Resolution Time](https://www.w3.org/TR/hr-time-3/) defines the monotonic clock (published working draft checked 4 November 2026); [WHATWG animation frames](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#animation-frames) defines animation callbacks. Neither specification turns two callbacks into proof of physical display presentation.
