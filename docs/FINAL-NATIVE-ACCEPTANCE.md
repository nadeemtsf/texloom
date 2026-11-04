# Native acceptance in progress

Updated 4 November 2026. Native tests resumed at the owner's request. English, Arabic and Hebrew are the current language scope; CJK IMEs and installing Windows input languages are deferred. All projects are disposable synthetic fixtures. No .text file was compiled, and the default example's earlier saved98bp position is preserved.

## Exact candidates

| Candidate | Executable SHA-256 | Observed scope |
| --- | --- | --- |
| First interaction build | E3FDA2786D0C4C98B580B29D0E9F087E6B09BDB5A810E87D122D3E6F1EB18986 | pdfLaTeX signature, source/input, crash/save/conflict |
| Seventh | 4B188A1F0606A6D7B0AF0E32AA1C2465758E29D83856BEC76CE787A14FAA464C | LuaTeX mapping/group/edit, catalog/recovery, project/adapter flows, XeLaTeX signature and compile failure |
| Tenth | 42CA509B0D0CC43C740C018837718B9CA6E8159D5B153BBCCED8509B056192C2 | Arabic/Hebrew search, label drag, repeated keyboard nudges, controls, real cancellation and concurrent typing |
| Eleventh | FA022284F2F4FDE6955DD068524F23AEECD4DC400E5C86508C06E21B44D8970D | Long-document zoom-position correction, lazy reading/navigation |

The eleventh NSIS SHA-256 is916356F24EF2BE3765844FA1D62C8BAE050FFEB844B0420C5CB2951877AB1421. It is built but installation is unexecuted. Exact dawn receipts and source hashes are in evidence/dawn-native-acceptance.json. Later documentation-only repackaging does not change the executable hash.

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

## Regressions exposed and corrected

Native tests found Windows active-draft locked-marker read ordering, LuaTeX verbatim ./ input paths, shaped-Arabic search, a drag label ignoring pointer input, keyboard focus lost after PDF publication, and zoom preserving pixels rather than the reading position. Clean-checkout CI found an absent generated-resource directory and concurrent bibliography test timestamp collisions. Failed attempts/logs are retained; a correction is counted only after its relevant repeat.

## Open gates

- Real mid-gesture Escape/pointer cancellation/lost capture. Controller tests cover these cancellation events, but the current Windows input API cannot inject a held drag plus an intervening key/cancel event.
- Windows DPI scale-change, multiple-monitor and documented reference-machine latency checks. Windows Settings did not expose a targetable window through the tool; no display setting was changed. A passing zoom test is not a DPI test.
- Actual interrupted Save was stopped during staging; original bytes remained intact, the3.7MB protected source/history recovered, and a separate exact new copy passed. Final disk restore/reopen and deliberate draft discard remain: the current unsaved-editor discard prompt was cancelled pending action-time confirmation.
- End-to-end latency distributions and an unassisted human usability trial remain. A20-minute exact-process-tree sample completed117 snapshots:548.83–824.18MiB working set,680.23MiB final; private peak494.54MiB,389.15MiB final. Shared WebView descendants included a second app and its crash/relaunch during the sample; this is not an isolated reference-machine benchmark, leak pass or long-session certification.
- Exact installed artifact on a clean Windows account/machine, dependency/setup/resource layout, uninstall, and interrupted shutdown. Windows UI installation requires action-time confirmation. Signing/public Store release requires owner decisions/credentials.

The project is not release-complete. Automated suites and headless real-engine proof do not replace unexecuted native/installation gates. Raw synthetic observations remain in the continuation workspace work/native-acceptance/ui-observations.json; committed receipts identify exact sources/artifacts without private user documents.
