# Windows preview installer checks

Use the exact unsigned Windows x64 `Setup.exe` being considered for release.
Record its source commit and SHA-256, the Windows version, and the outcome for
each route you tested. Use a disposable project and test installation and
removal on a disposable Windows account or machine.

## With MiKTeX or TeX Live already installed

1. Run Texloom setup and expand its details. Confirm it reports a successful
   sample compile and leaves the existing LaTeX installation in place.
2. Open Texloom, choose the Article template in Projects, and compile it.
   Verify that the PDF opens and a time appears beside Auto Compile. Make a
   small source change and check the time after the next completed preview.

## With no LaTeX distribution installed

1. Start setup on Windows 11 x64. Confirm it offers MiKTeX after installing
   Texloom. Choose Yes and follow the official MiKTeX wizard. Record whether
   the Texloom installer reports that the sample successfully compiled.
2. Open Texloom and compile an Article. Confirm the PDF and compile time.
3. On a separate clean test run, skip or cancel MiKTeX. Verify that Texloom
   opens and shows setup guidance when you try to compile.
4. In the disposable environment, uninstall Texloom and verify that the
   LaTeX installation and the sample project are still available. Reinstall
   Texloom and reopen the sample project.

Setup checks a basic document. Projects that use additional fonts or LaTeX
packages can still require those dependencies. The setup issue tracks the
results and any untested routes. [Windows setup issue](https://github.com/nadeemtsf/texloom/issues/34)
