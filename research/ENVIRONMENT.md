# Development environment snapshots

The read-only `tools/environment_manifest.py` records explicitly chosen executable, font and input-file fingerprints plus a limited machine/Python description. It probes only fixed version switches for supported tools. It does not compile, start Texloom, install packages or enumerate user projects. Output creation is exclusive: a previous manifest cannot be overwritten.

`--context development-preflight` means the snapshot was collected before a future development activity; `development-posthoc` means after it. Neither mode creates a final evaluation freeze. The final held-out set remains empty and `evaluation/freeze.json` remains `not_frozen`.

## Executed snapshot, 5 November 2026 local time

`artifacts/research-environment-20261105/posthoc-snapshot.json` records seven available tool binaries (the three LaTeX engines, PDF text extractor, rasterizer, Python and Node), four explicitly declared Windows Arial font files, five existing reference PDFs, the movement support package, dependency lockfiles, research requirements and the PDFium pin. All declared files/version probes resolved at collection time. Three environment-collector diagnostics bring the research helper suite to 22 passing checks; these are infrastructure checks, not 22 application accuracy cases.

This is a **posthoc, partial environment snapshot**. It does not prove that the current Arial files were the exact resolved fonts during the earlier pilot, or establish absence of font fallback. Fontspec declarations and retained logs are supporting evidence, not an immutable pre-run font binding. No font files are copied or licensed for redistribution by this process. The source documents can be shared separately from proprietary font binaries.

The manifest intentionally leaves RAM, WebView2 and display configuration unset. It does not inventory every shared library, TeX package or background task. Do not use it to claim a sealed reference machine, a clean-machine replication, cross-machine pixel identity or a final latency result. The existing pilot outputs are unchanged; the new snapshot does not rerun their comparisons or repair a historical provenance gap.

## Before a final evaluation

Record the product/analysis/tool hashes and complete inputs before running; retain native build receipts/recorder inputs and record actual font resolution, fallback and permitted font acquisition. Specify reproducible, appropriately licensed fonts for shareable final fixtures. Capture OS/CPU/RAM/WebView2/display/DPI/power/background-load information and the exact diagnostic-build flag. Seal fixture/oracle manifests and independent labels together. Refresh the environment record and record any changes; preserve the posthoc pilot snapshot as its own artifact.

The collector reports missing/changing executables or unreadable inputs as unresolved, never as evidence of a successful setup. Full local paths and the account's home path are omitted/redacted from output; filename/hash alone is still insufficient to reconstruct a complete installation.
