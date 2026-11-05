# PDFium build-source follow-up - 5 November 2026

Scope: read existing wheel metadata and primary upstream source, without installing/loading a DLL, rebuilding, running tests, scanning advisories or changing a dependency pin. This is not validation of the new application source, a reproducible build, a complete license review or a release decision. The owner's all-tests pause remains in force.

## Recorded identities

The existing pin remains PDFium 156.0.8076.0 from pypdfium2 5.14.0. The cached wheel's two version records were read as data. `pypdfium2_raw/version.json` declares `origin: pdfium-binaries`, `flags: []`, and **`hash: null`**. The wrapper version record also has a null hash. These files do not bind an exact upstream PDFium source commit. Their original bytes and SHA-256 values are retained, not rewritten.

The primary [PDFium-binaries release page](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F8076) names PDFium 156.0.8076.0 and branch `chromium/8076`. Its repository commit resolves to [`f2e9a1c45bb17b85b540abf1af30146ef65416ac`](https://github.com/bblanchon/pdfium-binaries/commit/f2e9a1c45bb17b85b540abf1af30146ef65416ac). That is the **build-repository commit**, not a verified PDFium source commit or a digest of the packaged DLL. These identities must not be substituted for one another.

## Static configuration observations

| Source at that build-repository commit | Observation | What it does not establish |
| --- | --- | --- |
| [Reusable build workflow](https://github.com/bblanchon/pdfium-binaries/blob/f2e9a1c45bb17b85b540abf1af30146ef65416ac/.github/workflows/build.yml) | Declares `enable_v8` default false and passes it to configuration; Windows jobs select Windows 2022. The workflow contains build/test steps. | No historical workflow run, argument values or upstream test result was collected or executed here. A workflow definition is not proof that its checks passed for this wheel. |
| [Checkout step](https://github.com/bblanchon/pdfium-binaries/blob/f2e9a1c45bb17b85b540abf1af30146ef65416ac/steps/02-checkout.sh) | Synchronizes `origin/${PDFium_BRANCH}` with shallow/no-history options; non-V8 checkout requests the minimal configuration. | A branch name is not an immutable record of the exact PDFium checkout used for a historical artifact. |
| [GN configuration template](https://github.com/bblanchon/pdfium-binaries/blob/f2e9a1c45bb17b85b540abf1af30146ef65416ac/steps/05-configure.sh) | Sets standalone mode, ties `pdf_enable_v8` and `pdf_enable_xfa` to the same input, and sets `pdf_use_partition_alloc = false`. | Source defaults and empty wheel flags are consistent with a non-V8 build, but do not independently prove the packaged DLL's actual flags, allocator or enabled features. |
| [License collection step](https://github.com/bblanchon/pdfium-binaries/blob/f2e9a1c45bb17b85b540abf1af30146ef65416ac/steps/08-licenses.sh) | Derives component names from `build.ninja`, copies/extracts known notice files, and emits a warning for an unknown name. | A warning rather than a hard failure means successful collection is not proof that every newly encountered component was covered. No specific missing license in Texloom is established by this observation. |

The wheel's metadata lists the notice paths retained by the current eighteen-entry pin. Two captured notice texts identify the build distributor and PDFium; other components retain separate notice paths. Presence and prior hash checks are useful packaging evidence, not a full component-version/SBOM inventory or a legal compatibility conclusion. The complete actual build graph was not reconstructed in this continuation.

No upstream shell script or workflow step was executed. The GitHub connector retrieved read-only public files and metadata. Its first release-tag API URL rejected an encoded repository path; the available public release page and pinned repository file reads supplied the source context. The error and decoded source responses are retained under ignored `artifacts/dependency-source-review-20261105/`.

## What remains before public distribution

1. Bind the actual PDFium checkout, build arguments, patches, toolchain and nested component revisions to the wheel/DLL through recorded build artifacts or a separately reviewed reproduction. A publish attestation alone authenticates the wheel publication, not this complete nested build relation.
2. Derive component/license coverage from the actual compiled graph and retained notices, including unknown-library warnings and any platform-specific omissions. Determine redistribution requirements for the exact distribution separately; do not infer a blanket conclusion from root license labels.
3. Review relevant primary advisories and version changes against those exact component/build identities. No vulnerability scan or claim of vulnerability absence was made here. The package-index latest-version observation remains the dated 4 November check.
4. If a dependency changes, repeat exact-artifact engine/verification/packaging acceptance when tests resume. Do not silently replace the development pin while all validation is paused.

The current worker remains a separate bounded process, not an OS security sandbox. Nothing in this static follow-up certifies its isolation, the dependency's security or public-release readiness. Signing and publication remain later owner decisions.
