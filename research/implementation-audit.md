# Implementation audit - 4 November 2026

## 7 November status reconciliation

The baseline audit below is historical; it is not a final freeze. Current prepared artifact r8 has ordinary executable SHA-256 `60ef2e8ea6e790c570f0cb3c15f87741c5cc4976deda78f28763233f28dfad9b`. The following later evidence supersedes only the corresponding older status. No new Windows app input or installation was performed in this continuation.

| Area | Implemented change or current policy | Executed evidence and remaining boundary |
| --- | --- | --- |
| RTL paragraph identity | The XeTeX wrapper orders directional markers without relaxing the selection proof. | [Owner-follow-up regressions](../docs/evidence/owner-followup-2026-11-06.json) retain PDF.js identity checks and real-engine movement cases. The [headless native mapping pilot](../docs/evidence/rtl-native-mapping-2026-11-07.json) found three declared targets and refused six wrong-target controls. Actual Arabic/Hebrew pointer selection in r8 is pending. |
| Protected close and recovery | Dirty close keeps a protected draft; restoring requires an explicit project choice. | Automated checks are recorded in the owner-follow-up receipt. Earlier normal-close discard is not evidence of crash loss. Owner confirmation of the replacement close/recovery flow is pending. |
| Legacy support migration | Only an exact owned legacy support file is eligible for the current upgrade; modified files retain their separate refusal policy. | [Legacy pilot](../docs/evidence/legacy-support-pilot-2026-11-07.json): five intended native moves met source, independent-render, Undo and unchanged-disk conditions; 25 corrupted packets were refused before compilation. These reuse development conditions, not new accuracy samples or RTL-target moves. |
| Independent verifier packaging | The PDFium worker/DLL match r7. Packaging retains all distributor notice bytes and two checked supplemental texts. | [Matched artifact evidence](../docs/evidence/pdfium-matched-artifact-notices-2026-11-07.json) and [r8 validation](../docs/evidence/r8-build-validation-2026-11-07.json) bind the bytes. Exact nested checkout provenance and broader component/advisory review remain unresolved. |
| Gesture cancellation | Escape and loss of focus cancel an active held drag. | The owner reported both held-drag Escape and Alt+Tab cancellation on an earlier named disposable project. This is an owner observation on that copy, not a new r8 test. |
| Compile failure and publication | Failed candidates should preserve the accepted PDF; a corrected compile should publish a new one. | Earlier owner evidence covers the failure/last-good-preview half. A [fresh fixture preflight](../docs/evidence/compile-followup-fixture-preflight-2026-11-07.json) confirms intended TeX success/error/correction states independently. Actual corrected publication in Texloom remains pending. |

Current required regressions record 291 frontend, 70 core unit, 18 core integration, 40 research-helper and two explicit native-verifier passes. These counts overlap with historical suites and are not an end-to-end accuracy denominator. Clippy completed with warnings retained. Current manual instructions are [the r8 checklist](../docs/MANUAL-FOLLOWUP-R8-2026-11-07.md). Methods, success criteria and the unfrozen final-study boundary are unchanged by this status reconciliation.

## Historical baseline audit

Audited product baseline: `c26a23851155f7e4e542eef80fae92e35c51f707`. This is a source/code audit plus a bounded review of recorded evidence. It is not a new correctness experiment or a final evaluation.

Supplement: local `0576198` adds a window-blur listener that cancels the active canvas gesture while retaining the accepted selection. Controller drag/marquee regressions pass; actual Windows input remains pending because the owner is away. The native acceptance, source planning and verification policies audited below are unchanged. The later independent-reference development pilot is documented separately in ORACLE-PILOT.md.

5 November supplement: `5e59e77` adds read-only setup guidance and opt-in static-category timing, with 252 frontend/88 default core checks and an ordinary Windows build passing. It does not change the native source planner/acceptance policies. Milestones distinguish build return (already including native layout approval for visual candidates), PDF.js preparation, build receipt recheck, commit and publication. The two-animation-callback endpoint is a presentation proxy, not observed hardware display latency; overhead/native validation remain pending. Environment metadata collected after the development pilot is explicitly partial and posthoc (ENVIRONMENT.md).

A development-only native candidate-entry harness now connects actual native prepare/build/approval/commit/Undo to independent byte-source and Poppler references (NATIVE-REFERENCE-PILOT.md). Five existing-object moves meet the corrected protocol and 25 deliberately altered packets fail source-plan rederivation before compilation. An initial TeX-box reference added an unrelated blank line; its failed comparisons and zero/five result remain intact. A corrected reference plus gold-only geometry health preceded a fresh run with unchanged native executable/packets. This is neither actual PDF selection nor a full-pipeline/final accuracy study.

The provisional question describes the main problem, but overstates uniformity if applied to every feature. A more testable question is: **For a declared subset of single-page LaTeX text moves, how effectively do constrained source patches and verification after compilation reject incorrect transformations, and what do they cost in valid-edit refusal and end-to-end latency?** Treat replacement/reflow and richer adapters as separate exploratory strata.

## Actual workflow

| Stage | Current implementation | What it establishes / does not establish |
| --- | --- | --- |
| PDF interaction | `web/core/canvas-controller.js`; `web/core/pdf-surface.js` | Click, group/marquee, drag outline and keyboard nudge use compiled text geometry. Pointer motion does not itself commit source. Event-boundary tests are separate from real held-gesture cancellation. |
| Source evidence | `web/app.js:resolveSourceSelection`; `src-tauri/src/service/visual.rs:resolve_text_selection`; `crates/texloom-core/src/selection_hints.rs` | Native SyncTeX file/line hints plus conservative source structure. Bounded hint enumeration does not authorize a guessed source range. Ordinary wrapped prose may expand to the whole paragraph. |
| Selection proof | `web/core/selection-proof.js` | A private zero-offset wrapper build must retain page geometry, runs and pixels in PDF.js; the marker must be unique and contain all clicked runs. A proof is cached only against its baseline view. This is a selection capability, not a source commit. |
| Native source plan | `src-tauri/src/service/visual.rs:plan_object_edit`; `src-tauri/src/service/layout_gate.rs:Plan::prepare`; `crates/texloom-core/src/document.rs` | Native code rederives the exact candidate from current sources, operation and expected content. UTF-16/editor offsets and UTF-8/source bytes remain distinct. Unknown grammar or changed preconditions can refuse. |
| Candidate build | `web/app.js:commitSources`; `src-tauri/src/service/document.rs:compile_document`; `crates/texloom-build/src/engine.rs` | Immutable snapshot, fixed recipe, engine/dependency receipts, bounded jobs and revision/cancellation checks. Compiler success alone does not authorize a normal-text visual edit. |
| Independent inspection | `src-tauri/src/service/layout_gate.rs:approve_layout`; `crates/texloom-build/src/inspection_worker.rs` and `verification.rs` | A pinned isolated PDFium worker is independent of PDF.js and the source planner. It is not an independent formal specification; prediction and inspection share PDFium. |
| Acceptance or refusal | `src-tauri/src/service/document.rs:document_commit`; `web/core/transactions.js` | A private one-shot approval binds project/revision/build/cancellation lease. Native commit and frontend preparation must both succeed. Stale or failed candidates retain accepted source/history/PDF. |
| Undo and durability | `crates/texloom-core/src/document/`; `crates/texloom-build/src/persistence.rs` and `recovery.rs`; `web/core/typing-recovery.js` | Shared history, explicit Save, external-file fingerprint refusal, staged multi-file recovery and separate typing protection. Recovery grants no old visual authority. Multi-file Save is recoverable, not a filesystem-wide atomic rename. |

## Contracts that must stay separate

1. **Displacement / identity.** `verification.rs` checks page structure, uniquely matched glyph geometry, non-text geometry and complete same-renderer raster equality against an expected translation. Current constants are target 0.25 bp, unaffected glyphs 0.02 bp, and non-text geometry 0.000001 bp. The fractional displacement path uses a separately rendered prediction from the baseline; candidate pixels do not construct that prediction. Target isolation, budget and page/annotation constraints can refuse.
2. **Protected text replacement / reflow.** `layout_gate.rs:reflow` checks multiplicities of extracted non-target text, non-text object count and visible target content. Positions and pages can legitimately change. This is not proof that every unrelated graphic or glyph retained its appearance/meaning.
3. **Semantic parameter/style adapters.** `Policy::Semantic` rederives an exact source plan, compiles and requires independent readable inspection evidence. It deliberately permits reflow and does not use the displacement preservation contract.
4. **Legacy positioned StudioBlock operations.** `Policy::Legacy` retains native source planning and the existing frontend acceptance path; it is not independently certified normal-text displacement.

The movement package retains the fragment's layout contribution while applying drawing translation. It does not generally rearrange TeX flow, solve arbitrary constraints, or invert macro expansion. Repeated printed strings need occurrence/provenance evidence; matching text alone is insufficient.

## Implemented, observed, planned

- Implemented in source: guarded text movement/replacement, shared source history, independent movement checks, bounded adapters, compile scheduling, project transfer/encoding/templates and recovery.
- Observed development evidence: automated suites, real-engine corpora and synthetic native signature/source/save/recovery workflows. Each observation is tied to its own source/artifact, not silently promoted to every later build.
- Not established: universal correctness, arbitrary package/shaping support, real held-drag Escape/lost capture, Windows DPI/multi-monitor transitions, clean-machine missing-dependency setup, uninstall, an unassisted usability trial, and reference end-to-end latency distributions.
- Research work still planned: an independently labelled final set, an oracle independent of the runtime verifier, paired offline ablations, traceable end-to-end timing and a sealed final evaluation version.

Older paragraphs in `docs/NATIVE-LAYOUT-GATE.md`, `docs/DISPLACEMENT-POLICY.md` and `VALIDATION.md` describe earlier checkpoints. For current behavior, source and later receipts take precedence. In particular, the old integer-pixel-only description does not describe the later prediction path. Historical failure records must remain intact.
