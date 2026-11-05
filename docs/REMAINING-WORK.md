# Current remaining work - 5 November 2026

The initial supported workflow is implemented: open/compile sources, select supported PDF text, move or edit it through checked candidates, Undo/Redo, explicit Save/export and recovery. Project transfer, encodings, templates, image import and bounded image/table/typography/spacing controls are present. Implemented scope is narrower than arbitrary visual LaTeX editing.

Latest instruction: continue development/research **except tests**. All test execution is now paused, including automated and headless checks. Later local source adds commit/presentation/uncertain-response handling, interaction completion and schema-2 diagnostic identity, but is **untested and unbuilt**. The last tested product source remains `5e59e77`; saved r4 binaries are unchanged. See COMMIT-PRESENTATION.md and FEATURE-SOURCE-COVERAGE.md. Do not promote old pass totals to the new source.

The continued source audit found and implemented automatic PDF-to-source highlighting, explicit Reset position versus Remove positioning, formatting/bidirectional editing hints with protected-command display, and editor placement near the selected PDF unit with a centered fallback. They remain unbuilt/unrendered/untested. See FEATURE-SOURCE-COVERAGE.md; source presence does not close any acceptance gate.

## Implementation and engineering

- Read-only prerequisite guidance is now implemented, including contextual compiler/package/font help. Native appearance, keyboard use and clean Windows setup acceptance remain pending; a developer-account install is insufficient evidence. See SETUP-HELP.md.
- Opt-in local handler-to-publication/frame-proxy timing is implemented. Validate its actual Windows boundaries, measure overhead and distributions, and address measured missed targets. Compiler IPC timings include native verification; two animation callbacks do not establish physical presentation. See PERFORMANCE-DIAGNOSTICS.md.
- Run a controlled single-instance resource/long-session workload. The earlier two-instance sample remains limited evidence.
- Remediate the newly identified PDFium advisory blocker, then complete remaining source/build/component/license review and exact-artifact validation before public distribution. The current tested development pin is PDFium156/pypdfium2 5.14.0; the new finding concerns that actual distributor build. Retaining it for reproduction is not security approval. See DEPENDENCY-ADVISORY-REVIEW.md; no pin change or test occurred during the pause.
- Fix further reproducible behavior failures exposed by these checks. Focus-loss cancellation and current setup/timing source `5e59e77` pass automated checks/builds; native repeat remains pending.
- Validate the source-only `c2ef008` commit/presentation/uncertain-response and cleanup changes, plus ordinary workflow regressions, only after the owner resumes tests. Source review is not a reproduced failure or a passing fix.
- Include the new interaction source and object double-click refusal guard (`391332d`, `4e403ca`), schema-2 diagnostics and all new record/admission/timing modules in that resumed validation. Neither code presence nor recorded file hashes close their behavior gates.
- Validate the source-only bitmap-initialization return guard (`8e23d15`) on the exact rebuilt verifier/app. Candidate dependency intake supplies archive/header/notice observations and an upgrade worksheet, not adopted resources or a runtime compatibility result. See DEPENDENCY-CANDIDATE-INTAKE.md.

## Explicitly pending while the owner is away

All tests requiring approval or opening Texloom: actual held-drag Escape/Alt+Tab/lost capture, new-build native repetition/installation, DPI/multiple-monitor transitions, clean-machine missing dependencies/uninstall, and an unassisted human usability trial. Keep them unrun. Do not request more approvals or operate the app to work around this deferral. The previous uninstaller launch was blocked by the control tool; do not bypass it.

## Research implementation

The new v1 study-record implementation and template separate commit, display, Save, uncertainty, required postconditions, intent/packet identities and artifact references. Separate source-only tools now specify declared byte-reference consistency and timing batches with duplicate/loss/censor accounting. They are unexecuted; their review/diagnostics remain pending. See ../research/RECORDS.md, ../research/REFERENCE-ADMISSION.md and ../research/TIMING-RECORDS.md. Annotation/adjudication and primary-source follow-up are prepared; the uncompiled report source integrates these methods and literature without new results. Read-only Windows inventory adds later observations, not app acceptance or earlier environment binding. Dependency source/advisory notes retain unresolved provenance/security/license work without tests or a changed pin.

The audit, evidence inventory, primary-source notes, evaluation definitions and editable technical-report draft exist. The owner removed the earlier eight-page constraint; the report expands methods, evidence and appendices without a page cap. Independent Poppler/source references also connect to the real native candidate-entry prepare/build/approval/commit/Undo path: five corrected-protocol matches and 25 source-plan refusals, with the first invalid-gold run retained. This still bypasses PDF click/mapping and frontend selection proof. The 27 previously executed research-helper checks are infrastructure diagnostics, not accuracy samples or validation of the new v1 records. Next work is independent contract annotation/review, ambiguous/reordered-layout adjudication, full interactive pipeline comparisons, final fixture labels, actual timing/overhead and complete pre-run machine/font manifests. Seal a fixed version and final inputs/oracles before the final evaluation; retain every failure and version change.

## Later scope and owner decisions

Arbitrary math, TikZ/table-cell editing, cross-page direct manipulation and additional languages/CJK IMEs remain future work. Signing, automatic updates and Store/public publication follow verified release preparation and separate owner decisions. No publication/submission is authorized. Continue using synthetic fixtures, preserving projects, and never compile `.text` files.
