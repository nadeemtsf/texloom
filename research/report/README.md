# Editable technical report draft

`main.tex` and `references.bib` are the editable sources. This is an eight-page technical report draft, updated 5 November 2026. Methods, contract boundaries and limitations precede the proposed evaluation. Optional timing instrumentation is described without measured native latency or overhead results; posthoc environment metadata is explicitly limited. Final results are pending; the audit commit is not an evaluation freeze.

The document uses neutral system descriptions and makes no individual-contribution claim. The primary-source inspection notes and limitations are in `../related-work.md`. No publication or submission is authorized.

## Rebuild

Run from this directory using an existing LaTeX installation with the packages listed in `main.tex`. Compile **only `main.tex`**, with shell escape disabled; never compile `.text` files. Do not enable automatic package installation for this build.

On this Windows/MiKTeX host, the reviewed build used the following sequence. Replace the output directory with a fresh local directory when preserving another version:

```powershell
$reportOutput = Join-Path (Resolve-Path '../..') 'artifacts/research-report/draft-v0-20261104'
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

The reviewed draft is eight pages with ten bibliography entries. Development component results are separate from final results: the native-entry pilot includes a preserved invalid-reference run and a corrected protocol, without pooling reused cases or claiming actual PDF mouse selection. The initial missing `booktabs` dependency was removed from the source in favor of ordinary table rules; no global package was installed. Build logs, intermediate PDFs and page renders remain under ignored `artifacts/research-report/`, outside tracked research sources.

Future measured results require the freeze, fixture/oracle manifests and run records described in `../evaluation-plan.md`. Do not fill result cells from regression-test totals.
