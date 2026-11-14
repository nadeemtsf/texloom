# Source-draft service cost: development evidence, 14 November 2026

Status: two completed headless development runs, not a frozen performance study.
Protocol: `SOURCE-INTENT-COST-PLAN.md`; receipt:
`../docs/evidence/source-intent-cost-2026-11-14.json`.

The initial two-arm run retained 210 raw rows (180 measured, 30 warmup).
Its inputs and executable were preserved before the reply-shape amendment.
The amended three-arm run retained 315 rows (270 measured, 45 warmup), nine
groups of five warmups and 30 measured replacement/Undo trials. Do not pool
the runs or count repeated trials as independent documents. No planned trial
failed or was dropped. Independent Python calculations matched every native
summary, and the recorded input hashes were unchanged during each run.

Each synthetic project has a 16 KiB BOM/CRLF main file and unrelated,
unreferenced comment-only auxiliary files, totalling exactly 1, 8 or 32 MiB.
All arms replace the second equal paragraph with the same independently
declared escaped source. Every trial checked the complete canonical source,
one-step Undo and protected acknowledgements; original files remained exact.
No TeX, app GUI, Tauri dispatch, frontend publication or Save was exercised.

The release example runs seeded interleaved arms on equivalent separate
projects. Size order is blocked, background load and filesystem cache are
uncontrolled. Machine: Windows 11 Pro build 26300; Intel i7-11370H; RAM
16,856,203,264 bytes; existing Balanced power scheme. Toolchain: recorded
rustc 1.99.1. No system settings or user workloads were changed.

## Amended run results

Times are milliseconds, nearest-rank p95 from 30 measured trials per row.
Response bytes are medians. Combined p95 is computed from each observed
whole interval, not by adding component p95s.

| Project MiB | Arm | Native service + reply | JSON encoding | Combined | Response bytes |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | Existing source edit, delta | 2.438 | 0.033 | 2.456 | 16,666 |
| 1 | Typed, experimental changed-file reply | 4.633 | 0.037 | 4.675 | 16,878 |
| 1 | Typed, production full snapshot | 4.701 | 0.731 | 5.325 | 1,049,026 |
| 8 | Existing source edit, delta | 5.316 | 0.058 | 5.340 | 16,666 |
| 8 | Typed, experimental changed-file reply | 47.167 | 0.085 | 47.252 | 16,878 |
| 8 | Typed, production full snapshot | 54.677 | 12.915 | 66.705 | 8,389,191 |
| 32 | Existing source edit, delta | 4.012 | 0.022 | 4.022 | 16,666 |
| 32 | Typed, experimental changed-file reply | 139.286 | 0.066 | 139.328 | 16,878 |
| 32 | Typed, production full snapshot | 171.722 | 42.415 | 217.754 | 33,555,471 |

The two typed modes share the same planner, recovery preflight/write,
canonical mutation, history and operation ledger. Only reply construction
and encoding differ. The experimental variant is gated by the benchmark
feature and has no Tauri command. Cached/history cases unable to form a
safe delta fall back to the current full snapshot. Thirteen focused native
controls passed, including cached retry after Undo and metadata-only refusal.
The installed app retains its full-snapshot protocol.

A smaller reply removes the project-sized payload and most observed encoding
cost, but the changed-file typed service still scales with unrelated source
bytes. Encoding alone therefore does not explain the remaining measured cost.
The code still prepares/checkpoints project state and preflights the future
session. These are candidates for subsequent component profiling, not costs
isolated by this experiment. Differences between service p95s cannot be read
as an exact causal effect under uncontrolled load. The source-edit arm has
different planning/protection semantics; it is context, not an interchangeable
safe substitute.

The first run's typed full-snapshot combined p95 was 4.941/36.400/115.345 ms
at 1/8/32 MiB, versus 5.325/66.705/217.754 ms in the amended run. The substantial
variation is retained rather than selecting the faster run or claiming a
reproducible speedup. Neither run supplies physical input latency, tracing
overhead, a rare-tail guarantee, a memory profile, an accuracy rate or a
general workload distribution. Target enumeration is separately recorded;
the main interval excludes it, assertions, reply destruction, Undo and file
checks. Rust `Instant` is the timer; its maintained documentation describes
clock/platform limitations, not a guaranteed steady physical clock.

## Next engineering question

Profile project-sized planning/checkpoint/preflight work with the same
protection and exact-source invariants before choosing an optimization.
Do not weaken recovery or rendered approval to improve timings. A production
delta protocol would separately need strict generated/runtime validation,
base-revision admission, immutable receipts and current-state fallback.
Visual typing, PDF proof and owner-run input remain later work.
