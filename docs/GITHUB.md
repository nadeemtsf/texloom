# GitHub plan

GitHub fits this project well for version history, issues, reviews and repeatable Windows builds. Start with a private repository named `texloom`; publish it later if an open-source release is desired. Repository visibility is an owner decision. No remote repository has been created or pushed by this preparation.

The repository contains application source, lockfiles, design notes, synthetic regression fixtures and CI. It excludes Rust/npm caches, generated builds, user projects and private documents. The MIT declarations in the existing manifests are accompanied by LICENSE; third-party components retain their own licenses.

The prepared Actions workflow checks source transformations and frontend transactions, builds Windows x64, and uploads an unsigned portable development artifact. It uses read-only repository permissions, pinned action commits and no signing secrets. It does not publish releases or upload user documents. The action references were checked against their upstream v7 tags on 2026-10-30.

After creating an empty private repository under the desired GitHub account, connect this directory using `git remote add origin <repository-url>` and `git push -u origin main`. An empty remote avoids an unrelated initial README commit. Review the source-only file list before pushing. Require the Windows check before merging once the first remote workflow succeeds.

GitHub repository hosting stores and builds the desktop application's source. GitHub Pages cannot run the native Rust compiler service or local TeX engines. The desktop application remains local. A future browser-hosted editor would need a separate authenticated compile service or a tested WebAssembly compiler, with quotas, project isolation and different operating costs.

GitHub account plans and included Actions quotas can change; check the account's current billing page before enabling frequent private builds. No paid service or spending limit was configured here.
