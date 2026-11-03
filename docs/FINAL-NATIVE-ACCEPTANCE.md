# Deferred final native acceptance

On 3 November 2026 the owner explicitly asked to continue all implementation phases and defer app-control tests until after implementation is finished. This changes test order. It does not convert missing native evidence into a pass.

Native mouse, keyboard/IME and end-to-end interactive acceptance are queued below. Do not request or retry app control during the remaining implementation work. Source/domain/controller tests, headless worker checks, real-engine CI, failure injection, builds and artifact checks continue normally. No clock-based reminder or automatic desktop-control task is created by this sequencing change.

## Final acceptance queue

Run this queue after the implementation backlog through P9 is complete and the exact candidate is built. Record the executable/package hash, OS/WebView/engine versions and case results. Use a generated public fixture; preserve private correspondence.

| Area | Deferred interaction |
| --- | --- |
| Source editing | Actual typing, composition with installed IMEs, Unicode/BOM/CRLF, multiple cursors, clipboard/find and context-menu/keyboard undo/redo |
| Reading | PDF text selection/copy/search, zoom, scroll and reading access after a failed compile |
| Selection | Click, Shift-click and marquee; repeated text and columns; visible logical-unit expansion; source navigation |
| Movement | Group drag/nudge, zoom/scroll coordinates, cancellation/Escape/lost capture and fractional displacement once implemented |
| Text/spacing | Protected text edits, locked commands, raw fallback, supported numeric gaps, reflow failure and undo |
| Complete signature | Select name plus two lines; move 12 bp down; edit name; undo/redo; Save, close/reopen, independently compile and compare |
| Persistence | Interactive Save/reopen, external-edit conflicts, project copy, interrupted-Save recovery and unsaved-typing recovery once implemented |
| Installation | Exact installed artifact on a clean machine; dependency/setup, resource layout and uninstall checks |
| Usability/performance | Real editor-paint/gesture/scroll timing and long-session behavior on the user's laptop and a documented Windows reference machine |

The previous native access rejection remains evidence of an unverified test, not a test failure in source logic. The prior access denial must not be bypassed by renaming apps, indirect launch or substituting automation methods. The owner's latest deferral means those checks should wait even if the desktop reconnects.

## Completion language

Track implementation and acceptance separately. An implementation item can be coded and automatically checked while its native acceptance is pending. Mark that state explicitly. A release claim still requires the final queue and remaining engine, coordinate, provenance, recovery and distribution gates to pass.
