Generated native layout-verification resources live here. Run
`node scripts/prepare-native-verifier.mjs` before a Windows desktop build.
The Rust worker, PDFium DLL, exact dependency notices and manifest are copied
into the installer/portable package; none are required from Python at runtime.

This is a development dependency pin, not an approved public-release renderer.
See `docs/PDFIUM-WORKER.md` for the version, checksums and open release review.
Source editing remains available if a development build has no verifier;
normal-text visual transactions fail with an explicit reason.
