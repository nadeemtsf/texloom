# P4 selection checkpoint

Source remains authoritative. A SyncTeX hit is a proposal, not permission to move text. Selection now follows this protocol:

1. Native code checks the project, acknowledged revision, eligible build and dependency receipt before and after mapping. It tries at most three bounded positions within the clicked glyph lines and resolves compatible contiguous source units and projects byte ranges through the same BOM/CRLF/UTF-16 mapping as Monaco.
2. A private candidate wraps that source with a new movement marker at zero displacement. No canonical source, history or displayed PDF is changed.
3. The candidate compiles to convergence and renders offscreen. The runtime compares every page's text/geometry and every canvas pixel with the baseline at the same scale. It checks page geometry, exact marker occurrence count, complete single-page output and coverage of all clicked text.
4. Only then does the app show the frame using actual tagged glyph bounds. A whole paragraph may expand a clicked visual line. A source fragment rendered more than once, overlapping duplicate glyphs, wrong occurrence, changed appearance, page crossing or missing identity is refused.
5. The probe token and PDF are always discarded. A bounded, build/view-specific cache avoids recompiling an already proven unit; dependency receipts are still checked. Typing, cancellation, a new view or project switch invalidates pending results.

This chooses the existing portable marker adapter plus a verified probe for the bounded supported text shapes. It avoids treating a broad SyncTeX rectangle as a reliable source-object identity. It does not establish general TeX provenance or replace the planned parser/Lua tracing evaluation. The upstream tree-sitter-latex grammar is structural and best effort; it does not provide PDF output identity: https://github.com/latex-lsp/tree-sitter-latex.

The first selection of a unit needs a full private build. This trades latency for evidence; no live-selection latency target has been established. Cache size is limited to 64 successful selections per prepared view. Verification is limited to 16 million pixels per page and 96 MiB of baseline pixels per document. Rendering itself still needs P7 virtualization/budgets.

Rotated pages, nonzero crop origins and non-default UserUnit remain explicitly unavailable for visual selection. Reading/source editing remains available. Independent runtime PDFium verification remains P6; the probe's pixel comparison uses PDF.js only. A successful selection does not certify a subsequent displacement or replacement.

Local checks: 51 core, 15 native service and 54 frontend/PDF tests passed. The unchanged 17 build-infrastructure tests passed at the parent checkpoint. The new real-engine selection corpus runs after the movement experiment in CI, combining actual SyncTeX hits, the real Rust unit resolver, PDF markers and PDF.js pixels. Native mouse/IME acceptance remains open. Local MiKTeX CLI execution is blocked by this session's registry permissions; no local engine pass is inferred.

The first real-engine selection run passed 30/33 expected outcomes. Included-file hints under pdfLaTeX/XeLaTeX need alternate hit samples. The pdfLaTeX mixed-format fixture changes PDF.js raster output under the zero wrapper despite stable text geometry and a passing PDFium movement experiment; this profile remains refused by the unchanged strict runtime check. The corpus records this measured capability limit explicitly, rather than relaxing pixel equality. Raw first run: docs/evidence/selection-first-run.json.
