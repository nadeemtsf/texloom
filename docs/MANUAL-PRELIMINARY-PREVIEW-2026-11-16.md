# Owner check: preliminary PDF view

Historical checklist for the candidate at `0bf8f8c`. On 18 November 2026 the
owner reported that the candidate worked well and chose to defer the early PDF
view. The current desktop configuration waits for the final build. The steps
below apply to builds with the experimental view enabled. Use only the supplied
synthetic project or another disposable project copy.

The supplied `Texloom-preliminary-preview-synthetic-project-20261116` folder
has `main.tex` and a LuaLaTeX project profile. Its second pass deliberately waits
eight seconds to make the temporary phase observable. No shell escape is used.

- [ ] Open `main.tex` in the candidate and Compile. The early-preview/checking
  label appears during the wait. It shows an image without selection/edit
  handles. The final PDF replaces it and resolves the section reference.
- [ ] Compile again and Cancel during the temporary phase. The earlier PDF
  returns, the source remains available, and no delayed preview flashes back.
- [ ] Change `\PreviewFailurefalse` to `\PreviewFailuretrue` and Compile.
  The later pass deliberately fails. The earlier PDF remains readable and the
  edited source is retained. Revert the flag and confirm normal compilation.
- [ ] During the wait, type newer source, resize/scroll, dismiss with Show
  earlier PDF, or open another synthetic project. Confirm the latest source
  and project win; focus and scrolling remain usable.
- [ ] After successful catch-up, repeat the previously accepted selection,
  movement, Undo/Redo, Save/reopen and PDF text-copy workflow on a disposable
  supported example. Confirm the temporary layer cannot linger over the final
  page or grant edits/export while compilation is active.

Report the candidate filename and each pass/failure with what was observed.
Automated/headless results do not mark these boxes complete. This is internal
development acceptance, without a public release or installation claim.
