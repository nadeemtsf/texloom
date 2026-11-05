# Editable technical report draft

`main.tex`, `sections/*.tex` and `references.bib` are the editable sources. The owner removed the eight-page constraint on 5 November 2026; this is now a full technical report draft with no page cap. Methods, contract boundaries and limitations precede measured development evidence. Detailed protocols, the failed reference/corrected rerun, timing boundaries, environment/provenance and reproduction appendices are included. Native latency/overhead and final results remain pending; the audit commit is not an evaluation freeze.

Later source-only work is documented in `../RECORDS.md`, `../ADJUDICATION.md` and `../related-work-followup-2026-11-05.md`. Its additional bibliography is `followup-references.bib`, intentionally not included by `main.tex`. The owner paused every test; no report rebuild or new experimental result accompanies these notes. The existing v5 PDF and its fourteen compiled references retain their original source/evidence scope. New product source `c2ef008` and the v1 record implementation remain untested.

The document uses neutral system descriptions and makes no individual-contribution claim. The primary-source inspection notes and limitations are in `../related-work.md`. No publication or submission is authorized.

The 5 November v5 review copy has 21 pages, including references and reproduction appendices. This is an observed length, not a new target or limit. All pages were rendered and reviewed as contact sheets, with enlarged review of the evidence tables, source-contract example, fingerprints, commands and bibliography. The final log has no overfull boxes, unresolved references or missing-glyph warnings; a page-bounds check found no text outside the page. These document checks are not application tests or a final research evaluation.

## Rebuild

Run from this directory using an existing LaTeX installation with the packages listed in `main.tex`. Compile **only `main.tex`**, with shell escape disabled; never compile `.text` files. Do not enable automatic package installation for this build.

On this Windows/MiKTeX host, the reviewed build used the following sequence. Replace the output directory with a fresh local directory when preserving another version:

```powershell
$reportOutput = Join-Path (Resolve-Path '../..') 'artifacts/research-report/NEW_REPORT_BUILD'
New-Item -ItemType Directory -Path $reportOutput -Force | Out-Null
pdflatex --disable-installer -no-shell-escape -interaction=nonstopmode -halt-on-error -file-line-error "-output-directory=$reportOutput" main.tex
$reportSource = (Get-Location).Path
$previousBibInputs = $env:BIBINPUTS
try {
    $env:BIBINPUTS = "$reportSource;"
    Push-Location $reportOutput
    try { bibtex main } finally { Pop-Location }
} finally { $env:BIBINPUTS = $previousBibInputs }
pdflatex --disable-installer -no-shell-escape -interaction=nonstopmode -halt-on-error -file-line-error "-output-directory=$reportOutput" main.tex
pdflatex --disable-installer -no-shell-escape -interaction=nonstopmode -halt-on-error -file-line-error "-output-directory=$reportOutput" main.tex
```

Check every process exit status. `--disable-installer` is specific to MiKTeX; use the corresponding no-auto-install configuration on another distribution. Render all pages with Poppler and inspect the latest rendering after layout changes. Successful compilation alone does not establish visual quality or the truth of research claims.

The expanded draft cites fourteen primary references. Earlier eight-page versions remain preserved; they no longer set a length target. Development component results are separate from final results: the native-entry pilot includes a preserved invalid-reference run and a corrected protocol, without pooling reused cases or claiming actual PDF mouse selection. The initial missing `booktabs` dependency was removed from the source in favor of ordinary table rules; no global package was installed. Build logs, intermediate PDFs and page renders remain under ignored `artifacts/research-report/`, outside tracked research sources.

Each numbered section and appendix has its own source file so later excerpts can be selected without treating an eight-page budget as a completeness criterion. A selected short version will need its references/cross-references and claims checked again.

Future measured results require the freeze, fixture/oracle manifests and run records described in `../evaluation-plan.md`. Do not fill result cells from regression-test totals.
