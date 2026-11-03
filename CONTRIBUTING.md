# Working on Texloom

Read docs/SYSTEM-DESIGN.md and VALIDATION.md before changing visual editing. The source buffer is authoritative. A visual candidate must compile, render, pass layout checks, and still match the current project revision before it changes source or undo history.

Use a small feature branch and a pull request. Run `npm ci`, `npm test`, `cargo test --locked -p texloom-core`, and `npm run desktop:build -- --no-bundle --target x86_64-pc-windows-msvc` on Windows. A local MiKTeX or TeX Live installation is needed for native compiler round trips. CI builds the executable but does not substitute for native gesture and TeX-engine checks.

Keep source transformations in texloom-core, native process/file work in src-tauri, PDF geometry in web/core, and gesture handling in CanvasController. Add regression coverage when changing parsing, transaction ordering or source-to-PDF mapping. Include exact engine/version information with fixture results.

Use synthetic documents in tests. Keep user projects, build caches, credentials, generated installers and personal correspondence out of Git. The checked-in lockfiles are part of the build contract.

Windows filesystem packaging checks in npm test prefer PowerShell 7 (pwsh.exe), matching CI, and fall back to system Windows PowerShell when version 7 is absent. They do not change execution policy. A host that denies creation of the test junction reports that specific link case as skipped; do not count it as a passed refusal check.

The ordered document client is authored in frontend/protocol/*.ts. Its web/core JavaScript is generated and must not be edited directly. After a TypeScript change run npm run protocol:emit; after a Rust DTO change first run npm run protocol:rust:generate. npm test/build fail on stale runtime output, and native tests with --features typegen fail on stale Rust bindings/fixtures. See docs/TYPED-DOCUMENT-PROTOCOL.md. The remaining frontend migration is separate work.
