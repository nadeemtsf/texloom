# Compile request ownership - source-only follow-up

Prepared 5 November 2026. All tests are paused. The new code has not been built, type/syntax checked, executed or observed in Texloom. The finding below comes from source control flow, not a demonstrated native incident.

Ordinary compile previously awaited source synchronization before capturing its project/client identity. That wait could overlap opening another project or another locking action. A delayed request must not take its compilation permission from one project and then use the new project's client. Similarly, an automatic timer or queued manual retry belongs to the client/cancellation generation that scheduled it.

The source now captures the requesting client and cancellation generation before flushing. After that wait it refuses superseded client/generation, opening/saving/history/composition work, revoked compilation permission or a disabled automatic-compile setting. Actual build and receipt recheck use the captured client. A stale old-client synchronization error does not pause the new client's editor. Source revisions within the same client can still settle through the flush; newer typing remains governed by the existing revision-bound build protocol.

Opening a project clears pending/debounced compilation. Automatic timers and queued manual retries check their originating client/generation before invoking compilation. Automatic work rechecks its setting rather than firing after the user unchecks Auto compile. This does not add trust for arbitrary source, custom commands, shell escape or `.text` compilation.

Deferred checks include: delayed flush followed by opening an unapproved project; project-open loading/failure; saved/recovered/new-template workflows; cancellation between queue and dispatch; disabling Auto compile before debounce/queue expiry; manual compile while automatic work is active; typing and history during compile; and normal three-engine behavior. All remain unrun. Existing r4 binaries do not contain this change.
