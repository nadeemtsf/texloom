# Project durability experiment

The standalone persistence component adds disk fingerprints, explicit source-save preconditions, source and support-file staging, flushed originals/intended bytes, checked manifests, per-project locking, recoverable multi-file replacement and flat cleanup. Recovery can finish or roll back a partial save only while every affected file still matches a known before/after fingerprint. Unknown external edits, corrupt evidence, foreign project paths and unrecognized metadata folders are refused.

This component is not connected to the application Save command yet. Its fault-injection suite covers changed/removed/new files, Unicode/BOM/CRLF, creation/deletion, interruption, locked sources, corrupted evidence, project identity, concurrent instances and cleanup boundaries. CI has not yet validated this checkpoint. The local command runner is unavailable because the supplied workspace configuration contains an invalid Windows permission path (`file:///workspace`); local command requests are rejected before starting. Direct workspace file access and GitHub CI remain available.

A journal makes several file replacements recoverable; it does not make independent filesystem renames globally atomic. Disk preconditions detect external edits before staging and replacement, and recovery refuses unknown states. Do not describe these checks as an operating-system compare-and-swap guarantee against every concurrent third-party replacement.

Next: pass the component tests, bind loaded disk state and revision checks to native Save, add conflict/recovery/save-copy flows, implement unsaved typing recovery, measure performance and finish native acceptance. Earlier native input/layout gates and P8/P9 remain open. No release is certified.
