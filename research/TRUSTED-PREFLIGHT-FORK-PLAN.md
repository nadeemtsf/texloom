# Trusted live-session preflight continuation

Prepared 14 November, before any fork implementation or changed-code measurement.
The source-cost checkpoint's full gate must finish before mutating its staged tree.

The measured typed transaction currently creates its future state through
`Session::from_checkpoint(slot.session.checkpoint())`. That path clones state,
validates an untrusted checkpoint and replays retained forward/inverse history.
The source is already a validated live Session with private fields. Explore an
explicit independent-owner `Session::fork_for_preflight()` that copies those
fields directly. This is not Clone: Prepared tokens and entity references from
the original must remain foreign. Serialized recovery must continue through
the existing bounded codec and full from_checkpoint replay; never add a raw
data adoption API or skip normal candidate prepare/commit guards.

Required before native adoption: exact source/profile/revision/selection and
undo/redo preservation over a mixed history including redo and file/profile
changes; independent mutation/Undo/Redo; original/fork Prepared and entity
references rejected across owners; no-op preservation; existing invalid
checkpoint/inverse/accounting tests continue to pass. Native typed preflight
must still prepare and commit the same minimal patch on its independent future
state, protect recovery before canonical publication, and retain refusal,
uncertainty, cached retry and original-file controls.

Keep the completed cost runs and immutable report v15 unchanged. Record the
new code identity and tests separately. If a post-change cost run is made,
use the same release three-arm harness, sizes, five warmups/30 measured trials
and independent analysis, in a fresh output directory. Keep all315rows,
failed trials and all timing boundaries. Background load is still uncontrolled;
cross-run changes cannot establish an exact causal speedup or physical latency.
No new fonts/packages/languages, TeX, .text files, GUI/input or Save. A full
installed27-step gate, push and hosted checks are required for a new commit.
Stop new work when five-hour allowance is30%remaining; preserve unfinished work.
