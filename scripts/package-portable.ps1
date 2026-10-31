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
Compress-Archive -LiteralPath $packageRoot -DestinationPath (Join-Path $artifactRoot 'Texloom-Desktop-Windows-x64.zip') -Force
Get-FileHash -LiteralPath (Join-Path $artifactRoot 'Texloom-Desktop-Windows-x64.zip') -Algorithm SHA256
