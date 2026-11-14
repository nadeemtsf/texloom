#requires -Version 7.0
<#
Read-only, partial Windows research inventory. No app/engine launch or tests.
Only explicitly selected non-identifying properties are written. No settings
are changed, and an existing output file cannot be overwritten.
#>
param([Parameter(Mandatory=$true)][string]$OutputFile)

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'This inventory requires Windows.' }

function Read-CimContext {
    param([string]$Class, [string[]]$Properties)
    try {
        $items = @(Get-CimInstance -ClassName $Class -Property $Properties -OperationTimeoutSec 10 |
            Select-Object -Property $Properties)
        return [ordered]@{status='observed';source=$Class;items=$items}
    } catch {
        return [ordered]@{status='unresolved';source=$Class;error_type=$_.Exception.GetType().Name;items=@()}
    }
}

function Read-WebViewRegistration {
    param([string]$Scope, [string]$RegistryView, [string]$Path)
    $entry = [ordered]@{scope=$Scope;registry_view=$RegistryView;status='unresolved';registered_version=$null}
    try {
        $properties = Get-ItemProperty -LiteralPath $Path -Name pv -ErrorAction Stop
        $entry.registered_version = [string]$properties.pv
        $entry.status = if ($entry.registered_version -match '^\d+\.\d+\.\d+\.\d+$' -and
                $entry.registered_version -ne '0.0.0.0') { 'observed' } else { 'unresolved' }
    } catch [System.Management.Automation.ItemNotFoundException] {
        $entry.status = 'not_registered_at_this_key'
    } catch {
        $entry.error_type = $_.Exception.GetType().Name
    }
    return $entry
}

# The documented application GUID is global, not a device/account identifier.
$runtimeId = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
$machineKey = if ([Environment]::Is64BitOperatingSystem) {
    "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\$runtimeId"
} else {
    "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\EdgeUpdate\Clients\$runtimeId"
}
$registrations = @(
    Read-WebViewRegistration 'per_machine' $(if ([Environment]::Is64BitOperatingSystem) {'32_on_64'} else {'32'}) $machineKey
    Read-WebViewRegistration 'per_user' 'current_user' "Registry::HKEY_CURRENT_USER\Software\Microsoft\EdgeUpdate\Clients\$runtimeId"
)
$overrides = [ordered]@{}
foreach ($name in @('WEBVIEW2_BROWSER_EXECUTABLE_FOLDER','WEBVIEW2_RELEASE_CHANNEL_PREFERENCE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')) {
    $overrides[$name] = -not [string]::IsNullOrEmpty([Environment]::GetEnvironmentVariable($name, 'Process'))
}

$inventory = [ordered]@{
    schema = 1
    kind = 'read_only_windows_context_not_test_or_final_environment'
    context = 'development-posthoc'
    recorded_utc = [DateTimeOffset]::UtcNow.ToString('o')
    collector_file = 'research/tools/windows_context.ps1'
    collector_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    powershell_version = $PSVersionTable.PSVersion.ToString()
    os = Read-CimContext 'Win32_OperatingSystem' @('Caption','Version','BuildNumber','OSArchitecture')
    memory = Read-CimContext 'Win32_ComputerSystem' @('TotalPhysicalMemory','NumberOfLogicalProcessors')
    cpu = Read-CimContext 'Win32_Processor' @('Name','NumberOfCores','NumberOfLogicalProcessors')
    video_controller_reports = Read-CimContext 'Win32_VideoController' @('Name','DriverVersion','CurrentHorizontalResolution','CurrentVerticalResolution','CurrentRefreshRate')
    webview2_registry = $registrations
    collector_process_override_presence = $overrides
    actual_texloom_loaded_webview2 = $null
    physical_monitor_topology = $null
    effective_per_monitor_dpi = $null
    texloom_device_pixel_ratio = $null
    power_and_background_load_during_experiments = $null
    app_opened = $false
    tests_or_experiments_executed = $false
    limitations = @(
        'A current posthoc observation does not establish the machine or runtime during an earlier experiment.'
        'Registry values describe registrations, not a successfully loaded or functional Texloom runtime.'
        'Environment override presence belongs to this collector process, not a verified application launch.'
        'Video-controller values are provider reports, not a physical monitor inventory or a DPI test.'
        'TotalPhysicalMemory is OS/provider-reported memory, not an inventory of individual installed DIMMs.'
        'No hostname, username, machine serial/UUID, device instance IDs, full paths, project data or environment values are recorded.'
        'No runtime files, fonts, packages or tools are installed, copied, compiled or launched.'
    )
}
$destination = [IO.Path]::GetFullPath($OutputFile)
$json = $inventory | ConvertTo-Json -Depth 10
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json + "`n")
$stream = [IO.File]::Open($destination, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
Write-Output 'Read-only Windows context saved. No application, compiler or test was run.'
