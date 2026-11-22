# Direct LaTeX editing on Windows

Use the branch's Windows development build and a disposable copy of the
example. An existing project can still contain its own style files.

1. With the sample compiled, choose **+ Paragraph**. Enter a single line such
   as “Bread & butter.” Check that it appears at the end of the PDF and that
   the source contains the escaped ampersand. Check the file list for new
   style files.
2. Select the new text in the PDF and double-click its frame. Change the
   wording, then check that the existing LaTeX source and PDF both show the
   change. Repeat with a formatted line or short paragraph and check the
   formatting around the changed words.
3. Place the source cursor on a typeset line, then choose **Show in PDF**.
   Check that the corresponding area comes into view and no source changes.
4. Undo and redo, checking the source and the updated PDF. Save the
   disposable project and reopen it. The saved folder should build from
   its ordinary LaTeX files with the same engine and declared dependencies.

Dragging a selection uses the existing movement adapter. Check the
paragraph and edit steps before dragging anything in this fixture.
