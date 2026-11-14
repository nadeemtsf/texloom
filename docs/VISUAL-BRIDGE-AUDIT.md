# Visual bridge audit — 3 November 2026

Audited the actual Rust/Tauri desktop source at recovery checkpoint d67c5af37717091e64a973964c3322b5b55e7ad2, then corrected two concrete gaps. Performance-plan changes were paused for this review. The frontend is JavaScript, Monaco and PDF.js; React components and strict TypeScript bindings do not currently exist. Generated/strict IPC typing remains an earlier-phase implementation gap.

## 1. Frontend IPC and drag frequency

> Can you show me the exact TypeScript code for the drag handlers in our React components? I need to see if we are calling `invoke('document_prepare')` on every single mouse movement, or if we are safely debouncing it and tying it to `onMouseUp` or `onDragEnd`.

Actual source: [canvas-controller.js](../web/core/canvas-controller.js), `drag`, and [app.js](../web/app.js), `moveText`, `textOperation` and `commitSources`. These are Pointer Events handlers in plain JavaScript.

The pointer-move body is:

```js
const move=e=>{
  const snap=v=>e.altKey?Math.round(v*100)/100:Math.round(v/2)*2;
  if(Math.hypot(e.clientX-origin.x,e.clientY-origin.y)<3&&!delta)return;
  delta={x:snap((e.clientX-origin.x)/factor),y:snap((e.clientY-origin.y)/factor)};
  frame.style.transform=`translate(${delta.x*this.view.scale}px,${delta.y*this.view.scale}px)`;
  frame.classList.add('dragging');
};
```

It performs no IPC and no source mutation. The actual release handler unbinds the pointer listeners, calls `this.move(selection, delta.x, delta.y)` once for a nonzero delta, and clears the preview in `finally`. The call path is:

`pointerup → moveText → textOperation → plan_text_object → commitSources → DocumentClient.prepare → document_prepare`.

The existing positioned-block drag path also changes CSS during pointer movement and calls `patchBlock` on pointer release. Source auto-compilation has a separate 800 ms typing debounce. Release starts one logical candidate transaction; verification can require multiple TeX passes and private zero-offset probes. Selection resolution itself can also compile a private probe. This is not a promise that every click produces zero builds.

The normal drag preview currently moves an outline. PDF ink stays at the accepted position until a verified candidate is published. The real controller tests exercise twenty pointer movements with zero operations, then one operation on release, plus jitter, Escape, cancellation, capture loss and zoom conversion. Those are DOM-boundary tests, not native pointer acceptance.

## 2. SyncTeX implementation and the discovered fallback gap

> What is currently inside the `backend.inverse` and `backend.forward` functions in `src-tauri/src/service.rs`? Are they just placeholders, or are they fully implemented using a SyncTeX parser crate to read `.synctex.gz` files?

Both are real CLI integrations in [service.rs](../src-tauri/src/service.rs). They locate the compiler's sibling `synctex.exe`/`synctex`, with an installed-program fallback. They query the immutable build's PDF:

```rust
command
    .current_dir(&build.directory)
    .arg("edit")
    .arg("-o")
    .arg(format!("{page}:{x}:{y}:{}", pdf.display()));
```

```rust
command
    .current_dir(&build.directory)
    .arg("view")
    .arg("-i")
    .arg(format!("{line}:0:{}", build.directory.join(file).display()))
    .arg("-o")
    .arg(build.directory.join(".out/preview.pdf"));
```

The installed SyncTeX executable reads the generated mapping; Rust parses its response. There is no Rust SyncTeX-parser crate in this implementation.

The audit found real problems: `forward` substituted 62/112/300/12 when geometry was absent, enlarged small boxes, and clipped negative coordinates. Both queries ignored the process success boolean. Inverse parsing could combine the first Input and Line from separate records.

The correction moves query validation/parsing to [synctex.rs](../src-tauri/src/service/synctex.rs):

- Refuse failed, timed-out, cancelled or truncated output, including valid-looking partial records from a failed process.
- Require complete, unambiguous source/line or page/h/v/W/H fields with valid numeric values. No invented coordinates, minimum-width inflation or clipping.
- Require the current canonical revision and an integrity-checked, sealed `.out/preview.synctex.gz`, rechecked before and after the query.
- Check the query executable's fingerprint before/after and retain project-relative path checks.
- Use an independent bounded read scope. A private probe can supersede the old compiler lease while the accepted PDF remains a valid read artifact; the compiler token is not reused as its read lifetime.

SyncTeX remains a hint for selecting a candidate source unit. Normal canvas editing also requires supported source structure, unique marked output, zero-offset equivalence, native operation approval and the relevant PDF comparison. A mapping answer alone never authorizes an edit.

## 3. Compiler flags

> In the `compile_document` function, how exactly are we executing the LaTeX compiler process? Are we passing the `-synctex=1` flag so that the engine generates the mapping file we need for the visual editor?

Yes. `compile_document` delegates to the build service. The argument vector in [engine.rs](../crates/texloom-build/src/engine.rs) already contains:

```rust
let mut args: Vec<String> = [
    "-interaction=nonstopmode",
    "-halt-on-error",
    "-file-line-error",
    "-synctex=1",
    "-recorder",
    "-no-shell-escape",
    "-jobname=preview",
    "-output-directory=.out",
]
.map(str::to_owned)
.into();
```

MiKTeX adds `--disable-installer`; the main filename is a separate argument. Rust uses `process::run(CommandSpec, cancellation)`, copied build inputs, bounded logs/time/memory and process-tree supervision. It does not construct a shell command from the source filename. No missing SyncTeX flag needed adding in this audit. The direct-engine recipe still does not orchestrate BibTeX/Biber.

## 4. Failure and preview rollback

> If a user drags an element and the `document_prepare` backend transaction fails or we call `document_discard`, how does our React state handle the rejection? Can you show me the code that makes the visual element snap back to its original position?

The canonical sources and accepted PDF are kept until validation/commit. In [app.js](../web/app.js), the relevant sequence is:

```js
nativeCandidate=await client.prepare(sources,state.result.build_id,operation);
const candidate=await stageCandidate({sources,revision:nativeCandidate.revision,identity:expectedIdentity,getIdentity:()=>sessionIdentity(state),
  compile:()=>client.compile(nativeCandidate.token),prepare:prepareResult,policy,previousPageCount:state.view.pageCount,requireVerified:true,verify:result=>client.verify(result.build_id)});
prepared=candidate.prepared;
if(sessionIdentity(state)!==expectedIdentity)throw new StaleCandidate();
state.mutating=true;editorReadOnly();
await client.commit(nativeCandidate.token,candidate.result.build_id);
state.file=file;state.lastHistory=0;syncEditor();publishPrepared(prepared,candidate.result);prepared=null;
```

Failure returns false, keeps accepted source/PDF/history, destroys an unpublished prepared PDF and discards the native pending token. Normal text dragging clears `frame.style.transform` in `finally`. `document_discard` removes only an uncommitted pending candidate; undo is a separate history operation after a successful commit.

The audit caught a second real gap in the legacy positioned-block handler: it previewed left/top/width, but did not reliably reset them after a rejected release. It now captures `originalStyle`, awaits the transaction and restores/repaints accepted bounds in `finally`. Pointer cancellation, lost capture and the shared Escape cancellation hook restore those bounds without creating a transaction. Tests cover native-prepare rejection, compiler rejection and resize cancellation while retaining original sources/PDF/history.

## 5. Source mutation strategy

> When the visual drag finishes and we resolve the coordinates to a specific line of LaTeX, how are we actually modifying the `.tex` file in Rust? Are we using regex and string replacement, or are we converting the string into an Abstract Syntax Tree (AST) to safely mutate the node?

The normal text adapter uses constrained structural scanning and guarded byte-range patches. It does not build a full TeX AST and does not regenerate the whole document through a pretty-printer. [objects.rs](../crates/texloom-core/src/objects.rs) skips comments/literal regions, reads balanced arguments, rejects duplicate/nested IDs and grants capabilities only for supported fragments.

For an existing object, movement plans changes only to its displacement arguments:

```rust
if let Some(value) = dx { patches.push((object.arguments[1].clone(), number(value, false)?)); }
if let Some(value) = dy { patches.push((object.arguments[2].clone(), number(value, false)?)); }
```

```rust
patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
let mut updated = source.to_string();
for (range, text) in patches { updated.replace_range(range, &text); }
text_objects(&updated)?; Ok(updated)
```

Unwrapped supported text is enclosed in the portable TexloomObject adapter and proved against the original rendering. Rust derives minimal `FileChange`/`Replacement` patches, checks expected original text, UTF-8 boundaries, session ownership and revisions, and records exact inverses. The native visual gate recomputes the declared operation against canonical source; frontend offsets or candidate strings alone do not grant commit authority.

A successful drag changes canonical buffers/history and their private recovery journal. Original `.tex` files are written only by explicit guarded Save. Normal movement retains the source unit's flow contribution. Legacy StudioBlock positioning has a separate compatibility policy; it is not the normal-text full-raster contract.

A lossless syntax representation can broaden structural coverage later. An AST alone does not prove macro-expansion identity or PDF geometry/appearance, so compiled-result verification remains necessary. Unknown/unsupported cases keep a source-editing fallback.

## Validation and open gates

Local audit validation: 70 core, 41 native default and 118 frontend checks passed; frontend and Windows x64 release builds passed. The five new native tests include real nonzero/truncated child processes and current/sealed trace identity. The five new frontend cases cover legacy preview rejection/cancellation. Exact source/log/executable receipts: evidence/visual-bridge-audit-local.json. The prior recovery checkpoint d67c5af passed Windows 41607398490, proof 41607398601 and compile corpus 41607398503; this audit needs its own remote run.

No native mouse/IME/app-control test ran. Strict/generated IPC typing, full syntax/provenance work, fractional movement, remaining phase implementation and final native/release acceptance are still open. The corrected executable was not launched. No new portable ZIP/installer or all-phase completion is claimed.
