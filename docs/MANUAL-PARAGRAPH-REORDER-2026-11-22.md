# Windows paragraph reorder check

1. Open and compile the disposable Article. Select only the first paragraph
   in the PDF. It wraps over several lines. Drag the frame to just below
   the second paragraph until **Reorder here** appears, then release it.
2. Verify that the whole first paragraph follows the second in the PDF
   and in the source. The third paragraph and the blank separators should
   stay in place. The project files should still show only main.tex.
3. Undo and redo. Select the moved paragraph again and use **Move before**
   in the inspector, or press Alt+Up while its frame is focused. It should
   go back above its previous neighbor.
4. Try a free drag away from the drop line. For single typeset lines it
   should adjust the position. Hold Alt to bypass the reorder cue.

This pass covers adjacent plain paragraphs on one page in the main file.
The same paragraph, a nonadjacent destination, or a source structure such
as a section break must refuse a reorder.
