# Texloom visual editor architecture and roadmap

Planning baseline: 7 November 2026, application source `6f55baca3100ef8af48d0a3cadb419855c125339`. This package is an architecture proposal and execution backlog. It does not implement the proposed features, approve a new release, or replace recorded failures with passes.

The product goal is a familiar, Word-like visual authoring experience backed by ordinary LaTeX: select, replace, delete, reorder, adjust spacing and formatting, and eventually place a caret and type directly in the visual document. Images and tables need their own supported editing capabilities. Dragging is one input method, not the document model.

The repository is public for source inspection. See [LICENSE](../../LICENSE) for code permissions. The product roadmap targets internal Windows builds and retains all third-party notices.

Original LaTeX and assets remain authoritative. New supported source-native operations should not require Texloom macros or a Texloom support package in saved output. Ordinary classes, fonts, packages, bibliography files and assets remain normal project dependencies. Existing Texloom-object projects need explicit compatibility and migration behavior.

## Read in this order

1. [Roadmap](ROADMAP.md): scope, stages, dependencies, completion gates and the first implementation sequence.
2. [Architecture](ARCHITECTURE.md): source model, operation contracts, input, rendering, assets and verification.
3. [Standards and audit evidence](STANDARDS-AND-EVIDENCE.md): formal standards versus established patterns versus Texloom-specific decisions, with primary sources and code anchors.
4. [Decision register](DECISIONS.md): retained choices, recommendations and unresolved decisions.
5. [Execution backlog](BACKLOG.md) and [structured issue definitions](backlog.json): actionable tasks and acceptance criteria.
6. [Validation scope](VALIDATION.md): checks performed on the plan, GitHub relationships and the separately requested private-project cleanup.

The live issue index is recorded in [github-roadmap.json](github-roadmap.json) after synchronization. Existing tracker #25 and current-build acceptance issues #26–#39 retain their own scopes. New implementation tasks remain open/backlogged; completing this planning package does not complete them.

8 November implementation continuation: [foundation contract v1](FOUNDATION-CONTRACT.md) records the adopted capability/output contract and the first read-only native source projection. ARCH-02 is in progress; new visual command handlers, change mappings and caret/input are not yet enabled. The existing owner-created GitHub Project is retained; its API lookup currently lacks `read:project`, so no duplicate board was created or board-status update claimed.

## Current versus intended

14 November current checkpoint: [native source-draft transport](SOURCE-INTENT-TRANSPORT.md)
is pushed as `76b2228` in draft PR #70. Its 27-step local gate and all five
triggered hosted checks passed. The separate 14 November refusal extension
distinguishes admitted pre-write refusal from unknown/uncertain execution;
its dated diagnostic is source/service evidence, not visual command acceptance.
The owner-reported r11/r12 navigation acceptance is reconciled in
`docs/evidence/navigation-ci-status-2026-11-13.json`. Earlier r8/r10-pending
paragraphs below retain their historical scope; broader Windows/research gates
remain open.

10 November continuation: [position mapping](POSITION-MAPPING.md) now keeps canonical source selections attached through unchanged intervals and invalidates destructive edits in both commit paths. [Typed source-intent planning](SOURCE-INTENT-PLANNING.md) prepares literal paragraph Replace/Delete candidates without visual authority. ARCH-02/03/04 remain in progress. The dated continuation receipt separates domain/native-API checks, one English three-engine compile/text pilot, the v11 technical-report artifact and unrun app acceptance.

The current app already has source editing, native source/history authority, guarded visual transactions, independent inspection, recovery, bounded parameter adapters, and legacy positioned-block insertion/deletion. Normal text-object editing and movement exist. General source-backed visual deletion, a visual caret, in-flow typing, and the generalized operation/anchor layer are proposed work.

The old bounded-core completion statements describe the earlier movement/editing release scope. They do not mean the expanded Word-like product is complete. The prepared r8 build remains unaccepted by the owner; no new app launch, installation, input, engine experiment or final evaluation was performed for this architecture exercise.

## Planning boundaries

This roadmap authorizes no implementation by itself. It records what to build and how to assess it after implementation is requested. English, Arabic and Hebrew remain the current language scope; later composition/language support must fit the design without installing new Windows input languages now. Never compile `.text` files. Preserve real projects and drafts, use synthetic fixtures, and leave all app-control tests to the owner. No source push, public release, signing, report submission or binary/report upload follows from issue creation.


12 November continuation: core `897c121` adds narrowly checked preamble forms and explicit typed source-draft/shared-history receipts. These remain source-domain APIs with unresolved render proof and no new IPC/UI handler. [Dated engineering checks](source-draft-validation-2026-11-12.json) record146 default/147 recovery checks; the [source/reference diagnostic](../../research/SOURCE-INTENT-REFERENCE-RESULTS.md) preserves initial refusal/setup failures and demonstrates four wrong source deletions with identical PDFs. This does not complete ARCH-02/04/05/09/10 or enable general visual Delete/typing. The separate r10 zoom correction is built from earlier source 663b7e2 and is owner-input pending. Report v12 and all final-study cells remain clearly labelled development evidence/pending final evaluation.
