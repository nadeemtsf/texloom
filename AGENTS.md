# Texloom repository

The primary repository is https://github.com/nadeemtsf/texloom. Use main and origin/main for ongoing development. This Rust/Tauri desktop implementation is the active project; the earlier Python app and downloaded ZIPs are historical artifacts.

Read docs/SYSTEM-DESIGN.md and VALIDATION.md before visual editing changes. Keep source authoritative, and compile and render candidates before committing source/history. Preserve user projects and private correspondence.

Run npm test and cargo test --locked -p texloom-core for source changes. Native PDF interaction and engine compatibility checks are required before claiming a release is ready. Keep caches, generated builds and private documents out of Git.

Before every commit, stage all intended files and run the installed pre-commit gate (`npm run hooks:install` configures this checkout). It runs `scripts/check-ci.ps1` against the staged tree, including native inspection, real-engine corpora, build/package and benchmark checks. Failures, unstaged/untracked inputs or changes during testing block the commit. Never bypass the hook or claim hosted CI has passed from local results. Push completed commits to the current development branch, then inspect GitHub checks; investigate failures before treating the branch as ready to merge. Actual app GUI/input checks remain owner-run.

Merge completed work into `main` without additional owner confirmation once the full local gate and applicable GitHub checks have passed for the exact changes being merged, and GitHub's review and branch rules are satisfied. Resolve conflicts and validate the integrated result first. Continue from updated `main` after merging. Reuse the current branch for unfinished work instead of creating copies of feature branches.
