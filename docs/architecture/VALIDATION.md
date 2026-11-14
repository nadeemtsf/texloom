# Validation of this planning package and private-project cleanup

Date: 7 November 2026. This record does not validate any proposed visual editing feature or change the existing app/research acceptance status.

## Architecture and backlog checks

- 24 unique task definitions across four new milestones; dependency references resolve and the planned graph is acyclic.
- Local architecture-document links checked, with a complete Markdown and structured backlog/index.
- GitHub parent #44 has 24 native sub-issues #45–#68. All 24 task identities, open states and milestone assignments were read back.
- 97 planned blocking dependencies were configured and read back in three bounded batches: 14, 36 and 47 added edges. Each issue's planned prerequisites were checked through the blocked-by API. No task was closed as implemented.
- Existing tracker #25 and feature inbox #42 now point to the broader roadmap. Existing acceptance issues stay open. Issue #36 records the private-only distribution decision without treating pending usability evidence as passed.
- Design review corrected explicit requirements for noncancelable composition updates, bounded visible-text/source mapping, cluster-level caret geometry, clipboard/deletion ordering and legitimate dependent numbering/reference/contents effects.

These are plan/metadata checks, not application correctness experiments or independent scientific review of final labels/oracles.

## Separately requested private-project cleanup

The owner requested removal of project contribution/licensing files and stated that the repository should remain private now and in the future.

- Root `LICENSE` and `CONTRIBUTING.md` were deleted on GitHub `main` through the contents API and removed from the local working tree. Remote root contents were read back without either file. Previous versions remain recoverable in Git history; no history rewrite was performed.
- Texloom-owned Rust manifests no longer declare MIT and now disable package publishing. All three local packages were checked with locked offline Cargo metadata. Only the two package manifests already present on remote `main` were edited there; unpublished application code was not copied to the remote.
- The README replaces Store planning with private-project direction and the live roadmap link. The architecture/product roadmap excludes public/Store distribution. Repository privacy was read back as true.
- Dependency licensing metadata and notices were retained. The reviewed Brotli and libyuv texts still match their expected SHA-256 bytes; no dependency source/resource changes were made.

## Required existing regressions after metadata changes

| Check | Observed result | Scope |
| --- | --- | --- |
| `npm test` | 291 passed, 0 failed, 0 skipped | Existing frontend/protocol/filesystem regressions |
| `cargo test --offline --locked -p texloom-core` | 70 unit and 18 integration passed | Existing core source/history/domain contracts |
| `cargo metadata --offline --locked --no-deps --format-version 1` | Three local Texloom packages parse; no own license declaration; publishing disabled | Package metadata only; does not build or install the app |
| Roadmap validator / `git diff --check` | Passed after completion | Plan structure, local links and diff formatting |

Full local regression logs are under ignored `artifacts/architecture-roadmap-20261107/`. The initial pre-generation link check correctly reported missing `BACKLOG.md`; the file was generated and the completed package was checked again. No missing document was represented as a pass.

No app launch, installation, native mouse/keyboard control, TeX-engine experiment, new application release build, final research freeze/evaluation, signing, public release or report submission occurred. r8 remains the previously prepared, owner-unaccepted runtime candidate. The architecture tasks remain proposals for later implementation.
