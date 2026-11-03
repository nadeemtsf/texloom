# Texloom Desktop - Rust desktop prototype

**This archive contains application source. A separate Windows x64 portable archive contains the compiled Texloom.exe.** The corrected source now passes the Rust tests and a complete Tauri release build. The executable opens a native window and compiles and renders an actual LaTeX PDF. It remains a development prototype; the full native editing workflow and installer are not certified. See VALIDATION.md for the exact checks completed.

This is a separate desktop implementation. It does not replace or convert the earlier Python application or its saved projects.

## Development status

Version 0.3 is under development in the primary repository, https://github.com/nadeemtsf/texloom. The Rust source/history, compile service, bounded visual adapters, independent verification, guarded Save and unsaved-work recovery have automated coverage. Native interaction and distribution gates remain open. The owner deferred mouse/keyboard/IME checks until implementation through the remaining phases is complete. See docs/WORK-CHECKPOINT.md and docs/FINAL-NATIVE-ACCEPTANCE.md. A successful test run does not establish reversible editing of arbitrary LaTeX.

## Implemented in source

- A native Tauri 2 window, with a Rust backend and no Python HTTP server.
- Monaco for source editing, including highlighting, find, keyboard editing, and shared source/canvas history.
- PDF.js for rendering actual compiler output inside the desktop window.
- Native file dialogs for opening a main .tex file and exporting a PDF.
- Existing MiKTeX/TeX Live detection, pdfLaTeX/XeLaTeX/LuaLaTeX selection, bounded compilation time, compiler logs, and SyncTeX source lookup.
- Dragging, width resizing, exact coordinates, base font sizes, block text editing, source-to-PDF compilation, and guarded conversion of a complete source selection into a movable block.
- **Visual transactions:** a mouse edit is compiled as a candidate before its source change is committed. Invalid edits preserve the original buffer and the last successful PDF.
- UTF-16-to-UTF-8 mapping between editor selections and Rust source offsets, including Arabic and supplementary Unicode characters.
- A durable source/history recovery journal, protected-revision status, review/restore/discard UI and source salvage when an original project folder is missing. Recovery preserves original Save conflict fingerprints and grants no old visual approval.
- Ordered changed-file typing acknowledgements that retain other project buffers; full opening/history/visual snapshots still handle file-set changes. Actual source-service timings and limits are in docs/SOURCE-PERFORMANCE.md.
- Rust domain, native service, filesystem/process, independent renderer and frontend/controller checks. Exact counts, source/artifact receipts and limitations are in VALIDATION.md.
- A Windows build script that locates Rust, or downloads and verifies the official Rust bootstrapper into the project when needed, then tests and builds an NSIS installer.

The interface uses HTML/CSS/JavaScript within WebView2; the document model, files, native compiler process, and source transformations use Rust. This is not an all-Rust widget toolkit.

## Build on Windows

You need the Windows build tools and access to official Rust, npm, and Cargo downloads, or an existing local dependency cache. Cargo.lock and package-lock.json record the dependency versions used for the verified build.

1. Extract the complete folder.
2. Ensure Microsoft C++ Build Tools has **Desktop development with C++** and a Windows SDK installed. The build script checks for this component and explains if it is missing. Official installer: https://visualstudio.microsoft.com/visual-cpp-build-tools/
3. Install a current Node.js LTS if it is not already installed: https://nodejs.org/
4. Double-click **Build-Portable.cmd** for an executable, or **Build-Desktop.cmd** for an NSIS installer.

The build script uses your existing Rust installation if available. Otherwise it downloads rustup-init.exe and its SHA-256 checksum from static.rust-lang.org, checks the file, and installs a minimal Rust toolchain under `.tools/` without changing the global PATH. It selects the Windows MSVC target, compiles and runs the core tests, installs locked frontend dependencies with npm ci, and runs the Tauri release build. Portable mode skips the installer download and packaging step.

The development executable is `target/x86_64-pc-windows-msvc/release/texloom-desktop.exe`. Its editor/PDF.js assets are embedded and need no local server. Independent visual verification also requires the matching packaged `verification` resources, including the worker, pinned PDFium DLL and notices. Use the complete development package rather than distributing a bare executable. WebView2 and a working MiKTeX/TeX Live installation are runtime prerequisites. A build is not installed-artifact acceptance.

To update an already extracted source project, replace its source files with this archive while retaining its `.tools/`, `node_modules/`, and `target/` directories. The missing source-map type annotation reported by the original build is corrected in src-tauri/src/service.rs. The script can reuse the existing Rust installation and Cargo build cache.

A successful full build produces an installer under:

```
target/x86_64-pc-windows-msvc/release/bundle/nsis/
```

Errors stop the script and remain visible. The presence of source files or successful frontend tests does not mean that the native app builds successfully. Resolve any compiler or dependency errors before treating the executable as usable.

Rust and Node.js are **build-time** dependencies. The installed app does not need Python or Node.js. It currently needs a working local LaTeX installation; its installer can arrange WebView2 if it is missing. No LaTeX distribution is bundled in this source prototype.

## Source checks

Frontend transaction tests, requiring only Node:

```
node --test tests/transactions.test.cjs
```

Rust core tests after installing Rust and the Windows build tools:

```
powershell -NoProfile -ExecutionPolicy Bypass -File Build-Desktop.ps1 -CoreOnly
```

The PowerShell execution-policy setting in the launcher applies to that process; the script does not change the machine's execution-policy settings.

## Expected workflow

Open a main .tex file with its supporting styles/images and select its engine. Rust owns canonical sources and shared undo/redo. Builds use private copied snapshots; typing does not write the original files. **Save** checks the loaded disk fingerprints and performs a recoverable multi-file write. External changes are refused, with the editor retained and a project-copy escape.

Select objects mode resolves supported PDF text to a unique source unit; Read text permits highlighting/copying. A group frame shows the selected source scope. Supported normal-text movement retains the unit's contribution to flow and requires a compiled, independently checked candidate before committing. Double-click opens protected literal fields or an explicit LaTeX fallback; recognized numeric line-break spacing has separate reflow checks. Existing StudioBlock positioning remains an isolated compatibility adapter. Unsupported macros, geometry, paint or ambiguous identity receive a refusal and source fallback.

Automatic recovery is separate from Save. Protection status stays visible beside the source, and storage errors leave editing/Save available. Recover work opens verified closed drafts for review with source and undo/redo; it never automatically overwrites originals. Earlier crash drafts stay until explicitly discarded. Missing-folder recovery can save source files to a new directory but cannot reconstruct missing images/assets. See docs/UNSAVED-RECOVERY.md.

## Current limits and required verification

- Rust compilation, native linking, Tauri production asset embedding, startup, PDF rendering, and actual compiler integration have passed. A complete native drag/edit/resize/save/export round trip remains to be verified.
- This prototype opens a main .tex file with supporting files in the same project tree; ZIP import, a dedicated image-upload flow, persistent recent-project lists, and project ZIP export are not included yet.
- Ordinary arbitrary LaTeX is not automatically reconstructed into independent visual objects. The source editor remains the fallback for complex macros, tables, and generated or repeated content.
- Source conversion preserves the selected LaTeX tokens, but cannot reproduce every surrounding formatting context. Review the resulting layout, especially when converting content from lists, columns, or locally changed fonts.
- Source mapping is provided by the installed native synctex utility. Some generated content may not have useful mappings.
- Bibliography tool orchestration and custom shell-escape build recipes are not included.
- Build/recovery caches have explicit retention and integrity checks. Native long-session, quota, replay/performance and usability acceptance remains open.
- Source-service benchmarks are measured; WebView/editor paint, native typing/IME and full interaction latency have not passed acceptance. Fractional free movement and broader paint/coordinate cases remain gated.
- Use your own trusted LaTeX projects. Turning shell escape off does not make native TeX a full sandbox.
- The portable executable is an unsigned development build. An NSIS installer has not been generated or tested, and no Store approval or signing certificate is included.
- The development PDFium pin still needs a production security/license review. A passing renderer fixture is not approval to distribute that dependency as a release.

## Microsoft Store planning

As checked on 30 October 2026, Microsoft's developer-account documentation states that new Individual and Company accounts have no registration fee when created through the new flow at https://storedeveloper.microsoft.com/ . Start there; legacy entry points can show a different onboarding flow. Identity or business verification still applies.

Tauri documents a Microsoft Store route for EXE/MSI installers. It requires an offline installer, silent-install support, appropriate publisher details, and code signing. Registration being free does not establish that every signing or distribution choice will be free. Code signing, compiler redistribution, dependency installation, licensing, and certification need to be resolved before submission.

`src-tauri/tauri.microsoftstore.conf.json` contains the offline-WebView2 build override. It is a packaging starting point, not a complete Store submission configuration.

Sources:
- https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account
- https://v2.tauri.app/distribute/microsoft-store/
- https://v2.tauri.app/start/prerequisites/

## Project structure

| Path | Purpose |
| --- | --- |
| `crates/texloom-core/` | Dependency-free Rust parser, validation, and source edits |
| `src-tauri/src/service.rs` | Project files, compiler jobs, and SyncTeX |
| `src-tauri/src/lib.rs` | Desktop window and native command boundary |
| `web/` | Monaco editor, PDF canvas, and transaction UI |
| `resources/` | Starter document and portable LaTeX block package |
| `tests/` | Frontend transaction checks |
| `scripts/` | Packaging local PDF.js support assets |

Build dependencies and their bundled components retain their upstream licenses. A release needs a dependency and redistribution review for the selected compiler and fonts.
