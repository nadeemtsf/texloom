# Editable technical report draft

Latest 10 November artifact: v11 is a 26-page technical report draft with 21 references. It adds typed source-position/selection mapping, conservative literal intent planning, preserved regressions and a one-document three-engine compile/text pilot. It was compiled, all pages rendered, first 20 pages checked byte-identical against their reviewed render and final six reviewed; changed method/evidence pages were inspected at readable size. No overfull/undefined/missing-character/LaTeX-warning matches were found in the final log. The v11 engineering/artifact receipt is `docs/architecture/foundation-continuation-validation-2026-11-10.json`. Final evaluation and owner app acceptance remain pending; older artifacts below retain their named-version scope.

8 November source/evidence supplement: [foundation development note](../FOUNDATION-DEVELOPMENT-2026-11-08.md) adds bounded headless engineering evidence and future diagnostic comparisons. It did not rebuild or silently update the v10 PDF described below. Final evaluation remains unfrozen/unrun.

Latest7November artifact: v10 has26pages/21references and includes the legacy-support migration pilot plus matched-artifact/notice follow-up. All26pages were rendered/reviewed, with readable review of the new evidence sections. No overfull/undefined/missing-character/LaTeX-warning matches occur in the final log. This remains a technical draft; final evaluation and native owner/performance acceptance are pending. Earlier versioned notes below retain their historical scope.

Latest artifact (7 November): v9 is a25-page,21-reference draft including the headless native mapping pilot. All pages were rendered/reviewed; after the date-only correction pages2–25 body pixels stayed exact and the final title page was reviewed. No final-log overfull/undefined/missing-character/LaTeX-warning matches were found. Final evaluation, native owner acceptance and performance remain pending. Earlier artifact notes below retain their historical scope.

Current artifact note (6 November): the v8 PDF includes the owner-follow-up subsection and was compiled and visually reviewed at 25 pages with 21 references. Every page was rendered with Poppler; all contact sheets and the new subsection/bibliography pages were inspected. The final log has no overfull boxes, unresolved citations/references or missing-character warnings. It records bounded RTL-target regressions and recovery changes; final evaluation and owner acceptance of the rebuilt app remain pending. Older source-only/v5 paragraphs below are historical notes, superseded for artifact status by this notice. Earlier v7 artifacts remain unchanged.

`main.tex`, `sections/*.tex` and `references.bib` are the editable sources. The owner removed the eight-page constraint on 5 November 2026; this is now a full technical report draft with no page cap. Methods, contract boundaries and limitations precede measured development evidence. Detailed protocols, the failed reference/corrected rerun, timing boundaries, environment/provenance and reproduction appendices are included. Native latency/overhead and final results remain pending; the audit commit is not an evaluation freeze.

The latest **uncompiled source revision** integrates the source-only interaction/failure-handling notes, expanded primary-source review, v1 outcome/reference methods and schema-2 timing plan. `main.tex` now includes `followup-references.bib`; inspection limits remain explicit for abstract-only sources. The owner paused every test; no report rebuild, new experimental result or page count accompanies this revision. The existing v5 PDF and its fourteen compiled references retain their original scope. New application and research code remain untested. RECORDS.md, REFERENCE-ADMISSION.md, TIMING-RECORDS.md and ADJUDICATION.md give the detailed specifications.

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

The reviewed v5 PDF cites fourteen primary references; the revised source includes additional citations that have not been compiled or rendered. Earlier eight-page versions remain preserved; they no longer set a length target. Development component results are separate from final results: the native-entry pilot includes a preserved invalid-reference run and a corrected protocol, without pooling reused cases or claiming actual PDF mouse selection. The initial missing `booktabs` dependency was removed from the source in favor of ordinary table rules; no global package was installed. Build logs, intermediate PDFs and page renders remain under ignored `artifacts/research-report/`, outside tracked research sources.

Each numbered section and appendix has its own source file so later excerpts can be selected without treating an eight-page budget as a completeness criterion. A selected short version will need its references/cross-references and claims checked again.

Future measured results require the freeze, fixture/oracle manifests and run records described in `../evaluation-plan.md`. Do not fill result cells from regression-test totals.
