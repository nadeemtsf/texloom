# Core completion and later work — 7 November 2026

Expanded-product planning: this checklist describes the earlier bounded core and its pending acceptance. The complete Word-like authoring goal is not implemented. [The new roadmap](architecture/ROADMAP.md) and [GitHub #44](https://github.com/nadeemtsf/texloom/issues/44) plan the shared architecture, source-native edit/Delete/reorder/spacing workflow and later visual typing/object editing. Architecture preparation and current owner acceptance can proceed in parallel; do not equate the older bounded-core completion wording with completion of the new product scope.

Latest7November update: ordinary r8 supersedes r7 as the prepared acceptance candidate; its notice supplements preserve the same DLL/worker. No r8 installer/app interaction has run. The legacy-support native pilot passed five intended moves and25 corrupted-packet refusals with independent reference/source/Undo/disk checks. Current draft v10 has26pages/21references; final evaluation remains unfrozen/unrun. GitHub tracker#25 now includes18child tasks with3scoped implementation tasks closed. The active usage floor is57%weekly remaining, replacing older values below.

The approved bounded core is implemented; acceptance is incomplete. This checklist supersedes old pending classifications only where newer evidence is explicitly named. It does not promise arbitrary LaTeX visual editing. Latest prepared candidate: r8, ordinary executable SHA-256 `60ef2e8ea6e790c570f0cb3c15f87741c5cc4976deda78f28763233f28dfad9b`. No new UI input was issued by the agent. Use `MANUAL-FOLLOWUP-R8-2026-11-07.md` with the fresh r8 synthetic fixtures; older named-artifact results keep their original scope.

## Core acceptance before expanding features

- [ ] Owner installs/opens the exact r8 candidate; identify it separately from earlier installed and portable copies.
- [ ] Owner checks Arabic and Hebrew appearance, paragraph selection, a small accepted move, unrelated-content preservation and Undo in r8. Also check the English paragraph in that same project.
- [ ] Owner confirms current-build English/Arabic/Hebrew source entry, rendered output and PDF search/copy. Earlier builds have evidence; the recent owner response did not execute this entire case.
- [ ] Owner checks Keep draft and close, explicit recovery-project selection, exact recovered text, Undo/Redo and unchanged original files using the fresh r8 disposable fixture. Do not discard real drafts.
- [ ] Confirm successful new PDF publication after removing a compile error. Expected failure and retained good PDF were reported; subsequent successful publication was not independently established.
- [ ] Final-build smoke check of the ordinary supported workflow: signature/group/content edit/history/Save/reopen/export and independent compilation of the saved project. Reuse older evidence where code and scope justify it; do not erase prior passes or rerun every case without a reason.
- [ ] Fix any reproducible failure, preserve its evidence and repeat the affected checks. There is no known large missing core feature to implement before these checks.

Already reported by the owner: signature selection/group movement/name replacement/Undo/Redo/Save/reopen, intended repeated occurrence, and held-drag Escape/Alt+Tab cancellation on earlier copies. Ampersand escaping was not exercised by the owner's actual replacement literal. Arabic/Hebrew object-marker and close/recovery changes are implemented with headless/native-backend checks; actual owner acceptance of the latest candidate remains open.

## Performance and reliability work

- [ ] Validate timing boundaries in the real Windows app: source edit/feedback, source mapping, zero-probe compilation, release-to-verified-move, text edit/reflow, publication and Save. Distinguish browser paint opportunity from physical display presentation.
- [ ] Measure ordinary-versus-diagnostic tracing overhead and account for failed, cancelled, superseded, missing or dropped samples.
- [ ] Collect controlled latency distributions for small and large synthetic projects, with recorded engines/fonts/hardware and background-load limitations.
- [ ] Run an isolated single-instance long-session/resource test. The older mixed-instance sample does not establish a leak pass.
- [ ] Optimize only demonstrated problems and repeat the relevant measurements. Instrumentation exists; acceptable end-to-end performance is not yet established.

## Wider Windows and release gates

- [ ] DPI/scale changes and multiple-monitor behavior; another monitor is currently unavailable.
- [ ] Clean Windows account/machine setup, WebView2, missing TeX/font/package guidance and first successful compile.
- [ ] Uninstall/reinstall while retaining projects and recovery data. Existing-account install evidence does not replace this; an earlier uninstall route was blocked and was not bypassed.
- [ ] Current named-artifact representative adapter checks, including source-preserving spacing/typography/layout/image/table-placement controls. Historical GUI and current headless matrix evidence have different scopes.
- [ ] Complete bounded verifier dependency provenance/component/license/advisory review. Archive publication attestation and hashes do not establish every nested source-to-DLL input or security absence.
- [ ] Conduct an unassisted usability trial if making usability claims.
- [ ] Decide signing, updates and public distribution separately. They are not authorized actions or prerequisites for developing locally.

## Research work, separate from core feature completion

- [ ] Stabilize the core first, then seal a fixed source/binary/environment version and independently reviewed fixture/oracle/label manifests.
- [ ] Run the predeclared final evaluation on fresh synthetic cases: repetition, ambiguity, multilingual targets, engines, unsupported structures, compile failures and unintended source/render changes.
- [ ] Run feasible paired baselines/ablations with identical labelled requests, preserving failures and separating operation families.
- [ ] Analyze incorrect/correct acceptances, valid refusals, source/render preservation and latency with the correct denominators and uncertainty. Regression counts are not accuracy trials.
- [ ] Replace pending final results in the draft only after measurements, then rebuild and review the report. Current v10 is a 26-page, 21-reference technical draft; final evaluation is unfrozen/unrun.

## Explicit later scope

Additional languages/CJK IMEs, arbitrary math/table-cell/TikZ editing, unrestricted floats/macros, cross-page or rotated/complex-page manipulation and general WYSIWYG LaTeX are not missing promised core features. They need separate designs/capabilities and acceptance cases. Current table/image/layout support is through bounded parameters, not unrestricted internal editing. Keep unsupported cases safely refused.

The owner controls every mouse/keyboard test. Never compile `.text` files, add input languages, touch real projects for tests, publish or submit. The active usage floor is 57% remaining in the weekly window.
