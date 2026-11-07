# Bounded dependency build review — 7 November 2026

Later matched-artifact review is in PDFIUM-NOTICE-FOLLOWUP-2026-11-07.md: the exact Windows x64 job's artifact matches all49 active archive files, including args.gn, and exposes two concrete license-collector warnings. The source-only uncertainty statements below retain their original scope. Conservative supplemental notices are now retained; exact nested checkout and wider review remain open.

This read-only follow-up checks the distributor build repository at the previously verified attestation's commit `0eb94b28c2252d2baae3dfe15dc6b237131d8cd8`. No upstream script was executed, DLL changed, exploit run or security clearance issued. GitHub issue#35 remains open.

The active archive's SHA-256 still matches `scripts/pdfium-pin.json`. Its VERSION and CMake package metadata declare159.0.8138.0. Those files identify the distributed package; they do not identify every build input.

At that immutable distributor commit, [checkout](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/steps/02-checkout.sh) uses shallow `gclient sync` at `origin/${PDFium_BRANCH:-main}`. [Configuration](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/steps/05-configure.sh) defaults ENABLE_V8 to false and assigns both pdf_enable_v8 and pdf_enable_xfa from it. These are reviewed configuration declarations, not independent proof of this DLL's actual flags or feature reachability.

The [license collector](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/steps/08-licenses.sh) has explicit ignored cases and a warning for unknown libraries. Retaining the archive's declared notices is therefore valuable but does not itself prove complete component/license coverage. The release workflow's branch description and publication attestation do not close the nested source-to-DLL gap.

Next useful evidence is the exact build invocation's resolved checkout/DEPS, generated GN arguments, patch identities, component/dependency graph and license warnings. Where unavailable, leave the relationship unresolved. Applicable advisory/reachability review remains separate. Captured source bytes/URLs/hashes are recorded in `docs/evidence/dependency-build-followup-2026-11-07.json` and ignored raw artifacts. No new application test or full security/license audit is claimed.
