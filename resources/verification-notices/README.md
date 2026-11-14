# Supplemental PDFium component notices

The matching upstream Windows x64 build job compiled Brotli/libyuv and emitted unknown-library license warnings. The adopted release archive has no named notice files for these components. These two upstream license texts are retained conservatively in addition to every existing distributor notice.

Revisions come from DEPS at the previously observed PDFium source commit5f092b52993bfd98f94f04deff391918fe0fdcfd. Exact DLL nested checkout/DEPS provenance remains unresolved; this is not a complete SBOM, license clearance or vulnerability audit. URLs, revisions and byte hashes are in manifest.json. Build preparation requires both declared files and verifies their exact bytes before appending them to PDFIUM-NOTICE.txt and recording them in verifier-manifest.json. It performs no license download at build time.
