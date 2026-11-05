# Development environment snapshots

The read-only `tools/environment_manifest.py` records explicitly chosen executable, font and input-file fingerprints plus a limited machine/Python description. It probes only fixed version switches for supported tools. It does not compile, start Texloom, install packages or enumerate user projects. Output creation is exclusive: a previous manifest cannot be overwritten.

`--context development-preflight` means the snapshot was collected before a future development activity; `development-posthoc` means after it. Neither mode creates a final evaluation freeze. The final held-out set remains empty and `evaluation/freeze.json` remains `not_frozen`.

## Executed snapshot, 5 November 2026 local time

`artifacts/research-environment-20261105/posthoc-snapshot.json` records seven available tool binaries (the three LaTeX engines, PDF text extractor, rasterizer, Python and Node), four explicitly declared Windows Arial font files, five existing reference PDFs, the movement support package, dependency lockfiles, research requirements and the PDFium pin. All declared files/version probes resolved at collection time. Three environment-collector diagnostics bring the research helper suite to 22 passing checks; these are infrastructure checks, not 22 application accuracy cases.

This is a **posthoc, partial environment snapshot**. It does not prove that the current Arial files were the exact resolved fonts during the earlier pilot, or establish absence of font fallback. Fontspec declarations and retained logs are supporting evidence, not an immutable pre-run font binding. No font files are copied or licensed for redistribution by this process. The source documents can be shared separately from proprietary font binaries.

The manifest intentionally leaves RAM, WebView2 and display configuration unset. It does not inventory every shared library, TeX package or background task. Do not use it to claim a sealed reference machine, a clean-machine replication, cross-machine pixel identity or a final latency result. The existing pilot outputs are unchanged; the new snapshot does not rerun their comparisons or repair a historical provenance gap.

## Before a final evaluation

Record the product/analysis/tool hashes and complete inputs before running; retain native build receipts/recorder inputs and record actual font resolution, fallback and permitted font acquisition. Specify reproducible, appropriately licensed fonts for shareable final fixtures. Capture OS/CPU/RAM/WebView2/display/DPI/power/background-load information and the exact diagnostic-build flag. Seal fixture/oracle manifests and independent labels together. Refresh the environment record and record any changes; preserve the posthoc pilot snapshot as its own artifact.

The collector reports missing/changing executables or unreadable inputs as unresolved, never as evidence of a successful setup. Full local paths and the account's home path are omitted/redacted from output; filename/hash alone is still insufficient to reconstruct a complete installation.

## Read-only Windows context supplement, 5 November 2026

The separate `tools/windows_context.ps1` was used once for system inventory after the owner paused all tests. It reads selected local CIM properties and the documented WebView2 registration keys. No Texloom/engine/installer/version-probe program was launched, no settings were changed and no tests or experiments ran. The collector itself and Windows providers participate in this read-only activity; the snapshot is not evidence that any application workflow executed. Its exclusive output is `artifacts/research-windows-context-20261105/posthoc-context.json`.

Observed provider/registry data:

| Item | Observation | Limit |
| --- | --- | --- |
| Windows | Windows 11 Pro, version 10.0.26300, build 26300, 64-bit | Current posthoc provider report, not a sealed prior execution environment |
| CPU | Intel Core i7-11370H; four cores/eight logical processors | No sustained clock, load or thermal measurement |
| RAM | 16,856,203,264 OS/provider-reported bytes | Not an inventory of individual DIMMs or free memory during a run |
| WebView2 | Per-machine registered version 156.0.4314.58; no entry at the inspected per-user key | Not proof that Texloom selected/loaded it, that installation is healthy, or that an earlier process used that version |
| Graphics | Intel Iris Xe, driver 31.0.101.5333; controller reports 1920x1080 at 144 Hz | Not an independently verified physical monitor topology, effective DPI or sustained presentation rate |

The three selected WebView2 override variables were absent in the collector process; that does not establish an application's actual environment. Effective per-monitor DPI, physical monitor arrangement, Texloom's device-pixel ratio, loaded runtime identity and experimental power/background load remain null. The 60 Hz performance target in the protocol must not be presented as a measured property of this 144 Hz controller report. Choose and record an actual reference display/mode before timing work.

The earlier seven-tool/four-font partial snapshot is unchanged, including its null RAM/WebView2/display fields. This supplement is later evidence and cannot fill those fields retrospectively. Actual engine/font resolution, complete inputs and pre-run sealing remain open.

Primary technical references inspected 5 November: Microsoft's [WebView2 distribution/detection documentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution), [Win32_ComputerSystem](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-computersystem) and [Win32_VideoController](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-videocontroller). Registry versions and provider properties are observations with the limits above, not prerequisite/DPI acceptance tests. No hostname, account name, serial/UUID, device instance identifier or environment-variable value is exported.
