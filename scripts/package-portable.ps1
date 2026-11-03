param([string]$CargoSourceRoot = $env:CARGO_HOME, [string]$ArtifactRoot)
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

Copy-PackageFile (Join-Path $projectRoot 'target/x86_64-pc-windows-msvc/release/texloom-desktop.exe') 'Texloom.exe'
foreach ($name in @('VALIDATION.md', 'LICENSE', 'README.md')) { Copy-PackageFile (Join-Path $projectRoot $name) $name }
$verifierSource = Join-Path $projectRoot 'src-tauri/resources/verification'
$manifestPath = Join-Path $verifierSource 'verifier-manifest.json'
$manifestReceipt = Get-PortableFileReceipt -Path $manifestPath -RelativePath 'verification/verifier-manifest.json'
if ($manifestReceipt.bytes -gt 65536) { throw 'Native verifier manifest exceeds size limit.' }
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema -ne 2 -or $manifest.protocol -ne 2 -or $manifest.dll_sha256 -ne 'b2c15cd4537d8efaf23b85b3353abe7a0b863e9c52b2f017dd87bb86a55f9c72') { throw 'Invalid native verifier manifest.' }
Copy-PackageFile $manifestPath 'verification/verifier-manifest.json' $manifestReceipt.sha256
foreach ($entry in @(@('pdfium_worker.exe', $manifest.worker_sha256), @('pdfium.dll', $manifest.dll_sha256), @('PDFIUM-NOTICE.txt', $manifest.notice_sha256))) {
    if ($entry[1] -notmatch '^[a-f0-9]{64}$') { throw 'Invalid native verifier receipt.' }
    Copy-PackageFile (Join-Path $verifierSource $entry[0]) ('verification/' + $entry[0]) $entry[1]
}
foreach ($name in @('main.tex', 'studio.sty', 'texloom-objects.sty')) { Copy-PackageFile (Join-Path $projectRoot "resources/$name") ('Example/' + $name) }

if (-not $CargoSourceRoot) { $CargoSourceRoot = Join-Path $env:USERPROFILE '.cargo' }
$noticeRoots = @((Join-Path $projectRoot 'node_modules'), (Join-Path $CargoSourceRoot 'registry/src'))
$notices = [Text.StringBuilder]::new()
foreach ($noticeRoot in $noticeRoots) {
    Assert-PlainPackagePath -Path $noticeRoot -Directory $true
    # License aggregation is a development notice bundle, not a completed
    # dependency/license audit. Do not follow cache links into unrelated paths.
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($noticeRoot)
    $licenses = [Collections.Generic.List[string]]::new()
    while ($pending.Count -gt 0) {
        foreach ($item in Get-ChildItem -LiteralPath $pending.Pop() -Force) {
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Linked dependency cache entry; review notices before packaging.' }
            if ($item.PSIsContainer) { $pending.Push($item.FullName) }
            elseif ($item.Name -match '^(LICENSE|LICENCE|NOTICE|COPYING|ThirdPartyNotice)') { $licenses.Add($item.FullName) }
        }
    }
    foreach ($path in @($licenses | Sort-Object)) {
        if ((Get-Item -LiteralPath $path).Length -gt 16000000) { throw 'Dependency notice exceeds size limit.' }
        [void]$notices.AppendLine('===== ' + $path.Substring($noticeRoot.Length).TrimStart('\', '/') + ' =====')
        [void]$notices.AppendLine([IO.File]::ReadAllText($path))
        if ($notices.Length -gt 64000000) { throw 'Dependency notice bundle exceeds size limit.' }
    }
}
$noticePath = Join-Path $packageRoot 'THIRD-PARTY-NOTICES.txt'
[IO.File]::WriteAllText($noticePath, $notices.ToString())
$files.Add((Get-PortableFileReceipt $noticePath 'THIRD-PARTY-NOTICES.txt'))
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
