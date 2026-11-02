param([string]$CargoSourceRoot = $env:CARGO_HOME)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$artifactRoot = Join-Path $projectRoot 'artifacts'
$packageRoot = Join-Path $artifactRoot 'Texloom-Desktop-Windows-x64'
New-Item -ItemType Directory -Path $packageRoot -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $projectRoot 'target\x86_64-pc-windows-msvc\release\texloom-desktop.exe') -Destination (Join-Path $packageRoot 'Texloom.exe')
Copy-Item -LiteralPath (Join-Path $projectRoot 'VALIDATION.md') -Destination $packageRoot
Copy-Item -LiteralPath (Join-Path $projectRoot 'LICENSE') -Destination $packageRoot
Copy-Item -LiteralPath (Join-Path $projectRoot 'README.md') -Destination $packageRoot
$verifierSource = Join-Path $projectRoot 'src-tauri/resources/verification'
$manifest = Get-Content -LiteralPath (Join-Path $verifierSource 'verifier-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.schema -ne 2 -or $manifest.protocol -ne 2 -or $manifest.dll_sha256 -ne 'b2c15cd4537d8efaf23b85b3353abe7a0b863e9c52b2f017dd87bb86a55f9c72') { throw 'Invalid native verifier manifest.' }
foreach ($entry in @(@('pdfium_worker.exe', $manifest.worker_sha256), @('pdfium.dll', $manifest.dll_sha256), @('PDFIUM-NOTICE.txt', $manifest.notice_sha256))) {
    if ((Get-FileHash -LiteralPath (Join-Path $verifierSource $entry[0]) -Algorithm SHA256).Hash -ne $entry[1]) { throw "Native verifier receipt mismatch: $($entry[0])" }
}
$verifierTarget = Join-Path $packageRoot 'verification'
New-Item -ItemType Directory -Path $verifierTarget -Force | Out-Null
foreach ($name in @('pdfium_worker.exe','pdfium.dll','PDFIUM-NOTICE.txt','verifier-manifest.json')) {
    Copy-Item -LiteralPath (Join-Path $verifierSource $name) -Destination $verifierTarget -Force
}
$exampleRoot = Join-Path $packageRoot 'Example'
New-Item -ItemType Directory -Path $exampleRoot -Force | Out-Null
foreach ($name in @('main.tex','studio.sty','texloom-objects.sty')) {
    Copy-Item -LiteralPath (Join-Path $projectRoot "resources\$name") -Destination $exampleRoot
}
if (-not $CargoSourceRoot) { $CargoSourceRoot = Join-Path $env:USERPROFILE '.cargo' }
$noticeRoots = @((Join-Path $projectRoot 'node_modules'), (Join-Path $CargoSourceRoot 'registry\src'))
$notices = [System.Text.StringBuilder]::new()
foreach ($noticeRoot in $noticeRoots) {
    if (-not (Test-Path -LiteralPath $noticeRoot)) { throw "Dependency notices not found: $noticeRoot" }
    Get-ChildItem -LiteralPath $noticeRoot -File -Recurse | Where-Object { $_.Name -match '^(LICENSE|LICENCE|NOTICE|COPYING|ThirdPartyNotice)' } | Sort-Object FullName | ForEach-Object {
        $relativeName = $_.FullName.Substring($noticeRoot.Length).TrimStart('\')
        [void]$notices.AppendLine("===== $relativeName =====")
        [void]$notices.AppendLine([System.IO.File]::ReadAllText($_.FullName))
    }
}
[System.IO.File]::WriteAllText((Join-Path $packageRoot 'THIRD-PARTY-NOTICES.txt'), $notices.ToString())
$provenance = [ordered]@{
    schema = 1; purpose = 'Development checkpoint; native interaction and release gates remain open'
    desktop_sha256 = (Get-FileHash -LiteralPath (Join-Path $packageRoot 'Texloom.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
    worker_sha256 = $manifest.worker_sha256; dll_sha256 = $manifest.dll_sha256
}
[System.IO.File]::WriteAllText((Join-Path $packageRoot 'ARTIFACT-RECEIPT.json'), ($provenance | ConvertTo-Json) + "`n")
Compress-Archive -LiteralPath $packageRoot -DestinationPath (Join-Path $artifactRoot 'Texloom-Desktop-Windows-x64.zip') -Force
Get-FileHash -LiteralPath (Join-Path $artifactRoot 'Texloom-Desktop-Windows-x64.zip') -Algorithm SHA256
