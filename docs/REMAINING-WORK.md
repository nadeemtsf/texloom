# Current remaining work - 5 November 2026

The initial supported workflow is implemented: open/compile sources, select supported PDF text, move or edit it through checked candidates, Undo/Redo, explicit Save/export and recovery. Project transfer, encodings, templates, image import and bounded image/table/typography/spacing controls are present. Implemented scope is narrower than arbitrary visual LaTeX editing.

Latest instruction: continue development/research **except tests**. All test execution is now paused, including automated and headless checks. Local product source `c2ef008` adds known-commit/presentation failure, uncertain commit response and cleanup handling, but is **untested and unbuilt**. The last tested product source remains `5e59e77`; saved r4 binaries are unchanged. See COMMIT-PRESENTATION.md. Do not promote old pass totals to the new source.

The continued source audit found and implemented automatic PDF-to-source highlighting, explicit Reset position versus Remove positioning, formatting/bidirectional editing hints with protected-command display, and editor placement near the selected PDF unit with a centered fallback. They remain unbuilt/unrendered/untested. See FEATURE-SOURCE-COVERAGE.md; source presence does not close any acceptance gate.

## Implementation and engineering

- Read-only prerequisite guidance is now implemented, including contextual compiler/package/font help. Native appearance, keyboard use and clean Windows setup acceptance remain pending; a developer-account install is insufficient evidence. See SETUP-HELP.md.
- Opt-in local handler-to-publication/frame-proxy timing is implemented. Validate its actual Windows boundaries, measure overhead and distributions, and address measured missed targets. Compiler IPC timings include native verification; two animation callbacks do not establish physical presentation. See PERFORMANCE-DIAGNOSTICS.md.
- Run a controlled single-instance resource/long-session workload. The earlier two-instance sample remains limited evidence.
- Complete the bounded runtime dependency/provenance review's remaining security/version/license checks before public distribution. The current tested pin is PDFium156/pypdfium2 5.14.0; no upgrade is justified solely by historical PDFium126 references.
- Fix further reproducible behavior failures exposed by these checks. Focus-loss cancellation and current setup/timing source `5e59e77` pass automated checks/builds; native repeat remains pending.
- Validate the source-only `c2ef008` commit/presentation/uncertain-response and cleanup changes, plus ordinary workflow regressions, only after the owner resumes tests. Source review is not a reproduced failure or a passing fix.

## Explicitly pending while the owner is away

All tests requiring approval or opening Texloom: actual held-drag Escape/Alt+Tab/lost capture, new-build native repetition/installation, DPI/multiple-monitor transitions, clean-machine missing dependencies/uninstall, and an unassisted human usability trial. Keep them unrun. Do not request more approvals or operate the app to work around this deferral. The previous uninstaller launch was blocked by the control tool; do not bypass it.

## Research implementation

The new v1 study-record implementation and template separate commit, display, Save, uncertainty, required postconditions, intent/packet identities and artifact references. They are unexecuted; their own review/diagnostics remain pending. A detailed annotation/adjudication procedure and primary-source follow-up are prepared. Read-only Windows inventory records later RAM/CPU/registry/controller observations, not app acceptance or an earlier sealed environment. The pinned dependency build-source follow-up identifies specific remaining nested provenance/notice-coverage questions without running tests or changing the pin. See ../research/RECORDS.md, ../research/ADJUDICATION.md, ../research/related-work-followup-2026-11-05.md and DEPENDENCY-SOURCE-REVIEW.md.

The audit, evidence inventory, primary-source notes, evaluation definitions and editable technical-report draft exist. The owner removed the earlier eight-page constraint; the report expands methods, evidence and appendices without a page cap. Independent Poppler/source references also connect to the real native candidate-entry prepare/build/approval/commit/Undo path: five corrected-protocol matches and 25 source-plan refusals, with the first invalid-gold run retained. This still bypasses PDF click/mapping and frontend selection proof. The 27 previously executed research-helper checks are infrastructure diagnostics, not accuracy samples or validation of the new v1 records. Next work is independent contract annotation/review, ambiguous/reordered-layout adjudication, full interactive pipeline comparisons, final fixture labels, actual timing/overhead and complete pre-run machine/font manifests. Seal a fixed version and final inputs/oracles before the final evaluation; retain every failure and version change.

## Later scope and owner decisions

Arbitrary math, TikZ/table-cell editing, cross-page direct manipulation and additional languages/CJK IMEs remain future work. Signing, automatic updates and Store/public publication follow verified release preparation and separate owner decisions. No publication/submission is authorized. Continue using synthetic fixtures, preserving projects, and never compile `.text` files.
