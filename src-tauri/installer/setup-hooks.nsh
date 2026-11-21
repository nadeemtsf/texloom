!define TEXLOOM_LATEX_SETUP "${__FILEDIR__}\setup-latex.ps1"

!macro NSIS_HOOK_POSTINSTALL
  ${IfNot} ${Silent}
  ${AndIf} $PassiveMode != 1
  ${AndIf} $UpdateMode != 1
    Push $0
    DetailPrint "Checking for an existing LaTeX installation..."
    nsExec::Exec '"$INSTDIR\${MAINBINARYNAME}.exe" --check-latex-for-setup'
    Pop $0
    ${If} $0 == 0
      DetailPrint "The existing LaTeX installation compiled the Texloom Article template."
    ${ElseIf} $0 == 10
      MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON1 "Texloom is installed. Install MiKTeX now to compile LaTeX documents? Setup downloads the official installer (about 142 MiB) and any packages needed by the bundled templates. You can also install MiKTeX or TeX Live later." IDNO texloom_latex_done
      InitPluginsDir
      File "/oname=$PLUGINSDIR\texloom-setup-latex.ps1" "${TEXLOOM_LATEX_SETUP}"
      DetailPrint "Setting up MiKTeX. Follow its separate setup wizard when it opens."
      nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\texloom-setup-latex.ps1" -TexloomPath "$INSTDIR\${MAINBINARYNAME}.exe"'
      Pop $0
      ${If} $0 == 0
        DetailPrint "MiKTeX compiled the Texloom Article and multilingual templates."
      ${Else}
        MessageBox MB_OK|MB_ICONEXCLAMATION "Texloom is installed. LaTeX setup did not complete the template compiles. Check the installer details or finish setting up MiKTeX packages before compiling."
      ${EndIf}
    ${Else}
      MessageBox MB_OK|MB_ICONEXCLAMATION "Texloom is installed. The existing LaTeX installation could not compile its Article template. Check your MiKTeX or TeX Live setup and confirm the geometry package is installed."
    ${EndIf}
    texloom_latex_done:
    Pop $0
  ${EndIf}
!macroend
