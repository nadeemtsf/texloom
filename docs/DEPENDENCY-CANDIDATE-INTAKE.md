# PDFium replacement intake - data and source only

Prepared 5 November 2026 while all tests are paused. Public release metadata, two compressed archives, headers/notices and selected upstream source were read as inert data. No library was loaded, archive member installed/extracted, app opened, source compiled, exploit/scanner/experiment run or packaged pin changed. This is not a compatibility pass or completed remediation.

## Recorded archive relationships

The GitHub connector's release **collection** endpoint supplied both releases after its encoded-tag route failed. Binary responses are outside that connector's text capability, so official release assets were downloaded into an ignored intake directory through HTTPS. Their observed archive digests agree with the distributor API's published digests. This is an integrity observation, not verified signed build provenance.

| Archive | Observed SHA-256 | DLL SHA-256 inside archive |
| --- | --- | --- |
| [chromium/8076 Windows x64](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F8076) | `808d36da9bc5a3104315fb307c80998121f565ee53953633bf33e80d7429e5ac` | `69f1e860f6742fbaafcc4db80e785eec9a4ee0da36ea6d1dc0867ea0f18baea6` |
| [chromium/8138 Windows x64](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F8138) | `8d48973e3eb472dda0f5f2435e595ac043f9f2219dd71883392f549445afa8a5` | `71a3e63e6ab8f5746a0fc9de9bc3d0c0dc07eb60990dfd37d9db7fc891821943` |

The older archive's DLL matches the previously pinned wheel/packaged DLL byte identity. This strengthens the distributor-artifact link beyond the wheel's null source-hash metadata, without proving the entire source-to-DLL build. Its `VERSION` says 156.0.8076.0; the newer archive says 159.0.8138.0. Both ship `args.gn` declaring Windows x64 release, V8/XFA false and partition allocator false. Those are distributor-supplied build-argument records associated with the archive, not an independent binary feature attestation.

The archive inventories record 45 and 49 regular entries respectively. Reads are bounded and no archive paths become output destinations. The newer notice list adds Highway, HarfBuzz and two Dragonbox notice files relative to the old list. Copying the current eighteen wheel notice paths would therefore be insufficient as a replacement packaging recipe. Presence is not a completed legal/component-coverage assessment.

## Source and API observations

The distributor's newer release metadata points at build-repository commit `0eb94b28c2252d2baae3dfe15dc6b237131d8cd8`. This is distinct from the observed PDFium branch head `5f092b52993bfd98f94f04deff391918fe0fdcfd`. The latter source's AGG file has the corrected initialization. Its OpenJPEG README records upstream revision `6c4a29b00211eb0430fa0e5e890f1ce5c80f409f`, version 2.5.4 and the image-header-copy patch; the imported `src/image.c` returns/propagates failure. These source observations are relevant to the remediation leads, not proof that every applicable fix is present in the actual DLL.

A reading aid located the 31 function declarations used by the worker in both archives' headers. The declaration texts are identical across those two header sets. This neither checks symbol exports/struct layouts at runtime nor proves semantic/render compatibility. The initial lookup also matched comment references; it and its incomplete counts are retained, with a corrected comment-stripping lookup separately labelled. Neither is an application test.

Manual comparison exposed an existing mismatch: the headers declare `FPDFBitmap_FillRect` as returning `FPDF_BOOL`, while the Rust binding declared no result. Source now uses an integer return and checks it before rendering/reading a raster. A false result refuses inspection; it cannot become approved output evidence. The code change is unbuilt and untested, and does **not** repair the separate AGG vulnerability. Old binaries/receipts do not contain this guard.

## Replacement work still required

8138 is a **candidate for further review**, not the selected or deployed replacement. Before adopting any candidate:

1. Bind exact source, patches, build arguments/toolchain, distribution signatures and nested components to the DLL; review all applicable fixes and component notices.
2. Update `scripts/pdfium-pin.json`, the fixed constants/entry layout in `scripts/prepare-native-verifier.mjs`, `inspection_worker::DLL_SHA256`, API declarations if needed, and generated verifier resources/manifest together. Direct distributor TGZ and pypdfium2 wheel layouts are different; the current wheel-only extractor cannot simply be pointed at this archive.
3. Resume required worker, source/layout/refusal, three-engine, packaging and actual Windows acceptance on the exact candidate and rebuilt app. Rendering/text extraction changes can affect tolerances and Arabic/Hebrew cases even when C declarations agree. Preserve failures and versions; do not tune the held-out set.

Raw intake data and reading limits are under ignored `artifacts/dependency-candidate-intake-20261105/`. One source request used the old OpenJPEG path and returned not-found; reading the primary tree located `src/`. No upstream instructions in READMEs were executed. Existing r4 outputs and installed r2 remain unchanged. Public distribution stays blocked.

## Source-only preparation for the different archive layout

`scripts/pdfium-review-candidate-8138.json` records the observed candidate archive and complete regular-file inventory, explicitly `adopted: false`. It is separate from the active wheel pin. `scripts/pinned-pdfium-tar.mjs` is an unexecuted in-memory reader intended for that exact digest and declared entries. It checks compressed identity before bounded decompression, tar header checksums/types/ranges, portable unique names, entry digests and complete declared-file coverage. Links, extension records and special files are refused. No archive path is used for a filesystem write.

The active preparation script does not import this module. No test, syntax/type check, module import or parser invocation has run; the manifest's presence and source review are not successful archive-reader validation. It does not fetch, select, install or load a replacement, verify signed build provenance, rebuild the worker, or close remediation/compatibility gates. Integrating it into packaging remains dependent on review and resumed tests. The archive's raw GNU header layout was read separately as metadata only; that reading is not execution of the new implementation.
