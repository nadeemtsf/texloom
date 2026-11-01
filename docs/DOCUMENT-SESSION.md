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
| document_prepare | Derive minimal byte replacements; retain private visual candidate |
| compile_document | Compile immutable current or candidate sources |
| document_commit | Require the candidate's own successful native build ID |
| document_discard | Drop abandoned candidate |
| save_document | Explicitly save canonical revision through existing save service |

DocumentClient serializes edits and selection messages against native acknowledgements. Canonical snapshots publish only when the queue drains. flush includes edits added while waiting. Optimistic edits immediately make the canvas stale; acknowledgements reconcile the browser revision.

Rejected source deltas freeze editing and stop dependent deltas while retaining the visible buffer. Refused visual commits do not poison subsequent typing. Keyboard, toolbar, explicit editor context-menu actions and Monaco model undo/redo route to native history. Dedicated context keys follow Rust history after Monaco resets; the context-menu, clipboard and standard multi-cursor contributions are loaded explicitly. Composition blocks compilation, canvas actions and history until it ends.

Visual edits prepare sources in Rust, compile, prepare/render and validate the PDF in the frontend, recheck identity, then briefly freeze editing for native commit/publication. Typing invalidates the token. Wrong-revision results are rejected before rendering/publication. Independent runtime PDF verification remains P6.

## Evidence and limits

Core tests cover exhaustive Unicode range/replacement round trips, multi-file atomicity, byte preservation, stale/foreign candidates, support-file/profile undo, EOL/BOM and selections. Native tests use the real service/Serde boundary and save/reopen. Frontend tests use named doubles for delayed IPC, failed/stale candidates, cursor forwarding, composition gates and undo routing.

Production startup/sample compile passed. Native typing, actual IME, multi-cursor and context-menu acceptance are unverified because foreground capture prevented reliable input testing. P2 is not complete.

The frontend remains JavaScript; strict TypeScript/generated IPC types remain planned. Unsupported encodings produce a UTF-8 error; read-only viewing/conversion is unfinished. Failed optimistic buffers remain in the window without a durable journal. String storage is dependency-free; performance, rope/index optimization and session/build cleanup need measurement.

Save is not yet multi-file atomic or conflict-aware. Core removals do not constitute completed filesystem file management. Compilation still copies disk dependencies and runs two fixed passes; P3 must make dependency snapshots reproducible. Main/engine choices have session history but no persisted project profile yet.
