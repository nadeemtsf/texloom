# Native acceptance in progress

5 November supplement: product source `5e59e77282fd5e4b4f70440437b8f841ccea1364` adds setup help and optional timing plumbing. 252 frontend/88 default core checks and the ordinary Windows release/NSIS build pass. No new native/UI/setup/installation or performance observation was made. Both the ordinary and diagnostic variants need their own recorded native checks after the owner returns. The owner-away deferral below continues to apply.

Latest instruction: the owner is away from the laptop and explicitly defers every test requiring approval or opening Texloom. Local 0576198 adds focus-loss gesture cancellation; 239 frontend/88 default core and Windows build pass, but its new executable (SHA-256 3e8207d43704fd017bb05b385772069348d69c898635462e85af46bb675bf2ce) has not been launched or installed. Keep these native checks pending; no repeated approval request is needed while this deferral applies. Existing installed r2 observations below retain their own artifact identity.

Updated 4 November 2026. Native tests resumed at the owner's request. English, Arabic and Hebrew are the current language scope; CJK IMEs and installing Windows input languages are deferred. All projects are disposable synthetic fixtures. No .text file was compiled, and the default example's earlier saved98bp position is preserved.

## Exact candidates

| Candidate | Executable SHA-256 | Observed scope |
| --- | --- | --- |
| First interaction build | E3FDA2786D0C4C98B580B29D0E9F087E6B09BDB5A810E87D122D3E6F1EB18986 | pdfLaTeX signature, source/input, crash/save/conflict |
| Seventh | 4B188A1F0606A6D7B0AF0E32AA1C2465758E29D83856BEC76CE787A14FAA464C | LuaTeX mapping/group/edit, catalog/recovery, project/adapter flows, XeLaTeX signature and compile failure |
| Tenth | 42CA509B0D0CC43C740C018837718B9CA6E8159D5B153BBCCED8509B056192C2 | Arabic/Hebrew search, label drag, repeated keyboard nudges, controls, real cancellation and concurrent typing |
| Eleventh | FA022284F2F4FDE6955DD068524F23AEECD4DC400E5C86508C06E21B44D8970D | Long-document zoom-position correction, lazy reading/navigation |
| Installed eleventh | 8B7BD9542BA78202451DAA9A5DB031B739870933F92822AAB7AB17F15C6F3AE9 | Existing-account NSIS installation, verified nudge/Save/Undo/reopen, Letter/CV templates and shutdown during compilation |
| Twelfth / r2 portable | 20520866982788619F7E128B1A9B9B0345977C75B944FE42E6B94DB1C35AB5CD | Modal feedback correction and line-spacing/table-placement native repeat |
| Installed twelfth / r2 | A3C4225BE0FBC9BF6F79AD0C4234AC9CD0CDAEFCF6002F5EB2846A28AD5D067A | Approved existing-account reinstall/update, matching resources, verified nudge, exact Save/Undo, clean close and relaunch |

The eleventh NSIS SHA-256 is `916356F24EF2BE3765844FA1D62C8BAE050FFEB844B0420C5CB2951877AB1421`; its approved normal per-user installation completed on the existing developer account. Installed/portable EXEs differ in exactly three bytes at offset 13489842: Tauri's `NSS` versus `UNK` bundle marker; all other bytes are identical. See evidence/installed-acceptance-2026-11-04.json. This is not a clean-machine/account result.

The twelfth/r2 installer SHA-256 is `AE8567329A095445417E010069A27372317444F6DCE30F1B701037977C500CF1`. The owner separately approved it; the existing-account reinstall/update and installed native repeat completed. Its installed/portable EXEs differ only in the three-byte Tauri bundle marker at offset 13489778. Setup advanced between read-only observations; maintenance/default destination/extraction and the resulting installed launch were observed, but the completion page and individual clicks are not claimed. Documentation-only portable repackaging preserves the EXE hash. See evidence/installed-r2-acceptance-2026-11-04.json and the separate feedback-fix CI receipt.

## Observed passes

- Open BOM/CRLF sources by path, review without execution, explicitly compile, reopen through recents.
- Actual pdfLaTeX name click and three-line marquee, exact12bp move, protected bold-name edit retaining both2pt gaps. Undo2/Save restores main.tex SHA A91EC3F6AAD7B7E77C28719D29CEE2273932EA8DC442394D8D10E4A6CB4B66A1; Redo2/Save exact, normal close/reopen and recompile.
- Actual LuaTeX single-name selection after wrong bounded candidates are rejected, three-line marquee,12bp group move and explicit LaTeX name fallback. Undo2/Save restores all source bytes SHA fc39c665d24eed31a998fb235363fe4e75b8dbcbe49a6186abd397da466f9e20.
- Actual XeLaTeX signature at91% zoom, three-line marquee,26bp frame drag, literal name edit with bold/two2pt gaps preserved. Undo2/Save restores original bytes; Redo2/Save restores moved/edited bytes exactly.
- In the tenth build, drag the label itself (+30bp), Down(+1bp) and Shift+Down(+10bp) after separate PDF refreshes without refocusing. Saved source differs only in dy26->67. Escape clears selection without source change; Undo3/Save exact original.
- PDF word double-click/Copy/paste into the real source editor; search and navigation. Existing physical English/Arabic/Hebrew keyboards yield a/ش/ש; Undo restores source, keyboard cycle returned English. Actual Ctrl+D multicursor edits undo together.
- Real undefined-control-sequence compile retains invalid source and the last good PDF, disables stale visual edits, and still permits reading/copy/paste. Undo restores valid source.
- Cancel a real bounded20-million-iteration engine job through the toolbar; engine processes are gone afterward. Typing during a real build supersedes the old result and preserves newer input/last good PDF. The newer source subsequently compiles; explicit Save writes exactly original source plus typed prefix. An earlier input attempt missed editor focus and is not counted.
- Abrupt exact-process termination/relaunch recovers protected multilingual source and history; Undo/Redo retained and explicitly unsaved. External notes.sty modification blocks Save without overwriting disk/editor. Native Save-copy preserves exact BOM/CRLF/Unicode bytes. Missing-folder source salvage/copy and simultaneous native catalog access pass; original fixture folder restored.
- Native Article and English/Arabic/Hebrew template creation, explicit pdfLaTeX/XeLaTeX compilation, readable three-language output. Native image copy/insertion,35mm aspect-preserving resize, margin change, selected-source bold, title size18->20pt and table column28->35mm pass. Complete corresponding Undo/Save restores all original bytes.
- Native ZIP export contains unsaved source, exact image and engine/main profile while saved source remains unchanged. Import opens exact three-file roundtrip in a new folder and compiles. Unsaved switch Cancel preserves editor.
- Native Windows1252/1256/1255 readonly previews show Western/Arabic/Hebrew text correctly. Conversion creates exact UTF-8 copies and updates inputenc; original hashes unchanged. Copy explicitly compiles.
- Ordinary Arabic query finds shaped PDF letters and highlights the correct original word; Hebrew search passes. A100-page project paints/searches distant pages75/100. Eleventh build zoom preserves page75 and reading position; native wheel scroll paints page76.
- Saved project independently rebuilds under all three engines outside app caches:12bp group displacement and neighbors match; pdfLaTeX whole-page pixels equal native exported PDF.

The eleventh build also edits only the second identical phrase, with exact Undo/Save. A marquee spanning independent columns refuses safely: “Select adjacent lines; unselected source occurs between these items.”

Later installed-artifact checks passed normal installation completion/launch, resource hashes/notices, per-user 0.3.0 registration and desktop/Start menu shortcuts. A synthetic example opened for review, explicitly compiled, mapped the name, accepted a verified +1bp Right nudge, kept disk unchanged until Save and restored all three original files with one Undo/Save. Clean close/relaunch/recents/recompile passed. Closing while an actual pdfLaTeX process was running left no app/compiler/worker processes. Native Letter and CV creation/explicit compilation passed.

On the twelfth/r2 portable build, an out-of-range factor 4 stays safely refused with its explanation readable inside the modal. A fresh controlled synthetic copy accepted line spacing 1.05 to 1.15 and table placement htbp to H. Explicit Save changed only those values, retaining every other byte and the final CRLF; two Undo operations plus Save restored original SHA `c9a794cbb70c9141288d3345c8c156dba3c4f77721ee31b912e9407dc6d578eb`. An earlier mixed-input session lost the final newline; its strict byte assertion failed and is retained as inconclusive because intervening input prevents attributing the change. It is not silently promoted to a preservation pass. See evidence/feedback-fix-native-2026-11-04.json.

The installed r2 repeat opened a fresh synthetic project without automatic compilation, explicitly compiled, mapped the name, accepted a verified +1bp Right nudge and kept disk unchanged until Save. Save changed only the expected wrapper/package; one Undo/Save restored all three original files exactly. Twelve protected synthetic fixture/copy files retained their hashes across reinstall. Clean close left no app/worker processes, and relaunch succeeded. The launch helper initially reported no targetable window; a fresh inventory found the running app. A first byte-check regex omitted the wrapper's terminating newline; the corrected exact reconstruction and the original assertion failure are distinguished in the receipt.

The approved final interrupted-Save restore reopened the exact original source/profile. Rollback intentionally retained a `rolled_back` journal; the separately approved staged-draft discard removed only that journal. Original hashes and the separately saved recovered-edit copy stayed exact. An earlier harness assumption that rollback itself deletes the journal was corrected after source/state inspection; it is not counted as a product defect.

## Regressions exposed and corrected

Native tests found Windows active-draft locked-marker read ordering, LuaTeX verbatim ./ input paths, shaped-Arabic search, a drag label ignoring pointer input, keyboard focus lost after PDF publication, zoom preserving pixels rather than the reading position, and a correctly refused numeric input whose explanation was blurred behind the modal. The last issue was corrected at c26a238; actual rebuilt-window feedback is visible inside the modal. Clean-checkout CI found an absent generated-resource directory and concurrent bibliography test timestamp collisions. Failed attempts/logs are retained; a correction is counted only after its relevant repeat.

## Open gates

- Real mid-gesture Escape/pointer cancellation/lost capture. Controller tests cover these cancellation events, but the current Windows input API cannot inject a held drag plus an intervening key/cancel event.
- Windows DPI scale-change, multiple-monitor and documented reference-machine latency checks. Windows Settings did not expose a targetable window through the tool; no display setting was changed. A passing zoom test is not a DPI test.
- Both exact eleventh and r2 installations have existing-account evidence; neither establishes a clean-machine dependency/setup result.
- End-to-end latency distributions and an unassisted human usability trial remain. A20-minute exact-process-tree sample completed117 snapshots:548.83–824.18MiB working set,680.23MiB final; private peak494.54MiB,389.15MiB final. Shared WebView descendants included a second app and its crash/relaunch during the sample; this is not an isolated reference-machine benchmark, leak pass or long-session certification.
- Clean Windows account/machine installation and missing-dependency setup remain. The existing-account install/resource/shutdown checks passed, but uninstaller launch returned “product policy blocks this app”; no uninstall ran and no alternate route was used. The app remains installed and project/recovery data are preserved. Signing/public distribution requires owner decisions/credentials.

The project is not release-complete. Automated suites and headless real-engine proof do not replace unexecuted native/installation gates. Raw synthetic observations remain in the continuation workspace work/native-acceptance/ui-observations.json; committed receipts identify exact sources/artifacts without private user documents.
