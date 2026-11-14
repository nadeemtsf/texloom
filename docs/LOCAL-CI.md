# Before-commit validation

The owner requests automated CI-equivalent checks before every commit. Run
`npm run hooks:install` once per checkout. The tracked `.githooks/pre-commit`
hook applies to command-line and VS Code commits in that checkout. Stage all
intended files first. `npm run ci:local` invokes the same gate manually.

The hook checks that the working files match the index, rejects untracked
inputs, records the staged tree and HEAD, runs the tests, and checks both
identities again. Any failure blocks the commit. Do not bypass the hook.
The checks run again for each commit; no stale passing-result cache is used.

| GitHub workflow | Local checks in `scripts/check-ci.ps1` |
| --- | --- |
| `ci.yml` | Locked frontend install, core/recovery/frontend/build-service/native protocol tests, pinned native renderer and independent reference, actual native inspection/approval, Windows build, portable package and source benchmark |
| `proof.yml` | Fixture planners, verification policy, two-renderer regressions, real three-engine movement/selection/protected text, native inspection of those compiled PDFs |
| `build-service.yml` | Real three-engine recipe/fault corpus and BibTeX/Biber recipes |

This is a Windows x64 gate. It requires PowerShell 7 (`pwsh`), Rust/MSVC, Node/npm, Python and the
installed TeX/BibTeX/Biber tools. It may take several minutes. Missing tools
or packages fail rather than silently skip a test. No Texloom GUI is launched;
physical input tests remain owner-run. No `.text` file is compiled.

Each run keeps step logs/results and a gate receipt under a fresh ignored
`artifacts/local-ci/` directory. Synthetic proof and corpus reports use that
directory too, preserving earlier evidence. Pre-commit desktop binaries are
validation outputs; their source-head field is null for staged source changes,
and the gate receipt identifies the tested tree.

The native CI jobs now prepare the same reviewed PDFium distributor pin as
the runtime. Python's separately pinned renderer remains an independent
development reference; its DLL is not substituted for the native verifier.
This fixes the observed hosted checksum refusals without weakening validation.

Local checks cannot establish a clean GitHub Windows/Linux runner result,
GitHub artifact transfer, external security checks or physical UI behavior.
After pushing, inspect the hosted checks and resolve failures before merging.
Keep the workflow and local check lists aligned when adding new CI steps.

The synthetic Git repositories in the gate tests remove inherited `GIT_*`
settings from both their Git commands and their staged-tree check. Linked
worktree hooks can provide absolute repository/index paths; passing those into
a disposable fixture would operate on the owning checkout instead. The real
gate keeps the committing process's environment, including its actual staged
index. This isolation changes no source-check or hook requirement.
