# Current remaining work - 4 November 2026

The initial supported workflow is implemented: open/compile sources, select supported PDF text, move or edit it through checked candidates, Undo/Redo, explicit Save/export and recovery. Project transfer, encodings, templates, image import and bounded image/table/typography/spacing controls are present. Implemented scope is narrower than arbitrary visual LaTeX editing.

## Implementation and engineering

- Complete the missing-prerequisite guidance/setup acceptance on a clean Windows account. Current discovery/preflight diagnostics and WebView2 installer configuration exist; a successful developer-account install is insufficient evidence of the clean setup flow.
- Add or validate end-to-end diagnostic timestamps for source feedback, selection and verified visible results; measure overhead and address actual missed performance targets. Existing service/compile timings exclude user-visible transport/paint.
- Run a controlled single-instance resource/long-session workload. The earlier two-instance sample remains limited evidence.
- Complete the bounded runtime dependency/provenance review's remaining security/version/license checks before public distribution. The current tested pin is PDFium156/pypdfium2 5.14.0; no upgrade is justified solely by historical PDFium126 references.
- Fix further reproducible behavior failures exposed by these checks. New local 0576198 cancels active drag/marquee on blur, with automated tests/build passing; native repeat remains pending.

## Explicitly pending while the owner is away

All tests requiring approval or opening Texloom: actual held-drag Escape/Alt+Tab/lost capture, new-build native repetition/installation, DPI/multiple-monitor transitions, clean-machine missing dependencies/uninstall, and an unassisted human usability trial. Keep them unrun. Do not request more approvals or operate the app to work around this deferral. The previous uninstaller launch was blocked by the control tool; do not bypass it.

## Research implementation

The audit, evidence inventory, primary-source notes, evaluation definitions and editable eight-page draft exist. An independent Poppler reference comparator and five-condition development pilot now run, with separate predicate replay. A prototype declared-byte source-contract oracle has five diagnostics plus CLI health checks; it does not prove TeX semantics. Next work is independent contract annotation/review and integration, ambiguous/reordered-layout adjudication, full pipeline comparisons, final fixture labels, end-to-end timing and machine/font manifests. Seal a fixed version and final inputs/oracles before the final evaluation; retain every failure and version change.

## Later scope and owner decisions

Arbitrary math, TikZ/table-cell editing, cross-page direct manipulation and additional languages/CJK IMEs remain future work. Signing, automatic updates and Store/public publication follow verified release preparation and separate owner decisions. No publication/submission is authorized. Continue using synthetic fixtures, preserving projects, and never compile `.text` files.
