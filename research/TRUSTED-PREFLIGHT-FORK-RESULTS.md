# Trusted live-session preflight: development note, 14 November 2026

The native draft path previously reconstructed its future state with
`Session::from_checkpoint(slot.session.checkpoint())`. That API validates
untrusted recovery data and replays retained forward/inverse history. The
input here is already a live Session with private validated state. The new
`fork_for_preflight` copies its source, profile, revision, history, cursor,
selection and typing groups, with a fresh owner. It accepts no raw data.
Normal candidate prepare/commit still runs on the fork, and durable recovery
protection still precedes canonical publication. Serialized recovery continues
through the unchanged bounded codec and complete checkpoint replay.

Three focused core controls passed: mixed file/profile/source history and
redo stay exact; edits remain independent; Prepared tokens and entity references
are foreign in both directions; no-ops retain redo and declared typing groups;
ordinary expected-source guards still reject invalid patches. All 13 native
source-draft controls passed, retaining refusal, uncertainty, retries, recovery
and original-file boundaries. The full installed pre-commit gate and hosted
checks are recorded separately for the resulting commit.

## Separately retained post-change cost run

The follow-up protocol was saved before implementation or execution. The same
release three-arm harness ran in a fresh directory: nine groups, five warmups
and 30 measured trials each, 315 raw rows. No failed or dropped trial. Exact
full source, one-step Undo, protected acknowledgements and unchanged originals
passed. Independent Python summaries matched, with source hashes stable during
the run. Environment and entry/timing exclusions match the recorded cost
protocol; compiler, binary and source hashes are separately captured.

Times below are nearest-rank p95 milliseconds; reply bytes are medians.
Combined percentiles come from whole observed intervals, not component sums.

| MiB | Arm | Native | JSON | Combined | Reply bytes |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | source_edit_delta | 2.387 | 0.017 | 2.397 | 16,666 |
| 1 | typed_preprotected_changed_file | 4.712 | 0.036 | 4.730 | 16,878 |
| 1 | typed_preprotected_full_snapshot | 3.813 | 0.897 | 4.441 | 1,049,026 |
| 8 | source_edit_delta | 2.417 | 0.019 | 2.427 | 16,666 |
| 8 | typed_preprotected_changed_file | 11.702 | 0.057 | 11.745 | 16,878 |
| 8 | typed_preprotected_full_snapshot | 13.374 | 7.075 | 20.456 | 8,389,191 |
| 32 | source_edit_delta | 3.011 | 0.019 | 3.026 | 16,666 |
| 32 | typed_preprotected_changed_file | 41.718 | 0.069 | 41.779 | 16,878 |
| 32 | typed_preprotected_full_snapshot | 48.905 | 30.781 | 79.426 | 33,555,471 |

At32MiB the post-change full-reply combined p95 is79.426ms and benchmark-only
changed-file p95 is41.780ms. Prior amended observations were217.754/139.328ms,
while the earlier full-reply observation was115.345ms. These remain distinct
runs under uncontrolled background load and filesystem caches, not an exact
causal speedup estimate. The changed code removes a concrete recovery-replay
path from trusted live-state preflight; it does not isolate every remaining
cost or establish typing responsiveness. Project-sized copying/planning and
full-snapshot encoding remain. The source-edit arm has a different protection
and planning contract and is not an interchangeable safe substitute.

The code still performs strict candidate guards, recovery writes and canonical
transaction checks. It grants no rendered approval or visual editing command.
No TeX, GUI, input, WebView publication, Save, memory/overhead measurement or
final evaluation ran. Thirty correlated repeats do not estimate a population
or rare tail. See `docs/evidence/trusted-preflight-fork-2026-11-14.json` for
exact identities and raw paths. Immutable compiled report v15 describes the
earlier named runs; this later note is material for the next report revision.
