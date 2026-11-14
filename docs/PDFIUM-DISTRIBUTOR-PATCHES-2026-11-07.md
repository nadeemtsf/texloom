# Recorded distributor patches — bounded provenance follow-up

This read-only review uses distributor commit `0eb94b28c2252d2baae3dfe15dc6b237131d8cd8` and the already matched [Windows x64 job](https://github.com/bblanchon/pdfium-binaries/actions/runs/41803647331/job/126485258606). The job's artifact matches all 49 regular release-archive files, as recorded in PDFIUM-NOTICE-FOLLOWUP-2026-11-07.md. No patch, upstream script or DLL was executed or changed by this review.

The immutable [patch-selection script](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/steps/03-patch.sh) and four patch files were captured as data. All five captures match their Git blob identities as well as having SHA-256 fingerprints. The matching job records four patch invocations, six relative target paths and 11 successful hunks. Several hunks applied at line offsets; the extracted patch-step log contains no failed/rejected/reversed/fuzzy-application marker. These are observations of the recorded job, not a local replay.

| Logged patch | Reviewed change | Application directory |
| --- | --- | --- |
| [shared_library.patch](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/patches/shared_library.patch) | Changes the library target, includes Windows resources and adjusts public export macros. | PDFium root |
| [public_headers.patch](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/patches/public_headers.patch) | Rewrites includes in the C++ deleter/scoper headers for their packaged layout. | PDFium root |
| [clang_rt.patch](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/patches/clang_rt.patch) | Alters unsupported-platform assertions and conditional runtime-library lookup. The log establishes application, not which modified branches ran. | Nested `build` checkout |
| [win/build.patch](https://github.com/bblanchon/pdfium-binaries/blob/0eb94b28c2252d2baae3dfe15dc6b237131d8cd8/patches/win/build.patch) | Changes the resource-compiler command from a Python wrapper to direct invocation with an environment define. | Nested `build` checkout |

The recorded invocation has Windows/x64, shared build, V8 disabled and version 159.0.8138.0. Those settings agree with the script's selection of these four patches. They do not independently prove absence or reachability of every binary feature. The script also generates `resources.rc` through environment substitution; its exact rendered file is not recovered here.

An initial local capture incorrectly trimmed trailing blank lines. Git blob verification detected the mismatch before the receipt was accepted. The first capture remains under the ignored raw-evidence directory; exact connector strings were then restored and all five blob checks passed. This data-handling failure is separate from the upstream build record.

This narrows the earlier unresolved patch-identity item. Exact resolved PDFium/nested checkout revisions, the complete post-patch tree, source-to-DLL reproducibility, component inventory and applicable advisory/reachability analysis remain unresolved. No security or legal clearance is claimed, and issue #35 stays open. The immutable receipt is `docs/evidence/pdfium-distributor-patches-2026-11-07.json`; raw captures and the extracted log are under ignored `artifacts/continuation-20261107-weekly57/distributor-patches/`.
