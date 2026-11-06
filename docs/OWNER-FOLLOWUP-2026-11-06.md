# Owner follow-up: 6 November 2026

This is development evidence, not final evaluation or release certification. The unedited owner response is retained privately in the chat workspace. No owner name or private project source is copied into this repository. The owner controls all native input.

## Reported outcomes

| Area | Evidence and limit |
| --- | --- |
| Signature workflow | Owner reports grouped selection/movement, replacement, Undo/Redo, Save and reopen working. Their replacement differs from the proposed literal; do not count its ampersand condition as executed. |
| Repeated occurrence | Owner reports the intended middle occurrence moved and others stayed in place. No new independent native recording was made. |
| Failed compilation | Owner reports explicit Compile, expected undefined-command failure and retained good PDF. Explicit Compile after opening is intentional. Removing the bad line cleared the error; new successful PDF publication still needs confirmation. |
| Arabic/Hebrew selection | Owner reports refusal to identify one complete object. They clarified the targets were Arabic and Hebrew. English was not tried in that project. Source typing, search and initial display correctness remain unconfirmed. |
| Recovery | Owner closed normally with the old discard confirmation and reported no useful recovery / a different project appearing. This does not demonstrate loss after a crash. Previous behavior intentionally discarded the current draft; the dialog also implicitly selected an inventory entry. |
| Environment | Another monitor was unavailable. Exact new installation identity, clean-machine prerequisites, uninstall preservation and unassisted usability are not established. |

## Implemented response

The close controller offers **Keep draft and close**. It flushes edits, requires the exact protected current revision, and closes without discarding a dirty draft. If protection is incomplete it keeps the window open and explains the Save/copy/retry alternatives. Clean saved documents retire their unnecessary draft. A native test covers close, clean exit, a fresh backend, exact recovered source, Undo and unchanged original disk bytes.

Recover work starts with **Choose a project to recover**. Restore and discard remain unavailable until a choice is made. Reopening the dialog or changing projects clears an earlier choice. Restoration still opens the chosen project and retains the earlier draft for review.

For XeTeX RTL targets, the owned v0.4 wrapper places marker/transform boundaries in an LTR span while retaining the original text's RTL span. This addresses the observed ordering defect without relaxing glyph, geometry or raster checks. An exact generated v0.3 support file can upgrade within the candidate; customized support files are retained and refused. Save remains explicit.

## Executed checks and retained failures

- 287 frontend checks pass; required core checks pass (70 unit, 18 integration). The combined desktop/core library configuration passes 71 core-library and 53 native tests, with two native tests still ignored in that default invocation. Counts/configurations overlap.
- PDF.js zero-offset selection/appearance proofs pass for one synthetic Arabic, Hebrew and English target under local XeLaTeX.
- Independent PDFium prediction checks pass for two declared moves per RTL language. Zero-offset full-page raster paint is unchanged. Nine Latin fractional moves across pdfLaTeX, XeLaTeX and LuaLaTeX pass with the new wrapper.
- The original empty-marker reproduction and a failed leavevmode-only experiment are retained. The first production wrapper failed pdfLaTeX because of conditional scanning; the corrected helper passed the repeated matrix. An initial proof-script English target literal was absent from the template; that failed run is preserved separately from the corrected three-target run.

Reports and exact source/log/PDF fingerprints are recorded in `evidence/owner-followup-2026-11-06.json`. Fresh raw outputs are under ignored `artifacts/owner-followup-20261106/`. These cases are reused development fixtures. They do not estimate correctness rates, broad RTL support, latency, usability or full PDF-selection workflow accuracy.

The existing v7 report PDF remains a historical artifact. The v8 PDF includes this follow-up, compiled at 25 pages with 21 references; all pages were rendered and visually reviewed, including readable views of the new subsection and bibliography. Compilation and document review do not establish application correctness. The final study remains unfrozen and unrun.
