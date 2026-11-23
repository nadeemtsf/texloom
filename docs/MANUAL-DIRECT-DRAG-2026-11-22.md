# Direct drag check on Windows

Use a disposable copy of the three-page Article. Compile before starting.

1. Select the first sentence in the PDF and drag it down and right. Check that
   the sentence moved and the project file list still contains only main.tex.
   The source should show \raisebox and \kern around that sentence.
2. Click the moved sentence and drag it again. Check that the same source
   command changes its offsets. The neighboring paragraph and the other two
   pages should look the same.
3. Undo once and redo once. Check both the PDF and the offsets in the source.
   Save, reopen, then run the same engine from the saved project directory.
4. Try a soft-wrapped paragraph. Free dragging should refuse it and keep the current source and PDF.

This check covers free positioning. A drop that changes the order of source
units is tracked separately in issue #56.
