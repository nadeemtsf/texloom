# Headless source-draft service cost development protocol

Prepared 14 November before the new harness or measurement. This is a paired source-equivalent development diagnostic, not a final performance study, physical input-to-display measurement, usability result, or interchangeable-contract comparison.

Question: how do target discovery, native service/recovery/reply construction, UTF-8 JSON encoding and response size scale with unrelated project source bytes for the new typed draft versus the existing source-edit delta endpoint?

Use one release-mode executable, the actual Backend endpoints, a fixed 16KiB BOM/CRLF main document containing two equal plain paragraphs, and unrelated comment-only .tex files. Declare exact total project sizes1MiB/8MiB/32MiB; auxiliary files are at most1MiB each. They are deliberately unreferenced to isolate project-size effects, not representative included-file compilation. No TeX, fonts, app, window or network service runs.

For each size open two separate, equivalent synthetic projects/caches, one per endpoint. Five warmup and30measured replacement/Undo trials per arm. Use unique operation IDs below the128receipt limit. Each trial replaces the second occurrence with an independently declared escaped source string; full expected maps are assembled from declared segments. No planner offsets/returned source construct gold. For the source-editor arm derive the known editor offset independently from BOM/CRLF-normalized declared prefix and UTF-16 length. Require exactly one source/history step, protected acknowledgement, exact full-source Undo and unchanged original files. Failed/refused/uncertain trials are recorded as such and never zero-filled or silently dropped.

Interleave endpoint arms with deterministic seeded paired ordering; record actual ordering and blocked size order. This controls neither background load nor OS file-cache state. Record OS/CPU/RAM/power scheme, compiler and exact source/protocol/runner/binary fingerprints before execution. Existing user workloads are not stopped or reconfigured.

Measure target enumeration separately for typed drafts. The main native endpoint interval starts immediately before the actual service call and ends after reply construction; a second interval covers serde_json UTF-8 encoding. The combined endpoint ends before source assertions, reply/payload destruction, Undo, original-file checks or UI transport. Warmups, status, raw samples, JSON response bytes, sample counts, median, nearest-rank p95 and max remain available. Thirty correlated repeats are not30independent documents; p95 is descriptive of this run, without population confidence or a rare-error guarantee.

Contracts differ: typed intent includes target rederivation, strict recovery protection before canonical publication and a full-source snapshot reply; source editing uses its current changed-file path and after-commit recovery flow. Equal source outcomes do not make them equivalent alternatives. Component timing alone cannot isolate every cost or establish that weaker protection is acceptable. Do not weaken safety or infer typing responsiveness. Compare observed service/encoding costs only under these declared bounds.

Use a fresh directory strictly under repository artifacts, preserve all failures and raw logs. No .text compilation, no new input languages/fonts/packages, no owner projects. Final evaluation remains not_frozen.

## Pre-execution amendment for reply-shape ablation

After the first two-arm run, before implementing/running this ablation: that
run met all integrity checks. At32MiB it observed typed full-response p95 about
115ms and a33.6MB response, versus3.3ms/16.7KB for the existing source delta.
These are not equivalent contracts or independent document samples. Preserve
the original210rows, pre-run hashes, helper/harness/protocol text and executable.

Add a benchmark-only changed-file reply variant of the typed endpoint. Both
typed reply modes must use the same native planner, recovery preflight/write,
canonical mutation and operation ledger. Only reply construction/encoding may
differ. No new Tauri/UI command is enabled. Duplicate/stale cases that cannot
safely form a delta must fall back to the full current snapshot, and explicit
native controls must retain exact outcome/Undo/uncertainty behavior.

Run all three arms in one release executable, on separately equivalent
projects, with the same5warmups/30measured trials and1/8/32MiB sizes. Seeded
triad ordering replaces paired ordering; retain actual arm order. Expected raw
rows315, nine summaries with30measured repetitions each. The original two-arm
run is historical development evidence, not an independent replication or a
poolable baseline. Re-check source/Undo/protection/originals and independently
derive summaries. Do not sum component p95s or infer UI responsiveness from
native/encoding endpoints. Report whether reply size/encoding is sufficient
to explain the remaining cost; do not weaken protection to improve numbers.
