# Shared by the development packager and filesystem tests. No application input
# controls or deletion operations. Every archive entry is explicitly receipted.
Add-Type -AssemblyName System.IO.Compression

function Assert-PlainPackagePath {
    param([string]$Path, [bool]$Directory)
    $item = Get-Item -LiteralPath ([IO.Path]::GetFullPath($Path)) -Force -ErrorAction Stop
    if ($item.PSIsContainer -ne $Directory) { throw "Unexpected package path type: $Path" }
    $current = $item
    while ($null -ne $current) {
        if (($current.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Reparse point in package path: $($current.FullName)"
        }
        if ($current -is [IO.FileInfo]) { $current = $current.Directory }
        else { $current = $current.Parent }
    }
}

function New-PortableOutputLayout {
    param([string]$ArtifactRoot)
    $root = [IO.Path]::GetFullPath($ArtifactRoot)
    if (-not (Test-Path -LiteralPath $root)) {
        Assert-PlainPackagePath -Path ([IO.Path]::GetDirectoryName($root)) -Directory $true
        New-Item -ItemType Directory -Path $root -ErrorAction Stop | Out-Null
    }
    Assert-PlainPackagePath -Path $root -Directory $true
    $identity = [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N')
    $stage = Join-Path $root ('.texloom-package-' + $identity)
    # No -Force: a staging directory must be new, never an old app/project copy.
    New-Item -ItemType Directory -Path $stage -ErrorAction Stop | Out-Null
    $package = Join-Path $stage 'Texloom-Desktop-Windows-x64'
    New-Item -ItemType Directory -Path $package -ErrorAction Stop | Out-Null
    $archive = Join-Path $root 'Texloom-Desktop-Windows-x64.zip'
    if (Test-Path -LiteralPath $archive) {
        $archive = Join-Path $root ('Texloom-Desktop-Windows-x64-' + $identity + '.zip')
    }
    [pscustomobject]@{ PackageRoot = $package; ArchivePath = $archive }
}

function Get-PortableFileReceipt {
    param([string]$Path, [string]$RelativePath)
    Assert-PlainPackagePath -Path $Path -Directory $false
    [pscustomobject]@{
        path = $RelativePath
        bytes = (Get-Item -LiteralPath $Path -Force).Length
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function Write-VerifiedPortableArchive {
    param([string]$PackageRoot, [string]$ArchivePath, [object[]]$ExpectedFiles)
    Assert-PlainPackagePath -Path $PackageRoot -Directory $true
    $root = [IO.Path]::GetFullPath($PackageRoot)
    $archive = [IO.Path]::GetFullPath($ArchivePath)
    Assert-PlainPackagePath -Path ([IO.Path]::GetDirectoryName($archive)) -Directory $true
    if ($ExpectedFiles.Count -eq 0) { throw 'An empty portable package is invalid.' }
    $names = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $ExpectedFiles) {
        $name = [string]$entry.path
        if ($name -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*$' -or
            @($name.Split('/') | Where-Object { $_ -in @('', '.', '..') }).Count -ne 0 -or
            -not $names.Add($name) -or $entry.sha256 -notmatch '^[a-f0-9]{64}$' -or $entry.bytes -lt 0) {
            throw 'Invalid or duplicate portable package receipt.'
        }
    }
    # Walk without following links. Unknown files, including hidden files, must
    # fail before an archive is created. Compression never enumerates a folder.
    $directories = [Collections.Generic.Stack[string]]::new()
    $directories.Push($root)
    $found = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    while ($directories.Count -gt 0) {
        foreach ($item in Get-ChildItem -LiteralPath $directories.Pop() -Force) {
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Link in portable staging.' }
            if ($item.PSIsContainer) { $directories.Push($item.FullName); continue }
            $relative = $item.FullName.Substring($root.Length + 1).Replace('\', '/')
            if (-not $names.Contains($relative)) { throw "Unreceipted file in portable staging: $relative" }
            [void]$found.Add($relative)
        }
    }
    if ($found.Count -ne $names.Count) { throw 'Portable package files are missing.' }
    # CreateNew preserves an existing archive even if it appeared after layout
    # selection. Keep failed staging/output for inspection; never recursively
    # remove or move an earlier build or a user directory.
    if (Test-Path -LiteralPath $archive) { throw 'Portable ZIP already exists; previous bytes preserved.' }
    $stream = [IO.File]::Open($archive, [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    $zip = $null
    try {
        $prefix = [IO.Path]::GetFileName($root) + '/'
        $zip = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Create, $true)
        foreach ($entry in @($ExpectedFiles | Sort-Object path)) {
            $path = Join-Path $root ([string]$entry.path)
            Assert-PlainPackagePath -Path $path -Directory $false
            $contentStream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
            try {
                $sha = [Security.Cryptography.SHA256]::Create()
                try { $hash = [BitConverter]::ToString($sha.ComputeHash($contentStream)).Replace('-', '').ToLowerInvariant() }
                finally { $sha.Dispose() }
                if ($contentStream.Length -ne $entry.bytes -or $hash -ne $entry.sha256) { throw "Staged receipt mismatch: $($entry.path)" }
                $contentStream.Position = 0
                $output = $zip.CreateEntry($prefix + $entry.path, [IO.Compression.CompressionLevel]::Optimal).Open()
                try { $contentStream.CopyTo($output) } finally { $output.Dispose() }
            } finally { $contentStream.Dispose() }
        }
        $zip.Dispose(); $zip = $null
        $stream.Flush($true)
        $stream.Position = 0
        $zip = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Read, $true)
        if ($zip.Entries.Count -ne $ExpectedFiles.Count) { throw 'Portable ZIP entry count mismatch.' }
        $checked = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($entry in $ExpectedFiles) {
            $name = $prefix + $entry.path
            $packed = $zip.GetEntry($name)
            if ($null -eq $packed -or -not $checked.Add($packed.FullName) -or $packed.Length -ne $entry.bytes) { throw 'Portable ZIP identity/size mismatch.' }
            $contentStream = $packed.Open()
            try {
                $sha = [Security.Cryptography.SHA256]::Create()
                try { $hash = [BitConverter]::ToString($sha.ComputeHash($contentStream)).Replace('-', '').ToLowerInvariant() }
                finally { $sha.Dispose() }
                if ($hash -ne $entry.sha256) { throw "Portable ZIP content mismatch: $name" }
            } finally { $contentStream.Dispose() }
        }
    } finally {
        if ($null -ne $zip) { $zip.Dispose() }
        $stream.Dispose()
    }
    [pscustomobject]@{
        ArchivePath = $archive
        ArchiveSHA256 = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        PackageRoot = $root
        Files = $ExpectedFiles.Count
    }
}
