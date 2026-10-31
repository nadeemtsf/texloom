# Texloom repository

This directory is the active Rust/Tauri desktop project. Use its main branch and configured origin for ongoing work. The intended primary GitHub repository is nadeemtsf/texloom; confirm that publication and the push have succeeded before describing it as published. The earlier Python implementation and downloaded ZIPs are historical artifacts.

Read docs/SYSTEM-DESIGN.md and VALIDATION.md before visual editing changes. Keep source authoritative; compile and render candidates before committing source/history. Preserve user projects and private correspondence.

Run npm test and cargo test --locked -p texloom-core for source changes. Native PDF interaction and engine compatibility checks are required before claiming a release is ready. Keep caches, generated builds and private documents out of Git.
