# Single-file source-draft preparation

Typed paragraph edits already rederive one replacement in an existing file.
They now use the same single-file preparation and history implementation as
source-editor changes. A caller can explicitly materialize the full candidate
from the matching session for reference comparisons. This requires the original
owner and base revision. Native preflight uses the single-file prepare and
commit methods on a separate proposed session before writing recovery.

All 12 core intent tests and 13 native draft tests passed, including refusal,
retry, operation receipts, recovery, revision ownership and exact Undo. The
same release-mode three-arm diagnostic retained all 315 rows. Exact sources,
single-step Undo, protection, original-file checks and independent summary
recalculation passed. The source and harness identities are recorded in
`docs/evidence/source-intent-single-file-2026-11-14.json`.

| Project MiB | Source delta | Typed changed-file | Typed full snapshot |
| ---: | ---: | ---: | ---: |
| 1 | 3.541 | 3.727 | 7.183 |
| 8 | 7.626 | 9.839 | 22.813 |
| 32 | 3.897 | 24.741 | 102.965 |

Each cell gives the combined service and JSON encoding p95 in milliseconds
from 30 repeated measurements. At 32 MiB, the changed-file arm's service
p95 was 24.657 ms, compared with 41.718 ms in the earlier trusted-fork run.
Full-snapshot combined p95 rose from 79.426 to 102.965 ms while encoding p95
rose from 30.781 to 60.867 ms. The workload and timing boundaries match
the earlier run. Background load and filesystem caches were uncontrolled,
so these observations cannot isolate a causal speedup from run variation.
The changed-file reply remains an experiment. The production snapshot still
contains all source files, and the journal can use the complete proposed
state for rotation. Physical input latency and final evaluation remain open.
