# P5 protected text checkpoint

Mixed-format selections can now open as separate plain-text fields. Each field is a literal source run with its inherited style label. Formatting commands and braces, colour arguments, macro calls, comments and forced line breaks remain protected. Simple existing line templates retain the earlier plain editor; LaTeX mode remains available.

Rust describes the fragment and applies edits by run ID against the exact expected original content. It escapes new text, rejects duplicate/missing IDs and stale content, preserves bytes outside the chosen runs and retains CRLF inside edited multiline runs. It never serializes a syntax tree over the full source. Existing object content is patched directly so unrelated trailing bytes are preserved.

The model is deliberately bounded: 4 MB source/text, 128 literal runs, 1,024 protected commands and 32 formatting levels. The existing movable-text validator still refuses unknown effects, links, math and structures without an adapter. This is not a general LaTeX semantic parser.

The dialog sends only changed field IDs and values. Switching to LaTeX mode first derives its source through the same native edit planner, retains the field edits, and blocks competing submission until that transfer finishes. Revision checks protect both asynchronous model loading and submission.

Changes use the existing private candidate build, PDF preparation and guarded shared-history commit. Text replacement can legitimately reflow the document; it does not use the position-only neighbor-invariance rule. Native editing/reflow acceptance remains open.

Validation: 58 core, 17 native and 58 frontend/PDF tests passed locally. The unchanged 17 build-infrastructure tests passed at P3 (150 combined tests). Frontend production build passed. CI adds three actual engine compilation cases using the real Rust editor, checking the exact unchanged command scaffold and visible replacement text. All three cases passed at source 6eb2b5a in workflow 41402245293; raw results are in docs/evidence/protected-text-verified-run.json. P4's 26 selections and seven measured refusals are preserved in docs/evidence/selection-verified-run.json.

## Explicit line-break spacing checkpoint

The structured editor also exposes existing `\\[length]` and `\\*[length]` gaps. Supported decimal lengths in pt, bp, mm, cm and in retain their unit; the patch replaces only numeric bytes. Whitespace, comments, optional stars, Unicode, line endings and the rest of the fragment remain unchanged. Numerically unchanged inputs preserve their original spelling. Macro lengths, font-relative units, flexible glue and values beyond 400 bp remain read-only with a source-editing explanation. The adapter never invents a gap for an ordinary line break.

Literal text and spacing edits can be submitted together against the same expected fragment. Switching to LaTeX mode transfers both through the native planner. Spacing uses the reflow policy rather than displacement's unchanged-neighbor policy.

The expanded local suites pass 64 core, 19 native service and 67 frontend tests. The engine workflow adds three explicit-gap cases through the real Rust adapter: only `[2pt]` becomes `[14pt]`, the preceding line stays fixed, and the following signature lines move by 12 TeX points. Real-engine results for this new checkpoint are pending CI; these fixtures do not certify native reflow acceptance.

Remaining P5 work includes broader spacing adapters and editing/reflow fixtures. P6 independent runtime inspection, P7 recovery/conflict/performance hardening, P8 installed-artifact acceptance and P9 richer adapters remain open. Native mouse/typing/IME acceptance remains blocked: desktop control rejected the test app with "Computer Use was not approved to use texloom-p5." No full application release is certified by this checkpoint.
