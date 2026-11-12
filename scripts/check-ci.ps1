param([Parameter(Mandatory=$true)][string]$ArtifactRoot)
#requires -Version 7.0
$ErrorActionPreference='Stop'
Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)
if(-not (Get-Command cargo -ErrorAction SilentlyContinue)){
    if(-not (Test-Path -LiteralPath '.tools/cargo/bin/cargo.exe')){throw 'Install Rust before running local CI.'}
    $env:CARGO_HOME=Join-Path (Get-Location) '.tools/cargo'
    $env:RUSTUP_HOME=Join-Path (Get-Location) '.tools/rustup'
    $env:PATH="$env:CARGO_HOME\bin;$env:PATH"
}
$ArtifactRoot=[IO.Path]::GetFullPath($ArtifactRoot)
New-Item -ItemType Directory -Path $ArtifactRoot -Force | Out-Null
$reportPath=Join-Path $ArtifactRoot 'steps.json'
if(Test-Path -LiteralPath $reportPath){throw 'Use a fresh artifact directory; earlier evidence will not be overwritten.'}
$script:steps=@()
function Invoke-Check([string]$Name,[string]$Program,[string[]]$Arguments){
    $log=Join-Path $ArtifactRoot ($Name+'.log')
    $step=[ordered]@{name=$Name;command=@($Program)+$Arguments;status='running';started_utc=[DateTime]::UtcNow.ToString('o');log=$log}
    $script:steps+=,$step
    ConvertTo-Json -InputObject @($script:steps) -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
    Write-Host "Checking $Name..."
    try{
        Get-Command $Program -ErrorAction Stop | Out-Null
        # Windows PowerShell 5 treats redirected native stderr as error records;
        # compiler progress/warnings must not turn exit 0 into a failed step.
        $ErrorActionPreference='Continue'
        & $Program @Arguments *> $log
        $nativeExit=$LASTEXITCODE
        $ErrorActionPreference='Stop'
        if($nativeExit -ne 0){throw "$Name failed (exit $nativeExit). See $log"}
        $step.status='passed'
    }catch{
        $step.status='failed';Get-Content -LiteralPath $log -Tail 18 -ErrorAction SilentlyContinue;throw
    }finally{
        $ErrorActionPreference='Stop'
        $step.finished_utc=[DateTime]::UtcNow.ToString('o')
        ConvertTo-Json -InputObject @($script:steps) -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
    }
}
# Same test commands as ci.yml, proof.yml and build-service.yml. Each invocation
# creates fresh synthetic outputs; nothing opens Texloom or writes user projects.
$env:PYTHONDONTWRITEBYTECODE='1'
$env:PYTHONPATH=Join-Path $ArtifactRoot 'python'
$env:TEXLOOM_PROOF_REPORT_DIR=Join-Path $ArtifactRoot 'proof'
$env:TEXLOOM_SELECTION_REPORT_DIR=Join-Path $ArtifactRoot 'selection'
$env:TEXLOOM_PROTECTED_TEXT_REPORT_DIR=Join-Path $ArtifactRoot 'protected-text'
$env:TEXLOOM_BUILD_REPORT_DIR=Join-Path $ArtifactRoot 'build-service'
$env:TEXLOOM_BIBLIOGRAPHY_REPORT_DIR=Join-Path $ArtifactRoot 'bibliography'
Invoke-Check 'locked-frontend-dependencies' 'npm.cmd' @('ci')
Invoke-Check 'core' 'cargo' @('test','--locked','-p','texloom-core')
Invoke-Check 'recovery-codec' 'cargo' @('test','--locked','-p','texloom-core','--features','recovery-serde')
Invoke-Check 'frontend' 'npm.cmd' @('test')
Invoke-Check 'build-service' 'cargo' @('test','--locked','-p','texloom-build')
Invoke-Check 'native-protocol' 'cargo' @('test','--locked','-p','texloom-desktop','--features','typegen','--lib')
Invoke-Check 'prepare-pinned-native-verifier' 'node' @('scripts/prepare-native-verifier.mjs')
Invoke-Check 'independent-python-reference-dependencies' 'python' @('-m','pip','install','--disable-pip-version-check','--target',$env:PYTHONPATH,'-r','scripts/proof/requirements.txt')
Invoke-Check 'independent-python-reference' 'python' @('scripts/proof/inspect_pdf.py','tests/fixtures/objects/baseline.pdf',(Join-Path $ArtifactRoot 'reference/baseline.json'))
$env:TEXLOOM_PDFIUM_TEST_DLL=Join-Path (Get-Location) 'src-tauri/resources/verification/pdfium.dll'
$env:TEXLOOM_PDFIUM_TEST_WORKER=Join-Path (Get-Location) 'src-tauri/resources/verification/pdfium_worker.exe'
$env:TEXLOOM_PDFIUM_REFERENCE=Join-Path $ArtifactRoot 'reference/baseline.json'
Invoke-Check 'native-inspection' 'cargo' @('test','--locked','-p','texloom-build','--test','pdfium_worker','--','--ignored')
Invoke-Check 'native-approval' 'cargo' @('test','--locked','-p','texloom-desktop','--lib','--','--ignored')
foreach($example in @('proof_candidate','selection_hint','literal_candidate','spacing_candidate')){
    Invoke-Check "example-$example" 'cargo' @('build','--locked','-p','texloom-core','--example',$example)
}
Invoke-Check 'verification-policy' 'cargo' @('test','--locked','-p','texloom-build','--test','verification')
Invoke-Check 'example-displacement' 'cargo' @('build','--locked','-p','texloom-build','--example','displacement_check')
Invoke-Check 'proof-regressions' 'node' @('scripts/movement-proof.mjs','--prepare')
Invoke-Check 'three-engine-movement' 'node' @('scripts/movement-proof.mjs')
Invoke-Check 'three-engine-selection' 'node' @('scripts/selection-proof.mjs')
Invoke-Check 'three-engine-protected-text' 'node' @('scripts/protected-text-proof.mjs')
$env:TEXLOOM_NATIVE_CORPUS=Join-Path $env:TEXLOOM_PROOF_REPORT_DIR 'compiled'
Invoke-Check 'native-three-engine-corpus' 'cargo' @('test','--locked','-p','texloom-build','--test','engine_worker_corpus','--','--ignored','--nocapture')
Invoke-Check 'three-engine-build-corpus' 'cargo' @('test','--locked','-p','texloom-build','--test','engine_corpus','--','--ignored','--nocapture')
Invoke-Check 'bibliography' 'cargo' @('test','--locked','-p','texloom-build','--test','bibliography','--','--ignored','--nocapture')
Invoke-Check 'windows-build' 'npm.cmd' @('run','desktop:build','--','--no-bundle','--target','x86_64-pc-windows-msvc')
Invoke-Check 'portable-package' (Join-Path $PSHOME 'pwsh.exe') @('-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/ci-package.ps1','-ArtifactRoot',(Join-Path $ArtifactRoot 'portable'))
Invoke-Check 'source-benchmark' 'cargo' @('run','--locked','--release','-p','texloom-desktop','--features','benchmark','--example','source_benchmark','--target','x86_64-pc-windows-msvc','--',(Join-Path $ArtifactRoot 'source-benchmark'))
Write-Host "All local CI steps passed. Evidence: $ArtifactRoot"
