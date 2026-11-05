param(
    [string]$CargoSourceRoot = $env:CARGO_HOME,
    [string]$ArtifactRoot,
    [Parameter(Mandatory = $true)][string]$DesktopExecutable,
    [Parameter(Mandatory = $true)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$ExpectedDesktopSha256
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'portable-archive.ps1')
if (-not $ArtifactRoot) { $ArtifactRoot = Join-Path $projectRoot 'artifacts' }
$layout = New-PortableOutputLayout -ArtifactRoot $ArtifactRoot
$packageRoot = $layout.PackageRoot
$files = [Collections.Generic.List[object]]::new()

function Copy-PackageFile {
    param([string]$Source, [string]$Name, [string]$ExpectedHash)
    $before = Get-PortableFileReceipt -Path $Source -RelativePath $Name
    if ($ExpectedHash -and $before.sha256 -ne $ExpectedHash) { throw "Input receipt mismatch: $Name" }
    $destination = Join-Path $packageRoot $Name
    $parent = [IO.Path]::GetDirectoryName($destination)
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent -ErrorAction Stop | Out-Null }
    [IO.File]::Copy($Source, $destination, $false)
    $copied = Get-PortableFileReceipt -Path $destination -RelativePath $Name
    $after = Get-PortableFileReceipt -Path $Source -RelativePath $Name
    if ($copied.sha256 -ne $before.sha256 -or $after.sha256 -ne $before.sha256 -or $copied.bytes -ne $before.bytes) { throw "Input changed during packaging: $Name" }
    $files.Add($copied)
}

# Tauri can build target/release or a target-triple subfolder. Never silently
# select a stale binary from a different build directory.
Copy-PackageFile (Resolve-Path -LiteralPath $DesktopExecutable).Path 'Texloom.exe' $ExpectedDesktopSha256.ToLowerInvariant()
foreach ($name in @('VALIDATION.md', 'LICENSE', 'README.md')) { Copy-PackageFile (Join-Path $projectRoot $name) $name }
$verifierSource = Join-Path $projectRoot 'src-tauri/resources/verification'
$manifestPath = Join-Path $verifierSource 'verifier-manifest.json'
$manifestReceipt = Get-PortableFileReceipt -Path $manifestPath -RelativePath 'verification/verifier-manifest.json'
if ($manifestReceipt.bytes -gt 65536) { throw 'Native verifier manifest exceeds size limit.' }
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$pin = Get-Content -LiteralPath (Join-Path $projectRoot 'scripts/pdfium-pin.json') -Raw | ConvertFrom-Json
if ($manifest.schema -ne 2 -or $manifest.protocol -ne 3 -or $manifest.dll_sha256 -ne $pin.dll_sha256 -or $manifest.pdfium -ne $pin.pdfium) { throw 'Native verifier manifest differs from the active pin.' }
Copy-PackageFile $manifestPath 'verification/verifier-manifest.json' $manifestReceipt.sha256
foreach ($entry in @(@('pdfium_worker.exe', $manifest.worker_sha256), @('pdfium.dll', $manifest.dll_sha256), @('PDFIUM-NOTICE.txt', $manifest.notice_sha256))) {
    if ($entry[1] -notmatch '^[a-f0-9]{64}$') { throw 'Invalid native verifier receipt.' }
    Copy-PackageFile (Join-Path $verifierSource $entry[0]) ('verification/' + $entry[0]) $entry[1]
}
foreach ($name in @('main.tex', 'studio.sty', 'texloom-objects.sty')) { Copy-PackageFile (Join-Path $projectRoot "resources/$name") ('Example/' + $name) }

foreach ($name in @('DEPENDENCIES.json', 'SOURCE-AVAILABILITY.txt', 'THIRD-PARTY-NOTICES.txt')) {
    Copy-PackageFile (Join-Path $projectRoot ('src-tauri/resources/distribution/' + $name)) $name
}
$receipt = [ordered]@{
    schema = 2
    purpose = 'Development checkpoint; native interaction and release gates remain open'
    desktop_sha256 = ($files | Where-Object { $_.path -eq 'Texloom.exe' }).sha256
    worker_sha256 = $manifest.worker_sha256
    dll_sha256 = $manifest.dll_sha256
    receipt_excludes_itself = $true
    files = @($files | Sort-Object path)
}
$receiptPath = Join-Path $packageRoot 'ARTIFACT-RECEIPT.json'
[IO.File]::WriteAllText($receiptPath, ($receipt | ConvertTo-Json -Depth 6) + "`n")
$files.Add((Get-PortableFileReceipt $receiptPath 'ARTIFACT-RECEIPT.json'))
Write-VerifiedPortableArchive -PackageRoot $packageRoot -ArchivePath $layout.ArchivePath -ExpectedFiles $files.ToArray()
