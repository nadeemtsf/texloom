# Texloom visual editor architecture and roadmap

Planning baseline: 7 November 2026, application source `6f55baca3100ef8af48d0a3cadb419855c125339`. This package is an architecture proposal and execution backlog. It does not implement the proposed features, approve a new release, or replace recorded failures with passes.

The product goal is a familiar, Word-like visual authoring experience backed by ordinary LaTeX: select, replace, delete, reorder, adjust spacing and formatting, and eventually place a caret and type directly in the visual document. Images and tables need their own supported editing capabilities. Dragging is one input method, not the document model.

The owner intends to keep the repository private now and in the future. The product roadmap targets private authoring/internal Windows packaging, with no public distribution or Store work. Project-level license/contribution files and MIT package declarations are removed separately; dependency notices remain retained.

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

The current app already has source editing, native source/history authority, guarded visual transactions, independent inspection, recovery, bounded parameter adapters, and legacy positioned-block insertion/deletion. Normal text-object editing and movement exist. General source-backed visual deletion, a visual caret, in-flow typing, and the generalized operation/anchor layer are proposed work.

The old bounded-core completion statements describe the earlier movement/editing release scope. They do not mean the expanded Word-like product is complete. The prepared r8 build remains unaccepted by the owner; no new app launch, installation, input, engine experiment or final evaluation was performed for this architecture exercise.

## Planning boundaries

This roadmap authorizes no implementation by itself. It records what to build and how to assess it after implementation is requested. English, Arabic and Hebrew remain the current language scope; later composition/language support must fit the design without installing new Windows input languages now. Never compile `.text` files. Preserve real projects and drafts, use synthetic fixtures, and leave all app-control tests to the owner. No source push, public release, signing, report submission or binary/report upload follows from issue creation.
