# LaTeX setup guidance

Implemented 4 November 2026. The application now offers **Setup help** in the file sidebar. Known compiler, initialization, missing-package and font failures also expose a contextual help button. This is read-only guidance; it does not download, install, execute a setup program, change PATH or write project files.

Compiler discovery and build preflight remain authoritative. A generic LaTeX syntax error is not classified as a missing installation. Access-denied, timeout and failed initialization messages recommend inspecting the compiler output and repairing the existing distribution, rather than concluding that the compiler is absent. Missing `.sty`/`.cls`/`.ldf` guidance distinguishes distribution packages from project-supplied files. MiKTeX package installation remains disabled during Texloom builds.

The dialog contains selectable, copyable official addresses. It does not add a general-purpose URL-opening native command. A new project or successful eligible compilation clears the old contextual diagnosis. The general guide is always available. App-wide Save/Compile/Undo shortcuts do not run behind an open dialog.

## Evidence and limits

Automated checks exercise the actual shipped shell and module: missing compiler versus initialization versus unrelated errors; missing packages/fonts; readable contextual help; no native operation or source change when opening help; and keyboard input retained by the dialog. These checks do not establish that the dialog looks correct in Windows or that a novice can complete setup.

Native appearance, keyboard navigation, clean-account missing-distribution/package/font scenarios and successful recovery after installation remain **pending**. The owner deferred all tests requiring Texloom to open or approval while away. WebView2 is needed before an in-app guide can appear; the existing NSIS bootstrapper configuration requires its own clean-machine test. No new installation was attempted.

## Primary references checked 4 November 2026

- [MiKTeX download](https://miktex.org/download): Windows distribution installer.
- [MiKTeX Console](https://miktex.org/howto/miktex-console): distribution management, updates, per-user/shared modes and package settings.
- [TeX Live Windows guide](https://tug.org/texlive/windows.html) and [quick installation](https://tug.org/texlive/quickinstall.html): Windows setup, program search path and TeX Live Manager. The Windows page was available through search indexing; direct retrieval timed out during this check. No installer version or download checksum is hard-coded in the UI.

These references describe the distributions. The separate statement that Texloom disables MiKTeX's automatic installer is established by `crates/texloom-build/src/engine.rs`, not by a distribution recommendation.
