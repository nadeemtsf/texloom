# Typed native source-draft transport

13 November 2026 continuation of ARCH-04/05 (#48/#49). This is a native
source-editor boundary, not generalized visual Delete, PDF target identity,
render approval, caret input or automatic Save.

14 November extension: admitted pre-mutation failures now have an explicit
cached refusal outcome. The original 13 November/75-assertion evidence remains
bound to its named source; the newer refusal diagnostic has a separate protocol.

## Authority and requests

`document_source_targets` derives literal paragraph candidates from canonical
source at the expected revision. Returned UTF-16 ranges and text are display
information. A later request supplies an opaque native-issued handle, not raw
source offsets or a caller-constructed core entity reference. Handles belong
to one open session and revision; Undo restoring the same bytes does not revive
them. Projection remains conservative and unsupported source remains opaque.

`document_source_intent` accepts a strict tagged literal Replace or paragraph
content Delete payload, the expected project/revision and a 32-character
lowercase hexadecimal operation ID. Native code resolves the private handle
and rederives the exact core plan. Plain replacement text is escaped with
ordinary LaTeX spellings; structural/control input refuses. The operation
changes no other file/profile and adds no Texloom support commands.

Changed drafts are protected through the existing recovery journal before
canonical in-memory publication, using a validated proposed shared-history
state. A successful application consumes the privately constructed core plan,
invalidates pending visual candidates and cancels obsolete compilation. It
does not replace the last-good PDF or write the original project files. A
no-op retains revision, selection, redo, pending candidates and running work.

## Acknowledgements and scope of idempotency

The immutable operation outcome records the original revisions/change status
and `render_verification: unresolved`. Every application/lookup reply includes
a separately fresh canonical snapshot. Retrying after Undo therefore returns
the old operation receipt alongside the current undone source, rather than
reapplying the edit or republishing its old snapshot.

`source_draft_refused` records `requested_revision`, `observed_revision` and a
finite reason for a valid, bounded request admitted to the receipt ledger:
changed revision, unavailable target, unsupported target, empty literal,
paragraph break, control character, oversized literal or rejected source guard.
It runs before canonical/recovery mutation, retains selection/redo/pending jobs,
and is cached just like an applied result. A refused request is not re-evaluated
after later source changes. Confirming an applied or refused outcome clears
that operation's client uncertainty and permits subsequent editing.

Transport-schema/ID failures, a missing session, payload-ID conflict, receipt
capacity and an already unresolved protection state remain outside this
admitted refusal contract. An exception or unknown lookup still does not prove
nonexecution, and never triggers automatic retry. The ledger's existing bounds
are unchanged; this is not an unlimited-session or durable-idempotency claim.

Same-session retries with exactly the same operation ID, base revision and
payload return the recorded outcome. Reusing an ID for a different request
refuses. `document_source_intent_status` resolves a lost acknowledgement without
performing a write. Receipts are session-local, with a 128-operation/4 MB payload
budget and no eviction; capacity refuses new work while old receipts remain
resolvable. Targets have separate 4096-handle/8 MB text budgets.

Session reopen/recovery deliberately drops these handles and operation receipts.
An `unknown` result means there is no receipt in this session. It cannot prove
that a previous process did not act, and must not trigger automatic retry with
a new handle or operation ID. Reconcile current/recovered source explicitly.
This is bounded session idempotency, not durable exactly-once execution.

A recovery-write error can follow partial filesystem writes. Native canonical
source remains unchanged and an explicit `protection_uncertain` outcome is
cached, including on identical retries. It is not a clean refusal asserting
that no proposed draft could later be recovered. The client must retain
uncertainty, stop dependent writes and reconcile the protected/current state.

## Evidence boundary

Generated Rust/TypeScript records and runtime validation describe the transport.
Headless service tests and the separately predeclared development diagnostic
exercise source authority, acknowledgement handling and recovery. They do not
exercise a running Tauri dispatch, packaged GUI, physical input or rendered
operation approval. Exact tested source/log identities and failures belong in
the dated validation receipt before any passing-result claim is made.

The report's source/render ambiguity result remains unchanged: equal complete
PDF output can hide a wrong source-occurrence deletion. ARCH-04 remains open
for broader intents/integration and ARCH-09/10 for rendered proof and visual
target identity. No final evaluation, release or publication follows from this
source-draft boundary.

## Headless cost follow-up, 14 November

The predeclared release source-draft diagnostic and reply-shape ablation are
recorded in `research/RESULTS-SOURCE-INTENT-COST-2026-11-14.md` and the dated
JSON receipt. Two typed reply modes share the same mutation/protection/ledger
path; the changed-file mode is benchmark-feature-only and not Tauri IPC. The
production full-snapshot contract is unchanged. Cached/history cases fall
back to a current full snapshot. Thirteen focused native controls passed.

The smaller reply removed project-sized serialization but native service
cost still grew with unrelated source bytes. Profile project-sized planning/
checkpoint/preflight work before changing it; preserve strict protection and
exact-source semantics. This is not physical typing latency or a final study.
