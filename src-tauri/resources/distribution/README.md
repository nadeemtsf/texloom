# Generated distribution notices

This tracked file keeps the resource directory present in a clean checkout,
including native library and protocol tests that run before desktop packaging.

`npm run desktop:build` runs `scripts/prepare-distribution-notices.mjs` to create
`DEPENDENCIES.json`, `SOURCE-AVAILABILITY.txt` and `THIRD-PARTY-NOTICES.txt` from
the locked Windows Rust graph and installed production JavaScript packages.
Those generated files are ignored by Git. The portable packager requires all
three files; the desktop bundle includes this directory.
