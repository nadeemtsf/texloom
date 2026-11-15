# Binary compile-reply development diagnostic

The desktop compiler command now returns a TLP1 envelope: four version bytes,
a little-endian 32-bit metadata length, UTF-8 JSON metadata with an empty
`pdf` array, then the original PDF bytes. Metadata is bounded at 64 MiB and
PDF output at the existing 60 MiB limit. Native service callers retain
`CompileResult`; only the desktop IPC boundary uses the envelope.

The browser validates the envelope and the existing logical build contract.
Revision, engine, eligibility, source mapping and candidate checks retain
their authority. A byte view avoids JSON number parsing and per-byte number
validation. The surface retains its copy before PDF.js can transfer a worker
buffer. Tauri's postMessage fallback may return raw replies as number arrays;
that path checks each byte before converting the envelope.

## Paired diagnostic

Protocol: `COMPILE-COST-PLAN.md`. Node v25.2.1, Windows, same uncontrolled
host as the compile diagnostic. One warmup and five measured pairs at each
size, alternating order. Repeated synthetic PDF-pattern bytes are serialization
fixtures, not valid multi-megabyte documents. Both intervals include production
validation and the surface byte copy; exact-byte equality is checked afterward.
The legacy arm imports the preserved validator from `0c3cd89`.

| Buffer | Legacy median ms | Binary median ms | Median paired saving ms | JSON bytes | Binary bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 MiB | 126.041 | 0.998 | 124.965 | 3,705,458 | 1,048,780 |
| 8 MiB | 895.155 | 6.075 | 889.376 | 29,644,012 | 8,388,812 |

These are Node decoding observations, excluding Rust encoding, Tauri/WebView
delivery, TeX execution and visible painting. The earlier 267 ms observation
and two interim blocks using the revised validator on both arms are retained
as separate development data. Do not pool these blocks or infer an application
speedup from their ratios. Machine state, memory pressure and JIT execution
remain uncontrolled; five correlated repeats do not estimate population uncertainty.

## Regression evidence

The generated Rust binary fixture crosses the JavaScript decoder, ordinary
validator and ordered client. It covers all 256 byte values and Arabic
metadata. Other controls exercise version/length/UTF-8 corruption, embedded
PDF bytes in metadata, oversized output, invalid fallback bytes, failed
builds, candidate revisions, cancellation and discarded-token refusal.
The real baseline PDF fixture retains identical text/geometry and full-page
rendered pixels. PDF.js's per-load document prefix in internal font IDs is
normalized; font identity within each document and all geometry remain checked.
The first failed comparison is preserved as development evidence.

The full local gate and hosted CI are tracked separately from these focused
tests. Native app opening, actual WebView dispatch, physical input, installation
and end-to-end presentation have not been performed by the agent.

Primary implementation context:
[Tauri binary responses](https://v2.tauri.app/develop/calling-rust/#returning-array-buffers).
The locked Tauri 2.12.5 IPC protocol and fallback source were also inspected.
