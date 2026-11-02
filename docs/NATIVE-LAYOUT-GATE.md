# P6 native visual transaction checkpoint

The application now binds supported visual operations to a native source plan
and an independent PDFium check before committing canonical source or history.
This is an implementation checkpoint, not completion of native acceptance.

`document_prepare` accepts an optional typed operation. Rust checks its project,
revision, root, complete original source map and baseline input receipt, reruns
the source planner, and requires the proposed source map to match exactly.
The private plan stays with the pending candidate. A selection probe without
an operation may compile and be discarded; it cannot acquire commit authority.

The same logical compilation lease covers candidate compilation, private
identity probes and independent inspection. The frontend never supplies glyph
membership, a successful-verification flag, executable paths or expected hashes.
The packaged worker's SHA-256 is embedded by the desktop build. PDFium keeps its
separate fixed development DLL pin. Both inputs are checked for each inspection.

| Operation | Native verification |
| --- | --- |
| Move an existing text object | Unique marked-content target in both PDFs, requested delta derived from emitted source, strict displacement comparison |
| First wrap and move | Privately compile the same wrapper/package at zero offset, prove full appearance identity with the original, then verify the move |
| Protected text, raw fragment or explicit gap edit | Exact native source plan, eligible compile, unique target and inspected reflow; unrelated text multiplicities and non-text object count must remain |
| First wrap and edit | Establish zero-offset identity first; then apply the reflow policy |
| Detach | Verify return to zero displacement and prove the private zero wrapper has the same appearance as the unwrapped candidate |
| Existing StudioBlock controls | Isolated compatibility source planner and eligible build, followed by existing PDF.js acceptance; this is not independent normal-text displacement certification |

Movement and reflow are separate policies. Reflow can change positions and
pages; it does not inherit the unchanged-neighbor geometry rule. The present
reflow check is conservative about changes in extraction, hyphenation or page
number text. Reflow plus displacement is split into separate actions.

Approval is a private Rust value tied to the candidate build, baseline,
revision and cancellation lease. It pins artifacts against cache eviction.
Commit rechecks receipts and cancellation while holding the canonical document
lock, consumes the pending candidate once, and creates one history entry.
Cancellation, typing, profile/history changes, a discarded token, mismatched
builds or altered evidence cannot commit that candidate.

The renderer runs from its checked program directory with absolute evidence
paths. A test exposed Windows error 267 when a long evidence directory was
used as the process working directory; the corrected path works without
shortening or relocating the user's project. Evidence cleanup remains flat.

Dragging now shows a moving outline. It does not crop unrelated background ink
into a ghost or cover the verified page with a white rectangle. Pointer motion
does no source work; release dispatches one operation. Losing pointer capture,
Escape and pointer cancellation discard the outline.

Windows builds prepare the worker, checked DLL and notices without requiring
Python. Tauri resources and the portable package include the verification
folder. The portable packager checks resource hashes and writes an artifact
receipt. The old PDFium pin is still a development reference: security/version
review and exact installed-artifact acceptance are open before public release.
The first Windows PowerShell build hook failed its authorization check in this
environment. Preparation now uses the existing Node build runtime and checks
the pinned archive before reading two exact entries; no policy setting changed.

Tests distinguish source-plan authorization, forged candidates, probe refusal,
one-shot commit, exact undo/redo, cancellation, artifact tampering and recipe
binding. Explicit headless Windows tests use the real worker to reject neighbor
drift without touching source/history and to reject a changed packaged program.
These synthetic PDFs do not replace a real-engine/native mouse workflow.
The engine proof workflow now sends its compiled PDFs to a Windows worker job:
24 positive movement/identity cases and three overlap refusals must pass using
native mark-handle identity. The existing cross-page experiment stops at its
measured refusal before compiling moved variants, so it is explicitly separate.
This new job is pending, not a local pass.

Still open: native typing/IME/multicursor/context-menu undo; real mouse group
selection and the full signature edit/save/reopen/independent compile scenario;
ordinary fractional-pixel displacement; broader native marker/paint/coordinate
coverage; and the remaining P1/P3/P5/P7-P9 acceptance gates. The local MiKTeX CLI
still fails preflight with registry access denied in this environment. A fresh
native launch attempt was rejected by automatic approval review because the
earlier desktop-access denial is still in force. Existing
three-engine CI evidence is retained separately; it is not a new native gate pass.
