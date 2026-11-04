# Bounded runtime dependency review - 4 November 2026

The current packaged verifier uses **PDFium 156.0.8076.0**, extracted from the SHA-256-pinned pypdfium2 5.14.0 Windows x64 wheel. Historical PDFium 126 / pypdfium2 4.30 development reports are different artifacts. The app's Rust worker does not require Python.

The [official package index](https://pypi.org/project/pypdfium2/5.14.0/) and its JSON endpoints were checked on this date. The current package version was 5.14.0; the exact Windows wheel digest matched the repository pin and cached wheel, its DLL matched the packaged DLL, all pinned notice entries were present, and the packaged notice matched its manifest. The wheel's PDFium version metadata reports no optional feature flags. This is distributor metadata, not an independent proof of every build option.

PyPI provenance identifies the pypdfium2-team/pypdfium2 release workflow. Its metadata was fetched and fingerprinted; an independent cryptographic attestation verification was **not performed**. An empty package-index advisory field does not establish that embedded PDFium and every component are vulnerability-free. The [PDFium build documentation](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/README.md) distinguishes JavaScript/XFA build options; generic upstream defaults do not prove the flags of an arbitrary binary.

Source inspection confirms checked absolute library paths, a pinned DLL held against writes/deletion, bounded immutable PDF inputs, a separate worker, timeout and memory budgets. The worker is not an operating-system security sandbox. These mechanisms limit faults and evidence confusion; they are not a universal security guarantee.

Decision: retain the already tested development pin. No binary was upgraded or launched by this review. Before public distribution, complete the remaining provenance verification, component vulnerability/version review and dependency-license assessment, then rerun the relevant exact-artifact compatibility checks if the pin changes. This review supplies no legal opinion or release certification.

Receipts and primary-source response hashes: `evidence/dependency-review-2026-11-04.json`. All new work remains local.
