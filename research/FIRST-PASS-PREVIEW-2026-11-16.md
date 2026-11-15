# First-pass observation: development results

Protocol: `FIRST-PASS-PREVIEW-PLAN.md`. First direct-engine plan/script were
saved before execution under ignored `artifacts/compile-20261115/first-preview-spike/`.
The service-observer extension and later controls were recorded before their
respective runs. All inputs are disposable synthetic `.tex`; no app was opened.

## What is implemented

`engine::build_with_preview` is an opt-in build-library observer. After the
first successful process, runtime checks, recorder discovery and dependency
verification, it can yield owned PDF bytes with diagnostics and elapsed
native time. `UnverifiedPreview` carries no final receipt, eligibility flag,
mapping or export authority. Consumers must parse the PDF before showing it.
The read checks a plain file, a PDF header and a bounded 60 MiB input. Failed,
missing, malformed, truncated or oversized preview reads produce no event.
Cancellation before/after the callback prevents later build completion.

The ordinary desktop path does not enable the observer or read early output.
The existing convergence, final eligibility and native approval contracts
remain. Runtime banner normalization also now strips the trailing build clock
after the final parenthesis/double-space delimiter, retaining earlier spaced
engine and format text. A synthetic regression checks changed format dates.

## Distinct experiments

- Direct engines: 12 conditions, three engines by plain/reference/contents/
  deliberate later failure. Successful per-pass PDFs retain text and 96 dpi
  raster hashes. Plain first/last output matches; reference/contents changes
  settle by pass two/three. All three late failures follow a readable first PDF.
- Initial service run: 18 controls, using fresh engine identification. Retained
  separately with its source identities and nine matching final/baseline PDFs.
- Intermediate service runs: 21 controls, adding frozen-input mutation and
  explicit warm-profile use. Retained as development data.
- Final service run: 21 controls, using the final bounded read and banner
  normalization. The regular gate repeats the corpus as regression validation.

The seven final conditions under every engine are plain/reference/contents,
late failure, first-pass compile error, callback cancellation and frozen-input
mutation after observation. The final successful PDFs match their no-observer
baseline in text, character geometry and all-page 96 dpi raster hashes. Exactly
one event accompanies each eligible first process; compile errors produce none.
First-pass references/contents differ from final output. The auxiliary counter
stays at one after callback cancellation; changed frozen input is refused before
a final result. Original project source stays unchanged throughout.

Final observed successful builds reuse compiler identification primed by the
baseline and check the real runtime/format banner. Their timings differ in
probe conditions from the baseline and do not measure observer overhead.
There is one observation per engine/condition, with uncontrolled machine load,
cache and scheduling. No quantile, population uncertainty, whole-app speedup
or visible latency is inferred. Raw first-available and final-return endpoints
are kept in the receipt, independently of output comparisons.

## Next implementation boundary

Native dispatch and a clearly provisional desktop view still need implementation.
They must bind project/revision/request identity, reject late/cancelled events,
withhold geometry and verified export authority, and retain the last verified
view on later failure. Physical/native UI acceptance remains owner-run. A
readable first PDF alone cannot authorize a source/history commit or release.
