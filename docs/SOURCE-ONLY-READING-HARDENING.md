# PDF reading and cleanup follow-up - source only

Prepared 5 November 2026. All tests are paused. These are source-review findings and unbuilt/untested changes, not reproduced failures or passing fixes. No Texloom window, compiler, renderer diagnostic or actual project was opened.

## Latest search navigation wins

On a lazy long-document view, Next/Previous waits for the requested page to render. Multiple requests could complete out of order while sharing the same query serial, allowing an older request to scroll/highlight after a newer one and show a count from the newer global index. `PDFSearch` now has a separate navigation sequence and captures its match index. Only the latest request for the same query/view updates highlight, scroll, count or error notification. Reset invalidates pending navigation. It does not cancel actual page work or add new search/language support.

Deferred checks: delayed pages resolving in reverse order, rapid Next/Previous, query/view reset during navigation, delayed failure after a newer success, ordinary English/Arabic/Hebrew search and short/long documents. None has run for this change.

## Renderer cleanup does not own application state

PDF export disposes its private view and destroys its PDF with cleanup failures contained; an outer finally releases busy/read-only state. Ordinary compilation tracks its prepared view so an unadopted view is cleaned up after publication failure. A partially adopted failed view is marked stale and its selection cleared, without destroying an adopted PDF as private data. Old-renderer destruction errors are contained during publication and after a new project has been adopted, so they do not skip that project's native-close/recovery setup.

These paths keep operation outcome, source acceptance and cleanup separate. No automatic retry, rollback, save, recovery deletion or acceptance bypass is added. Failure-injection and normal workflow/build checks are deferred. See COMMIT-PRESENTATION.md; previous commit-path cleanup work remains versioned separately.

## Gesture ownership

Current canvas and legacy drag handlers filter by the initiating pointer ID. Marquee lost capture cancels its transient selection instead of leaving release listeners armed. Gesture cleanup drops the global cancellation hook only while it owns that hook. The pending Windows tests still determine actual native behavior; no input test was run to justify a pass. See GESTURE-VERIFICATION.md.
