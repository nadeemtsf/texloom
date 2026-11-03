# Explicit Save and recovery checkpoint

The application protects explicit project saves with native disk fingerprints and recoverable journals. Automatic recovery of typing that never reached Save is now a separate implemented path; see UNSAVED-RECOVERY.md for its format, UI and validation limits. Neither checkpoint certifies native interaction or a release.

Rust records the exact disk state when a project opens. Save holds the canonical document revision guard through filesystem work, so an earlier request cannot write a newer editor revision. Changed, removed or newly occupied source paths produce a conflict; editor bytes/history and the saved-state marker remain unchanged. Existing StudioBlock dependency behavior stays in its isolated compatibility path.

Before replacing originals, the persistence component stages and flushes intended bytes, backups and an integrity-checked manifest. It validates project identity, paths, file/byte budgets, staged contents and current before/after states. Per-project locking coordinates Texloom instances. Several filesystem replacements are recoverable; they are not globally atomic or an operating-system compare-and-swap guarantee against every concurrent third-party rename.

A failed replacement attempts a checked rollback. Interrupted or rolled-back saves retain their intended bytes. The application lists affected files and offers explicit actions: finish the prior Save, restore earlier files, save the current editor version as a separate project, or confirm deletion of the recovery draft. Recovery reopens a new checked project session and refuses a stale editor revision. It does not automatically write recovery data over project files on open.

Project copies use canonical editor sources and checked current assets in a newly created directory outside the original. Original external text changes remain untouched. Incomplete copies retain a marker and are not reported as successful. Copying or discarding recovery data does not mark the original editor version as saved.

Unknown external states, corrupt or foreign manifests, reparse/symlink paths and unexpected recovery contents are refused. Cleanup checks contents as well as filenames, removes only recognized derived files and never recursively descends into unknown folders. Recovery retention and size limits are explicit; broader failure/retention usability still needs acceptance.

At source 426d052e7001883ac4e1bfc3610aeed24579ce07, all three CI workflows passed:

- Windows executable, portable package and checks: 41532942547.
- Three-engine movement/selection/edit proof plus Windows native mark identity: 41532942542.
- Compile-service engine/fault corpus: 41532942522.

Windows ran 156 Rust and 77 frontend checks, 233 combined. These include actual isolated-worker approval, external-edit refusal, locked-file and replacement-boundary rollback/recovery, Unicode/BOM/CRLF, copy behavior, stale revisions, native recovery reopen, draft deletion and UI-controller boundaries. Exact results: evidence/project-durability-ci.json. The marker corpus separately records 24 positives and three expected overlap refusals in evidence/native-marker-verified-run.json.

Local commands resumed on 3 November. Exact remote history was reconciled without changing working file bytes, and all 233 existing automated checks passed locally, including seven explicitly invoked native worker/gate checks. Launch and observation succeeded, but native input remained denied. Inspection exposed a missing Cancel compile element that aborted shell startup; the correction supplies the control and four full-shell integration tests. The corrected 81 frontend checks, 64 required core checks and fresh Windows build passed. See evidence/desktop-startup-local.json. The corrected build has not received native input acceptance. No alternate launch/automation may bypass the current denial.

Still open: native crash/relaunch and broader recovery acceptance, performance/retention/usability, P6 fractional movement and broader paint/coordinate handling, earlier native typing/IME/mouse/signature acceptance, P8 installation/clean-machine gates and P9 richer adapters. Ordinary-PDF expansion is outside the current phase scope.
