# Visual LaTeX desktop editor — implementation and acceptance plan

7 November planning extension: the owner specified a broader Word-like authoring goal and requested architecture analysis and a complete backlog. See [the proposed architecture and roadmap](architecture/README.md) and GitHub roadmap #44. This adds a planned evolution path; it does not silently label future features implemented or extend this earlier approval into permission to start new application code now. Preserve the current baseline and acceptance evidence while preparing the shared editing foundation.

**Approved by the owner on 1 November 2026. Implementation begins with P0/P1.**

Primary repository: https://github.com/nadeemtsf/texloom  
Inspected baseline: e7b6d1eb334c879a4aa28c4f22b984fb3c17532b  
Working product name remains Texloom until a separate naming decision.

This plan defines the app we will build, the experiments that decide its architecture, and the evidence required to call it reliable. It incorporates the supplied design review, inspection of the existing repository, and primary-source research. Creating this document does not authorize implementation or change the app.

## 1. Recommendation and quality contract

Build a local Windows desktop application using Rust, Tauri, Monaco, and PDF.js. Retain the useful shell and fixtures. Replace the document/session machinery where necessary, and select the visual mapping and movement implementation through a controlled experiment.

The central promise is: **the user can select supported content in the actual compiled document, edit or move it, and obtain a small, understandable change to their original LaTeX source—with reliable undo and independently verified output.**

“Works perfectly” must mean observable behavior, not an unlimited claim about every possible TeX program. We will require:

- No silent loss or unrelated rewriting of source.
- No operation applied to the wrong occurrence of repeated text.
- No stale compile overwriting newer typing.
- No successful move that quietly changes neighboring content.
- Exact source undo, dependable save/recovery, and reproducible export.
- Immediate gesture feedback, with a clear distinction between a pending preview and a verified result.
- A visible explanation when a particular operation cannot be supported safely.

The first complete workflow is the one that motivated this project: select the name and two lines beneath a signature, move the group lower, edit the name, undo/redo, save, reopen, and independently compile the saved project.

This workflow is a release requirement. A working code editor, a rendered PDF, or passing mock tests alone will not satisfy it.

## 2. Current baseline: retain evidence, replace weaknesses

Repository inspection confirms an existing Rust/Tauri implementation, Monaco, PDF.js, source range transformations, revision checks, candidate compilation, portable movement syntax, and a real PDF regression fixture. The supplied review understates these existing protections.

| Area | Verified from current source / prior recorded work | Consequence |
| --- | --- | --- |
| Source and transaction foundations | Range-based edits, source checks, candidate snapshots, stale-result rejection already exist | Port and strengthen; do not discard blindly |
| Movement package | Uses graphics-state translation and marked content; it does not currently box every fragment | Compare it against alternatives experimentally |
| Known renderer bug | A fixture catches unselected text moving after unsafe direct PDF literals | Preserve this as a mandatory regression |
| Corrected package | Page-literal correction is present; complete native verification remains outstanding | No release claim until the corrected executable passes |
| History and text ownership | Important session/history logic remains in the frontend | Establish a single Rust authority |
| Compilation | Current service runs a fixed two passes | Replace with explicit build recipes and convergence handling |
| Geometry validation | Current neighbor check relies on PDF.js text runs and rounding | Add explicit tolerances, independent PDF verification, and non-text checks |
| Build documentation | Some status files are stale or contradict later publication | Reconcile after approval using actual evidence |
| Distribution | Prior downloadable package is an older baseline | Never label an old ZIP as the newly verified build |

The last recorded baseline was 33 Rust core tests and 13 frontend/fixture tests passing. Those checks were **not rerun for this planning review**. Remote CI status and installer readiness are not established by the local clean checkout.

## 3. What the first reliable release contains

| User action | Intended behavior | Initial boundary |
| --- | --- | --- |
| Open a project | Open normal .tex sources and assets; choose the correct root and engine | Local Windows projects; explicit handling of external dependencies |
| Edit source | Normal code editing, find, multiple files, diagnostics, auto compile, undo/redo | Source remains editable even when it temporarily fails to compile |
| Read PDF text | Highlight, copy, search, zoom, scroll | Real PDF text layer, separate from object selection |
| Select a signature line | Hover outline, click frame, source highlight | A complete logical line with a safe source boundary |
| Select a group | Shift-click or marquee adjacent compatible units | Same page and source context; expanded selection is visible before moving |
| Select wrapped prose | Select the logical paragraph | A visual soft-wrap line is not automatically an independent source object |
| Edit visible text | Edit supported literal runs in place while retaining formatting | Opaque commands remain protected; raw-fragment fallback is available |
| Move a block | Drag or nudge its drawing while retaining its space in flow | Verified single-page text units without unsupported side effects |
| Adjust spacing | Change a supported spacing parameter; allow appropriate downstream reflow | A distinct action with a distinct validation policy |
| Undo / redo | One history across source and visual edits | One successful drag equals one history entry |
| Save / reopen / export | Preserve project portability and reproduce the accepted result | No dependency on hidden app caches to compile the project |

Editing math, table cells, arbitrary floats, TikZ internals, rotated text, cross-page groups, or arbitrary macro expansions is outside the first direct-manipulation contract. These documents can still open, compile, and be edited as source. Unsupported elements must not disable unrelated supported content.

Arabic/RTL, CJK, complex shaping, and mixed-direction documents enter the test corpus from the start. Source editing and faithful display are required; canvas editing is enabled for each case only after its own mapping and movement tests pass. We will not infer support from a Latin-only demonstration.

After the core passes, the next adapters are images with aspect-ratio controls, heading typography, explicit paragraph spacing, table column widths, and template-aware CV/letter blocks. Each gets a source grammar, capability rules, and acceptance fixtures.

## 4. How visual editing should feel

On opening a successfully compiled project, the preview offers clearly labeled **Edit layout** and **Read text** modes. Editable regions show a subtle outline on hover. Clicking a region highlights the corresponding source without changing it. The selected frame explains its scope, such as “3 signature lines” or “whole paragraph.”

A marquee proposes the smallest compatible group. If it would include a table cell, unrelated column, different source file, hidden macro side effect, or page boundary, it shows the reason instead of guessing.

Double-click opens an editor anchored near the selected unit. Simple formatting appears as formatted text; unsupported commands are protected tokens. A source-fragment option is available for advanced editing. Typing a percent sign as text produces escaped LaTeX. Editing a displayed macro value never silently edits every use of its definition.

Dragging produces immediate feedback. Pointer movement does no compilation or file writing. Release submits one operation. A translucent target preview is used where target-only rendering is reliable; otherwise a moving outline is used. We must not white-paint the old position or crop unrelated background graphics into the drag image.

The old verified page remains available while compilation runs. On success, source, geometry, and the visible page advance together. On failure, the pending overlay disappears and the previous result remains. Escape and pointer cancellation leave the document unchanged.

Arrow keys make small, documented nudges; Shift increases the step; Alt disables snapping. Measurements can be shown in mm and bp. Every visual operation has a keyboard-accessible alternative.

## 5. Research findings that change the proposed design

### Mapping needs several kinds of evidence

SyncTeX records source-file and line information and has useful forward/inverse synchronization, but its original description documents accuracy limits and complications from output transformations. It is useful evidence for navigation, not sufficient authority for arbitrary source rewriting. [S1]

LuaTeX exposes node attributes and post-line-break access. This makes engine-assisted provenance and layout measurement promising. It does not prove that every source fragment survives macro expansion, shaping, copying, or shipout transformations with an exact identity. [S2]

PDF.js supports marked-content extraction and text transforms. We should retain its text layer for copying and use its PDF geometry as evidence, rather than treating the HTML text layer as the source of truth. [S3]

**Decision:** combine conservative source analysis, engine/source provenance, and verification against the final PDF. No single layer gets unchecked authority.

### Reuse parsing infrastructure before writing a full new parser

A lossless source store and guarded patches are essential. A completely new general LaTeX parser is not.

The existing tree-sitter-latex grammar is a candidate for incremental structural parsing; its own documentation explains that it is best effort and does not handle all TeX behavior. Rowan provides useful full-fidelity tree infrastructure but does not supply LaTeX semantics. [S4, S5]

**Decision:** start by evaluating tree-sitter-latex behind a small parser interface. Preserve raw source independently, including gaps and comments. Use conservative capability rules. Adopt a custom Rowan-based parser only if measured failures justify the additional maintenance. Never serialize the syntax tree back into the whole document.

### A compile service does not necessarily require rewriting latexmk

latexmk already coordinates repeated LaTeX runs and auxiliary tools. Its documentation also exposes control over configuration loading. [S6]

**Decision:** use a compile-backend interface. Prefer a controlled latexmk recipe where available; retain a narrowly specified direct-engine fallback. Do not present a two-pass fallback as bibliography-complete. Check dependencies during setup, including Perl where relevant. Do not automatically execute a project’s .latexmkrc.

### Very fast local previews are plausible, but different from final layout

Recent realtime-tex work describes a resident LuaLaTeX process for paragraph previews alongside a background full compile. Its documentation explicitly distinguishes local updates from delayed page layout, references, and other global state. The accompanying benchmark work isolates paragraph work from page building. These are upstream reports, not measurements of this app. [S7, S8]

**Decision:** evaluate reuse in a bounded performance experiment after correctness is established. An approximate or partially settled preview cannot authorize a move or become an exported PDF. Matching one baseline example does not establish safety for arbitrary future changes.

### Full rewrites and alternate engines have real costs

Tectonic is based on XeTeX and offers a useful packaging approach; it is not a substitute for LuaTeX’s callback model. TeXpresso documents a custom XeTeX engine and a different rendering stack, with tested platforms that do not establish our Windows integration. [S9, S10]

**Decision:** keep a standard-engine path and a replaceable compile interface. Do not make maintaining a TeX engine fork part of the initial project.

## 6. Architecture and ownership

| Component | Owns | Must not own |
| --- | --- | --- |
| Rust document/session core | Canonical buffers, revisions, patches, history, pending operations | Browser rendering |
| Syntax and capability layer | Source structure, editable-unit candidates, refusal reasons | Execution of arbitrary TeX semantics |
| Compile service | Immutable snapshots, engine jobs, diagnostics, dependency manifests | User-file saves |
| Layout/verification service | Build-specific provenance, geometry, PDF checks, validation result | Unverified edits |
| TypeScript interface | Monaco projection, gestures, selection display, page virtualization | Canonical history or filesystem writes |
| Project persistence service | Explicit save, recovery journal, external-file conflicts | Compiler output generation |
| TeX support kit | Versioned portable movement adapter and private tracing | Hidden dependencies needed only by the app |

Start with a small number of Rust crates: a pure domain core, an infrastructure/services crate if useful, and the Tauri host. Organize text, syntax, operations, compilation, and layout as clear modules. Split additional crates when dependency boundaries justify it; seven crates are not a prerequisite for correctness.

Convert frontend modules to strict TypeScript in manageable slices. Retain the existing package manager and useful tests unless a concrete problem requires a change. Generate IPC types from Rust using one selected tool, with runtime validation for incoming messages.

The durable document is the user’s LaTeX project. The layout sidecar, syntax tree, PDF, indexes, and editor view are derived artifacts.

## 7. Source storage, patches, and one undo history

Each file retains original bytes, encoding/BOM metadata, and line-ending information. UTF-8 is the initial editable encoding; unsupported encodings open without lossy conversion and offer an explicit conversion path. Monaco’s normalized representation must not silently rewrite untouched CRLF or mixed-EOL content.

A rope can store editable Unicode text efficiently. Ropey has byte/character/UTF-16 conversion APIs, but those index spaces remain distinct; patch ranges are always validated at actual character boundaries. [S11]

A patch contains:

- Project identity, base revision, and transaction ID.
- File identity and expected file state.
- Sorted, non-overlapping replacement ranges.
- Exact expected old bytes, with a strong fingerprint for fast checks.
- Replacement bytes and a precomputed inverse.
- Operation kind and validation policy.

Apply patches as one all-or-nothing session transaction. Reject overlap, invalid Unicode boundaries, stale revisions, changed expected text, missing files, and conflicting auxiliary-file creation. Bytes outside declared edits must remain identical.

Rust owns history. Monaco reports edits through an ordered, acknowledged protocol and receives non-echoing updates. Typing is optimistic for responsiveness, but there is only one canonical committed sequence. Visual actions wait for outstanding source changes and IME composition to settle.

Route keyboard undo, toolbar undo, context-menu undo, redo, multi-cursor edits, paste, completion, and file switching through the same history policy. Monaco’s own edit APIs have different undo effects, so this is a tested integration, not just rebinding Ctrl+Z. [S12]

Typing remains accepted even if it produces invalid TeX. Only proposed **visual transformations** require a successful candidate build before committing. This corrects the overly broad “nothing is accepted until it compiles” rule in the supplied review.

## 8. Revisions and the visual transaction protocol

Use distinct identifiers for project session, document revision, operation, build, layout schema, and toolchain/dependency fingerprint. A build identifies all source and asset inputs, root document, engine/options, support-package version, relevant dependencies, and render artifacts.

Every geometry object and selection belongs to a specific build. Revisions are monotonic; undo creates a new revision even when its bytes match an earlier state.

~~~mermaid
flowchart TD
    A["Gesture on current build"] --> B["Plan guarded source patch"]
    B --> C["Compile immutable candidate"]
    C --> D{"Validation passes?"}
    D -->|No| X["Keep source and last good PDF"]
    D -->|Yes| E["Prepare visible PDF pages"]
    E --> F{"Base revision still current?"}
    F -->|No| X
    F -->|Yes| G["Commit one history transaction"]
    G --> H["Publish matching source and preview"]
~~~

The exact publication protocol is:

1. Capture the current acknowledged revision and selected build.
2. Plan a candidate without mutating canonical source.
3. Compile the complete candidate snapshot to convergence under its recipe.
4. Validate source, target identity, geometry, and operation-specific layout.
5. Have PDF.js parse the candidate and prepare visible pages offscreen.
6. Enter a short ordered commit barrier: verify revision and dependency identity again; reject any superseded operation.
7. Commit source patches, inverse/history, and accepted-build identity in Rust.
8. Publish only that accepted revision in the frontend. Older events cannot move the editor or preview backward.

A source edit, undo, project switch, dependency change, or cancellation during steps 2–6 supersedes the candidate. Preserve user typing. Never rebase and commit a PDF that was verified against different inputs.

While the PDF is stale, reading remains available but visual transformations are disabled. Untouched source ranges are not enough to justify using old coordinates: an edit earlier in the document can move them.

If the UI crashes after the core commit, recovery restores the accepted source/history and rebuilds the view. It does not pretend a distributed UI/core operation is magically atomic on disk.

## 9. Geometry and provenance experiment: the first major decision

Before broad refactoring, build one fixture-driven command-line experiment. Compare:

| Approach | What is being tested | Main risk |
| --- | --- | --- |
| A. Existing marker adapter + SyncTeX | Whether the corrected baseline can safely identify and move supported units | Ambiguous source evidence and graphics-state interactions |
| B. Lua attributes + shipout sidecar + PDF verification | Whether source identity survives layout and yields reliable final positions | Shaping, copied boxes, package callbacks, transforms |
| C. Reuse selected upstream tracing components | Whether existing code reduces maintenance without changing the contract | API maturity, dependency assumptions, semantic mismatches |

Do not combine successful pieces without rerunning the same tests on the combination.

Prototype ten cases: a three-line signature; justified wrapping/hyphenation; bold/italic and ligatures; repeated identical names; a local name macro; minipage content; two columns; a page boundary; included source files; transformed or linked content expected to refuse.

Lua instrumentation must:

- Allocate its own attributes and cooperate with existing callbacks.
- Preserve original source with a reversible shadow-to-source offset map.
- Avoid inserting typeset nodes between letters merely to tag them.
- Test whether attribute changes themselves alter shaping or break ligatures.
- Capture appropriate post-line-break and shipout information, including content that never traverses ordinary paragraph breaking.
- Use the documented LaTeX callback infrastructure; distinguish kernel-defined shipout hooks from LuaTeX’s native callback API. [S13]
- Track nested boxes, shifts, glue setting, font expansion, directions, and final page transforms.
- Detect repeated use of one source fragment and ambiguous output instances.
- Produce a schema-versioned sidecar tied to the PDF hash and build manifest.

A sidecar unit needs an original file/range, a build-local identity, an optional persisted movement ID, page fragments, baseline/advance information, transform information, and explicit capability reasons. Persistent IDs must survive reload; ephemeral layout IDs must never be treated as persistent.

**Gate A: selection provenance.** On all mandatory fixtures, a click selects the intended source occurrence or returns the expected refusal. No wrong positive is allowed. Instrumented and clean builds retain the same visible layout under pinned conditions.

**Gate B: displacement.** A known translation moves exactly the selected material, leaves all other material unchanged, and produces a valid portable project.

If A fails, reduce the supported source shapes or select another mapping approach and rerun. If B fails, retain editing/navigation while replacing the movement adapter. Do not proceed by weakening the checks. A failure affecting the signature use case reopens the architecture; it cannot be hidden as “almost done.”

## 10. Movement, spacing, and portability

There are two separate user operations:

**Move without reflow.** Shift only the selected drawing. Its original contribution to the layout stays intact. Neighbors, page count, and text content must remain unchanged. Relative displacement is anchored to the source location, so later reflow repositions the anchor naturally.

**Adjust spacing.** Change a recognized source length or structural spacing operation. Following content may reflow. This cannot use the “all neighbors unchanged” validator, and the requested spacing change need not equal a simple physical displacement after page breaking or flexible glue.

Prefer editing an existing, well-understood source parameter where that matches the requested action. Do not invent a nearby vspace command simply because it appears visually convenient.

For free displacement, the persisted adapter is selected by Gate B. The existing macro, a carefully designed environment, and a Lua-specific adapter are candidates. Environment syntax alone does not solve boxing, verbatim, catcode, scope, or page-break problems.

The adapter must pass zero-offset identity and preserve inherited font/width/context for supported units. Unsupported footnotes, links, inserts, floats, page breaks, and ambiguous side effects are rejected before manipulation.

Portable support files are versioned and saved beside the source when required. Never silently overwrite a user-modified or incompatible package. No shadow instrumentation is required to compile an exported project.

“Reset position” sets displacement to zero. “Remove visual wrapper” preserves the current content and removes positioning syntax. “Undo” restores exact earlier bytes. These are different operations: removing a wrapper after editing its content must not resurrect the old wording.

A cross-engine portability claim requires a three-engine fixture matrix. Until it passes, portability means independently compilable with the project’s declared supported engine—not an unsupported promise about every journal toolchain.

## 11. Engine support policy

Do not silently switch an imported document’s engine.

| Engine situation | Guaranteed base behavior | Visual capability rollout |
| --- | --- | --- |
| Tested LuaLaTeX profile | Open, edit source, compile, read, export | Primary candidate for full text selection/edit/move |
| Existing pdfLaTeX project | Preserve its recipe and display | Prove the signature workflow with the existing/selected adapter early; enable only passing operations |
| Existing XeLaTeX project | Preserve its recipe and display | SyncTeX navigation first; additional operations require engine-specific gates |
| Unsupported or unusual recipe | Source access and a precise setup/build explanation | No invented compatibility |
| Proposed engine change | Build a separate comparison without overwriting the project | User chooses only after seeing layout and dependency differences |

The initial alpha may demonstrate full editing on LuaLaTeX. The first release advertised as covering the user’s existing pdfLaTeX signature workflow must pass that exact workflow under pdfLaTeX. If it does not, report the limitation and seek a scope decision rather than silently narrowing the goal.

## 12. Compilation, snapshots, and reproducibility

Compile immutable snapshots in a short app-managed path. Copy user files or use a proven copy-on-write mechanism. **Never hard-link writable compiler inputs to the user’s originals.** A compiler write through a hard link could alter the project.

Freeze sources, images, bibliography inputs, and allowed external dependencies. Detect asset edits, renames, and symlink/reparse-point changes. Use dependency-recorder information where available to build an input manifest. Unknown or changing dependencies invalidate verification.

Use separate build identities for display, candidate validation, and clean export. Cache only matching inputs and toolchains. Seed auxiliary files only from a compatible accepted state; keep a clean rebuild path.

The compile service owns:

- Toolchain discovery and a preflight check.
- Engine-specific arguments and auxiliary-tool availability.
- Controlled latexmk or an explicitly limited direct-engine recipe.
- Convergence and rerun diagnostics, bounded passes, elapsed-time and memory limits.
- Cancellation of the whole process tree.
- Bounded logs and typed, file-linked diagnostics.
- Missing-package reporting without a surprise install dialog.
- Immutable result publication and cache eviction.

MiKTeX documents a disable-installer flag; use the verified engine-specific invocation. Installing packages is a separate setup action. [S14]

Baseline and candidate comparisons must use matching settings, fonts, package versions, time inputs, and auxiliary state. Do not silently change the user’s date semantics for ordinary exports. Pin time for fixtures; where needed, freeze a comparison session’s time and expose the setting. Random or stateful output gets an instability diagnostic instead of a false geometry verdict.

Errors, missing identity, and nonconvergence reject visual candidates. Undefined references after the required reruns are errors for verified export. Underfull boxes and ordinary existing warnings are shown rather than blanket-rejected. New clipping, missing glyphs, or overfull content relevant to a visual operation follow an explicit diagnostic policy.

## 13. Independent verification, with meaningful comparisons

Use PDF.js for the visible app and an independently pinned PDFium inspection worker for validation/fixtures. Packaging and dependency notices are part of adopting that worker.

PDFium exposes character bounds in PDF user space and distinguishes tighter character boxes from looser glyph bounds. TeX advance boxes, PDF ink bounds, and selection padding are different things. We must compare corresponding quantities rather than assert that all rectangles should be identical. [S15]

Define typed spaces: source byte offsets, editor UTF-16 positions, TeX scaled points, TeX points, PDF user units/bp, CSS pixels, and device pixels. Normalize page origins, CropBox/MediaBox, UserUnit, rotation, zoom, and device scale through tested transforms.

Validation layers:

1. **Source:** unchanged bytes outside declared patches; exact inverse; no unintended file creation or deletion.
2. **Identity:** target provenance is unique and tied to the candidate build.
3. **Structure:** PDF parses; expected pages/boxes exist; relevant annotations and object roles are preserved.
4. **Geometry:** compare baselines/advances, transformations, target fragments, and unaffected material according to the operation.
5. **Appearance:** compare clean/instrumented and before/after raster output using the same renderer and deterministic configuration.
6. **Independent renderer:** verify the candidate in both PDF.js and PDFium, with the known broken-literal fixture required to fail.

Initial displacement target: requested translation within 0.25 bp for comparable target positions, and no unaffected-position change beyond a tighter declared numerical tolerance. Establish tolerances using exact fixtures before implementing the adapter. Do not round coordinates to one decimal place to manufacture agreement.

For zero-offset and instrumentation identity, require pixel equality in deterministic same-renderer fixtures. Cross-renderer raster equality is not expected because anti-aliasing differs; compare semantics and within-renderer before/after differences instead.

Target masking must not hide unrelated graphics or text. Check all pages for page/geometry changes; include non-text objects and annotations. If runtime verification cannot isolate a target from overlapping material, reject the unsupported operation or use a stronger inspection path. Do not use a large rectangular mask as proof that only the target changed.

Text replacement legitimately causes reflow. Its contract is exact source patching, expected edited content, valid build, and a preview of layout consequences—not a blanket requirement that all later pages remain unchanged. Heading edits may legitimately affect running heads and the contents.

## 14. Saving, recovery, and external edits

Save is explicit. Typing recovery is separate and does not overwrite the project.

Before saving, compare disk fingerprints with the loaded version. If another editor changed a file, offer reload/merge/save-copy behavior; never overwrite it silently.

A save that touches source plus a new support package is a recoverable multi-file transaction:

1. Validate every destination and stage new bytes in the same volumes.
2. Record a manifest of originals and intended replacements.
3. Flush staged data and replace files using tested Windows behavior.
4. Complete or recover the manifest after interruption.

Do not claim filesystem-wide atomicity for multiple independent renames. Simulate failures between replacements, disk full, permission denial, locked files, and interrupted support-package creation.

On startup, recovery checks journal integrity and project identity before restoring unsaved work. Keep private recovery data local with bounded retention. Save/reopen tests must not rely on in-memory undo state or cached PDFs.

Export compiles the chosen current revision cleanly without private tracing. If the current source is invalid, the UI may offer the last successful PDF only with its revision/stale status clearly stated.

## 15. Windows reliability and trust boundaries

Use Windows Job Objects for timeout/cancellation and process-tree cleanup. Assign the process safely before it can launch uncontrolled children; verify kill-on-close behavior and nested-job constraints in the actual host. Job Objects manage lifetime/resources, not filesystem isolation. [S16]

Keep shell escape off by default. Do not claim that this makes arbitrary TeX safe. TeX Live’s current documentation notes that openin_any no longer restricts reads, so that variable cannot be our security boundary. [S17]

For the first release, opening sources is separate from trusting a project to compile. Project configuration scripts and custom build commands require explicit trust. A future untrusted-project mode needs actual OS-level isolation and adversarial tests before being advertised.

Use narrow Tauri capabilities, safe path validation, a local asset protocol, and strict content handling. Treat logs and source text as text in the UI. PDF scripts, launch actions, embedded files, and external links do not execute automatically.

Handle long/Unicode paths, spaces, standard-user installs, OneDrive/file-lock behavior, multiple monitors, DPI changes, and app shutdown during a build. Do not ask users to disable antivirus or add exclusions as a routine performance fix.

## 16. Performance and the meaning of “live”

These are proposed acceptance targets, not achieved measurements. Measure on the user’s laptop and a documented Windows reference machine, recording median and p95 separately.

| Interaction | Initial target |
| --- | --- |
| Editor keystroke feedback | No compiler-blocking work; p95 visual feedback within 50 ms |
| Hover and selection on prepared layout | p95 within 50 ms |
| Drag preview | Target a 16.7 ms frame budget on the reference 60 Hz setup |
| Pointermove compilation | Exactly zero compiler jobs |
| Drag release | One logical candidate build, possibly several internal TeX passes |
| Warm small-document verified result | Goal: p95 within 3 seconds on a defined 1–10 page fixture suite |
| Cancellation response | UI acknowledges immediately; worker termination measured separately |
| Large-document scrolling | Virtualized pages, bounded cache, no rasterizing every page upfront |
| Resource use | Stable under a repeated-operation soak test; no unbounded retained builds/pages |

Measure snapshot, engine, auxiliary tools, sidecar extraction, PDF verification, and visible-page preparation separately. A slow bibliography cannot be hidden in an average gesture number.

Optimize in this order: coalesce superseded work; cache compatible inputs; reuse auxiliary state; virtualize rendering; reduce copies; profile the real bottleneck. Then test a resident LuaLaTeX/rtex backend behind the same protocol.

Local paragraph previews, if adopted, are marked pending until the full layout settles. Never cache partial layout as a verified export. Do not use include-only compilation as a correctness shortcut when counters, floats, or dependencies can change globally.

## 17. Test corpus and acceptance evidence

Start with about 20 compact synthetic fixtures and grow it with every reproduced defect:

- Signature: individual line, three-line group, bold name, forced breaks, and minipage context.
- Ordinary prose: justification, soft wraps, ligatures, hyphenation, font expansion.
- Identity: duplicate visible strings, repeated macros, copied boxes, ambiguous instances.
- Structure: comments, escaped braces, malformed input, verbatim/catcode regions.
- Project: includes, bibliography, root selection, relative assets, changed external assets.
- Layout: page edges, multiple pages, columns, nonzero crop origins, rotation.
- Content interaction: links, footnotes, tables, math, figures, colored backgrounds.
- Encoding: LF, CRLF, mixed endings, UTF-8 BOM, emoji, combining marks, CJK, Arabic/RTL.
- Classes: article, letter, KOMA-Script, one CV layout, and one appropriately licensed journal example.

Each fixture declares supported actions and expected refusals. The test suite must measure both correctness and useful coverage: a system that refuses every edit is safe but does not meet the product goal.

| Test layer | Required evidence |
| --- | --- |
| Unit/property | Patch inverse identity, outside-byte preservation, offset conversions, parser range safety |
| Parser fuzz | No panics/hangs, bounded resources, original bytes preserved even for invalid input |
| State-machine | All asynchronous interleavings: typing, undo, drag, cancellation, project switch, worker crash |
| Compile integration | Real engines, dependencies, bibliography, failure handling, convergence |
| PDF differential | Zero-offset identity, instrumented/clean identity, exact movement, unchanged non-target content |
| Native Windows UI | Actual mouse/keyboard interactions in the built Tauri app |
| Persistence | Save/reopen, external conflicts, crash/disk-error recovery, portable independent build |
| Performance | Recorded latency distribution, cache behavior, long-session resource stability |

Current Tauri documentation supports WebDriver testing. Use a pinned test harness against the actual Windows app, matching the native driver/runtime where that route is selected. Browser-only tests and mocked IPC do not count as native acceptance. Test-only control hooks must not ship enabled. [S18]

The signature acceptance scenario:

1. Open the synthetic letter from disk and compile.
2. Read/copy text successfully.
3. Select the name alone, then all three signature lines with the mouse.
4. Move the group down exactly 12 bp.
5. Verify matching source changes, unchanged surrounding content, and one history entry.
6. Edit the name while preserving bold formatting and line spacing.
7. Undo twice and compare every source byte with the starting project.
8. Redo, save, close, and reopen.
9. Rebuild with the app cache removed.
10. Compile the exported project outside the app and compare output.
11. Repeat while deliberately causing a compile error and while typing during a candidate build.
12. Repeat at supported DPI/zoom settings and under each engine advertised for this workflow.

## 18. Milestones and decision gates

| Stage | Concrete work | Exit evidence |
| --- | --- | --- |
| P0 — Establish baseline | Reconcile status, reproduce current tests, preserve regression artifacts, inventory engines and native build | One truthful capability/build report; a reproducible starting point |
| P1 — Prove mapping and movement | Run the A/B/C experiment; define coordinate/appearance contract; compare parser options | Gates A and B pass, or a written architecture/scope revision |
| P2 — Own source safely | Rust patches/history, encoding projection, ordered IPC, Monaco integration, recovery foundations | Property tests and IME/multi-cursor/undo native checks pass |
| P3 — Build reproducibly | Snapshot/dependency manifests, engine recipes, convergence, cancellation, diagnostics, clean export path | Real compile corpus and fault-injection tests pass |
| P4 — Select real content | Layout sidecar/index, PDF text mode, click/hover/marquee, source synchronization, stale-state behavior | Correct-or-refused mapping on every mandatory fixture; mouse group selection proven |
| P5 — Edit and adjust spacing | Literal-run editor, protected commands, raw fallback, parameter-specific spacing adapters | Formatting preserved; failures/races cannot corrupt source; reflow policy proven |
| P6 — Complete displacement | Chosen portable adapter, drag/nudge, source/PDF publication, independent verification | Full signature acceptance scenario passes in the native executable |
| P7 — Harden and broaden | Crash recovery, external conflicts, longer documents, engine matrix, performance, usability testing | No open data-loss/wrong-target defects; all advertised capabilities backed by fixtures |
| P8 — Package a release | Installer, clean-machine setup, support files/notices, migration, versioned evidence, release candidate | The exact installed artifact passes the release checklist |
| P9 — Add richer adapters | Images, typography, tables, templates, advanced shaping/cross-page research | Separate gates per feature; no weakening of existing contracts |

P1 is time-boxed around a finite matrix, not endless open-ended research. Its result decides the implementation route before broad coding. The benchmark and native validation environment must exist before calling it passed.

After approval, work proceeds through these stages without seeking permission for each routine implementation decision. A material change to the accepted product contract, a paid service, or public distribution is brought back with concrete evidence.

A reliable release is a multi-stage engineering project. Calendar estimates should be reset after P1 demonstrates the chosen mapping and movement route. Fast code generation does not remove the cost of compatibility and native testing.

## 19. First implementation backlog after approval

These are the first reviewable work packages, not work performed by this document:

| ID | Change | Acceptance |
| --- | --- | --- |
| B01 | Record baseline and reconcile contradictory status docs | Exact commit/build/test evidence; unverified items labeled |
| B02 | Create standalone PDF comparison harness | Known broken graphics-literal fixture is rejected by the intended checks |
| B03 | Define typed coordinate transforms | Synthetic crop/rotation/DPI/pt-to-bp cases round-trip |
| B04 | Prepare the ten P1 fixtures and engine manifests | Each independently compiles or has an explicit expected failure |
| B05 | Test the current corrected movement adapter | Zero-offset, translation, neighbors, non-text content, portable compile |
| B06 | Implement minimal Lua provenance spike | Safe unit IDs survive to final page evidence on mandatory cases |
| B07 | Compare parser options on the same sources | Exact source storage and conservative capability behavior documented |
| B08 | Select architecture in an ADR | Evidence table, limitations, selected fallback, no invented passes |
| B09 | Implement guarded patch/inverse core | Property tests including Unicode, BOM, CRLF, mixed EOL |
| B10 | Integrate ordered source protocol and history | Native typing/IME/undo paths behave consistently |
| B11 | Implement snapshot and build scheduler | Superseded jobs cannot publish; process cleanup and failure paths proven |
| B12 | Deliver first click-to-source native slice | Open project, compile, select exact signature source, no mutation |

Subsequent packages follow P5–P8. Keep changes reviewable by behavior; do not impose an arbitrary line count that forces one coherent change into unsafe fragments.

## 20. GitHub, migration, and release discipline

Keep https://github.com/nadeemtsf/texloom as the primary repository. No second replacement repository is needed. Preserve the inspected prototype commit and use short-lived branches for experiments and implementation.

The requested local parent folder remains a separate workspace-access issue; moving it is not required to validate this plan or architecture. Do not delete the active checkout to solve it.

Use gradual module replacement with regression tests. Preserve existing StudioBlock projects through an isolated compatibility adapter; do not silently rewrite them into the new object format. Introduce migrations only with backups, explicit version detection, and round-trip tests.

CI should grow with the milestones: formatting/lint/types and fast tests; pinned TeX/PDF corpus; then native Windows end-to-end gates. Record pinned toolchains and dependency provenance. Require actual results before marking a milestone complete.

Protect main using available repository controls; verify account/plan support rather than assuming every GitHub security or ruleset feature is enabled. Keep generated caches, user documents, credentials, and large build outputs out of Git. Retain small intentionally chosen binary regression fixtures when they cannot be faithfully regenerated from a future toolchain.

An acceptance expectation may be corrected when demonstrably wrong, but the change must explain why and preserve the original regression evidence. Updating a golden output because a broken implementation produced it is not a fix.

Every release candidate names the source commit, dependency/toolchain versions, artifact checksum, supported-engine matrix, passed tests, blocked tests, and known limits. A portable executable, installer, and Store listing are separate deliverables.

Use a standard-user Windows installer with tested WebView2 setup. End users must not need Python, Rust, or Node to run the packaged app; TeX is detected or set up through a clear flow. Tauri provides Windows installer/runtime distribution options to evaluate. [S19]

Microsoft Store publication, signing-provider selection, and automatic updates follow a verified installer. Recheck current eligibility, costs, and submission requirements at that stage. No account purchase, signing enrollment, or public submission is part of approving this implementation plan.

## 21. Definition of a release we can honestly call reliable

All of the following must be true:

- The exact signature workflow works repeatedly with the mouse in the packaged Windows app.
- Supported source and visual edits use one correct undo history.
- No open critical data-loss, wrong-target, stale-commit, or save-corruption defect remains.
- Every advertised engine/action combination passes its declared corpus.
- Unsupported operations provide a useful reason and a source fallback.
- The exported project compiles independently using its documented dependencies.
- Compile failure, cancellation, worker crash, and external edits preserve user work.
- Display scaling, zoom, keyboard operation, and text-copy behavior are tested.
- Performance results and supported document sizes are published from actual measurements.
- The installed artifact is tested on a clean Windows account/machine.
- A small usability trial demonstrates that people can select, move, edit, undo, and save without developer guidance. Wrong-target actions are a release blocker.
- The shipped documentation and download identify the same tested version.

## 22. Approval scope

Approval means proceeding with this Windows desktop plan, preserving the existing repository, running P0/P1 first, replacing internals where the evidence supports it, and implementing the gated text-editing workflow through packaging.

It also accepts an honest compatibility policy: safe supported editing first, explicit refusals elsewhere, and broader adapters added through separate evidence. The app will not claim universal reversible editing of arbitrary LaTeX.

The first result after approval should be a reproducible baseline and the mapping/movement experiment—not another visual redesign or a claim that the app is already finished.

## Research sources and limits

Primary sources consulted on 1 November 2026. Technical recommendations above are design judgments; upstream feature claims are not local test results. The linked TUG preprint was inaccessible during retrieval, so its contents are not treated as independently reviewed; the public benchmark repository was available.

- **S1 — SyncTeX author paper:** https://www.tug.org/TUGboat/tb29-3/tb93laurens.pdf — synchronization granularity and transformation limitations.
- **S2 — LuaTeX manual:** https://tug.ctan.org/systems/doc/luatex/luatex.pdf and https://www.luatex.org/documentation.html — attributes, node processing, callbacks, PDF literal modes; match documentation to the tested binary.
- **S3 — PDF.js API source:** https://mozilla.github.io/pdf.js/api/draft/api.js.html — marked content, transforms, text normalization, rendering APIs.
- **S4 — tree-sitter-latex:** https://github.com/latex-lsp/tree-sitter-latex — reusable grammar and documented limitations; repository identifies MIT licensing.
- **S5 — rust-analyzer syntax architecture:** https://rust-analyzer.github.io/book/contributing/syntax.html — lossless syntax principles and Rowan separation.
- **S6 — latexmk author documentation:** https://www.cantab.net/users/johncollins/latexmk/ and https://www.cantab.net/users/johncollins/latexmk/latexmk-488.txt — build orchestration and configuration controls.
- **S7 — realtime-tex:** https://github.com/HenryXiaoYang/realtime-tex/blob/main/README.md and https://github.com/HenryXiaoYang/realtime-tex/blob/main/docs/live-editing.md — resident preview architecture and explicit deferred-layout cases; not benchmarked locally.
- **S8 — public LuaLaTeX benchmark:** https://github.com/texlode/luatex-benchmark — explains paragraph timing and what it does not measure.
- **S9 — Tectonic:** https://tectonic-typesetting.github.io/en-US/ — engine and packaging alternative.
- **S10 — TeXpresso:** https://github.com/let-def/texpresso — architecture and tested-platform description.
- **S11 — Ropey API:** https://docs.rs/ropey/latest/ropey/struct.Rope.html — byte, character, and UTF-16 conversion interfaces.
- **S12 — Monaco API:** https://microsoft.github.io/monaco-editor/typedoc/interfaces/editor_editor_api.editor.ITextModel.html — model edits and undo behavior.
- **S13 — LaTeX kernel sources:** https://www.latex-project.org/help/documentation/source2e.pdf — kernel callback/shipout integration; also https://github.com/latex3/tagpdf/blob/main/tagpdf-mc-luacode.dtx as an existing callback-use reference.
- **S14 — MiKTeX documentation:** https://docs.miktex.org/manual/texfeatures.html — automatic package-install controls.
- **S15 — PDFium public text API:** https://pdfium.googlesource.com/pdfium/+/main/public/fpdf_text.h — character/loose boxes and coordinate semantics.
- **S16 — Microsoft Job Objects:** https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects — process grouping and termination.
- **S17 — TeX Live changes:** https://tug.org/texlive/bugs.html and https://tug.tug.org/pipermail/tex-live-commits/2026-January/035883.html — openin_any no longer constrains reads.
- **S18 — Tauri testing:** https://tauri.app/develop/tests/webdriver/ and https://learn.microsoft.com/en-us/microsoft-edge/webdriver-chromium/ — actual-app test routes and driver compatibility.
- **S19 — Tauri Windows distribution:** https://tauri.app/distribute/windows-installer/ — installer and WebView2 options.

Licenses, bundled binary notices, and exact compatibility must be reviewed for the pinned versions actually adopted. A repository badge is not a complete dependency-license audit.
