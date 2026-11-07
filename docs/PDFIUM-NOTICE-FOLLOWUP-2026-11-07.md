# Matched Windows build artifact and supplemental notices

## Evidence obtained

The primary [Windows x64 job126485258606](https://github.com/bblanchon/pdfium-binaries/actions/runs/41803647331/job/126485258606) completed successfully at distributor commit0eb94b28c2252d2baae3dfe15dc6b237131d8cd8. The associated artifact13309513718 has ZIP digest5cabaa06b4056ea11f72ccdf3fa26d54fdcf73305c6eb055f2709d6ff33df33d, matching the job's upload record. All49 regular files match the adopted release archive exactly: DLL, import library, public headers, VERSION, args.gn and notices. This narrows the earlier source-only configuration uncertainty without claiming a reproducible source-to-binary derivation.

The matching archived args.gn records release/static-component Windows x64 configuration, pdf_enable_v8=false, pdf_enable_xfa=false and pdf_use_partition_alloc=false. The job log records those settings too. These are build-record/configuration observations, not reverse-engineered proof that a particular feature is unreachable in every binary path.

## Concrete notice finding and response

The job compiled Brotli/libyuv objects and its license collector warned `Unknown library brotli` and `Unknown library libyuv`. The archive's notice files contain neither component name. This is a concrete notice-inventory finding, not a legal compliance verdict.

Development preparation now preserves all19 distributor notices and appends the two reviewed upstream license texts. The immutable component revisions are taken from DEPS at the previously observed PDFium source ref5f092b52993bfd98f94f04deff391918fe0fdcfd. The exact DLL's nested checkout remains unresolved, so these are conservative supplemental notices, not an exact-version SBOM or complete license/security clearance.

The local manifest binds file names, byte counts, SHA-256, source revisions and URLs. Packaging refuses changed bytes, missing/duplicate components, path escape or mismatched source identity. License files bypass Git newline conversion so a Windows checkout cannot silently change the hashed upstream bytes. No upstream script was executed. New development binaries/packages need their own build/resource receipt and owner acceptance; old artifacts remain historical.

GitHub issue#43 tracks this bounded packaging fix. Broader dependency review#35 stays open for resolved source/DEPS/patch/component provenance and applicable advisory review. Raw logs/artifact ZIP/metadata and failed test setup remain under ignored `artifacts/continuation-20261107-weekly57/`.
