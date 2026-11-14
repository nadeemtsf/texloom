# Texloom r8 follow-up checks — revision 2

Run these after coding stops, when you are ready. You control all input. Ordinary build: `Texloom-0.3.0-x64-development-2026-11-07-r8-setup.exe`. Save any real work first. Install this exact file and open its shortcut; report whether installation and launch worked. Keep any delete-application-data option unchecked. Do not uninstall or clear existing drafts for these checks.

The fresh disposable projects are in the chat's outputs folder under `Texloom-r8-followup-fixtures-2026-11-07`. Earlier projects and recovery entries are left intact. A refusal is a separate result from an accepted edit; report its message.

## 1. Arabic and Hebrew selection

Open `multilingual/main.tex` from the new folder, choose XeLaTeX and click Compile. First report whether the Arabic and Hebrew look correct. Select the Arabic paragraph in Canvas mode. If it gets a frame, move it slightly downward, wait for the result, then Undo. Repeat for the Hebrew paragraph. The selected paragraph should move; the other content should stay in place; Undo should restore it. You do not need to save this project.

Also try selecting the English paragraph beginning “This editable project”. Report each language separately, including any refusal or changed unrelated text.

## 2. Keep a draft when closing without Save

Open the new `recovery/main.tex`. In source mode, replace only `ORIGINAL R8 RECOVERY TEXT` with `KEPT R8 RECOVERY TEXT`. Wait for the bottom status to say **Recovery copy protected**. Close Texloom. The dialog should offer **Keep draft and close**; choose it.

Reopen Texloom and click Recover work. It should initially say **Choose a project to recover**, with restore unavailable. Choose the entry whose path ends with the new `recovery/main.tex`, then restore it. The editor should contain `KEPT R8 RECOVERY TEXT`. Undo should restore `ORIGINAL R8 RECOVERY TEXT`; Redo should bring back the kept text.

Do not select an unrelated draft or discard any entries. Leave the recovered project open without saving when you report the result. If the right draft is missing, stop this case and report what the list shows.

## 3. Good PDF after a failed compile

Open the fresh `compile-recovery/main.tex`, choose pdfLaTeX and Compile. Check that the preview says `BEFORE R8 PDF PUBLICATION`.

In source mode, change only `BEFORE` to `AFTER` and add a new line containing `\TexloomDeliberateCompileError` immediately after `\begin{document}`. Compile. This is an intentional error: report the message and whether the last successful `BEFORE` PDF remains visible.

Remove only the `\TexloomDeliberateCompileError` line, leaving `AFTER` in the source, and Compile again. The preview should now say `AFTER R8 PDF PUBLICATION`, and the compile error should clear. Report the failure and correction separately. Do not save changes to a real project.

The fixture was preflighted with an independent TeX command and PDF text extractor: the baseline and correction compiled with the expected text; the deliberate undefined command failed as intended. These checks do not establish Texloom preview retention or corrected publication. Those remain pending until your report.

Report normal sentences with what passed, failed or was refused; no special answer format is needed. Another monitor, clean-machine installation, uninstall preservation and an unassisted usability trial remain separate pending gates.

## 4. Signature and saved-project smoke check

Use the fresh signature/main.tex in the same fixture folder and compile with pdfLaTeX. Select the bold name and two lines below it, move them slightly downward, then edit only the name to Dr. Revised Researcher & Colleague. Check bold formatting, the other two lines and unrelated paragraphs. Undo/Redo the name edit, then Save and reopen the project. Compile explicitly and verify the saved result. Report any refusal rather than trying to work around it. Use only this disposable copy.

The r8 packaging update preserves all previous verifier notices and adds two supplemental license texts. Its DLL and worker are byte-identical to r7. Existing reported results keep their original version scope; this packet does not imply that earlier passes disappeared. Broader DPI, clean-machine, uninstall and research/performance gates remain pending.
