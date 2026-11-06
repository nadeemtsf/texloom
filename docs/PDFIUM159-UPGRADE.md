# Development verifier upgrade to PDFium 159

6 November current status: the replacement engine matrix and ordinary/NSIS/portable builds completed; older pending statements below describe their earlier checkpoints. The later build-pairing guard and independently checked 16-entry portable archives supersede the earlier 15-entry packaging format. The r7 follow-up also passes both explicit native verifier checks, three-language zero probes, four RTL moves and nine Latin fractional moves; actual r7 installed/UI acceptance and wider provenance/license/security gates remain pending. See OWNER-FOLLOWUP-2026-11-06.md. No public release is certified.

The active development pin selects distributor `chromium/8138`, version metadata 159.0.8138.0. Archive SHA-256: `8d48973e3eb472dda0f5f2435e595ac043f9f2219dd71883392f549445afa8a5`; DLL SHA-256: `71a3e63e6ab8f5746a0fc9de9bc3d0c0dc07eb60990dfd37d9db7fc891821943`. Old binaries, failures and references remain historical evidence.

## Implemented

The active pin declares all 49 member hashes. The bounded in-memory TGZ reader distinguishes review and active manifests and rejects altered data, unsafe paths and links. Desktop preparation checks the exact archive/DLL, retains all 19 root/component notices (FreeType uses Windows-1252), builds the isolated worker and fingerprints resources. Native parent/child share version/hash constants checked against the packaging pin. The historical Python reference preparer separately reads `pdfium-proof-pin-156.json`.

Portable packaging now requires an explicit executable path and expected SHA-256, instead of silently selecting a stale target-triple directory. It checks verifier resources against the active pin. Actual replacement archive creation/roundtrip validation remains pending.

## Evidence boundary

GitHub CLI verified the archive SLSA statement against `bblanchon/pdfium-binaries`, `.github/workflows/build-all.yml` and build-repository commit `0eb94b28c2252d2baae3dfe15dc6b237131d8cd8`. Wrong-repository and altered-byte controls failed. Observed PDFium source ref `5f092b52993bfd98f94f04deff391918fe0fdcfd` has the corrected AGG initialization discussed in DEPENDENCY-ADVISORY-REVIEW.md. These distinct identities and release-assembly attestation do not fully establish every nested source/build input or absence of other vulnerabilities.

Replacement checks: 273 frontend/protocol passes; 55 build-library passes; five worker passes, including the retained historical reference's glyph/graphics/raster comparison; two native approval passes. First approval invocation failed for a missing worker-path variable, then its corrected repeat passed. Logs: `artifacts/resumed-validation-20261105-run1/`, generally `pdfium159-` prefixed.

At the computer-control stop, adapter/fractional/template/marked-content repeats and release/NSIS build remained active. Inspect existing logs/sessions before repeating. Actual replacement GUI/installation/portable acceptance, bounded nested provenance/license/security review and wider environment/performance/final-study gates remain open. Public distribution is unapproved; no security certification or final evaluation is claimed.

## Checkpoint follow-up

The replacement ordinary release and NSIS build completed. The real portable package passed its own receipt checks and an independent Python CRC/exact 15-entry/per-file hash audit. Two initial auditor path assumptions failed; the unchanged archive passed after its actual enclosing-folder layout was used. Exact artifacts and log hashes are recorded in `docs/evidence/resumed-validation-2026-11-05.json`. Replacement engine gates at checkpoint: already running; do not start duplicates. Replacement GUI/installation and wider release gates remain open.
