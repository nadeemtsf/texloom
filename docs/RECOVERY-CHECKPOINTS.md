# Source/history recovery checkpoints

This is the core checkpoint boundary for P7 automatic recovery. The checkpoint itself has no filesystem/UI authority. The application journal and review UI that use it are described in UNSAVED-RECOVERY.md.

A checkpoint captures canonical UTF-8 sources, main/engine profile, monotonic revision, retained undo and redo entries, their cursor and byte-based selections. It contains no Prepared ownership token, compiler artifact or visual approval.

Session::from_checkpoint validates schema, source/profile/selection bounds, history count and byte accounting, revision ordering, and matching forward/inverse step counts. It walks backwards to the retained base, then replays every retained step including the redo branch. Each forward step must change state and its inverse must restore every byte and profile exactly. The reconstructed cursor must match the checkpoint. Merged adjacent-deletion inverses are compared by resulting state, not by a guessed structural equality.

Restoration allocates a fresh owner identity, so pre-crash prepared candidates remain foreign. Previous typing groups are broken at the new session boundary without changing the retained undo/redo grouping. Serialized accounting fields are not trusted.

The core keeps no filesystem/compiler/UI dependencies. Its default library has no dependencies; recovery-serde is an explicit optional codec feature, using the already locked Serde version. The actual Session is never deserializable. Infrastructure must bound the serialized input and bind project/disk identities before calling the validated constructor. JSON codec tests do not prove durable filesystem recovery.

Infrastructure can enter a validated Replay, apply the durable tail while retaining its recorded typing groups, then call finish once to adopt a fresh UI session and break old groups. Direct Session::from_checkpoint performs that finish immediately. Prepared::recovery_patch exports validated plain change data without its commit authority.

The persistence layer flushes accepted mutations before reporting them protected, handles partial records and storage failures visibly, retains conflict fingerprints, and checkpoints without losing retained undo/redo. Native interaction remains deferred under FINAL-NATIVE-ACCEPTANCE.md.
