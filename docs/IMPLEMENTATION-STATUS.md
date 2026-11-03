# Implementation and acceptance status

Updated 4 November 2026. Scope: APPROVED-IMPLEMENTATION-PLAN.md, with the owner's later instruction to complete native tests now and limit input-language acceptance to English, Arabic and Hebrew.

| Area | Current implementation and observed evidence | Remaining acceptance |
| --- | --- | --- |
| Source/history and IPC | Rust authority, exact byte preservation, recovery journal, strict generated document/build/workspace contracts. 89 core, 54 build-library, 50 native default and 231 frontend checks pass locally. | Exact final-candidate native matrix; long-session timings |
| Compilation | Immutable receipts, cancellation/convergence, declared BibTeX and Biber. All 24 expected three-engine corpus outcomes and both bibliography recipes pass. | Native error/cancellation/concurrent-typing scenarios |
| Selection | Private zero-offset probes. 28 positive and 6 expected refusals across three engines, including a LuaTeX minipage box-hint fallback which rejects wrong single-name candidates. | Exact rebuilt native LuaTeX fallback; repeated/column mouse cases |
| Movement | Pinned PDFium 156 worker/protocol 3 independently predicts target translation. 24 positive and 6 oracle refusals; nine fractional moves pass. Actual native mouse line and three-line marquee, 12 bp move, protected name edit, two Undo/Redo, Save and reopen passed on the recorded development candidate. | Exact final candidate, all advertised engines/zoom; independent saved-project rebuild |
| Richer source controls | Images/aspect ratio, supported table columns, explicit lengths/margins/font/leading/line factors/float placement, selected source styles. 27 real adapter cases pass across three engines. | Native controls and image insertion matrix |
| Project workflows | Paths, recents, guarded ZIP import/export, explicit encoding review and UTF-8 copy, profile persistence, editable Article/Letter/CV/multilingual templates. 19 template/class cases pass, including Arabic/Hebrew glyphs under XeLaTeX. | Native ZIP/encoding/template/image workflows |
| Reading | PDF text layer, occurrence search and bounded lazy long-document painting. Six real PDF.js crop/rotation/UserUnit render/coordinate cases pass; such pages explicitly refuse layout edits. Native PDF word selection/copy into source passed. | Native long-document/zoom and failed-compile reading |
| Durability | Native multilingual crash/relaunch restored source and history; Undo/Redo passed; a subsequent actual external edit correctly blocked Save. | Missing-folder/source-copy, multiple instances, final-candidate repetition |
| Distribution | Unsigned NSIS and verified 15-entry portable packages generated. Cargo audit: zero vulnerabilities; inactive-platform warnings retained. npm audit: zero known vulnerabilities. Exact Windows dependency inventory, MPL source availability and notices are generated for portable and installer. | Clean-machine/account installation, uninstall, human usability trial; signing/public distribution require owner decisions |

This is an implementation and evidence report, not a completed release claim. Old deferral statements and old artifact receipts elsewhere describe historical checkpoints. FINAL-NATIVE-ACCEPTANCE.md records the current interactive queue. Never compile .text files; all fixtures are synthetic .tex projects.
