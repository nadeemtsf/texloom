# Rust document session: P2 checkpoint

## Ownership and coordinates

texloom-core::document::Session owns UTF-8 sources, main/engine profile, monotonic revision, selections and history. It has no compiler, filesystem, UI or TeX parser dependency. Invalid intermediate TeX remains editable.

Monaco is an optimistic LF/UTF-16 view. Rust hides the leading BOM, projects CRLF/lone CR to LF, and maps valid UTF-16 boundaries to UTF-8 byte positions. Surrogate interiors are refused. Inserted newlines use the local terminator, or the preceding terminator at EOF. Unchanged bytes retain mixed EOL/BOM.

## Transactions and history

A patch names its base revision and file operations. Replacements carry byte ranges and exact expected text; file creation/removal and profile changes also have preconditions. Duplicate files, overlap, invalid boundaries, stale revisions, conflicting creates and invalid final profiles are refused before canonical mutation.

Prepared candidates are immutable, tied to their creating session and base revision. Undo/redo advance revisions, so identical restored bytes never revive an old candidate. Touching deletion inverses are joined to avoid ambiguous insertion offsets.

Source edits, canvas commits and profile changes share native history. Explicit typing groups coalesce below a 1 MiB patch budget; canvas commits remain separate entries. Limits are 120 entries and a 32 MiB patch budget, retaining the newest entry even if it alone exceeds the budget. These do not bound every application allocation.

Primary and secondary selections use byte boundaries and project back for undo/redo. Selection metadata does not change revision or invalidate candidates. Navigation, file switches, history and composition boundaries break frontend typing groups.

## Protocol

| Command | Role |
| --- | --- |
| document_snapshot | Sources/profile/revision/history capabilities |
| document_edit | Apply one Monaco change event at its acknowledged revision |
| document_selection | Update all cursors through the ordered queue |
| document_profile | Change main/engine through history |
| document_history | Shared undo/redo |
| document_prepare | Replan a typed source operation; retain a private receipt-bound candidate or a discard-only selection probe |
| compile_document | Compile immutable current or candidate sources |
| document_commit | Require the candidate's own successful native build ID |
| document_discard | Drop abandoned candidate |
| save_document | Explicitly save canonical revision through existing save service |

DocumentClient serializes edits and selection messages against native acknowledgements. Canonical snapshots publish only when the queue drains. flush includes edits added while waiting. Optimistic edits immediately make the canvas stale; acknowledgements reconcile the browser revision.

Rejected source deltas freeze editing and stop dependent deltas while retaining the visible buffer. Refused visual commits do not poison subsequent typing. Keyboard, toolbar, explicit editor context-menu actions and Monaco model undo/redo route to native history. Dedicated context keys follow Rust history after Monaco resets; the context-menu, clipboard and standard multi-cursor contributions are loaded explicitly. Composition blocks compilation, canvas actions and history until it ends.

Visual edits are replanned in Rust from a typed operation and exact baseline receipt. A candidate compiles privately, receives operation-specific inspection from the isolated PDFium worker and is prepared/rendered by the frontend before native commit/publication. Native one-shot approval binds the exact source revision, build, toolchain and cancellation lease. Selection probes cannot commit. Typing invalidates the token, and wrong-revision results cannot publish. See NATIVE-LAYOUT-GATE.md for implemented policies and remaining fractional/paint/native-acceptance limits.

## Evidence and limits

Core tests cover exhaustive Unicode range/replacement round trips, multi-file atomicity, byte preservation, stale/foreign candidates, support-file/profile undo, EOL/BOM and selections. Native tests use the real service/Serde boundary and save/reopen. Frontend tests use named doubles for delayed IPC, failed/stale candidates, cursor forwarding, composition gates and undo routing.

Earlier prototype startup/sample observations do not certify the current executable. On 3 November a fresh native launch exposed a missing Cancel compile element that aborted initialization. The correction passes full-shell integration tests and builds a new executable; those tests use mock native/Monaco/DOM boundaries. Native input was rejected with "Computer Use was not approved to use texloom-p5." Native typing, actual IME, multi-cursor and context-menu acceptance remain unverified. P2 is not complete.

The ordered document client and reply validator now have strict TypeScript sources, Rust-generated request/response DTOs, runtime validation and checked generated JavaScript. See TYPED-DOCUMENT-PROTOCOL.md for the exact covered commands and remaining migration. Other frontend/IPC paths remain JavaScript or return unknown into their existing operation-specific checks. Unsupported encodings still produce a UTF-8 error; read-only viewing/conversion is unfinished. Accepted canonical work is durably journaled (UNSAVED-RECOVERY.md); a rejected or unacknowledged optimistic editor buffer remains in the window and must be copied before reopening. Canonical storage is dependency-free. Single-file preparation avoids copying untouched sources; SOURCE-PERFORMANCE.md records measured limits, with longer-document/selection/paint work still open.

Explicit Save now checks loaded disk fingerprints and the canonical revision, uses staged recoverable replacements, and offers conflict-preserving project copies and explicit interrupted-Save recovery. Multiple replacements are recoverable, not globally atomic against all third-party writes. Core removals still do not constitute completed filesystem file management. See PROJECT-DURABILITY.md.

Compilation uses immutable copied source/asset snapshots and checked compiler/dependency receipts, two to six convergence passes, cancellation, resource limits and guarded export. The direct-engine recipe does not orchestrate bibliography tools; see BUILD-SERVICE.md. Main/engine choices have session history but no persisted project profile yet. Recovery of accepted never-saved typing/history is implemented separately from explicit-Save journals and build caches; native crash/relaunch acceptance remains deferred. See UNSAVED-RECOVERY.md.

## Typed document protocol checkpoint

Actual Tauri handlers construct shared Rust request records without changing the flat camelCase argument format. The optional ts-rs generator exports those records and real reply structs. The strict TypeScript client treats every native result as unknown until its command-specific checks pass. Full snapshots, changed-file replies, cursor/recovery metadata and prepared tokens have distinct shapes and revision rules. Malformed successful acknowledgements freeze dependent mutations even for otherwise recoverable Save/visual commands; ordinary native Save conflicts and visual refusals remain nonfatal. Project opening validates the snapshot before replacing the current editor/PDF.

Normal cursor acknowledgements update the client's selection without publishing another source snapshot when protection is unchanged; protection failures still publish a persistent warning. This avoids redundant shell updates, not native cursor-event journal flushes. No end-to-end input/paint performance claim is made.

The runtime JavaScript is committed compiler output used by the existing app/tests. npm test/build check strict TypeScript, compile-time negative contracts and output freshness; checks do not silently rewrite files. CI/local native tests enable the optional schema check. 271 checks passed in this follow-up (78 core, 42 native with type generation, two explicit headless native approval and 149 frontend/filesystem), with one local junction skip. The unchanged build/worker suites retain their earlier evidence and are not included again in this count. Native GUI acceptance and full P2 completion remain open.
