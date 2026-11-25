# PDF text editing in Texloom

## How Word’s editing behavior translates

Word’s documented file structure stores main-story paragraphs and text runs. Its desktop shortcuts describe moving and extending a selection by character, word, line, and paragraph. Together these support an interaction model with a logical position in the text and a visual position on the current page. The public specification describes the document structure and observable behavior. Word’s private implementation is outside this comparison. [WordprocessingML structure](https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document) · [Word keyboard shortcuts](https://support.microsoft.com/en-us/accessibility/word/keyboard-shortcuts-in-word)

| Interaction | Implication for Texloom |
| --- | --- |
| Click between letters | Project the caret into a known range of the current LaTeX source from the PDF already on screen. |
| Arrow and Shift-arrow movement | Keep one text selection across consecutive paragraphs. Wrapped rows share offsets in the paragraph text. |
| Backspace at a paragraph start | Delete the paragraph separator and reflow the joined text. At a soft wrap, Backspace removes the preceding logical character. |
| Tab and spaces | Keep focus inside the document and preserve the spacing in the LaTeX source. Word uses configurable paragraph tab stops. This Texloom implementation inserts a fixed four-space tab using standard LaTeX spacing commands. |
| Continue typing during layout | Keep the draft and its caret while Texloom verifies a compiled update. A result for an older draft must preserve newer keystrokes. |
| Undo | Record an accepted text change in the same history as source edits. |

[Word tab stops](https://support.microsoft.com/en-us/word/clear-or-remove-tab-stops-in-word)

## Cursor placement and compilation

The rendered PDF and current source provide a direct mapping for a plain document body on one page. Elsewhere, SyncTeX can help identify a supported run in the existing compile. Caret placement runs no TeX. On an actual edit, the background transaction compiles and checks the proposed source. An independently marked baseline confirms the original screen location before source and history change. This protects repeated text from edits to the wrong occurrence.

The editor updates the affected draft on screen and batches edits before verification. A verification build still typesets the complete project. Page breaks, labels, generated lists, counters, and package code can affect later output after a local edit. A fragment compile would need evidence that the downstream output stays valid. Texloom’s existing repeated-build work reuses compiler identification and narrows dependency-check costs while preserving full build verification. [Technical report source](../research/report/sections/compile-cost.tex)

A future incremental experiment should compare full and incremental PDFs and SyncTeX mappings on the same inputs. Include paragraph reflow, page breaks, tables, floats, cross-references, and bibliography changes. Measure click-to-caret, keystroke-to-visible-text, and time to verified publication separately. An unchanged command and dependency inventory can identify some safe reuse opportunities. It cannot establish that two different TeX documents produce equivalent later pages.

## Compositor as a direct comparison

Compositor advertises direct editing of typeset LaTeX, a source editor, standard LaTeX files, local operation, and visual equation editing on macOS and Windows. Its author describes a source-backed editing model and a typeset view handling keyboard input in a 2018 article. The 2026 release notes document incremental typesetting work, including avoidance of a full pass in one scrolling workflow. Those notes establish that Compositor does incremental work without specifying the coverage of every edit in the current product. [Compositor](https://compositorapp.com/) · [Author’s engineering article](https://compositorapp.com/blog/2018-09-29/Testing/) · [Incremental typesetting release notes](https://compositorapp.com/blog/2026-10-12/Compositor-Windows-092/)

Texloom’s comparison should focus on preserving existing project files, using the project’s chosen local compiler, and verifying the source changes made by a visual edit. Compositor’s published limitations list file flattening and a TeX Live 2018 package base. Test those differences with actual project fixtures before making compatibility or performance claims. A related-work comparison should include editing latency, source and project fidelity, and the scope of visual operations each product accepts. [Compositor limitations](https://compositorapp.com/#current-limitations)
