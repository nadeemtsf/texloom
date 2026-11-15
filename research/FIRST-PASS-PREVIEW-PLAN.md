# First-pass preview engineering protocol

## Direct-engine feasibility phase

The predeclared protocol and script were saved before execution under ignored
`artifacts/compile-20261115/first-preview-spike/`. Its 12 conditions use one
fresh synthetic project for each combination of pdfLaTeX/XeLaTeX/LuaLaTeX
and plain/reference/contents/deliberate later failure. The first three take
three direct engine passes; the last takes two. Retain per-pass PDF copies,
logs, text and full 96 dpi page raster hashes. This phase bypasses the build
service and does not grant application authority or measure app presentation.

The observed control outcomes motivate the next phase: plain first/last
output matches, reference/contents output changes then settles, and an early
readable PDF can precede a failed second pass. Raw artifacts remain separate
from subsequent production-service runs.

## Production-service observation phase

Record this extension before execution. Add an opt-in first-pass observer to
the existing supervised build library. Leave ordinary desktop builds and
their convergence rules unchanged. This is preparation for a later explicitly
provisional UI, not an implemented early-preview feature.

The observer may receive owned PDF bytes after the first successful process,
runtime-banner check, recorder discovery and frozen-input/dependency check.
It receives an explicitly unverified type with diagnostics and elapsed native
time. It receives no verified-build receipt, mapping or export capability.
Do not send truncated, missing, oversized or non-PDF-header output. No observer
means no early PDF read/copy. Cancellation is checked before and after the
callback; it must stop subsequent passes rather than make a final build.

Use fresh disposable synthetic snapshots across all three installed engines.
For plain/reference/contents/late-failure conditions, verify exactly one event
after pass one. Save its immutable bytes and later compare to the settled
result using text and full-page rasters. The first three conditions still need
the existing convergence passes and receipt verification. The later-failure
control must remain failed and ineligible despite the early event. Add a
cancellation-at-callback control and a first-pass compile-error control.
Compare no-observer and observed results for source/output equivalence.

Measure callback availability and final return separately with the production
service's monotonic clock. One run per engine/condition is descriptive; no
quantiles, confidence interval, visible-latency result or app speedup follows.
Preserve source/compiler identities, exact arguments, pass counts, diagnostics
and original-source checks. Run the normal gate and GitHub checks before merge.

Later native transport and UI work must bind project/revision/request identity,
discard stale/cancelled events, keep the last verified PDF available on failure
and withhold geometry/edit/export authority from provisional output. Actual
native behavior remains an owner-run acceptance gate.

## Follow-up controls after the first service run

Retain the first 18-condition service run as initial development evidence.
Before the final run, add a frozen-snapshot mutation inside the callback:
the next pass must refuse it while the original project stays unchanged.
Use an auxiliary pass counter to check callback cancellation prevents a
second completed pass. Exercise the production compiler-profile route:
prime successful identification with the no-observer baseline and reuse it
for the observed build, checking the real runtime/format banner. Record
probe reuse explicitly. Baseline and observed durations now have different
version-probe conditions and must not be treated as observer-overhead pairs.
