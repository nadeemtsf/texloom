# Native source-intent transport development protocol

Prepared 13 November 2026 before the new endpoint diagnostic is executed.
Status: development protocol; no final evaluation freeze or held-out cases.

## Question and entry boundary

Does the actual native document service preserve exact source-occurrence and
shared-history authority when a typed paragraph Replace/Delete request crosses
the transport boundary, including a lost acknowledgement and a later retry?
This extends the earlier read-only occurrence diagnostic. It does not test PDF
selection, rendered approval, a running WebView2, physical input or Save.

The implementation baseline is `4cfe4616c98e85a879e5a60296f5d3a8c09405dc`
on `feature/remaining-desktop-features`. The continuation branch starts at those
same bytes. Record the tested source files, protocol, runner and executable
hashes before running. Use a new ignored artifact directory; retain every
failed attempt and amendment separately. Do not install packages or compile
`.text` files. No TeX compilation is required for this source/draft diagnostic.

## Independent source expectation

Build the synthetic source from separately declared prefix, first paragraph,
separator, second equal paragraph and suffix. Expected replacement and deletion
are assembled from those declared segments, without reading planner offsets,
app escaping output or the returned source to construct the expectation.
Include BOM/CRLF, English/Arabic/Hebrew literals, a separate unchanged project
file and a replacement containing an ampersand and percent with explicitly
declared expected LaTeX escaping. Do not infer exact source from rendered text.

Enumerate targets through the real native service. Select the second occurrence
in declared source order; a token is source-model identity, not proof that a
visible PDF selection intended it. All requests use native-issued handles and
expected session/revision metadata as required by the implemented schema.

## Declared conditions

1. Replace the second equal paragraph; compare the entire source map/profile to
   the independent expected state and verify one exact Undo/Redo history step.
2. Delete only the second paragraph's content, retaining separators and all
   unrelated bytes; verify exact Undo/Redo.
3. Re-submit the same operation after an acknowledgement is discarded. Resolve
   by lookup and retry without another mutation, revision or history entry.
4. Retry after Undo: return the original immutable operation receipt and fresh
   current state, without reapplying or publishing an old source snapshot.
5. Reuse an operation ID with a different payload: refuse without mutation.
6. Submit stale, foreign-session and reopened-session handles: refuse without
   mutation, including when Undo restored identical source bytes.
7. Submit invalid/unknown tagged payload fields and unsupported literal input:
   reject at the appropriate schema or native planner boundary.
8. A no-op must preserve revision, selection, redo and unrelated pending work.
9. Operation capacity must refuse new work without evicting old receipts or
   allowing a destructive duplicate. Already recorded requests remain resolvable.
10. A changed draft must enter protected recovery before canonical publication.
    Inject protection failure where feasible and require no live mutation;
    a partial/uncertain recovery write must retain an explicit unresolved
    outcome rather than claiming that no draft could later be recovered.
    reopen a protected draft and verify exact source/history. Session-local
    operation lookup after reopen remains unknown, so no automatic retry follows.

Refusal is correct only when the declared unsupported/stale/conflicting
condition is present. A missing implementation, runner failure or unavailable
setup remains a separate failure/unexecuted condition. Record conditions that
are covered only by unit/service tests separately from the diagnostic runner.

## Outcomes and reporting

For each diagnostic condition retain complete request/receipt, before/after
revision, independently expected and observed source hashes, exact comparison,
Undo/Redo where relevant, protection status and failure/error text. Synthetic
handles and operation IDs need not be stable across runs. Do not include user
projects, personal correspondence, credentials or external paths in the bundle.

Report engineering test totals separately from diagnostic cases, with their
actual entry layers. Tests that call the native service do not exercise Tauri
IPC dispatch or a packaged UI. JSON round trips and generated type checks are
additional transport checks, not observations of physical input. No proportions
from these tuned development cases estimate app accuracy or final coverage.

A source-draft receipt leaves rendered-content proof unresolved. It authorizes
neither a visual transaction nor original-file Save. Last-good PDF preservation,
build cancellation and recovery each require their own executed evidence.
The earlier equal-paragraph render result remains relevant: matching complete
PDF pixels cannot distinguish all wrong source-deletion origins.
