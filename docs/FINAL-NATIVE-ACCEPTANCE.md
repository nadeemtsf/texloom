# Native acceptance in progress

Updated 4 November 2026. The owner's latest instruction supersedes the earlier sequencing deferral: reopen the exact built app and complete the tests now. English, Arabic and Hebrew are the current input-language scope; CJK IMEs and adding other Windows input languages are deferred.

All interaction uses the packaged Windows app and disposable synthetic projects. Never compile .text files. The default example's earlier saved 98 bp name position is preserved.

## Observed native passes

The first recorded candidate executable SHA-256 is E3FDA2786D0C4C98B580B29D0E9F087E6B09BDB5A810E87D122D3E6F1EB18986. Later rebuilt artifacts require their own receipt and remaining native checks.

- Open a BOM/CRLF project by path, review sources without automatic execution, explicitly compile with pdfLaTeX, and reopen through recents.
- Click the name alone, marquee the three signature lines, apply exactly 12 bp down, and edit only the bold name while retaining both 2 pt gaps. Two Undo followed by Save restores main.tex to SHA-256 A91EC3F6AAD7B7E77C28719D29CEE2273932EA8DC442394D8D10E4A6CB4B66A1. Two Redo/Save restore the accepted changed source exactly; normal close/reopen and recompile preserve it.
- Native PDF search finds one Synthetic occurrence. Mouse double-click selects Synthetic; Copy and paste into the actual source editor yields exactly that word, followed by exact Undo. An earlier drag missed the text and left the clipboard unchanged; it is not counted as a pass.
- Actual physical key input under the three existing layouts yields English a, Arabic ش and Hebrew ש. Three Undo restore the exact source. The layout cycle is returned to English. This is installed-layout typing evidence, not CJK IME composition evidence.
- Two Ctrl+D selections and one typing action replace Alpha Alpha with Beta Beta. One Undo restores both words; the next restores the original fixture.
- Abruptly terminate only the hash-verified synthetic app process after protected revision 17, relaunch, open Recover work, and restore the exact multilingual unsaved comment plus undo history. Undo/Redo works; restored work remains explicitly unsaved.
- Change the original notes.sty externally after recovery. Save reports a conflict and preserves both the editor version and the exact external file. Save a copy through the native Windows folder chooser preserves BOM/CRLF/Unicode in a new project and retains the original external edit. Byte comparisons passed.

The crash catalog exposed a Windows locked-handle ordering issue: an active draft was correctly excluded but produced a misleading raw lock error. The fix acquires ownership before reading its marker through the same handle; the strengthened Windows test passes. The rebuilt candidate repeats the catalog without the error, including while a second native instance owns a live draft. Missing-folder recovery and a source-only copy also pass exact byte comparisons; the fixture folder was restored afterward.

## Remaining interactive queue

- Exact rebuilt app: LuaTeX minipage candidate fallback and current locked-draft fix; native image/layout/style/template/ZIP/encoding workflows.
- Independent saved-project rebuild passed with all three engines outside app caches: group displacement and neighbors match, and pdfLaTeX full-page pixels equal the native export.
- Native malformed compile, cancellation, concurrent typing, reading after failure, repeated text/columns, drag/nudge and cancellation, zoom/scroll/DPI, long documents and sustained resource measurements.
- Missing-folder source salvage/copy and simultaneous native catalog access passed. Interrupted-Save UI and deliberate disposable draft deletion remain (confirmation rules apply).
- Exact installer on a clean Windows account/machine, dependency/setup/resource layout and uninstall. Installer actions through Windows UI require action-time confirmation under the computer-use skill. Signing/public Store submission and a human usability trial remain owner/external gates.

Implementation and acceptance are tracked separately. A passing build, headless renderer or compiler corpus does not mark an unexecuted native case passed. Raw synthetic interaction observations are retained in the continuation workspace's work/native-acceptance/ui-observations.json; release evidence must name the exact source and artifact hashes.


The sixth candidate exposed an additional real Windows LuaTeX mapping failure: SyncTeX returned a verbatim path containing ./main.tex, which Windows rejects with error 123. The mapped-path resolver now normalizes that spelling before canonicalization while retaining the snapshot-directory fence, with a regression covering dot components and escape refusal. The exact seventh build (SHA-256 4b188a1f0606a6d7b0af0e32aa1c2465758e29d83856bec76ce787a14faa464c) repeats actual LuaTeX name selection successfully after the rejected candidates; source/history remain unchanged by selection. The earlier sixth-build refusal is retained as regression evidence.
