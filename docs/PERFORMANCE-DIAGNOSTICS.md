# Optional local timing diagnostics

## Native compile baseline by page count

The `compile_profile` example measures the production build service on disposable 1-, 20- and 100-page projects. Each page has one numbered sentence. Each trial starts with a fresh TeX snapshot. The two conditions alternate order and differ in whether they reuse the compiler identification probe. The initial pair warms the profile and is excluded from the results.

On Windows 11 Pro (build 26300) with an Intel i7-11370H and MiKTeX pdfTeX 4.23, five measured pairs per size gave these medians in seconds.

| Pages | Fresh identification | Reused identification | TeX processes with reuse | Build input checks with reuse |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1.514 | 1.275 | 1.167 | 0.067 |
| 20 | 1.756 | 1.155 | 1.065 | 0.058 |
| 100 | 1.756 | 1.444 | 1.342 | 0.068 |

All 15 measured pairs agreed on page count, extracted text, 36 dpi page rasters and the compared build-manifest fields. The build service also checked the recorded dependency receipts and original source. These sparse fixtures measure ordinary compilation through its native return. Package-heavy documents, PDF transport and painting, and independent PDF verification during visual edits need separate measurements. See [timing issue #32](https://github.com/nadeemtsf/texloom/issues/32).

Run from the repository root on Windows with pdfLaTeX and Poppler tools available. The output directory must be new. The comparison expects five measured pairs per size.

```powershell
$output = New-Item -ItemType Directory artifacts/page-count-baseline
$compiler = (Get-Command pdflatex.exe).Source
cargo run --locked --release -p texloom-build --example compile_profile -- (Join-Path $output.FullName 'paired-pdflatex') $compiler 5 --pages > (Join-Path $output.FullName 'paired-pdflatex.jsonl')
python research/tools/compare_compile_profiles.py $output.FullName (Join-Path $output.FullName 'summary.json') --pages
```

## Independent PDF inspection by window size

The `inspection_profile` example measures the packaged PDFium worker and host receipt checks against a fixed PDF. It alternates four- and eight-page requests and compares page geometry, links, text, graphics and raster bytes in physical page order. It rejects changes in page coverage or evidence. The separately recorded total includes the benchmark's comparison digest.

Three paired runs on the 47-page technical report used packaged PDFium 159.0.8138.0. The PDF SHA-256 was `3cc99ec79b9af64fa97091c6502317e39001c108ce6080ef837c49c96fa209c8`. All six runs returned equivalent checked evidence. The earlier three-pair run had worker medians of 4.84 and 3.69 seconds, respectively.

| Pages per request | Requests | Median worker and host checks (s) | Median including benchmark digest (s) | Largest returned raster payload (MiB) |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 12 | 4.08 | 4.66 | 22.97 |
| 8 | 6 | 3.59 | 4.17 | 45.94 |

This measures one unmarked PDF. A normal nonempty visual edit can inspect the original and candidate PDFs plus their two tagged probes. The benchmark excludes the TeX builds, comparison between tagged and unmarked PDFs, PDF.js and presentation. Production retains the four-page window. A larger window uses more memory and needs coverage on denser pages and larger media before adoption.

Use the verification directory from a trusted packaged Windows build. Run from the repository root and give the benchmark a new scratch directory. The example writes one JSON record per run.

```powershell
cargo run --locked --release -p texloom-build --example inspection_profile -- $pdf $verificationDirectory artifacts/inspection-profile 3
```

### Packaged report edit trace

The Windows diagnostic build for runtime source `0ae10ae` (executable SHA-256 `e3e27823a3d557e916d29d399412626035b6a567abf84817d8dd9fd2a03472cf`) ran the native WebView2 regression on a disposable copy of the full report. Both edits saved in their included file, matched a clean compile, and restored the original source on Undo.

| Edited page | Input to verified text hit (s) | Native build and approval reply (s) | PDF.js preparation (s) |
| ---: | ---: | ---: | ---: |
| 2 | 46.00 | 43.97 | 0.87 |
| 21 | 46.55 | 44.31 | 1.04 |

The native interval runs from `native-prepared` to `build-returned` and combines TeX, independent inspection and the reply. PDF.js preparation runs from `build-returned` to `pdf-prepared`. The snapshots include these milestones and publication while the traces are still active. The input-to-hit time includes the harness polling interval and measures WebView text availability. Two samples do not establish a latency distribution or physical display timing.

## Desktop tracing

Implemented 4 November 2026; native measurements and overhead evaluation are pending. Ordinary builds disable tracing. A separate build with `VITE_TEXLOOM_PERFORMANCE=1` opts in and exposes `window.texloomPerformance.snapshot()`, `.stop()` and `.clear()` for a controlled evaluation harness. Record that build's flag, complete source identity and executable hash. Do not describe it as byte-identical to the ordinary build.

5 November source-only continuation adds `committed-ui-failed` and `commit-unknown` outcomes. A later UI error does not erase a native commit or an already recorded publication milestone; an unusable commit response is not proof of refusal. These new categories and their handling are **untested** because the owner paused every test run. Earlier diagnostic build/check receipts apply to the preceding implementation, not these additions. See COMMIT-PRESENTATION.md.

Later source changes add automatic source highlighting/file activation before the controller's `selection-ready` boundary. Its cost is therefore inside future selection traces on that source version; it was absent from earlier build receipts. Reset position uses the `text-move` category and must be identified by the declared workload condition, not inferred from an unlabelled timing record. No latency or overhead measurement accompanies either change.

The next source-only revision changes exports to **schema 2**: a per-launch local random collector identity, clear epochs, issued-ID boundaries, explicit discard/invalid-start counters and cumulative erased-row counts. Disabled ordinary builds do not request an identity. Missing randomness leaves identity null rather than fabricating one. The unexecuted batch-analysis draft separates endpoints/outcomes and withholds distributions for incomplete current epochs. These additions are **unbuilt and untested**; old schema-1 receipts remain unchanged. See [timing record specification](../research/TIMING-RECORDS.md).

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

Storage holds 256 completed records, at most 16 active operations, and 64 milestones per operation. Evictions, dropped starts, explicit discards, invalid-clock starts and truncated milestones are counted. Export small batches; a final timing study must reject or explicitly account for nonzero losses/pending records. The collector does not infer missing events as successes. `.clear()` clears current pending/completed diagnostics and epoch loss counters, increments the epoch and retains cumulative erased-row counts; use it only between recorded batches.

## Required evaluation before performance claims

1. Build and hash ordinary and diagnostic variants from the same recorded source/dependency state. Keep the same release optimizations and machine configuration.
2. Check trace boundaries against actual Windows actions and external observation after the owner returns. Confirm publication, selection and cancellation on the exact diagnostic executable.
3. Seal a timing workload and an external measurement method. Randomize paired ordinary/diagnostic order, including warmups, to estimate tracing overhead. The ordinary build cannot supply its own trace-based overhead measurement.
4. Collect declared warm/cold, failed/censored and successful samples. Record the machine, WebView2, engine, font, dependency and display configuration. Report distributions by category and condition, with counts and losses.
5. Compare with the predeclared targets only for the boundaries actually measured. Leave physical presentation, sustained gesture-frame timing and usability claims pending unless separately measured.

Previously recorded headless tests cover the earlier collector's bounded storage, milestone plumbing, failure/pending handling and absence of an API/clock sampling in disabled mode. They do not validate the subsequent schema-2 revision or establish real app latency, acceptable overhead or native usability. No Texloom window was opened for those tests. All new diagnostics and app measurements remain paused.

Primary API basis: [W3C High Resolution Time](https://www.w3.org/TR/hr-time-3/) defines the monotonic clock (published working draft checked 4 November 2026); [WHATWG animation frames](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#animation-frames) defines animation callbacks. Neither specification turns two callbacks into proof of physical display presentation.
