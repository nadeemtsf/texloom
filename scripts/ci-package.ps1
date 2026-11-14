param([Parameter(Mandatory=$true)][string]$ArtifactRoot)
$ErrorActionPreference='Stop'
$targetRoot=if($env:CARGO_TARGET_DIR){$env:CARGO_TARGET_DIR}else{'target'}
$executable=(Resolve-Path -LiteralPath (Join-Path $targetRoot 'x86_64-pc-windows-msvc/release/texloom-desktop.exe')).Path
$package=& "$PSScriptRoot/package-portable.ps1" -DesktopExecutable $executable -ExpectedDesktopSha256 (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash -ArtifactRoot $ArtifactRoot
$package | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $ArtifactRoot 'portable-package.json') -Encoding utf8
