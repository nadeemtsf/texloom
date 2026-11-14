# Texloom r7 follow-up checks

Run these after coding stops, when you are ready. You control all input. Ordinary build: `Texloom-0.3.0-x64-development-2026-11-06-r7-setup.exe`. Save any real work first. Install this exact file and open its shortcut; report whether installation and launch worked. Keep any delete-application-data option unchecked. Do not uninstall or clear existing drafts for these checks.

The fresh disposable projects are in the chat's outputs folder under `Texloom-r7-followup-fixtures-2026-11-06`. Earlier projects and recovery entries are left intact. A refusal is a separate result from an accepted edit; report its message.

## 1. Arabic and Hebrew selection

Open `multilingual/main.tex` from the new folder, choose XeLaTeX and click Compile. First report whether the Arabic and Hebrew look correct. Select the Arabic paragraph in Canvas mode. If it gets a frame, move it slightly downward, wait for the result, then Undo. Repeat for the Hebrew paragraph. The selected paragraph should move; the other content should stay in place; Undo should restore it. You do not need to save this project.

Also try selecting the English paragraph beginning “This editable project”. Report each language separately, including any refusal or changed unrelated text.

## 2. Keep a draft when closing without Save

Open the new `recovery/main.tex`. In source mode, replace only `ORIGINAL R7 RECOVERY TEXT` with `KEPT R7 RECOVERY TEXT`. Wait for the bottom status to say **Recovery copy protected**. Close Texloom. The dialog should offer **Keep draft and close**; choose it.

Reopen Texloom and click Recover work. It should initially say **Choose a project to recover**, with restore unavailable. Choose the entry whose path ends with the new `recovery/main.tex`, then restore it. The editor should contain `KEPT R7 RECOVERY TEXT`. Undo should restore `ORIGINAL R7 RECOVERY TEXT`; Redo should bring back the kept text.

Do not select an unrelated draft or discard any entries. Leave the recovered project open without saving when you report the result. If the right draft is missing, stop this case and report what the list shows.

## 3. Good PDF after a failed compile

The earlier failure/last-good-preview behavior was reported. The remaining detail is whether removing the bad command and pressing Compile publishes a successful new PDF and clears the error. Use only the earlier disposable compile-failure project if you still have it. Otherwise leave this case pending.

Report normal sentences with what passed, failed or was refused; no special answer format is needed. Another monitor, clean-machine installation, uninstall preservation and an unassisted usability trial remain separate pending gates.
