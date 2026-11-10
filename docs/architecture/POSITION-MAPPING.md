# Source position mapping: first ARCH-03 slice

10 November 2026. This implements source-coordinate mapping and fixes canonical selection retention. It does not implement a visual caret, shaped-cluster geometry, a bidi navigation algorithm, source-native visual commands or semantic entity re-identification.

## Coordinate and authority boundaries

`document::position::SourceByteOffset` requires a UTF-8 scalar boundary. `EditorUtf16Offset` names an editor coordinate; conversion through `EditorProjection` validates it against the exact current source and rejects a surrogate interior, the removed BOM and an inaccessible CRLF interior. Existing numeric APIs remain compatible. A valid scalar boundary is not necessarily a valid grapheme or visual caret stop.

`LogicalRange` preserves anchor/head direction. `Affinity::Before` and `After` describe logical source order around an insertion, independent of physical left/right. These value types do not bind a project, file, revision or rendered artifact and grant no editing permission. Session ownership and revision checks remain at guarded preparation/commit boundaries. Existing entity references retain their strict session/revision invalidation policy; this coordinate map does not revive them.

## Mapping policy

`ChangeMap::new` validates the exact replacement set, including expected bytes, overlaps and UTF-8 boundaries. It produces an exact candidate string and a transient coordinate map. Guarded commits use a private borrowed map over already validated candidate bytes, avoiding another whole-file patch application. Unchanged replacements do not destroy identity.

- An insertion or deletion before an unchanged range shifts it by the exact byte effect. Repeated text is located by its original range, never a search for equal text.
- Insertions at the two edges of a nonempty selection remain outside its selected content. Reversed selection direction is retained.
- A collapsed caret uses `After` for insertion. A point inside replaced/deleted content becomes unavailable.
- Replacing/deleting any selected content, or inserting inside a nonempty selected interval, clears the selection. Numerically valid old offsets do not justify retaining it.
- Every secondary selection must survive. If one loses identity, the whole multi-selection clears; the command does not silently drop a selected region.
- A change to another file preserves selection in its own file. Removing the selected file clears it. No-op edits retain the current revision and selection.

Both generic prepared commits and the one-file typing path apply this policy before recording history. Existing Undo/Redo restores recorded selections with their corresponding source snapshots. A caller that intentionally creates a new selection after editing can use the existing revision-checked selection API; this policy does not guess that new intent.

The inverse coordinate view handles adjacent deletions that collapse into one insertion point. Affinity selects the boundary of the restored interval; removed interior identity is not uniquely recoverable. An inverse map is not an Undo capability. Reorder-as-delete/insert currently invalidates a selected moved fragment rather than claiming to follow it. More structured operation mappings remain ARCH-03 work.

## Work and coverage limits

Map construction sorts changed replacements; lookups use binary search in the ordered change intervals. Mapping does not scan the selected text once per secondary range. This avoids a patch-count multiplied by selection-count traversal under the existing bounded document/selection limits. These algorithmic observations are not a latency or responsiveness measurement.

This layer understands byte changes, not TeX expansion, supported semantic scopes, independent PDF identity, fonts or layout. A conservative clear can include an unchanged subrange when a caller supplies a broad enclosing replacement. Improving useful retention requires more precise patches or structural evidence, not nearest-text recovery.

## Executed checks and preserved failures

Three new pre-fix regressions reproduced wrong retained offsets after a prefix insertion, selection surviving deleted content, and a partially destroyed multi-selection. The failing run is retained in `artifacts/anchors-20261110/selection-before-fix.log`. The first full run after integration found an old recovery-test fixture that unwrapped a selection now deliberately invalidated by replacement. That failure is retained in `core-first.log`; the fixture now explicitly injects an invalid BOM boundary, preserving the recovery-corruption rejection check.

The domain tests include a separate tagged-character splice oracle: original-character tags distinguish equal bytes from replacement text, and expected preserved ranges are derived without coordinate-delta formulas. Tests enumerate forward/reversed ranges over mixed Unicode and structural characters, plus explicit affinity, inverse-deletion, typed-conversion, history and checkpoint cases. These are development regressions, not independent final-document trials.

The dated engineering receipt records completed checks and exact hashes. Actual Windows selection/caret/input behavior remains owner-pending. The existing prepared r8 binary has not been rebuilt, installed or opened by this change. No engine/PDF, latency, usability or final-evaluation result follows from source-coordinate tests.
