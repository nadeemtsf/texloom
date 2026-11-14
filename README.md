# Texloom Desktop — development candidate

**This repository contains application source. A separate Windows x64 portable archive contains the compiled Texloom.exe.** The executable opens a native window and compiles and renders actual LaTeX output. The core native signature workflow has passed under all three engines on the recorded candidates. One exact installer passed existing-account installation and native workflow checks; clean-machine setup, uninstall, held-gesture/DPI, performance and usability gates remain open. See VALIDATION.md for versioned evidence. This is a development build.

This is a separate desktop implementation. It does not replace or convert the earlier Python application or its saved projects.

## Development status

Version 0.3 is under development in the primary repository, https://github.com/nadeemtsf/texloom. Source/history, compiler/mapping and workspace messages have Rust-generated TypeScript contracts and runtime checks. The current candidate adds project ZIP import/export, recents, explicit encoding conversion into a new copy, editable templates, BibTeX/Biber recipes, image import and bounded source layout controls. English, Arabic and Hebrew are the current language scope. See docs/IMPLEMENTATION-STATUS.md, docs/FINAL-NATIVE-ACCEPTANCE.md and docs/REMAINING-WORK.md for actual results and remaining work. New native/installation tests are currently deferred at the owner's request; headless checks continue. The research draft, development pilot and later evaluation plan are described in research/README.md; no final evaluation or publication is implied.

## Implemented in source

- A native Tauri 2 window, with a Rust backend and no Python HTTP server.
- Monaco for source editing, including highlighting, find, keyboard editing, and shared source/canvas history.
- PDF.js for rendering actual compiler output inside the desktop window.
- Mouse-wheel/two-finger PDF scrolling, horizontal trackpad/Shift-wheel navigation and Ctrl-wheel PDF-only zoom. New-build device acceptance remains pending; see docs/PDF-NAVIGATION.md.
- Native file dialogs for opening a main .tex file and exporting a PDF.
- Projects dialog with direct paths, recents, bounded ZIP import/export, encoding review and new Article/Letter/CV/multilingual templates.
- Declared local BibTeX/Biber builds, image import and source-preserving image/table/typography/spacing controls.
- PDF search across unpainted pages and bounded lazy painting for documents longer than 20 pages.
- Existing MiKTeX/TeX Live detection, pdfLaTeX/XeLaTeX/LuaLaTeX selection, bounded compilation time, compiler logs, and SyncTeX source lookup.
- Read-only setup help for missing compilers, initialization failures, packages and fonts; see [setup guidance and its pending native checks](docs/SETUP-HELP.md).
- Optional local timing diagnostics in a separately identified build; ordinary builds disable them. [Measurement boundaries](docs/PERFORMANCE-DIAGNOSTICS.md) distinguish handler entry, publication and a frame proxy. No end-to-end performance result is claimed yet.
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
3. Install a current Node.js LTS (22.13 or newer) if it is not already installed: https://nodejs.org/
4. Double-click **Build-Portable.cmd** for an executable, or **Build-Desktop.cmd** for an NSIS installer.

The build script uses your existing Rust installation if available. Otherwise it downloads rustup-init.exe and its SHA-256 checksum from static.rust-lang.org, checks the file, and installs a minimal Rust toolchain under `.tools/` without changing the global PATH. It selects the Windows MSVC target, compiles and runs the core tests, installs locked frontend dependencies with npm ci, and runs the Tauri release build. Portable mode skips the installer download and installer creation. Each portable build uses a fresh staging directory and a verified, explicit file list. The script prints its actual ZIP path; an existing archive is preserved and a new name is chosen. See docs/PORTABLE-PACKAGING.md.

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

- Visual editing supports bounded, unambiguous single-page text units. Arbitrary math, table cells, TikZ, cross-page groups and opaque macro expansions remain source-editing workflows.
- Cropped, rotated and UserUnit-scaled pages can be read and rendered. Their visual layout edits are explicitly refused.
- Complex shaping is tested for faithful source/display. CJK input-language acceptance is deferred by the owner; no input languages are installed by the app.
- BibTeX and Biber run declared local recipes with fixed arguments. Remote bibliography sources, project configuration scripts and shell-escape recipes are refused.
- Image dimensions, supported table column widths, explicit lengths, margins, font size/leading, line factors, styles and float placement have source controls. Unsupported grammars retain source access.
- Imported projects open for source review before Compile runs LaTeX under the user's Windows account. Turning shell escape off is not an operating-system isolation boundary.
- Portable and NSIS development packages include the dependency inventory, source-availability notices and pinned PDFium notices. They are unsigned. Clean-machine installation/uninstall and a human usability trial remain private release gates. Public distribution and Store submission are outside the roadmap.
- Automated and real-engine results do not substitute for the exact native acceptance matrix. Current status and measured performance limits are documented rather than inferred from a successful build.

## Source access and development roadmap

Texloom's source is public on GitHub for inspection. Windows packages are development builds for internal use. Existing Store configuration is kept for historical reference.

All rights are reserved for the current project code. See [LICENSE](LICENSE) for permission requirements, GitHub's viewing and forking terms, third-party licenses and earlier MIT-licensed revisions.

The broader visual-authoring architecture and future backlog are tracked in [roadmap #44](https://github.com/nadeemtsf/texloom/issues/44). This is planned work, not a declaration that those features are implemented.

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
