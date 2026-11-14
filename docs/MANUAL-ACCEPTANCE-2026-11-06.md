# Texloom manual acceptance checks - 6 November 2026

Run these when the coding run has stopped. You control all mouse and keyboard input. Use only the disposable fixtures supplied with this packet. These are guided development checks, not the final research evaluation or an unassisted usability trial.

Record each case as PASS, FAIL, REFUSED, NOT RUN, or UNAVAILABLE, with the message and what happened. A safe refusal is recorded separately from a successful edit. Never mark an unexecuted case as passed.

## A. Install the named ordinary build

Save any work and close Texloom. Run `Texloom-0.3.0-x64-development-2026-11-06-r6-setup.exe` from outputs. This is the ordinary build; do not use the diagnostics ZIP for these checks. Report any installer message and whether Texloom opens from its shortcut. If Windows blocks it, report the exact message; do not bypass a security block or change security settings.

Keep any option to delete application data UNCHECKED. The generated uninstaller really can delete projects/recovery data when that option is selected. Installation on this existing account is not a clean-machine result.

## B. Signature group, name edit, Undo/Redo and reopen

1. Projects -> Open path: `<fixtures>/01-signature/main.tex`. Click Compile when requested.
2. Draw a box around the name and the two lines below it. The frame/inspector should identify a group containing all three lines. The source should highlight the corresponding group.
3. Drag its green frame a little lower. While dragging, an outline previews the move; the verified result appears after release. The surrounding paragraphs must stay put. Report any refusal instead of treating it as a passed move.
4. Double-click the frame. Change only the Bold/name field to `Dr. Manual Researcher & Colleague`. Keep the other two fields and line gaps unchanged. Apply. The name should stay bold and show a literal ampersand.
5. Click Undo once: only the name should revert, with the group still moved. Click Redo once: the revised name should return; the other lines should stay unchanged.
6. Save. Reopen the same path through Projects and Compile. The revised wording, formatting and position should return.

Tell me the outcome of steps2-6 separately. An unchanged preview alone is not proof that a source edit was accepted.

## C. English, Arabic and Hebrew

Open `<fixtures>/02-multilingual/main.tex`. Select XeLaTeX and Compile. Check that all three languages appear without missing-glyph boxes. Keep every LaTeX language/environment command unchanged.

In the Arabic source text replace `مرحبا بالعالم` with `أهلا بالعالم`. In the Hebrew source text replace `שלום עולם` with `שלום חברים`. Type or paste using your existing setup; do not add keyboard languages. Save and Compile. Check the revised words, Arabic joining and right-to-left appearance. In Read text mode, try Find in PDF for `العالم` and then `חברים`; record the counts/messages. Report whether you typed or pasted, since those are different input evidence. CJK IMEs and additional languages remain out of scope.

## D. Compilation failure and return to the last good preview

Open `<fixtures>/03-compile-failure/main.tex` and Compile successfully first. Add `\NotARealTexloomCommand` on a new line before `\end{document}` and Compile again. An error should be visible, and the previous good PDF should remain available as an older preview. Do not Save the broken source. Undo the added command and Compile; the normal preview should return. Report the error and whether the app remained usable.

## E. Repeated text: correct occurrence or explicit refusal

Open `<fixtures>/04-repeated/main.tex` and Compile. There are three visible occurrences of `Dr. Example Researcher`. Select the middle occurrence and try moving it slightly. The other two occurrences and surrounding paragraphs must stay unchanged. Report PASS only if that exact occurrence moved; report REFUSED if Texloom explains it cannot map safely, or FAIL if a different occurrence moved.

## F. Recovery, using a disposable project only

Open `<fixtures>/05-recovery/main.tex`, Compile and Save once. Change ORIGINAL to `UNSAVED RECOVERY TEST` without saving. Close Texloom normally and reopen it. Use Recover work and identify this exact project's path. Report whether its unsaved edit can be recovered and compiled. If a save/discard dialog appears, record it and keep the draft; do not discard recovery entries or touch entries belonging to real projects. Leave this case NOT RUN if you cannot identify the disposable draft confidently.

## G. Optional environment checks

If you already have another monitor, move the app between monitors and check that PDF frames, clicks and dialogs remain aligned. A normal zoom test does not establish Windows DPI behavior. Record your actual monitor/scale setup; mark unavailable hardware UNAVAILABLE.

Uninstall testing is optional and should follow the other cases. Keep delete-application-data UNCHECKED, then reinstall the exact same setup and confirm the disposable saved projects still reopen. Do not perform this if preserving your existing installation/data is uncertain. A separate clean Windows account/machine and missing-prerequisite check remain NOT RUN unless that environment is actually available.

## Reply format

A: outcome and installer message
B: selection / move / name edit / Undo / Redo / Save-reopen
C: display / typing-or-pasting / Arabic search / Hebrew search
D: error / last-good preview / successful return
E: middle occurrence moved, refused, or wrong occurrence
F: recovered edit, dialog, or not run
G: actual hardware/uninstall observation, or unavailable/not run

The developer will check saved files independently when work resumes. None of these checks authorizes publication, submission, adding Windows languages, or compiling `.text` files.
