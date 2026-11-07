# RTL source-mapping development pilot — protocol 1, 7 November 2026

Question: does the current native SyncTeX/source-unit/zero-wrapper path identify the intended Arabic, Hebrew and English paragraphs in the existing synthetic multilingual template, with unchanged frontend identity/appearance checks?

This protocol precedes execution. This is a reused development fixture, not the held-out final evaluation. The native harness is opt-in behind the existing benchmark feature and does not start Tauri, issue mouse/keyboard input, call Save, or commit a candidate. It opens only a fresh copy of the repository's multilingual template, compiled by the normal Backend under XeLaTeX. PDF.js extracts the actual native baseline and selects one exact normalized baseline-text line per predeclared language. NFKC handles Arabic presentation forms in label lookup only; it does not weaken production verification.

For each intended baseline line, query the real native resolver at its midpoint and 35% of its height above the baseline. Try the normal bounded source-unit candidates, at most16 in source order. The actual native planner creates each zero-offset candidate; private compilation produces its PDF. Retain every native error, rejected candidate and compile failure. Use the original intended baseline line as the click evidence, not candidate marker geometry. A language passes only when the unchanged frontend validateSelectionProbe accepts a complete marker owning that intended line with exact appearance. Its two other labelled language lines must be rejected as wrong-target controls, yielding six controls if all three languages succeed.

After every native request and at shutdown, require exact canonical source, revision0, empty Undo history and unchanged original synthetic disk source. Write result/log files in a fresh owned directory; never overwrite prior attempts. Record source/executable/PDF/log hashes. Unexpected outcomes stay failures; do not relabel valid refusals as passes.

This tests real headless mapping and zero-probe integration. It does not test native pointer behavior, frontend candidate-loop orchestration, movement commit, Save/recovery, general RTL support, latency or usability. Previous four RTL displacement checks retain their separate entry layer. Actual r7 owner checks remain pending even if this pilot passes.

## Observed run, 7 November

The corrected harness run passed all three declared languages. Candidate0 resolved each intended source paragraph (English line17, Arabic line20, Hebrew line24); each privately compiled zero probe passed unchanged identity/appearance checks. All six cross-language wrong-target controls were refused. Canonical source/revision/history and original synthetic source bytes stayed unchanged after every request. This is three reused template targets, not an accuracy sample.

The first harness build attempted a test-only Backend method and failed compilation. An orchestration attempt then found no executable. Both failures remain preserved. The harness was corrected to the existing production `document_prepare_visual` entry with no commit operation; the runner now checks executable existence before creating the run. Acceptance criteria were unchanged. No production acceptance policy was relaxed.

Reproduce with the existing opt-in benchmark feature, not a desktop window:

```powershell
cargo build --locked -p texloom-desktop --features benchmark --example native_selection_development
$env:TEXLOOM_RTL_MAPPING_DIR = 'ABSOLUTE_FRESH_OUTPUT_DIRECTORY'
node scripts/rtl-mapping-proof.mjs
```

The parent directory must exist; the requested output directory must not. Local Rust toolchain, XeLaTeX/template fonts and the pinned verifier preparation follow the repository's normal prerequisites. The harness is not bundled into the desktop app. Its diagnostic mode and the current r7 app have different executable identities.
