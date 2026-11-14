# PDFium advisory follow-up - 5 November 2026

**Later resumed work:** the active development pin now uses the reviewed 159 distributor archive. See [upgrade status](PDFIUM159-UPGRADE.md) for observed checks and unresolved nested provenance/security/release gates. The original source-only review below retains its original scope; its statement about retaining the old pin is historical.

**Public distribution remains blocked pending dependency remediation and validation.** The current development pin is retained for reproducibility; this is not security approval of that pin. This review reads public source/advisory metadata only. No app, DLL, exploit, vulnerability scanner, upstream code or test was executed. All tests remain paused.

## Finding and evidence strength

Texloom's recorded pin is the Windows x64 pypdfium2 5.14.0 wheel, PDFium 156.0.8076.0. Wheel/DLL identities remain those in `scripts/pdfium-pin.json`; the Rust application uses its DLL, not the Python wrapper at runtime. Wheel publication authentication and matching hashes do not establish vulnerability absence.

The [Chrome team's 13 October release notice](https://chromereleases.googleblog.com/2026/10/stable-channel-update-for-desktop_01668083673.html) identifies CVE-2026-110155 as a high-severity PDFium buffer overflow, bug 570167922. Its Chrome release number must not be used as a standalone PDFium version comparison. The [PDFium fix](https://pdfium.googlesource.com/pdfium/+/58e11305710a506ce3369581662eeb14440639cf) is a separate immutable source identity, dated 6 October 2026, changing AGG current-cell initialization and describing an out-of-bounds access prevented by that change.

The [pypdfium2 issue](https://github.com/pypdfium2-team/pypdfium2/issues/487) explicitly links release 5.14.0/build 8076 to that missing fix; a maintainer acknowledged the report. The [PyPA database issue](https://github.com/pypa/advisory-database/issues/401) proposes a package mapping. An issue proposal is not an already merged advisory database entry. An empty package advisory list cannot override this evidence.

Read-only Gitiles capture resolved `refs/heads/chromium/8076` to `8ca5b735df4263f43c830479b78213854a392c22`. Its AGG source retains the old initialization in constructor/reset; the fixed source contains the replacement. This corroborates the **source-level** finding for that branch. The wheel still lacks an exact PDFium source hash, so this observation does not independently recover the DLL's entire source/build/patch relationship or demonstrate a working Texloom exploit. Treat the pinned distributor mapping plus source finding as an unresolved release blocker, not a reason to claim a successful exploit or a complete affected-feature analysis.

## Other leads and upgrade boundary

The same upstream discussion identifies later fixes. Primary commit metadata was read for [OpenJPEG allocation handling](https://pdfium.googlesource.com/pdfium/+/0a42f6e64f28603f3aee118a1c140e9bd0647b71), [JPEG input repair](https://pdfium.googlesource.com/pdfium/+/4150fb564e0d83109f33a06385d22bb13cc3cd97), [JavaScript property enumeration](https://pdfium.googlesource.com/pdfium/+/cce5c5ed801f54cadb5b039d98ed43804994764c) and [form-field lifetime handling](https://pdfium.googlesource.com/pdfium/+/0a21f66dd0c665dc4d6dbdafc150e93528cb0020). These are follow-up leads; the corresponding CVE mappings, actual compiled paths and reachability in this DLL/worker were not fully adjudicated. Empty wheel flags and workflow defaults do not prove absence of a feature.

Consequently, selecting a build solely because it includes the AGG fix would not complete the dependency review. The latest-release API still returned pypdfium2 5.14.0 at this inspection. No replacement artifact has been selected, downloaded, loaded or substituted.

## Required next work

Select a replacement with recorded source/build provenance, reviewed applicable fixes and retained component notices. Bind its exact Windows DLL, compatible API declarations and packaged resources; preserve historical binaries as historical evidence. When tests resume, repeat verifier/engine/refusal/packaging and actual installed-app acceptance on that exact replacement. A separate bounded process is useful fault containment, but is not an OS sandbox or a substitute for remediation.

Raw GitHub responses, the rejected encoded release-tag endpoint, and Gitiles byte fingerprints are retained in ignored `artifacts/dependency-advisory-review-20261105/`. The browser fallback supplied release-page context; its branch-file view was unavailable, while direct read-only Gitiles data retrieval succeeded. No issue/comment was posted and no upstream contributor was contacted. Remaining complete component/license/advisory review is still open.
