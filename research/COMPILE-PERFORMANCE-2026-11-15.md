# Compile deadlines and native build cost

The native build service lets TeX passes and bibliography generation run
until they complete or the user cancels. Engine and tool discovery probes
have their own deadlines. The 2 GiB process memory limit and six-pass
convergence limit remain. In the real LuaLaTeX control, the first pass took
**121.83 seconds**, then the second pass settled. The eligible
output was returned in **122.97 seconds** and independently
contained the expected text. The test wrapper's 180-second watchdog applies
only to the experiment. Process tests cover cleanup with and without a
timeout, including stopping child processes on cancellation.

The baseline native plain-document medians were 1.10 s (pdfLaTeX), 1.82 s
(XeLaTeX) and 1.94 s (LuaLaTeX). Two real engine passes accounted for 0.76,
1.43 and 1.57 s of those builds. A separate version subprocess accounted
for roughly 0.26–0.29 s. Rust coordinates snapshots, dependencies, tool
processes and output evidence. The TeX engine performs the typesetting.

The revised path verifies the known and newly recorded inputs together
after each successful pass. It resolves duplicate recorder paths once per
pass. Repeat builds can reuse a successful engine identification within the
same project and executable/environment. Every pass still executes TeX and
checks dependencies. Its runtime and loaded format must match the applicable
engine header. A failed or cancelled repeat consumes the remembered profile.

## Paired timing results

All times below are milliseconds. Each group has five measured pairs with
alternating order, after separate warm-ups. The saving is the median of the
individual fresh-minus-warm differences. First pass is the warm build's
engine-process interval, including startup and exit.

| Engine | Fixture | Fresh | Warm | Paired saving | Warm first pass |
| --- | --- | ---: | ---: | ---: | ---: |
| pdfLaTeX | plain | 1074 | 782 | 280 | 365 |
| pdfLaTeX | references | 1140 | 826 | 365 | 383 |
| pdfLaTeX | large-project | 1248 | 994 | 254 | 384 |
| XeLaTeX | plain | 1911 | 1702 | 294 | 826 |
| XeLaTeX | references | 2045 | 1730 | 278 | 808 |
| XeLaTeX | large-project | 2089 | 1754 | 279 | 771 |
| LuaLaTeX | plain | 2005 | 1715 | 325 | 798 |
| LuaLaTeX | references | 2060 | 1796 | 233 | 878 |
| LuaLaTeX | large-project | 2175 | 1877 | 298 | 845 |

The independent checks matched 90 outputs across the baseline and final
blocked runs, plus 45 fresh/warm pairs. Every comparison matched extracted
text, complete 96 DPI page rasters, pass counts, arguments, version and used
project inputs. Repeats use synthetic one-page projects and a shared host.
Native intervals omit editor debounce and WebView rendering. Blocked runs
showed substantial variation in TeX time. Pairing reduces time-order bias
while background activity and file caches remain uncontrolled.

## Opportunities for larger gains

- **First preview.** The first-pass column quantifies the engine time before
  another pass becomes necessary. A provisional view can show a fresh PDF
  sooner while Texloom resolves references and approves the final result.
  Movement, verified export and other proof-dependent operations still need
  the settled build. UX acceptance needs a packaged Windows check.
- **Incremental convergence.** A guarded experiment can seed auxiliary data
  from the prior successful revision. Measure equivalence against clean
  builds across APIs, references, bibliography and changed or removed
  dependencies. [Latexmk's manual](https://tug.ctan.org/support/latexmk/latexmk.pdf)
  documents dependency-driven reruns and continuous preview.
- **Large PDF replies.** Encoding PDF-pattern bytes as numbers expanded
  8 MiB to 29.6 MB. Parse, validation and conversion took 267 ms median in
  Node. The diagnostic excludes WebView transport and rendering. A binary
  Tauri response can remove the array conversion and preserve checked build
  metadata. [Tauri binary replies](https://v2.tauri.app/develop/calling-rust/#returning-array-buffers)

The native engine still dominates these small-document timings. Comparing
another TeX implementation would require matching the document/engine
matrix, installed packages, glyph positions and SyncTeX behavior.
The raw observations and exact source identities are recorded in
[the evidence receipt](../docs/evidence/compile-performance-2026-11-15.json).

## 18 November follow-up

The desktop now publishes a new PDF after the compiler settles. The
first-pass observer remains available for experiments. The technical report
records the three-engine auxiliary reuse counterexample, the pdfLaTeX
preamble feasibility trial, and the paired compiler-identification results
across alternating projects. [Compile follow-up evidence](../docs/evidence/compile-followup-2026-11-18.json)

Short completed PDFs can prepare two pages at a time through the existing
render queue. Every page still has to finish before the new view replaces
the accepted view. An initial page failure cancels the rest of the
candidate's queue.

On one Windows host, headless Edge and PDF.js 6.2.108 prepared the six-page
fixture in a median 43.6 ms, down from 61.4 ms. The twenty-page fixture
took 108.4 ms, down from 184.2 ms. The median paired reductions were 17.5
ms and 79.9 ms from five measured pairs per fixture. All twelve pairs,
including warm-ups, matched complete canvas pixels, readable text, and
line and text-run geometry. These timings cover offscreen preparation
after PDF load. The final implementation passed 33 headless navigation
checks at device scale factors 1, 1.5 and 2.
[PDF preparation evidence](../docs/evidence/completed-pdf-paint-2026-11-18.json)

## Source coordinate mapping follow-up

The native compile service maps each legacy block and movable text object
back to the UTF-16 offsets used by the editor. Previously it built a full
projection for each range, then scanned the boundary list for each end. The
service now creates one projection per file when it needs a range. The
sorted boundary table supports binary search and retains the existing
rules for BOMs, mixed newlines and astral characters.

The projection-only comparison covered nine measured observations per
condition. With 128 ranges in 64 KiB, the median was 50.31 ms for the old
per-range setup, 4.23 ms with one shared projection and linear lookup, and
0.26 ms with the shared sorted table. With 256 ranges in 1 MiB the medians
were 2401.87, 137.28 and 6.01 ms. The 4 KiB, one-range case was about
0.014 ms in both the original and final conditions. The offset and
replacement checks matched for all 2,401 multilingual and newline
combinations.

Three alternating pairs through the normal native compile service used
one 61,154-byte pdfLaTeX source containing 64 text objects and 64 legacy
blocks. The median paired wall time reduction was 128 ms. After
subtracting the engine/build interval, the median paired reduction was
66 ms. The output fit on one page. Each pair matched the independently
expected ranges, exact metadata, PDF text and character geometry, and
all 96 DPI raster pixels. Original source and revision were preserved.
These tests did not measure desktop transport or display latency.
[Source coordinate evidence](../docs/evidence/source-coordinate-mapping-2026-11-18.json)

## PDF worker startup during compilation

An ordinary preview compile starts an empty PDF.js worker after dispatching
the native build request. The worker receives document bytes only after the
native build returns successfully for the current revision. The accepted view keeps
its own worker. A failed or superseded compile disposes its unused worker.
Once a worker starts loading a PDF, the document handle owns its disposal.
A startup failure falls back to normal PDF loading.

A headless browser test compared startup at PDF load with an already ready
worker. With the packaged worker asset served directly, median paired
load-to-prepared reductions were 72.0 ms for six pages and 86.9 ms for
twenty pages, from seven measured pairs each. All 16 pairs including
warm-ups matched complete pixels, line and run geometry, and readable text.
The implementation run also matched all 16 pairs and passed ownership
checks. The larger gains observed with the development URL are kept
separate. These component timings exclude actual concurrent TeX work,
resource overlap, WebView transport and physical display.
[PDF worker startup evidence](../docs/evidence/pdf-worker-startup-2026-11-18.json)


## Final PDF bytes and build receipt

The completed PDF is read once for delivery and sealed from those bytes.
The 60 MiB limit applies to the actual read. Later receipt checks still
reopen and verify the file. A changed PDF after the read fails verification.

In release-mode repeated pairs on one Windows host, removing the extra
read saved 0.07 ms at 256 KiB, 1.39 ms at 8 MiB, 5.71 ms at 32 MiB and
10.29 ms at 60 MiB by median paired difference. All 40 paired fixtures
matched exact bytes and SHA-256. A separate production control matched
all 18 baseline and candidate PDF comparisons across the three engines
and independently checked the output fingerprints. These tests isolate
file-read and hash cost from TeX and presentation.
[Output identity evidence](../docs/evidence/completed-pdf-output-identity-2026-11-18.json)


## Paired native compile checkpoint comparison

The pre-optimization benchmark and the current native service built four
synthetic projects per MiKTeX engine, using fresh snapshots and identical
inputs. One warm-up pair and three measured pairs covered each condition.
All 48 pairs matched recorded build inputs, invocation fields, pass counts,
PDF text, character geometry, links and 96 DPI pixels. Each output matched
its own receipt hash.

| Engine | Small | Source-heavy | Asset-heavy | Mixed |
| --- | ---: | ---: | ---: | ---: |
| pdfLaTeX | 577 ms | 533 ms | 947 ms | 995 ms |
| XeLaTeX | 302 ms | 687 ms | 960 ms | 762 ms |
| LuaLaTeX | 685 ms | 572 ms | 1247 ms | 1323 ms |

Each result is the median paired reduction from invoking the Rust build
service through its returned PDF and receipt. The extra files exercise
snapshot work while each compile reads the same small main source and
image. App dispatch and preview rendering are outside the measured interval.
[Native comparison evidence](../docs/evidence/native-compile-checkpoint-comparison-2026-11-18.json)
contains the absolute medians, individual pairs and fixture identities.

The source line indexing follow-up reduced standalone parsing of a synthetic
1 MiB file with 512 text objects and 512 legacy blocks from a median 299.7 ms
to 2.31 ms. The native compile control verified all ranges for 128 text
objects and 64 blocks in an included file. Borrowing expected pixels in the
raster verifier saved a paired median 41.5 ms for identity and 35.0 ms for
predicted displacement across twenty synthetic pages. Both routines preserved
their exact comparison results.
[Source and raster evidence](../docs/evidence/source-and-raster-comparison-2026-11-18.json)
includes the individual timed pairs and the native control PDF fingerprint.

## Report manuscript and compiler log follow-ups

The same build service checkpoints compiled the 40-page v24 report with
pdfLaTeX and BibTeX. Five measured pairs followed two warm-ups. The paired
median saving was 0.75 seconds, with one run 0.49 seconds slower and the
best pair 2.21 seconds faster. All seven pairs matched three-pass behavior,
receipt fields, and PDF text, geometry, links and pixels.
[Report compile evidence](../docs/evidence/technical-report-compile-followup-2026-11-18.json)
records inputs, individual timings and scope.

The process supervisor now retains compiler output in chunks. It drains
the pipe and preserves the original 128 KiB cap, prefix and tail. A
separate in-memory comparison measured paired median savings of 0.015 ms
at 4 KiB, 0.352 ms at the retention cap, and 48.871 ms for 8 MiB of
truncated output. The last case measures handling of unusually large
diagnostic output. Native subprocess tests verified output draining,
cancellation and descendant cleanup.
[Log capture evidence](../docs/evidence/compiler-log-capture-2026-11-18.json)
contains the measured pairs and native controls.

## Project-file checks and long PDF preparation

Reusing metadata from the link check reduced directory enumeration of
600 files by a paired median 16.6 ms. The separate native comparison
of that change saved a median 86 ms on the 600-file build. Individual
native differences crossed zero. The toolchain concurrency prototype
reduced verification costs but produced mixed total-build results.
[Native and component evidence](../docs/evidence/project-and-toolchain-verification-2026-11-19.json)
preserves both comparisons and the matched receipts and PDFs.

The reading view now requests page metadata in groups of eight and
keeps page order and exact dimensions. Paired median savings through
first-page paint were 6.5 ms at 40 pages, 25.4 ms at 250 pages and
108.8 ms at 1,200 pages. The headless test verified every page shell
and the first and last page rasters and text.
[Reading-view evidence](../docs/evidence/pdf-reading-page-fetch-2026-11-19.json)
records the individual timings and failure control.

The intermediate-draft experiment found small median savings on two
fixtures and nearly zero on the report. The saved-mode counterexample
produced a different final PDF after two full passes, ruling out the
unrestricted shortcut. The offline Tectonic fixture also exposed a
version-specific output-directory prefix in its prerequisite paths.
[Draft-pass evidence](../docs/evidence/intermediate-draft-pass-2026-11-19.json)
and [offline prerequisite evidence](../docs/evidence/tectonic-prerequisite-paths-2026-11-19.json)
record the compiler and artifact checks.

## Diagnostic parsing and recovery after source errors

The parser prepares source markers once and skips source matching on
output lines without a possible line number. It keeps the first 200
diagnostics in their original order. The 600-source standalone comparison
saved paired medians of 6.4 ms on the shorter log and 92.6 ms on the
verbose log. An independent native 600-file fixture that printed 950
notices per pass saved a paired median 263 ms. The receipt and PDF
properties matched in every pair.

After an attributed source error with a matching runtime banner, the
engine retains its existing identification profile. The failed build
remains ineligible. The next valid build still runs TeX and verifies
its own snapshot. Quiet paired repair runs saved medians of 275 ms
with pdfLaTeX, 292 ms with XeLaTeX and 183 ms with LuaLaTeX. Cold
failures and changed environments still required identification.
[Diagnostic and repair evidence](../docs/evidence/compiler-diagnostics-and-repair-2026-11-19.json)
contains the measured pairs and output controls.

## Source work around the native compile

The frontend compile stage uses the current revision and candidate token.
It no longer copies the source map into callbacks without source
parameters. The native service retains an owned revision snapshot for
the build and borrows it for compile. A private proof build constructs
its request with the supplied proof sources. The metadata pass derives
plain-text preview definitions once for all objects in the same build.

A Node preflight diagnostic measured paired median dispatch savings of
1.08 ms for 1 MiB, 12.80 ms for 8 MiB and 53.63 ms for 32 MiB. It
measures the old double copy at the ordinary frontend entry point and
uses native and rendering doubles. A separate optimized Rust copy
diagnostic at the same three sizes saved 0.23, 2.20 and 9.89 ms. Neither
time includes TeX execution.

Reading the main source definitions once saved 16.46 ms on the
256-object, 1 MiB metadata fixture. Both implementations returned the
same text or supported refusal, including for redefined macros,
mixed formatting and recursion. A headless two-file pdfLaTeX build
verified three objects at their expected source positions, including
a styled macro in an included source. The original files and draft
revision were preserved.
[Source preparation evidence](../docs/evidence/compile-source-preparation-2026-11-18.json)


## Snapshot capture and receipt verification

Snapshot capture uses a native asset copy and seals the bytes that TeX
will read. Capture verifies both the frozen file and its original before
TeX. Receipts with at least 8 MiB of frozen inputs verify frozen files and
the remaining dependencies in two threads after checking the compiler
and alias. Both finish before acceptance, with the same error priority.

The normal pdfLaTeX build service matched all 16 pre-change and candidate
pairs. The median paired reductions in snapshot and verification work
were 38.8 ms for a source-heavy fixture, 161.1 ms for an asset-heavy
fixture, and 156.2 ms for a mixed fixture. The small fixture was within
0.1 ms. TeX, delivery and rendering have separate endpoints.
[Snapshot evidence](../docs/evidence/snapshot-verification-work-2026-11-18.json)

## Tectonic feasibility

Paired command-line trials compared Tectonic 0.17.0 against MiKTeX
XeLaTeX in fresh working directories. Cold cache setup for the first
Tectonic fixture took 99 seconds. In a separate warm offline comparison
with explicit Letter paper and margins, five fixture medians favored
Tectonic by 594–856 ms. All 25 measured pairs matched PDF text and the
specified SyncTeX round trips. Exact character geometry differed by up
to 0.014 bp and one fixture matched pixels. The dependency rule did not
list the bundled class. A copied-cache control showed that changing its
class file changed output under the same bundle namespace.
[Engine feasibility evidence](../docs/evidence/tectonic-engine-feasibility-2026-11-18.json)
