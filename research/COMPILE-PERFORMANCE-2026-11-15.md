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
