# Desktop prototype verification

Date: 30 October 2026.

## Build failure corrected

The first user-run build installed Rust and dependencies and passed all twelve core tests, but native compilation stopped with E0282 in service.rs. The project source map now has the explicit type `BTreeMap<String, String>`, resolving an ambiguous string conversion. Cargo.lock and package-lock.json retain the dependency resolution used for the corrected build.

## Completed checks

- All twelve dependency-free Rust model tests compiled and passed locally with the Windows MSVC toolchain.
- All four frontend transaction tests passed: failed visual edits preserve the previous document; successful edits commit after compilation; newer typing wins over older candidate builds; stale asynchronous PDF parsing is discarded. These tests use explicit native/rendering test doubles.
- npm run build completed with Monaco, PDF.js, and Tauri dependencies. Vite reported a large-bundle warning, not a compilation failure.
- cargo build --offline --locked --release --target x86_64-pc-windows-msvc -p texloom-desktop completed successfully.
- The production command `tauri build --target x86_64-pc-windows-msvc --no-bundle --ci -- --offline --locked` completed successfully, embedding production assets and using the custom application protocol.
- The executable was launched on Windows. Its native window opened, and the startup project compiled with the installed MiKTeX pdfLaTeX engine. The UI reported success and displayed a real one-page PDF, its text layer, the Monaco editor, and movable-block controls.
- A copied portable executable also compiled and rendered the sample PDF. Its native Open dialog was opened and cancelled.
- The changed PowerShell script passed its syntax parser. The portable build mode uses the same no-bundle Tauri build command without CI/offline flags.
- Packaging checks validate manifests, resources, icon, and HTML control identifiers. Source archives exclude build caches and installed dependencies.

## Native UI checks still pending

Dragging, resizing, text replacement, source editing with automatic recompilation, failed-edit rollback, undo/redo, opening a chosen external project, explicit source save, and PDF export still need a complete verified round trip in this Rust executable. Startup and a rendered PDF do not establish that all editing controls work.

Windows automation encountered inconsistent modal focus, a concurrent-input guard, and then a foreground-window error during the mouse test. Those attempted actions are not recorded as passes. Existing user edits were not discarded to continue testing. The earlier Python app's browser tests do not count as native Rust interaction tests.

## Release work still pending

- No NSIS installer, clean-machine installation, Microsoft Store submission, certification, or code signing has been performed.
- The initial npm install reported one high-severity audit finding without naming the affected package in the supplied log. It has not been assessed or resolved here; no clean dependency audit is claimed.
- Cache cleanup, external-editor conflict detection, complex layouts, Arabic/RTL projects, bibliography orchestration, and clean-machine prerequisites remain open release concerns.

## Toolchain provenance

The initial session could not download Rust because its permission request was rejected. The user subsequently ran Build-Desktop.cmd outside that restriction, installing the official toolchain and dependencies. Verification used that existing local toolchain and copies of its cached dependencies with offline Cargo builds. No alternate download or permission workaround was used.
