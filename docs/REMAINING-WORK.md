# Current remaining work - 5 November 2026

The initial supported workflow is implemented: open/compile sources, select supported PDF text, move or edit it through checked candidates, Undo/Redo, explicit Save/export and recovery. Project transfer, encodings, templates, image import and bounded image/table/typography/spacing controls are present. Implemented scope is narrower than arbitrary visual LaTeX editing.

## Implementation and engineering

- Read-only prerequisite guidance is now implemented, including contextual compiler/package/font help. Native appearance, keyboard use and clean Windows setup acceptance remain pending; a developer-account install is insufficient evidence. See SETUP-HELP.md.
- Opt-in local handler-to-publication/frame-proxy timing is implemented. Validate its actual Windows boundaries, measure overhead and distributions, and address measured missed targets. Compiler IPC timings include native verification; two animation callbacks do not establish physical presentation. See PERFORMANCE-DIAGNOSTICS.md.
- Run a controlled single-instance resource/long-session workload. The earlier two-instance sample remains limited evidence.
- Complete the bounded runtime dependency/provenance review's remaining security/version/license checks before public distribution. The current tested pin is PDFium156/pypdfium2 5.14.0; no upgrade is justified solely by historical PDFium126 references.
- Fix further reproducible behavior failures exposed by these checks. Focus-loss cancellation and current setup/timing source `5e59e77` pass automated checks/builds; native repeat remains pending.

## Explicitly pending while the owner is away

All tests requiring approval or opening Texloom: actual held-drag Escape/Alt+Tab/lost capture, new-build native repetition/installation, DPI/multiple-monitor transitions, clean-machine missing dependencies/uninstall, and an unassisted human usability trial. Keep them unrun. Do not request more approvals or operate the app to work around this deferral. The previous uninstaller launch was blocked by the control tool; do not bypass it.

## Research implementation

The audit, evidence inventory, primary-source notes, evaluation definitions and editable eight-page draft exist. Independent Poppler/source references now also connect to the real native candidate-entry prepare/build/approval/commit/Undo path: five corrected-protocol matches and 25 source-plan refusals, with the first invalid-gold run retained. This still bypasses PDF click/mapping and frontend selection proof. All 27 research helper checks pass; they are infrastructure diagnostics, not accuracy samples. Next work is independent contract annotation/review, ambiguous/reordered-layout adjudication, full interactive pipeline comparisons, final fixture labels, actual timing/overhead and complete machine/font manifests. Seal a fixed version and final inputs/oracles before the final evaluation; retain every failure and version change.

## Later scope and owner decisions

Arbitrary math, TikZ/table-cell editing, cross-page direct manipulation and additional languages/CJK IMEs remain future work. Signing, automatic updates and Store/public publication follow verified release preparation and separate owner decisions. No publication/submission is authorized. Continue using synthetic fixtures, preserving projects, and never compile `.text` files.
