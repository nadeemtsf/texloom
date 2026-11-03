# Strict document IPC slice

The document client now has strict TypeScript sources in frontend/protocol. Rust remains the canonical text/history authority. Every native acknowledgement enters the client as unknown; it is checked before replacing the last acknowledged replica. No compiler result, rendering result or TypeScript type assertion grants permission to commit source.

## Contracts and generation

The actual Tauri handlers construct shared request records in src-tauri/src/document_protocol.rs and then call the existing backend. The external argument shape remains flat camelCase. The optional, exactly pinned ts-rs 11.1.0 feature exports these records and the real serialized reply types, plus a command map whose handler identifiers are checked by Rust. Revision JSON values use number with safe-integer runtime checks; native source/offset/size checks remain authoritative. The generator and fixture exporter are Rust-only developer programs and add no IPC command or application-control hook.

TypeScript 5.9.3 is pinned in package-lock.json. The two protocol modules compile with strict, noUncheckedIndexedAccess and exactOptionalPropertyTypes. Their checked-in JavaScript in web/core is generated output, preserving existing imports and allowing tests to execute the same runtime that Vite bundles. Edit the TypeScript files, not those JavaScript outputs. Neither npm test nor npm run build silently regenerates stale output.

| Change | Developer command |
| --- | --- |
| Rust request/reply DTOs | npm run protocol:rust:generate |
| TypeScript protocol implementation | npm run protocol:emit |
| Read-only TypeScript/runtime check | npm run protocol:check |
| Read-only Rust DTO/Serde fixture check | npm run protocol:rust:check |

After Rust DTO changes, regenerate both the bindings and the runtime, then run the checks. CI and Build-Desktop.ps1 run native tests with the typegen feature, including exact generated-declaration and Serde-fixture checks. Ordinary production builds omit this feature. Native request records are constructed by the flat Tauri handlers; their Serde unknown-field attribute is not a claim that Tauri rejects unknown top-level command arguments. Queue-owned project/revision fields are set after caller fields.

## Covered replies

| Command | Required acknowledgement |
| --- | --- |
| document_edit | Tagged changed-file reply, exact base, revision equal or +1, unchanged profile, one matching changed file or empty no-op |
| document_history | Full snapshot, equal or next revision, valid sources/profile/history flags/selections/protection |
| document_profile | Full snapshot, requested profile, unchanged source buffers |
| document_commit | Full snapshot after native one-shot approval; malformed success freezes further mutations |
| document_selection | Same revision, requested primary/secondary cursors, valid normalized UTF-16 boundaries and protection |
| save_document / retry_typing_recovery | Same revision and protection status; no source/history/profile fields permitted |
| document_prepare | Opaque nonempty token and equal/next candidate revision; no canonical-state mutation |

Full snapshots require all expected fields, a main file present in the source map, supported profile metadata and valid selections. Primary and secondary endpoints cannot split a surrogate pair or exceed the BOM-hidden, LF-normalized editor text. Protection cannot claim a future revision or a protected revision without a draft. No-op full replies cannot change canonical source/profile/history state. Unknown or missing fields are refused. Changed-file checks retain untouched immutable source strings and do not validate/copy the whole project on every keystroke.

The client preserves the last accepted snapshot after an invalid acknowledgement and stops dependent mutations. This includes malformed successful replies to otherwise nonfatal Save/commit operations: a native success with an unusable acknowledgement cannot safely be followed by another source edit. Ordinary rejected Save conflicts and refused visual candidates remain nonfatal. The existing UI retains optimistic text with a copy-before-reopen explanation. It does not claim to undo a backend commit after a transport/protocol failure. Native canonical history/recovery remains separate.

Ordinary selection acknowledgements update the replica's selection without triggering a shell source publication if protection metadata is unchanged. New protection errors still publish. Native cursor-event journal flushes are unchanged and remain separate performance work. Before opening a project into the UI, the new client validates its initial snapshot; rejection leaves the existing source, PDF and render lifetime intact.

## Evidence and remaining scope

Rust-generated Serde fixtures cross the runtime validator with BOM, CRLF, supplementary Unicode, byte-preserving sources, selections, deltas and recovery. New tests reject malformed snapshots, out-of-range/split-surrogate cursors, future protection, extra metadata sources, invalid revisions and malformed commit/preparation results. The full production-shell test checks that a bad snapshot cannot destroy the existing editor/PDF. A copied build fixture proves the check fails on stale JavaScript or invalid TypeScript without rewriting either file. Compile-time negative contracts cover missing editor-change fields, wrong cursor/group types, incomplete commits and queue bypass.

Local validation: 78 core, 42 native with typegen, two explicitly invoked headless native approval and 149 frontend/filesystem checks passed: 271, excluding unchanged historical build/worker suites. One junction fixture is locally skipped because creation is denied; the same unchanged packaging case passed on the previous checkpoint's Windows CI. Frontend, verifier and Windows x64 release builds passed. Exact source, generated output, dependency/log and executable receipts are in evidence/typed-protocol-local.json. Native GUI typing/IME/mouse and installed-artifact gates remain owner-deferred.

This is the first strict TypeScript slice. Monaco wiring, canvas/geometry, PDF surface, recovery controller and the rest of app.js remain JavaScript. Compile results, project/recovery inventory, mapping and richer visual-adapter IPC still need generated contracts and further runtime coverage; compile returns unknown into the existing semantic validation path. No full-frontend migration, complete P2/P7/P8/P9 gate or release is claimed. Unsupported-encoding viewing/conversion, parser/provenance, bibliography recipes, fractional displacement, long-document/cursor performance and richer LaTeX adapters remain open. Ordinary-PDF expansion is outside scope.
