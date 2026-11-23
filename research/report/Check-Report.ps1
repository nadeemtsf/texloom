#Requires -Version 7.0
[CmdletBinding()]
param([string]$ExpectedText)

$ErrorActionPreference = 'Stop'
foreach ($command in 'pdflatex', 'bibtex', 'pdftotext') {
    Get-Command $command -ErrorAction Stop | Out-Null
}

$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$name = [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$output = New-Item -ItemType Directory (Join-Path $root "artifacts/research-report/$name")
Write-Host "Report output: $($output.FullName)"

function Invoke-PdfLatex([int]$Pass) {
    $log = Join-Path $output.FullName "pass-$Pass.log"
    pdflatex --disable-installer -no-shell-escape -interaction=nonstopmode -halt-on-error -file-line-error "-output-directory=$($output.FullName)" main.tex *> $log
    if ($LASTEXITCODE -ne 0) {
        Get-Content -LiteralPath $log -Tail 15
        throw "pdfLaTeX pass $Pass failed. See $log."
    }
}

Push-Location -LiteralPath $PSScriptRoot
try {
    Invoke-PdfLatex 1
    $previousBibInputs = $env:BIBINPUTS
    Push-Location -LiteralPath $output.FullName
    try {
        $env:BIBINPUTS = "$PSScriptRoot;$previousBibInputs"
        bibtex main *> bibtex-run.log
        if ($LASTEXITCODE -ne 0) {
            Get-Content -LiteralPath bibtex-run.log -Tail 15
            throw 'BibTeX failed.'
        }
    } finally {
        $env:BIBINPUTS = $previousBibInputs
        Pop-Location
    }
    Invoke-PdfLatex 2
    Invoke-PdfLatex 3
} finally {
    Pop-Location
}

$diagnostics = Select-String -LiteralPath (Join-Path $output.FullName 'main.log') -Pattern @(
    'LaTeX Error:', 'Undefined control sequence', 'There were undefined (references|citations)',
    '(Citation|Reference).*undefined', 'Missing character:', 'Overfull \\[hv]box',
    'Label\(s\) may have changed'
)
if ($diagnostics) {
    $diagnostics | ForEach-Object { Write-Warning $_.Line }
    throw 'The final LaTeX log needs review.'
}

$pdf = Join-Path $output.FullName 'main.pdf'
$textFile = Join-Path $output.FullName 'report.txt'
pdftotext -enc UTF-8 -layout $pdf $textFile
if ($LASTEXITCODE -ne 0) { throw 'PDF text extraction failed.' }
$text = Get-Content -LiteralPath $textFile -Raw -Encoding utf8
if ([string]::IsNullOrWhiteSpace($text)) { throw 'The PDF yielded no text.' }
if ($ExpectedText -and -not (($text -replace '\s+', ' ').Contains(($ExpectedText -replace '\s+', ' ').Trim()))) {
    throw "Expected passage was not found. See $textFile."
}
Get-Item -LiteralPath $pdf, $textFile | Select-Object -ExpandProperty FullName
