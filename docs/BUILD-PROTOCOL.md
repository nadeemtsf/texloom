# Compiler and mapping IPC

The compiler, build acknowledgement and source/PDF mapping boundary now uses strict TypeScript with declarations exported from the real Rust handler records. It extends TYPED-DOCUMENT-PROTOCOL.md. This is implementation hardening, not native gesture or release acceptance.

## Covered commands

| Command | Acceptance before use |
| --- | --- |
| compile_document | Complete result, captured canonical/candidate revision, selected engine, finite timing, bounded recipe passes, safe integer job identity, byte-valid PDF header, typed diagnostics and object inventory |
| verify_build / pin_displayed_build | Exact serialized Rust unit acknowledgement (null); backend failure remains a rejection |
| cancel_compilation / document_discard | Exact unit acknowledgement; local cancellation still fences late results even if its acknowledgement fails |
| export_document_build | Ordered current revision and exact unit acknowledgement; native export still checks the build and destination |
| pdf_from_source / source_from_pdf | Complete finite rectangle or an existing source file and valid line; source changes while awaiting navigation invalidate the result |
| resolve_text_selection | Exact build/revision, captured original source bytes, normalized UTF-16 fragment, supported unit metadata and same-page rectangle hints |

The existing flat camelCase command arguments are unchanged. The compiler-purpose enum accepts preview, visual and export; candidate compilation continues to select its actual native visual purpose. The native scheduler job ID is independent of the browser cancellation sequence and is not compared to that sequence.

## Source and output ownership

Prepared tokens retain an immutable copy of their candidate source map and returned revision in the ordered client. Compile results are checked against that candidate, while the command retains its canonical base revision. Discard, cancellation and accepted source changes invalidate the local token. Native one-shot layout approval remains necessary for commit; a checked compiler reply grants no authority.

The native compiler now serializes positioned blocks and text-object metadata through actual Rust structs instead of untyped JSON values. Nullable plain-text previews and empty measured blocks remain representable. Object ranges cannot split a surrogate pair or point outside the captured source. Failed compiler results cannot carry PDF bytes or visual objects; eligible builds cannot contain error diagnostics. Successful readable builds with unresolved diagnostics remain usable as previews.

Selection validation checks the original bytes including BOM and line endings separately from the editor projection. The content must equal the captured projected range. Selection metadata and SyncTeX rectangles remain hints: the existing zero-offset PDF probe, independent layout checks and commit gate still run afterward. Negative genuine rectangle coordinates are preserved, without invented padding or clipping.

Malformed compiler/mapping/verification replies leave source typing usable. They fail the pending visual transaction and preserve the accepted source/history/PDF. A cancellation fence takes priority over any late compiler result, including malformed results. Malformed successful source commits and Save replies retain the stricter document-client freeze from the earlier slice.

## Verification and remaining work

Rust-generated Serde fixtures exercise actual wire shapes. Runtime cases cover wrong revisions and engines, invalid PDF bytes, missing fields, duplicate objects, invalid UTF-16 offsets, forged unit acknowledgements, wrong-source selections and navigation racing with typing. Production-shell transaction tests confirm refusal before rendering/planning/commit and disposal of unpublished candidate PDFs. Compile-time negative contracts and generated-output freshness checks remain mandatory in npm test/build.

Exact local checks and build receipts are recorded in evidence/build-protocol-local.json after execution. Native mouse/typing/IME/signature and installed-artifact checks remain owner-deferred. Project/recovery inventory, source-transform requests and protected-text models remain outside this strict slice; unsupported encodings, broader recipes/adapters, fractional displacement and release gates remain open.
