# Native draft refusal development protocol

Prepared 14 November 2026 before the refusal extension is implemented or run.
Baseline: `76b2228a71f6eeeaba1dd0ffcc11eca894b21327`; prior v1 transport
diagnostic and its 75 assertions remain separate, unchanged artifacts.

## Question and proposed contract

Can a native pre-mutation refusal be resolved without treating it as uncertain
execution, while lost acknowledgements and partial recovery writes still block
unsafe retries? This is source-draft transport development, not PDF selection,
rendered approval, a running Tauri/WebView2 experiment or a final study.

For a valid, bounded request admitted to the session receipt ledger, return and
cache `source_draft_refused` with the requested/observed revisions and a finite
reason: revision changed, target unavailable, unsupported target, empty literal,
paragraph break, control character, literal too large or source guard rejected.
Refusal occurs before canonical source/history or recovery mutation. It must
preserve selection, redo, pending candidates, current jobs and original files.

An identical retry/lookup returns the immutable refusal and a separately fresh
canonical snapshot; it must not re-evaluate the old request after later edits.
A confirmed applied/refused outcome resolves that operation's client freeze.
Unknown lookup and uncertain recovery protection do not. Invalid transport
schema/IDs, missing sessions, payload-ID conflicts and receipt-capacity errors
are outside this admitted receipt contract and remain conservative failures.
No receipt survives session reopen; there is no automatic retry across it.

## Pre-fix reproduction and engineering controls

Before changing the endpoint, add a native test that expects a stable refusal
receipt for a paragraph-break request and a browser-client test that expects
continued editing after a confirmed refusal. Keep their failing logs. These
exercise observable endpoint/client behavior rather than a private helper.

Then check exact same-source/history/selection/redo behavior on refusals,
status lookup after a deliberately discarded reply, duplicate refusal after a
later edit/Undo, conflicting payload, stale/foreign/reopened handles, and the
existing uncertainty/capacity controls. Strict generated Rust/TypeScript and
Serde/runtime negative controls must agree. A malformed claimed refusal cannot
clear a lost-acknowledgement freeze.

## Independent source diagnostic

Use three explicitly reused English/Arabic/Hebrew source conditions, with two
equal paragraphs, BOM/CRLF and a separate unchanged file. Construct full
expected source maps from independently declared segments and explicit escaping,
without planner offsets or returned source. No TeX compilation/fonts are needed;
the outcome concerns source bytes, not shaping, bidi caret or rendered language.

Submit four invalid literal classes, require the declared typed reason and exact
unchanged canonical state, discard one acknowledgement and resolve via lookup,
then apply a valid second-occurrence replacement. Replay an earlier refusal
after that edit and require the old outcome plus current source. Check stale
revision, unknown/foreign/reopened targets, unchanged originals and exact
Undo/Redo/recovery separately. Capacity and injected journal failure may remain
service-test-only conditions and must be identified as such.

Capture protocol/runner/source/executable hashes before execution, refuse an
existing output directory, retain requests/replies and every failed attempt,
and check source stability afterward. Record assertion counts within conditions;
do not manufacture independent user-request denominators or accuracy rates.
Tuned/reused development cases are ineligible as an untouched final set.

## Research writing boundary

Use the new diagnostic only as evidence of admitted source-request outcomes.
The prior wrong-source/same-PDF finding and missing final visual selection path
remain unresolved by this extension. Explain the established call-ID/failure
distinction with the inspected Birrell/Nelson RPC primary paper; do not claim
novel distributed-systems semantics or inherit that protocol's guarantees.
