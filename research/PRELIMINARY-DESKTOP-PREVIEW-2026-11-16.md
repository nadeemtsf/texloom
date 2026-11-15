# Preliminary desktop preview candidate

The candidate connects the first-pass observer to an optional raw Tauri channel
and a separate read-only current-page canvas. The native service enables it
only for canonical preview requests. Export and prepared visual builds remain
settled-only. The progress record carries project/revision/request/engine and
an explicit first-pass phase, without a build ID, receipt or verification flag.

The frontend validates progress separately from final results. An opaque
invocation ticket rejects older callbacks even when a reopened project has
matching visible IDs. Duplicate, cancelled, completed and stale messages cannot
mount a view. Loading/rendering/disposal failures are optional and preserve
the compiler's result. The existing settled PDF remains alive underneath;
only normal final publication changes `state.pdf`, `state.result` or `state.compiled`.
Typing, composition, history, project switching, cancellation and final
publication invalidate temporary rendering. Export is visibly disabled while
compilation or other conflicting work is active.

The image has an early-preview/checking label and a dismissal control. It has
no text-hit, source mapping or editing bindings. Only the current page is
rendered, clamped to the new PDF page count. The complete settled PDF retains
the existing reading/search/selection/verification path. This is an initial
temporary presentation, not a general incremental TeX engine or visual caret.

## Executed component checks

The 351-test frontend run passed, including the full production shell's
preliminary/final lifecycle and later-failure retention, strict generated Rust
progress bytes, optional channel failures, old-ticket/duplicate/cancelled races,
real PDF painting and disposal after an aborted load. Existing transaction tests
use the new transport adapter too. Earlier failed harness runs are preserved;
their missing injected imports were corrected, and the new export-button check
led to the explicit busy-state control above.

The headless native routing example creates a fresh synthetic project and
calls the actual Backend. Preview yielded one event, while export and a prepared
visual request yielded none. Source, history and the original file were
unchanged. Its first-pass and final PDFs parse with identical text and full-page
96 dpi pixels. This does not execute Tauri/WebView dispatch. The opt-in example
is `first_preview_transport_development`; its raw receipt is recorded separately.

The complete local gate, hosted checks and owner-run native acceptance are
separate checkpoints. See the source-bound development receipt and
`docs/MANUAL-PRELIMINARY-PREVIEW-2026-11-16.md`. The latest compiled manuscript
remains immutable v19, describing the preceding observer checkpoint.

No new end-to-end speed measurement is claimed. The earlier first-available
native endpoint, binary Node decoding diagnostic and physical presentation are
different intervals. The new view is intended to overlap first-page painting
with remaining TeX passes; actual native behavior needs owner verification.
