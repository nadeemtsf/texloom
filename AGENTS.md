# Texloom repository

The primary repository is https://github.com/nadeemtsf/texloom. Use main and origin/main for ongoing development. This Rust/Tauri desktop implementation is the active project; the earlier Python app and downloaded ZIPs are historical artifacts.

Read docs/SYSTEM-DESIGN.md and VALIDATION.md before visual editing changes. Keep source authoritative, and compile and render candidates before committing source/history. Preserve user projects and private correspondence.

Run npm test and cargo test --locked -p texloom-core for source changes. Native PDF interaction and engine compatibility checks are required before claiming a release is ready. Keep caches, generated builds and private documents out of Git.
