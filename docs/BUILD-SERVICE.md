# P3 compile service

The owner authorized P3 while deferring P2's native typing/IME/mouse acceptance to manual testing. This checkpoint does not sign off P2 or certify a release.

## Responsibilities

The new texloom-build crate owns source/asset snapshots, dependency receipts, direct engine recipes, diagnostics, process lifetime and newest-request scheduling. It does not parse PDF layout or modify canonical document history. The Tauri service binds each job to a project, acknowledged revision, purpose and private candidate token.

One compiler runs at a time. A newer request cancels the active request and supersedes older waiting requests. Request-ID cancellation fences also reject a request that reaches the worker after Cancel. Source edits, profile changes and history invalidate running work when they advance the canonical revision; no-op edits do not cancel a valid job. The frontend keeps the idle typing debounce; cancellation does not trigger a new compiler for every keystroke.

## Inputs and receipts

Canonical UTF-8 buffers define source membership. Text removed from the in-memory project cannot reappear from an old disk copy. Supporting packages are injected only when absent from canonical sources. Assets are copied, never hard-linked. Source aliases, case collisions and project symlinks/reparse points are refused.

The private snapshot manifest records SHA-256 hashes, compiler invocation and binary identity, version, arguments, selected TeX/time environment settings, source/asset inputs, recorder-discovered toolchain inputs, generated inputs, pass count and sealed PDF/SyncTeX/measurement/recorder outputs. Compiler invocation aliases are retained: resolving pdflatex to pdftex must not accidentally select the plain-TeX format.

Recorder inputs outside the snapshot must belong to declared installation/font/cache roots. Undeclared external user dependencies require an explicit import into the project. New dependencies require another full pass. Hashes are rechecked around passes, after PDF preparation, at visual commit and at export. Assets added, edited, renamed or removed invalidate the receipt.

This is a recorded build profile, not a hermetic OS image. Recorder coverage does not capture all registry state, dynamic libraries or arbitrary Lua side effects. Canonical source remains authoritative if an external editor changes its disk copy; conflict resolution remains P7. Normal date semantics are retained; SOURCE_DATE_EPOCH is preserved if already configured, not silently imposed.

## Engine recipe

Direct pdfLaTeX, LuaLaTeX and XeLaTeX jobs use file/line errors, SyncTeX, the dependency recorder, nonstop mode, halt-on-error and no shell escape. MiKTeX is identified by its version output and receives its disable-installer flag.

A build needs at least two passes, unchanged auxiliary/generated-input fingerprints, no new discovered dependencies and no rerun request. Limits: six passes, 120 seconds per build (at most 90 per engine pass), 2 GiB per process job. Exceeding limits, nonconvergence or compiler errors rejects the result. The total deadline is checked before acceptance; filesystem validation and cleanup may add return latency.

BibTeX/Biber orchestration is not implemented in this limited direct recipe. Bibliography-tool requirements are reported explicitly. Prebuilt bibliography fragments included as project inputs can be compiled. This does not claim support for every journal toolchain.

Logs retain bounded head/tail data while pipes continue draining. Truncated or interrupted logs prevent verification eligibility. Diagnostics include code, severity, message and a project source location when available. Unresolved references/citations and missing glyphs prevent visual edits and clean export; ordinary box warnings are shown. A converged PDF with unresolved diagnostics remains readable.

Eligibility means the result may enter the visual validation pipeline. Independent PDF validation and the complete signature manipulation scenario remain P6.

## Process lifetime

On Windows, CreateProcess receives a Job Object through PROC_THREAD_ATTRIBUTE_JOB_LIST before child code runs. Handle inheritance is restricted. The job has memory and kill-on-close limits; cancellation, timeout and parent completion terminate remaining descendants, then drain their output pipes. Failure to establish supervision fails the launch. There is no taskkill fallback.

Unix development/CI uses a separate process group and address-space limit. Actual Windows tests check quoted/Unicode argv, large output, cancellation, timeout and descendant heartbeat termination.

Implementation references:
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
- https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-setinformationjobobject
- https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock

## Publication, export and cache

PDF preparation and dependency validation precede publication. Cancelling while PDF preparation is still pending invalidates its frontend epoch and native candidate token. Older results cannot commit source/history or replace the last good PDF.

Export has a separate clean job and empty auxiliary state. The resulting PDF is prepared offscreen and checked for compilation eligibility; final export uses a brief edit barrier, rechecks revision/inputs and atomically renames a temporary output. It refuses to overwrite a PDF actually used as an input. Failure/cancellation preserves an existing destination and does not save source.

Displayed artifacts and in-flight SyncTeX/export readers are retained. Unpinned results are evicted toward twelve entries and 512 MiB; live pins/borrows take precedence. Failed/cancelled private workspaces are cleaned when safe. Each P3 process has a locked cache session. A subsequent process reaps only recognized, unlocked P3 sessions with an explicit clean-close marker. Interrupted sessions are retained because snapshots can contain unsaved source copies. The window close path marks a clean exit only after the existing unsaved-source decision; later canonical mutations remove that marker before committing. Another live instance, unknown folders and historical P2 caches are untouched. This preserves available copies, not a complete recovery journal or recovery UI. Failed filesystem cleanup is best effort, not an OS disk quota. There is no auxiliary-state reuse cache yet.

## Acceptance

Local automated suites: 51 core, 17 Windows build-infrastructure, 14 native service and 38 frontend/PDF tests passed (120 total). The real-engine corpus is an explicit ignored test locally. A fresh CI run at 3e40b99 passed all 24 expected outcomes (12 eligible builds, six readable/ineligible PDFs and six expected failures). Evidence is in docs/evidence/build-verified-run.json. CI runs eight scenarios on each of three engines: portable visual adapter compilation, references/TOC/includes, spaces in filenames, a real PDF asset, unresolved references, compile errors, deliberate nonconvergence and a required bibliography tool. It preserves synthetic receipts and reports.

P3's real-engine CI and production packaging status must be read in VALIDATION.md. Native app input acceptance remains deferred; the old P2 test executable remains unchanged. Broad engine/package coverage, fully hermetic toolchains, project trust/isolation, runtime independent PDF verification, source conflicts, crash recovery, strict frontend types and installer acceptance are not claimed by this checkpoint.

## Compiler initialization follow-up - 3 November

The installed MiKTeX engines were explicitly tested through the real build service on Windows. All 24 outcomes failed their version preflight with access-denied errors opening MiKTeX registry keys and creating its user configuration directory. This is failed/environment-blocked acceptance, not 24 expected successes or a document compilation result. Settings and permissions were not changed. The reviewed summary and raw-report hashes are in evidence/windows-engine-first-local.json; original reports remain in the ignored test workspace.

Compiler version failures now return an unsuccessful, ineligible build result with a short structured diagnostic and the bounded captured trace available in Compiler output. Timeout, truncated output, access-denied and other initialization failures have separate diagnostic codes. They record zero document passes, no accepted engine version, no PDF bytes and no output receipts. A successful exit with truncated version output is also refused. Input receipts are rechecked before the failed result is persisted; a PDF emitted by an unsuccessful version check is never sealed as an output.

Actual subprocess fixtures exercise a verbose nonzero version check that leaves an untrusted PDF behind and a successful-but-truncated version check. Neither can start a document pass or mutate originals. Frontend tests check the short error, retained trace and refusal before PDF preparation/verification. Normal LaTeX error extraction is retained. Source/history/last-good-PDF publication still follows the existing failure boundary. Local regression/build results are in evidence/compiler-preflight-local.json when available; native input and the local installed-engine acceptance remain open.

Later P6 independent inspection/native approval and P7 explicit Save/conflict/recovery work are described in NATIVE-LAYOUT-GATE.md and PROJECT-DURABILITY.md. The original P3 checkpoint counts above are historical; read WORK-CHECKPOINT.md for current phase status.
