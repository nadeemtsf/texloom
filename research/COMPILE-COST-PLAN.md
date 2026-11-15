# Native compile cost diagnostic

Compare the same production engine build service with the old verification
sequence and a consolidated sequence. Both versions have cancellable TeX
and bibliography execution without wall-clock deadlines. Use release mode,
the same installed MiKTeX executables and five measured runs following one
warm-up per case. Save each observation and manifest in a separate ignored
output directory. The runner uses a temporary independent watchdog for
the experiment.

Fixtures use small synthetic article sources, an auxiliary cross-reference and
24 MiB of unrelated source buffers. Each build creates a fresh snapshot.
Require eligibility, converged output, final receipt integrity and unchanged
original main source. Compare PDF text and full-page rasters across versions
for each measured condition. Keep engine arguments, pass counts and output
validity. If a fixture fails, keep the error and describe the corrected run
separately.

Report observed medians for snapshot setup, compiler preflight, TeX passes,
receipt checks, recorder processing, bibliography and finalization. Include
an end-to-end native interval and account for auxiliary state and other
work outside timed stages. These measurements use one Windows machine and
its existing fonts and tools. Warm-up does not control the operating
system's file cache, other processes or WebView rendering. This diagnostic
does not represent a distribution of real projects.

Run a clearly named LuaLaTeX duration control separately. Two passes of a
synthetic timed sleep should produce eligible output after more than 120
seconds. Use existing process tests to check descendant cancellation with
no deadline and bounded behavior for short probes.

## Amendment after the first baseline

The first 45 measured builds showed TeX accounting for most time, with a
separate version probe taking roughly 0.25–0.32 seconds on this installation.
For the follow-up, report both fresh-probe and warm-probe runs. The session
may reuse only a successful MiKTeX or TeX Live version banner for the same
project root, compiler alias, binary fingerprint and relevant environment.
TeX still runs normally on every pass and dependency integrity is verified.
A failed build clears the applicable remembered probe. Record probe reuse
explicitly and keep cold-start results separate.

The final candidate also compares the actual engine and format header
between successful passes and against the previous warm build. Preserve
initial optimization trials as development data. Measure the final
candidate separately and record its first TeX pass to quantify the time
available for a future early preview.

The optimized verification sequence records inputs before the post-pass
integrity check. That check validates both previously known and newly
discovered dependencies. It retains the pre-pass check and revalidates
bibliography outputs. Repeated identical lines within one recorder file
need only one path resolution. No dependency facts are reused between
recorder files. Compare pass counts, arguments and output equivalence before
interpreting any timing changes.

The separate final runs showed substantial variation in TeX time across
blocks. Follow with alternating fresh and warm pairs in one executable,
with the same cases and counts. Compute the median of paired duration
differences and compare output text, raster and manifest for each pair.
Report this separately from the baseline comparison. The final duration
control puts the entire delay in the first TeX pass and uses an auxiliary
marker to let the second pass settle promptly.

As a separate transport diagnostic, repeat the byte distribution of a
synthetic output PDF to fill 1 and 8 MiB buffers. Record JSON size and the
Node interval covering JSON parse, the regular compile-acknowledgement
validator and conversion to Uint8Array. Verify exact bytes after timing.
These repeated byte buffers are serialization fixtures. The measurement
excludes Rust serialization, Tauri/WebView transport and PDF rendering.

## Binary compile-reply follow-up

Replace the PDF's JSON number array on the desktop command boundary with a
versioned binary envelope. Keep the exact logical build identity, revision,
eligibility and mapping checks. Native callers continue to receive the same
CompileResult. Check malformed envelopes, failed builds, all 256 byte values,
Unicode metadata, candidate revisions and superseded replies. Run the full
local gate and hosted checks before merging.

Repeat the same 1 and 8 MiB transport diagnostic with paired alternating
legacy and binary decoding (one warmup and five measured pairs). Both paths
include the production reply validator and the PDF surface's Uint8Array
construction. Compare exact bytes outside the interval. Report payload
sizes and paired differences, retaining the earlier baseline independently.
This isolates browser decoding costs in Node; it does not measure the native
WebView, engine speed or PDF painting. Owner-run desktop preview acceptance
is needed for the changed native transport before calling it release-ready.

The first two follow-up blocks used the revised validator for both paths,
which admits either a number array or Uint8Array. Retain those as development
trials. For the final comparison preserve the baseline JavaScript validator
and its import from commit 0c3cd89, then load it in the legacy arm. This
avoids attributing a polymorphic-validator/JIT change to removal of JSON.
