# GitHub development

The private primary repository is https://github.com/nadeemtsf/texloom. Use main for the integration branch and feature branches for ongoing changes. Source control, issues, reviews and Windows builds belong here.

The repository contains application source, lockfiles, design notes, synthetic regression fixtures and CI. It excludes Rust/npm caches, generated builds, user projects and private documents. Third-party components retain their own licenses.

The Actions workflow checks source transformations and frontend transactions, builds Windows x64, and uploads an unsigned portable development artifact. It uses read-only repository permissions, pinned action commits and no signing secrets. It does not publish releases or upload user documents. The action references were checked against upstream v7 tags on 2026-10-30. Remote CI success must be verified separately from local tests.

GitHub Pages cannot run the native Rust compiler service or local TeX engines. The desktop application remains local. A browser-hosted editor would need a separate compile service or a tested WebAssembly compiler.
