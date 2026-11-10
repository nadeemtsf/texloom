//! Source-coordinate mapping, not a grapheme/caret/PDF identity or edit grant.
use super::{apply_replacements, EditError, EditResult, Replacement, Selection};
use std::{borrow::Cow, ops::Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceByteOffset(usize);
impl SourceByteOffset {
    pub fn checked(source: &str, value: usize) -> EditResult<Self> {
        if !source.is_char_boundary(value) {
            return Err(EditError::InvalidBoundary);
        }
        Ok(Self(value))
    }
    pub fn get(self) -> usize {
        self.0
    }
}

/// A coordinate in an editor projection; conversion checks surrogate boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorUtf16Offset(usize);
impl EditorUtf16Offset {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
    pub fn get(self) -> usize {
        self.0
    }
}

/// Logical source order, independent of physical left/right in bidi text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Affinity {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRange {
    pub anchor: SourceByteOffset,
    pub head: SourceByteOffset,
}

#[derive(Clone, Debug)]
struct Change {
    before: Range<usize>,
    after: Range<usize>,
}

/// An exact, transient map between two source snapshots. It does not re-resolve
/// entity references, follow moved content, or bind a file/session/revision.
pub struct ChangeMap<'a> {
    before: &'a str,
    after: Cow<'a, str>,
    changes: Vec<Change>,
}
impl<'a> ChangeMap<'a> {
    pub fn new(source: &'a str, edits: &[Replacement]) -> EditResult<Self> {
        let (after, _) = apply_replacements(source, edits)?;
        Ok(Self::build(source, Cow::Owned(after), edits))
    }
    /// Only document commit may use this after guarded patch preparation.
    pub(super) fn from_prepared(before: &'a str, after: &'a str, edits: &[Replacement]) -> Self {
        Self::build(before, Cow::Borrowed(after), edits)
    }
    fn build(before: &'a str, after: Cow<'a, str>, edits: &[Replacement]) -> Self {
        let mut edits = edits
            .iter()
            .filter(|e| e.expected != e.text)
            .collect::<Vec<_>>();
        edits.sort_by_key(|e| (e.range.start, e.range.end));
        let mut old_cursor = 0;
        let mut new_cursor = 0;
        let mut changes = Vec::with_capacity(edits.len());
        for edit in edits {
            let start = new_cursor + edit.range.start - old_cursor;
            let end = start + edit.text.len();
            changes.push(Change {
                before: edit.range.clone(),
                after: start..end,
            });
            old_cursor = edit.range.end;
            new_cursor = end;
        }
        Self {
            before,
            after,
            changes,
        }
    }
    pub fn before(&self) -> &str {
        self.before
    }
    pub fn after(&self) -> &str {
        &self.after
    }

    /// None means the point was inside removed/replaced content. Insertion
    /// affinity chooses the boundary before/after inserted bytes, never a glyph.
    pub fn map_position(
        &self,
        position: SourceByteOffset,
        affinity: Affinity,
    ) -> EditResult<Option<SourceByteOffset>> {
        let position = SourceByteOffset::checked(self.before, position.get())?.get();
        // Skip completed replacements, but retain an insertion at the same
        // boundary. Each lookup stays logarithmic in the patch count.
        let index = self.changes.partition_point(|change| {
            change.before.end < position
                || (change.before.end == position && !change.before.is_empty())
        });
        if let Some(change) = self
            .changes
            .get(index)
            .filter(|c| c.before.start <= position)
        {
            if position > change.before.start && position < change.before.end {
                return Ok(None);
            }
            if change.before.is_empty() {
                let value = match affinity {
                    Affinity::Before => change.after.start,
                    Affinity::After => change.after.end,
                };
                return SourceByteOffset::checked(&self.after, value).map(Some);
            }
            return SourceByteOffset::checked(&self.after, change.after.start).map(Some);
        }
        let (old_cursor, new_cursor) = index
            .checked_sub(1)
            .map(|i| (self.changes[i].before.end, self.changes[i].after.end))
            .unwrap_or((0, 0));
        SourceByteOffset::checked(&self.after, new_cursor + position - old_cursor).map(Some)
    }

    /// Preserve only ranges whose selected bytes were not changed. Boundary
    /// insertions stay outside a nonempty range. Any affected secondary range
    /// invalidates a multi-selection rather than silently dropping part of it.
    pub fn map_range(&self, range: LogicalRange) -> EditResult<Option<LogicalRange>> {
        let a = SourceByteOffset::checked(self.before, range.anchor.get())?.get();
        let h = SourceByteOffset::checked(self.before, range.head.get())?.get();
        if a == h {
            return Ok(self
                .map_position(range.anchor, Affinity::After)?
                .map(|p| LogicalRange { anchor: p, head: p }));
        }
        let (start, end) = (a.min(h), a.max(h));
        let index = self
            .changes
            .partition_point(|change| change.before.end <= start);
        if self
            .changes
            .get(index)
            .is_some_and(|change| change.before.start < end)
        {
            return Ok(None);
        }
        let Some(lo) = self.map_position(SourceByteOffset(start), Affinity::After)? else {
            return Ok(None);
        };
        let Some(hi) = self.map_position(SourceByteOffset(end), Affinity::Before)? else {
            return Ok(None);
        };
        // Checked patch construction copied this unmodified interval exactly.
        // Do not rescan the selected bytes for every overlapping selection.
        Ok(Some(if a < h {
            LogicalRange {
                anchor: lo,
                head: hi,
            }
        } else {
            LogicalRange {
                anchor: hi,
                head: lo,
            }
        }))
    }

    /// Inverse coordinate map. Deleted content has no unique recovered interior
    /// identity; collapsed boundaries use affinity. This is not automatic undo.
    pub fn inverse(&self) -> ChangeMap<'_> {
        let mut changes: Vec<Change> = Vec::with_capacity(self.changes.len());
        for change in &self.changes {
            let next = Change {
                before: change.after.clone(),
                after: change.before.clone(),
            };
            // Adjacent deletions become simultaneous insertions at one point.
            // Combine them so After crosses the entire restored interval.
            if let Some(last) = changes.last_mut().filter(|last| {
                last.before.is_empty()
                    && next.before.is_empty()
                    && last.before == next.before
                    && last.after.end == next.after.start
            }) {
                last.after.end = next.after.end;
            } else {
                changes.push(next);
            }
        }
        ChangeMap {
            before: &self.after,
            after: Cow::Borrowed(self.before),
            changes,
        }
    }

    pub(super) fn map_selection(&self, selection: &Selection) -> Option<Selection> {
        let pair = |a, h| {
            let range = LogicalRange {
                anchor: SourceByteOffset::checked(self.before, a).ok()?,
                head: SourceByteOffset::checked(self.before, h).ok()?,
            };
            let mapped = self.map_range(range).ok()??;
            Some((mapped.anchor.get(), mapped.head.get()))
        };
        let (anchor, head) = pair(selection.anchor, selection.head)?;
        let secondary = selection
            .secondary
            .iter()
            .map(|&(a, h)| pair(a, h))
            .collect::<Option<Vec<_>>>()?;
        Some(Selection {
            file: selection.file.clone(),
            anchor,
            head,
            secondary,
        })
    }
}
