# Texloom repository

The primary repository is https://github.com/nadeemtsf/texloom. Use main and origin/main for ongoing development. This Rust/Tauri desktop implementation is the active project; the earlier Python app and downloaded ZIPs are historical artifacts.

Read docs/SYSTEM-DESIGN.md and VALIDATION.md before visual editing changes. Keep source authoritative, and compile and render candidates before committing source/history. Preserve user projects and private correspondence.

Run npm test and cargo test --locked -p texloom-core for source changes. Native PDF interaction and engine compatibility checks are required before claiming a release is ready. Keep caches, generated builds and private documents out of Git.

While iterating, run the checks closest to the change. Stage the implementation, relevant tests and documentation together before running the full commit gate. Reuse Cargo build output within the same checkout. If using `CARGO_TARGET_DIR`, assign a distinct directory to each checkout. Let the verifier check any cached archive against its pin. Reuse the gate's portable package when its receipt matches the candidate tree and build configuration.

Before every commit, stage all intended files and run the installed pre-commit gate (`npm run hooks:install` configures this checkout). It runs `scripts/check-ci.ps1` against the staged tree, including native inspection, real-engine corpora, build/package and benchmark checks. Failures, unstaged/untracked inputs or changes during testing block the commit. Never bypass the hook or claim hosted CI has passed from local results. Push completed commits to the current development branch, then inspect GitHub checks; investigate failures before treating the branch as ready to merge. Actual app GUI/input checks remain owner-run.

Merge completed work into `main` without additional owner confirmation once the full local gate and applicable GitHub checks have passed for the exact changes being merged, and GitHub's review and branch rules are satisfied. Resolve conflicts and validate the integrated result first. Confirm feature behavior with the established automated tests. Reuse relevant recorded owner acceptance for workflows already tested. Ask the owner to test when physical input, installation or other manual behavior is necessary to verify the change. Continue from updated `main` after merging. Reuse the current branch for unfinished work instead of creating copies of feature branches.

Batch new hands-on checks into one checklist when they can use the same candidate. Check hosted CI by pull request and use quiet waits between checks.

## Technical report

Edit the canonical LaTeX project in `research/report/`. Put generated PDFs, logs, extracted text and any page renders in the Git-ignored `artifacts/research-report/` directory.

For prose edits, follow the report README to compile `main.tex`, check the final log for errors, unresolved citations or references, missing glyphs and overfull boxes, and verify the changed passage in extracted PDF text. Inspect page renders when the change affects layout, tables, figures or pagination, when checks reveal a layout concern, or when preparing a milestone PDF. Never compile `.text` files.
