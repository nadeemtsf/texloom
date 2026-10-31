param([switch]$CoreOnly, [switch]$Portable)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot

function Check-NativeExit([string]$Step) {
    if ($LASTEXITCODE -ne 0) { throw "$Step failed with exit code $LASTEXITCODE. See the output above." }
}

try {
    Write-Host 'Texloom Desktop: checking Windows build prerequisites...'
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $visualStudio = $null
    if (Test-Path -LiteralPath $vswhere) {
        $visualStudio = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    }
    if (-not $visualStudio) {
        throw 'Microsoft C++ Build Tools are required. Install Desktop development with C++ and a Windows SDK from https://visualstudio.microsoft.com/visual-cpp-build-tools/ , then run this script again.'
    }

    $tools = Join-Path $PSScriptRoot '.tools'
    New-Item -ItemType Directory -Path $tools -Force | Out-Null
    $localCargo = Join-Path $tools 'cargo\bin'
    if (Test-Path -LiteralPath (Join-Path $localCargo 'rustc.exe')) {
        $env:CARGO_HOME = Join-Path $tools 'cargo'
        $env:RUSTUP_HOME = Join-Path $tools 'rustup'
        $env:PATH = "$localCargo;$env:PATH"
    }
    $rustc = Get-Command rustc.exe -ErrorAction SilentlyContinue
    if (-not $rustc) {
        $existing = Join-Path $env:USERPROFILE '.cargo\bin'
        if (Test-Path -LiteralPath (Join-Path $existing 'rustc.exe')) {
            $env:PATH = "$existing;$env:PATH"
            $rustc = Get-Command rustc.exe -ErrorAction SilentlyContinue
        }
    }
    if (-not $rustc) {
        Write-Host 'Downloading the official Rust bootstrapper into this project...'
        $uri = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
        $installer = Join-Path $tools 'rustup-init.exe'
        $checksumFile = Join-Path $tools 'rustup-init.exe.sha256'
        Invoke-WebRequest -UseBasicParsing -Uri $uri -OutFile $installer
        Invoke-WebRequest -UseBasicParsing -Uri "$uri.sha256" -OutFile $checksumFile
        $expected = ((Get-Content -LiteralPath $checksumFile -Raw).Trim() -split '\s+')[0]
        $actual = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash
        if ($expected -notmatch '^[0-9a-fA-F]{64}$' -or $actual -ne $expected) { throw 'The Rust bootstrapper checksum did not match. Installation stopped.' }
        $env:CARGO_HOME = Join-Path $tools 'cargo'
        $env:RUSTUP_HOME = Join-Path $tools 'rustup'
        & $installer -y --no-modify-path --profile minimal --default-toolchain stable --default-host x86_64-pc-windows-msvc
        Check-NativeExit 'Rust setup'
        $env:PATH = "$localCargo;$env:PATH"
    }

    if (Get-Command rustup.exe -ErrorAction SilentlyContinue) {
        & rustup.exe target add x86_64-pc-windows-msvc
        Check-NativeExit 'Preparing the Windows MSVC Rust target'
    }
    Write-Host 'Running the dependency-free Rust document-model tests...'
    & rustc.exe --edition=2021 --target x86_64-pc-windows-msvc --test 'crates\texloom-core\src\lib.rs' -o '.tools\core-tests.exe'
    Check-NativeExit 'Compiling the Rust tests'
    & '.\.tools\core-tests.exe'
    Check-NativeExit 'Rust document-model tests'
    if ($CoreOnly) { exit 0 }

    if (-not (Get-Command node.exe -ErrorAction SilentlyContinue)) {
        $nodeDirectory = Join-Path $env:ProgramFiles 'nodejs'
        if (Test-Path -LiteralPath (Join-Path $nodeDirectory 'node.exe')) { $env:PATH = "$nodeDirectory;$env:PATH" }
    }
    if (-not (Get-Command npm.cmd -ErrorAction SilentlyContinue)) {
        throw 'Node.js is needed to build the editor assets. Install the current LTS from https://nodejs.org/ and rerun this script. Installed Texloom applications do not require Node or Python.'
    }
    Write-Host 'Installing desktop/editor dependencies...'
    if (Test-Path -LiteralPath 'package-lock.json') { & npm.cmd ci }
    else { & npm.cmd install }
    Check-NativeExit 'Installing frontend dependencies'
    & npm.cmd test
    Check-NativeExit 'Frontend transaction and PDF geometry tests'
    if ($Portable) {
        Write-Host 'Building the portable Windows desktop app...'
        & npm.cmd run desktop:build -- --target x86_64-pc-windows-msvc --no-bundle
    } else {
        Write-Host 'Building the Windows desktop app and NSIS installer...'
        & npm.cmd run desktop:build -- --target x86_64-pc-windows-msvc
    }
    Check-NativeExit 'Building Texloom Desktop'
    if ($Portable) {
        $executable = Join-Path $PSScriptRoot 'target\x86_64-pc-windows-msvc\release\texloom-desktop.exe'
        if (-not (Test-Path -LiteralPath $executable)) { throw 'The build returned success but its executable was not found.' }
        Write-Host "Portable executable: $executable"
        Write-Host 'This executable needs WebView2 and a local LaTeX installation.'
        exit 0
    }
    Write-Host ''
    Write-Host 'The build completed. Installers are in target\x86_64-pc-windows-msvc\release\bundle\nsis\'
    Get-ChildItem -LiteralPath 'target\x86_64-pc-windows-msvc\release\bundle\nsis' -Filter '*setup.exe' | Select-Object -ExpandProperty FullName
    exit 0
} catch {
    Write-Host ''
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host 'No working desktop executable is claimed unless the complete build succeeds.'
    exit 1
}
