# Architecture decision register

Status: planning decisions and proposals, 7 November 2026. "Retain" means the current design direction is supported by the audit; it does not create new passing evidence. "Recommend" is not an implemented feature. "Open" must be resolved before its dependent feature is enabled.

| ID | Status | Decision / rationale | Required follow-up |
| --- | --- | --- | --- |
| D01 | Retain | Canonical ordinary LaTeX and assets, with native source/history authority | Preserve exact source and opaque content through every new operation |
| D02 | Recommend | Partial lossless editable projection, with capability per operation | Define supported syntax and malformed/unknown handling; ARCH-02 |
| D03 | Recommend | Modular local application; extend existing core/services/frontend boundaries | Extract orchestration incrementally; no microservice or rich-text-authority rewrite; ARCH-07 |
| D04 | Recommend | Typed editing intents and source anchors independent of persistent wrappers | Specify payloads, effect contracts, change mapping, stale/unknown outcomes; ARCH-03/04 |
| D05 | Retain and extend | Draft input and verified visual transformation have different lifecycles within shared history | Model provisional feedback, compile failure and background publication; ARCH-05/06 |
| D06 | Owner goal; implementation policy open | New saved source-native operations add no Texloom macro/support package | Standard LaTeX constructs remain possible. If all new boxing/wrapping is also forbidden, declare which moves become unsupported; ARCH-01 |
| D07 | Open | Temporary instrumentation may remain private if identity is proved and nothing leaks to saved output | Compare marker-independent alternatives, extra builds, ambiguity/refusal and migration. Banning internal probes is a distinct choice; ARCH-10 |
| D08 | Recommend | Reorder and spacing/position adjustment are explicit different intents | Show insertion destinations or permitted reflow. No automatic inference that every upward drag changes reading order; CORE-02/03 |
| D09 | Open | Input host and provisional caret/layout technique | Spike actual WebView2 behavior; compare controlled editable host and EditContext/fallback; TYPE-01 |
| D10 | Recommend | Internal capability adapters, not a public plugin SDK yet | Stabilize built-in entity/operation contracts before third-party extension design |
| D11 | Recommend | Conservative staged asset lifecycle; delete reference by default | No guessed physical-file deletion or automatic collection of shared/unknown assets; ARCH-08 |
| D12 | Recommend | Versioned operation-specific verification | Do not relabel generic readable inspection as edit preservation; ARCH-09 |
| D13 | Retain | Existing projects open without silent conversion | Explicit migration/copy/equivalence checks; customized support protected; CORE-05 |
| D14 | Open | Supported document budgets and numerical performance targets | Measure declared endpoints/workloads first; set targets before release experiments, not after seeing favorable results; QUALITY-02/#32 |
| D15 | Target, not claim | Applicable WCAG 2.2 AA and actual Windows assistive-technology behavior | Non-drag pointer alternatives, keyboard/focus/input semantics and owner-run evidence; QUALITY-01 |
| D16 | Open for release | Compile trust model and any untrusted-project isolation claim | Existing process supervision is not OS sandboxing. Review actual boundaries before advertising safety; #35/QUALITY-02 |
| D17 | Retain | English/Arabic/Hebrew current scope, language/composition extension points | Pin actual algorithms/runtime behavior; no added Windows languages or CJK acceptance now |
| D18 | Scope boundary | No CRDT/collaboration, DOCX fidelity, track changes, arbitrary TeX/TikZ/math editing or universal Word parity implied | Separate owner requirements and feasibility work before adding these to a release promise |
| D19 | Owner decision | Repository remains private now and in the future; no public distribution or Store release planned | Retain internal packaging/Windows acceptance and dependency notices; remove project license/contribution files and own MIT declarations |

8 November contract adoption: the owner's implementation request activates the staged foundation work. [Foundation contract v1](FOUNDATION-CONTRACT.md) makes D06/D08 concrete: no new saved Texloom macros/support package for source-native operations, ordinary LaTeX constructs permitted under the declared contract, and explicit reorder versus spacing intentions. No new command is enabled by this policy alone. D07 source-native proof strategy and D09 input host still require their planned investigations.

## Required architecture review before implementation

Confirm entity families and operation capabilities, the saved-output policy, draft versus verified publication behavior, selection invalidation, migration safety, accessibility/input fallback and the exact first vertical slice. Resolve each high-risk decision with a small bounded synthetic feasibility study once implementation/testing is requested. Preserve negative outcomes and select a different design if the approach fails; do not silently reduce the user-facing guarantee.

Documentation research does not choose a new framework by popularity. The current source/history/build foundation is useful. The most consequential unknowns are reliable caret/source mapping after reflow, responsive provisional typing, source-native target identity without persisted markers, and operation-specific verification of allowed reflow.
