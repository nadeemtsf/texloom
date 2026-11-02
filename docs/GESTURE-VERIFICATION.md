# P6 gesture verification checkpoint

Six focused tests execute the production CanvasController against an explicit event/DOM boundary. They verify that repeated pointer movement produces only a transient ghost, release dispatches exactly one movement, click jitter dispatches none, Escape and pointer-cancel restore the preview without an edit, zoom/free movement retains PDF-point deltas, and stale/read surfaces do not start a drag.

The complete frontend suite passes 64 tests. Combined with the already passing 58 core, 17 native and 17 build-infrastructure checks, this is 156 local checks. No production executable code changed in this checkpoint; the P5 executable hash remains the recorded build.

P5 source 6eb2b5a also passed the real engine workflow: the existing independent movement proof, all 33 declared selection outcomes (26 selections/seven refusals), and protected literal edits compiled under pdfLaTeX/LuaLaTeX/XeLaTeX with the exact source scaffold and expected visible text. Raw edit evidence is in docs/evidence/protected-text-verified-run.json.

These tests do not emulate native Windows input and do not certify P6. A packaged independent PDFium worker, operation-specific comparison, overlap/annotation policy and the complete signature mouse/edit/undo/save/reopen scenario remain open. P2 manual input acceptance and P5 spacing/native reflow acceptance also remain open. P7 recovery/conflicts/performance, P8 installed-artifact acceptance and P9 richer adapters have not been completed.

The next P6 checkpoint adds an independent Rust comparison and twelve focused tests; see DISPLACEMENT-POLICY.md. The complete local run passes 64 core, 29 build/comparison, 19 native service and 67 frontend tests (179 combined). The real-PDF identity smoke passes and the retained broken-literal PDF is rejected for displaced target/neighbor geometry. New three-engine comparison CI remains pending. P5's six literal/explicit-gap engine cases and Windows packaging passed at 3900ac5; broader/native spacing acceptance remains open.
