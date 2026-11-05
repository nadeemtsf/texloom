# Commit, presentation and cleanup - untested development change

Prepared 5 November 2026. The owner paused **all tests** for this continuation, including automated and headless checks. The implementation below was reviewed as source only: it has not been executed, built, installed or observed in Texloom. Earlier passing suites and r4 artifacts belong to earlier source; they do not validate this change. No original document or recovery draft was modified.

## Failure boundary found by source review

`commitSources` calls native commit before synchronizing Monaco, publishing the prepared PDF and reselecting a text object. The native document client installs a validated snapshot before invoking its UI callback. Consequently an exception in that callback, editor update, PDF publication or reselection can occur after native source/history has advanced. The previous generic false return could tell an image-insertion caller that source insertion was not accepted. This is a code-path finding, not a reproduced native failure.

A second source-review finding was that a rejecting PDF-destruction promise in cleanup could skip clearing the busy/mutating flags. An editor model exception could also leave the programmatic-update suppression flag set. Neither finding has been exercised in this continuation.

## Proposed behavior now implemented in source

- A completed commit acknowledgement, or a validated client snapshot at the candidate's advanced revision with exactly the proposed sources, distinguishes a known commit from a failure before commit. A no-op baseline match alone is not evidence that a failed commit succeeded.
- If UI work then fails, retain the acknowledged native sources/history, mark the preview stale, clear the selection and explain that the source edit was applied. Undo/Compile remain the intended recovery actions when synchronization is healthy; Save stays explicit. The exception reaches callers so they cannot label it as a refused insertion or select an incomplete view.
- Text-object reselection failure after publication uses the same known-commit error. A PDF may already have appeared; the trace outcome does not imply that no pixels were shown.
- Once commit is attempted, an unusable acknowledgement without a confirmed advanced snapshot is `CommitStatusUnknown`. The current replica is stopped through the existing synchronization-error state. No commit is retried, no automatic rollback or Save occurs, and local editor text is retained for copying. The user is directed to reopen Texloom and review recovery.
- Engine/main-document changes and Open are disabled during the commit critical section. Compile/Save/profile controls reflect synchronization errors. This does not disable source typing throughout the preceding candidate compilation; existing staleness rules remain in place.
- Cleanup of an unadopted view is best-effort and cannot replace the original staging failure or skip releasing busy/mutating state. A partially adopted PDF is kept alive as a stale reading view until replaced. Programmatic editor suppression resets in `finally`.

The new optional trace outcomes are `committed-ui-failed` and `commit-unknown`. Neither is a pre-commit refusal or a successful end-to-end task. Existing completed milestones remain useful: a record can contain `published` and still fail during later reselection. No source text, project path or error message is added to the trace.

## Deliberate limits

The frontend has no structured native receipt establishing a negative commit in every exception case. Treating an unconfirmed attempted commit as unknown can therefore pause a request that native code actually refused safely. This is conservative availability loss, not proof of a transport failure or a new correctness guarantee. It needs review during later validation.

The change does not implement durable distributed transactions, prove that every UI exception is recoverable, independently verify a malformed acknowledgement, or reconstruct a missing native response. Reopening/recovery, stale reading surfaces and exact source history still need actual evidence. Cleanup exceptions are suppressed to preserve the original failure; this alone does not prove absence of resource leaks.

## Deferred validation scenarios

These are requirements for later work, not executed checks:

1. Inject editor/publication/reselection and changed-callback failures after a confirmed commit; inspect exact native/client source, one history entry, visible warning, ordinary Undo and explicit Save behavior.
2. Lose or invalidate the commit acknowledgement; establish whether native committed independently, verify the replica does not issue newer writes and verify the actual reopen/recovery path preserves available edits.
3. Make unadopted view disposal/PDF destruction reject while preserving the original compile/validation error and releasing busy/suppression state. Check the adopted-view path separately.
4. Repeat successful moves, replacements, legacy controls and image insertion; confirm ordinary completion and selection remain unchanged. Check configuration/Open controls around the commit boundary.
5. Inspect exact diagnostic categories and milestones without reclassifying an accepted edit as a refusal. Measure overhead separately when permitted.

No test or application launch is authorized by this document. The owner's current pause remains in force. Never compile `.text` files.
