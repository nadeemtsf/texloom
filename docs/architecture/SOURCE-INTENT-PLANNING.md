# Source-native intent planning: first ARCH-04 slice

10 November 2026. `document::intent` prepares read-only candidates for literal paragraph replacement and paragraph-content deletion. This is a contract/domain slice, not a completed native command, visual editing feature or publication path.

`Session::plan_source_intent` accepts a typed `SourceIntent` containing a private entity reference. It resolves session/revision ownership, rederives the recognized paragraph range from canonical source and creates one exact replacement in one existing file. It accepts no frontend-supplied raw offset or arbitrary supporting-file write. Repeated text retains its particular occurrence. Foreign/stale references refuse, including after Undo restores equal bytes.

Replacement means plain literal text, escaped using standard LaTeX spellings. Raw source commands in the input become literal text. Empty/whitespace-only input uses a distinct Delete intention; controls, paragraph breaks (including U+2028/U+2029) and literal payloads over 256,000 UTF-8 bytes refuse. Split/join, tabs and structural/raw-LaTeX input need separate future contracts. Existing candidate file/project limits still apply.

Delete removes the selected paragraph's content range and preserves surrounding whitespace. It neither removes positioning nor deletes an asset. No Texloom command, support package or project file is added. Existing unrelated source bytes and the build profile remain unchanged.

The source projection additionally recognizes the exact `\textbackslash{}`, `\textasciitilde{}` and `\textasciicircum{}` spellings produced by escaping. An initial regression showed that without this support a generated candidate could become entirely opaque on the next projection. The failing run is retained. Recognition requires the exact empty terminator; arbitrary arguments or missing/malformed terminators remain unsupported. These are lexical candidates, not proof of package/engine semantics or visual eligibility.

## Verification and commit boundary

`IntentPlan` exposes candidate sources, the exact source patch, base revision, operation kind, a required proof and an explicit document-reflow effect. Its prepared object is private. There is no visual-acceptance/commit method or fake passed-verification flag. Constructing `RequiredProof` records an obligation; it does not discharge it.

An eventual native executor must recheck current source and user/target identity, compile, inspect and verify the intended rendered replacement/deletion and unrelated content under a reviewed reflow policy. A readable PDF or success exit alone is insufficient. The current UI and native verification service have not been connected to these plans.

The existing source-edit lane can explicitly apply the same exact patch as a draft, with normal guarded history/recovery. That is source editing, not geometric acceptance. Operation IDs, acknowledgement resolution, idempotency, transport schema, structured targets and the final verifier integration remain open ARCH-04/05/09/10 work. Coordinate maps do not authorize reusing stale rendered geometry.

## Development experiments

`source_intent_candidate` is a read-only fixture CLI using the actual core planner. `research/tools/source_intent_portability.py` creates only a new directory below repository `artifacts/`, generates synthetic replacement/deletion source through that CLI, disables shell escape/automatic package installation, bounds subprocess time and preserves logs/PDFs/extracted text. It never opens Texloom or commits/saves a project.

On the recorded 10 November version, all nine English compile/text checks passed: baseline, replacement and deletion under pdfLaTeX, XeLaTeX and LuaLaTeX. Extraction was checked against independently declared full-document text, including the requested ampersand and percent. The original fixture remained byte-identical. This reuses one development document across engines and checks no glyph geometry, RTL rendering, app mapping/acceptance, native input, latency or held-out accuracy. Exact versions/hashes and earlier snapshot limits are retained in the engineering receipt.
