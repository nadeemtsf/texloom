# Unsaved source and history recovery

P7 now journals accepted source edits, profile changes, undo/redo, selections and approved visual commits independently of explicit Save. Canonical buffers stay in Rust. Recovery storage is private app data, under `.source-recovery`, and never overwrites project originals. An acknowledgement reports the protected revision and any recovery error separately from source acceptance.

Opening a project creates a flushed source/history checkpoint. Each accepted mutation appends a bounded record and calls `sync_all` before reporting that revision protected. If storage fails, the edit remains accepted, the protection acknowledgement reports the failure, and a persistent source-panel warning offers Save/copy and a protection retry. Compilation cannot hide that warning. Save uses the existing external-edit/recoverable-save protocol, then checkpoints the new saved baseline. A separate recovery failure cannot turn a successful disk Save into a reported failure.

## Record and checkpoint contract

Each frame contains a little-endian length and complement, JSON payload and a SHA-256 chain binding the prefix, previous frame and payload. The first frame identifies schema, draft, generation, original project/disk fingerprints and an opaque core checkpoint. Subsequent records identify base/new revisions and a typed mutation. They contain no compiler result, Prepared owner, approval token or permission to commit a visual operation.

Core restoration validates the entire retained undo/redo timeline and exact inverses. Infrastructure replays the journal tail with the recorded typing groups, then breaks those groups once when adopting a fresh UI session. This preserves the historical grouping across a checkpoint in the middle of typing while preventing new typing from joining pre-crash edits. Restoration requires a fresh compile before any visual operation.

A physical partial final frame restores only the preceding complete verified state and exposes a warning. Its bytes are preserved. A complete frame with bad checksum, revision ordering, schema, path or state transition is refused. A partial new checkpoint can fall back to the preceding complete generation with a warning. A corrupt complete checkpoint is not silently skipped.

## Bounded storage and cleanup

- 32 drafts; no unrecovered draft is evicted automatically to make room.
- 224 MiB maximum encoded frame; 256 MiB maximum generation; 2 GiB aggregate write budget coordinated by a store lock.
- Checkpoint rotation after 1,024 events or 16 MiB of tail records, or before a generation would exceed its limit.
- Ordinarily retain the current generation and its predecessor. Older files are pruned only against the expected byte length and SHA-256 accumulated when writing that closed generation.
- 64 files per draft; unknown entries, reparse points, unexpected content and poisoned generations are preserved. New protection can pause at a limit; the editor and explicit Save remain available.

Cleanup is flat and content checked. It never recursively deletes unknown directories. A failed append is never followed by another append to the uncertain tail; a successful full checkpoint repairs protection instead. Unknown store entries pause new writes but do not hide valid crash drafts from the read-only inventory. Files open in another Texloom instance are locked and excluded from crash recovery.

These are application-managed filesystem checks and cooperative instance locks, not OS isolation against a malicious same-account process or a filesystem-wide compare-and-swap guarantee. Flush/fault tests do not establish arbitrary device behavior under real power loss.

## Review and adoption

Recover work lists verified closed drafts across projects. Nothing is restored or written automatically. Opening a draft creates a fresh native document ID, canonical session and current journal; the earlier crash draft is retained for review. The restored editor is explicitly unsaved even when its initial displayed buffers happen to match another saved view. Recovered native state cannot reuse old visual tokens.

The original saved disk fingerprints and absent-file preconditions are retained. An external source change after a crash still blocks Save; recovery never recaptures that changed file as an approved overwrite baseline. Save a copy remains the escape path. Explicit draft deletion keeps original files and the current editor unchanged.

If the original folder is missing, source and history can still open. Save refuses to recreate/overwrite the old path. Save a copy writes a new source-only directory; images and other missing assets are not stored in this journal. A changed/rebound existing project path is refused. Incomplete/corrupt data remains available for inspection rather than being deleted by a guessed cleanup operation.

Closing is revision guarded. A deliberate close/discard can remove the current live draft; it does not delete earlier crash drafts. Cleanup failure returns a retention warning after closing instead of leaving the frontend attached to a half-closed native session. An explicitly closed unsynchronized editor retains its native draft because its last acknowledged revision may be stale.

Filesystem work runs on blocking worker tasks, outside the WebView command thread. Source commands remain ordered. No promise is made that fsync or replay of long retained history has met the native paint/latency target. The release benchmark now measures the actual journal-enabled edit path (schema 3); per-edit cursor events and checkpoint compaction remain outside its measured samples.

## Validation boundary

Automated checks cover every byte truncation of a final frame, corruption/reordering, locks, revision gaps, source/profile/history fidelity, unknown paths, closed-generation cleanup, actual write failures, recovery across a new backend, external-change refusal, successful Save with degraded recovery, guarded close/discard, and missing-folder source salvage. Frontend tests exercise the production recovery controller and shell boundaries with explicit native/DOM doubles.

Exact source/log/executable and benchmark receipts belong in `evidence/typing-recovery-local.json`. Native window kill/relaunch, mouse/keyboard/IME, power-loss/storage-device behavior, long-session retention/usability and clean-machine acceptance remain separate gates in FINAL-NATIVE-ACCEPTANCE.md. Implemented recovery does not complete P7 or certify a release.
