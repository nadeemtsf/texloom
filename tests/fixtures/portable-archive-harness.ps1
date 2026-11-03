param([string]$FixtureRoot, [string]$ArchiveHelper, [string]$Case)
$ErrorActionPreference = 'Stop'
. $ArchiveHelper
function Require([bool]$Ok, [string]$Message) { if (-not $Ok) { throw $Message } }
function Refused([scriptblock]$Action, [string]$Message) {
    $caught = $null
    try { & $Action | Out-Null } catch { $caught = $_.ToString() }
    Require ($null -ne $caught -and $caught.Contains($Message)) ('Expected refusal: ' + $Message + '; received: ' + $caught)
}
$out = Join-Path $FixtureRoot 'out'
New-Item -ItemType Directory -Path $out | Out-Null
if ($Case -eq 'isolation') {
    $old = Join-Path $out 'Texloom-Desktop-Windows-x64'
    New-Item -ItemType Directory -Path $old | Out-Null
    [IO.File]::WriteAllText((Join-Path $old 'private.tex'), 'Private old project; do not package')
    [IO.File]::WriteAllText((Join-Path $out 'Texloom-Desktop-Windows-x64.zip'), 'Previous archive; preserve')
}
$layout = New-PortableOutputLayout $out
$root = $layout.PackageRoot
$main = Join-Path $root 'main.tex'
$content = [Text.Encoding]::UTF8.GetBytes(([char]0xfeff + "Line`r`n" + [char]::ConvertFromUtf32(0x1f600)))
[IO.File]::WriteAllBytes($main, $content)
$receipt = Get-PortableFileReceipt $main 'main.tex'

switch ($Case) {
    'isolation' {
        Require ($root -ne (Join-Path $out 'Texloom-Desktop-Windows-x64')) 'Staging must be fresh.'
        Require ($layout.ArchivePath -ne (Join-Path $out 'Texloom-Desktop-Windows-x64.zip')) 'Existing archives need a new name.'
        New-Item -ItemType Directory -Path (Join-Path $root 'Example') | Out-Null
        $style = Join-Path $root 'Example/style.sty'
        [IO.File]::WriteAllText($style, '% known style')
        $styleReceipt = Get-PortableFileReceipt $style 'Example/style.sty'
        $result = Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt, $styleReceipt)
        Require ($result.Files -eq 2) 'Wrong archive file count.'
        Require (([IO.File]::ReadAllText((Join-Path $old 'private.tex'))) -eq 'Private old project; do not package') 'Old private project changed.'
        Require (([IO.File]::ReadAllText((Join-Path $out 'Texloom-Desktop-Windows-x64.zip'))) -eq 'Previous archive; preserve') 'Old archive changed.'
        $file = [IO.File]::OpenRead($result.ArchivePath)
        try {
            $zip = [IO.Compression.ZipArchive]::new($file, [IO.Compression.ZipArchiveMode]::Read)
            try {
                Require ($zip.Entries.Count -eq 2) 'Unexpected archive entries.'
                Require ($null -ne $zip.GetEntry('Texloom-Desktop-Windows-x64/Example/style.sty')) 'Missing nested entry.'
                $entry = $zip.GetEntry('Texloom-Desktop-Windows-x64/main.tex').Open()
                try {
                    $bytes = [IO.MemoryStream]::new()
                    try { $entry.CopyTo($bytes); Require ([Convert]::ToBase64String($bytes.ToArray()) -eq [Convert]::ToBase64String($content)) 'UTF-8/BOM/CRLF bytes changed.' }
                    finally { $bytes.Dispose() }
                } finally { $entry.Dispose() }
            } finally { $zip.Dispose() }
        } finally { $file.Dispose() }
    }
    'unknown' {
        $private = Join-Path $root 'private.tex'
        [IO.File]::WriteAllText($private, 'Never upload')
        [IO.File]::SetAttributes($private, [IO.FileAttributes]::Hidden)
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt) } 'Unreceipted file'
        Require (-not (Test-Path -LiteralPath $layout.ArchivePath)) 'Unknown files must fail before ZIP creation.'
    }
    'tampered' {
        [IO.File]::WriteAllText($main, 'Changed after receipt')
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt) } 'Staged receipt mismatch'
    }
    'existing' {
        # Simulate an archive appearing after name selection.
        [IO.File]::WriteAllText($layout.ArchivePath, 'Keep the previous bytes')
        $before = (Get-FileHash -LiteralPath $layout.ArchivePath -Algorithm SHA256).Hash
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt) } 'already exists'
        Require ((Get-FileHash -LiteralPath $layout.ArchivePath -Algorithm SHA256).Hash -eq $before) 'Existing archive was overwritten.'
    }
    'paths' {
        foreach ($name in @('../private.tex', '/private.tex', 'C:/private.tex', 'Example//style.sty')) {
            $invalid = [pscustomobject]@{ path = $name; bytes = $receipt.bytes; sha256 = $receipt.sha256 }
            Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($invalid) } 'Invalid or duplicate'
        }
        $duplicate = [pscustomobject]@{ path = 'MAIN.TEX'; bytes = $receipt.bytes; sha256 = $receipt.sha256 }
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt, $duplicate) } 'Invalid or duplicate'
        $missing = [pscustomobject]@{ path = 'missing.sty'; bytes = 1; sha256 = $receipt.sha256 }
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt, $missing) } 'files are missing'
        Require (-not (Test-Path -LiteralPath $layout.ArchivePath)) 'Invalid receipts created an archive.'
    }
    'link' {
        $private = Join-Path $FixtureRoot 'private'
        New-Item -ItemType Directory -Path $private | Out-Null
        [IO.File]::WriteAllText((Join-Path $private 'private.tex'), 'Outside staging')
        try { New-Item -ItemType Junction -Path (Join-Path $root 'linked') -Target $private | Out-Null }
        catch [UnauthorizedAccessException] {
            [pscustomobject]@{ case = $Case; passed = $false; skip = 'Host denied creation of the test junction; link refusal is unverified here.' } | ConvertTo-Json -Compress
            exit 0
        }
        catch [ComponentModel.Win32Exception] {
            if ($_.Exception.NativeErrorCode -notin @(5, 1314)) { throw }
            [pscustomobject]@{ case = $Case; passed = $false; skip = 'Host denied creation of the test junction; link refusal is unverified here.' } | ConvertTo-Json -Compress
            exit 0
        }
        Refused { Write-VerifiedPortableArchive $root $layout.ArchivePath @($receipt) } 'Link in portable staging'
        Require (-not (Test-Path -LiteralPath $layout.ArchivePath)) 'Link refusal created an archive.'
        Require (([IO.File]::ReadAllText((Join-Path $private 'private.tex'))) -eq 'Outside staging') 'Link target changed.'
    }
    default { throw 'Unknown filesystem test case.' }
}
[pscustomobject]@{ case = $Case; passed = $true } | ConvertTo-Json -Compress
