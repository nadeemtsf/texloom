param([Parameter(Mandatory)][string] $TexloomPath)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$package = 'basic-miktex-25.12-x64.exe'
$url = "https://miktex.org/download/ctan/systems/win32/miktex/setup/windows-x64/$package"
# Keep this version and digest in sync with https://miktex.org/download.
$expected = '14B42DD9F4B4A7813A8BFD69C8F99316C2888CC4EE26F631F397E163D85D6C62'
$directory = Join-Path ([IO.Path]::GetTempPath()) ('texloom-setup-' + [Guid]::NewGuid())
$download = Join-Path $directory $package
$created = $false
$result = 1
try {
    New-Item -ItemType Directory -Path $directory | Out-Null
    $created = $true
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $download -TimeoutSec 600

    # Keep the verified file open without permitting writes or replacement.
    $file = [IO.File]::Open($download, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ((Get-FileHash -InputStream $file -Algorithm SHA256).Hash -ne $expected) {
            throw 'The MiKTeX download did not match the published SHA-256 hash.'
        }
        Write-Output 'Verified the MiKTeX download. Opening its setup wizard.'
        $setup = Start-Process -FilePath $download -WorkingDirectory $directory -WindowStyle Normal -PassThru
        try {
            # The wizard can open a browser on completion. Wait only for setup.
            $setup.WaitForExit()
            if ($setup.ExitCode -ne 0) { throw "MiKTeX setup exited with code $($setup.ExitCode)." }
        } finally {
            $setup.Dispose()
        }
    } finally {
        $file.Dispose()
    }

    Write-Output 'MiKTeX wizard closed. Setting up packages and compiling the Texloom templates.'
    # Include paths added by MiKTeX while the Texloom installer was running.
    $env:Path = @([Environment]::GetEnvironmentVariable('Path', 'Machine'),
        [Environment]::GetEnvironmentVariable('Path', 'User'), $env:Path) -join ';'
    $probe = Start-Process -FilePath $TexloomPath -ArgumentList '--prepare-latex-for-setup' -WindowStyle Hidden -PassThru
    try {
        $probe.WaitForExit()
        if ($probe.ExitCode -ne 0) { throw 'LaTeX could not compile the Texloom templates after setup.' }
    } finally {
        $probe.Dispose()
    }
    $result = 0
} catch {
    Write-Output $_.Exception.Message
} finally {
    if ($created) {
        Remove-Item -LiteralPath $download -ErrorAction SilentlyContinue
        try { [IO.Directory]::Delete($directory) } catch { }
    }
}
exit $result
