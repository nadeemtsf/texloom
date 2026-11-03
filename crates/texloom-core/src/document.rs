//! Canonical UTF-8 source, guarded patches and one revisioned history.
//! No filesystem, compiler, UI, or TeX interpretation lives in this module.
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    ops::Range,
    sync::Arc,
};
pub mod projection;
pub mod recovery;

pub type Sources = BTreeMap<String, String>;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Revision(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    StaleRevision,
    ForeignSession,
    RevisionExhausted,
    MissingFile(String),
    ExistingFile(String),
    DuplicateFile(String),
    InvalidRange,
    InvalidBoundary,
    ExpectedTextMismatch,
    OverlappingEdits,
    InvalidProfile,
    LimitExceeded,
    InvalidEditorOffset,
    InvalidRecoveryCheckpoint,
}
impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EditError {}
pub type EditResult<T> = std::result::Result<T, EditError>;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct Replacement {
    pub range: Range<usize>,
    pub expected: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FileChange {
    Edit {
        file: String,
        edits: Vec<Replacement>,
    },
    Create {
        file: String,
        text: String,
    },
    Remove {
        file: String,
        expected: String,
    },
}
impl FileChange {
    fn file(&self) -> &str {
        match self {
            Self::Edit { file, .. } | Self::Create { file, .. } | Self::Remove { file, .. } => file,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct Profile {
    pub main: String,
    pub engine: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct ProfileChange {
    pub expected: Profile,
    pub replacement: Profile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct Patch {
    pub base: Revision,
    pub files: Vec<FileChange>,
    pub profile: Option<ProfileChange>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Origin {
    Source,
    Canvas,
    Settings,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypingGroup(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct Selection {
    pub file: String,
    pub anchor: usize,
    pub head: usize,
    pub secondary: Vec<(usize, usize)>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
struct Changes {
    files: Vec<FileChange>,
    profile: Option<ProfileChange>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
struct HistoryEntry {
    forwards: Vec<Changes>,
    inverses: Vec<Changes>,
    origin: Origin,
    group: Option<TypingGroup>,
    #[cfg_attr(feature = "recovery-serde", serde(skip))]
    bytes: usize,
    before_selection: Option<Selection>,
    after_selection: Option<Selection>,
    last_revision: Revision,
}

/// Only the creating session can commit this immutable candidate. The service
/// must validate compiler/PDF results before committing a visual candidate.
pub struct Prepared {
    owner: Arc<()>,
    base: Revision,
    sources: Sources,
    profile: Profile,
    forward: Changes,
    inverse: Changes,
    changed: bool,
}
impl Prepared {
    pub fn sources(&self) -> &Sources {
        &self.sources
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn base(&self) -> Revision {
        self.base
    }
    pub fn changed(&self) -> bool {
        self.changed
    }
    /// Validated plain data for a durable mutation log. It carries no Prepared
    /// ownership, compiler result or permission to commit a visual candidate.
    pub fn recovery_patch(&self) -> Patch {
        Patch {
            base: self.base,
            files: self.forward.files.clone(),
            profile: self.forward.profile.clone(),
        }
    }
}

pub struct Session {
    owner: Arc<()>,
    sources: Sources,
    profile: Profile,
    revision: Revision,
    history: Vec<HistoryEntry>,
    cursor: usize,
    selection: Option<Selection>,
}
impl Session {
    pub fn new(sources: Sources, profile: Profile) -> EditResult<Self> {
        validate_state(&sources, &profile)?;
        Ok(Self {
            owner: Arc::new(()),
            sources,
            profile,
            revision: Revision(0),
            history: Vec::new(),
            cursor: 0,
            selection: None,
        })
    }
    pub fn sources(&self) -> &Sources {
        &self.sources
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }
    pub fn can_redo(&self) -> bool {
        self.cursor < self.history.len()
    }
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }
    pub fn set_selection(&mut self, selection: Selection) -> EditResult<()> {
        validate_selection(&self.sources, &selection)?;
        if let Some(entry) = self
            .cursor
            .checked_sub(1)
            .and_then(|i| self.history.get_mut(i))
            .filter(|e| e.last_revision == self.revision)
        {
            entry.after_selection = Some(selection.clone());
        }
        self.selection = Some(selection);
        Ok(())
    }

    pub fn prepare(&self, patch: Patch) -> EditResult<Prepared> {
        if patch.base != self.revision {
            return Err(EditError::StaleRevision);
        }
        let forward = Changes {
            files: patch.files,
            profile: patch.profile,
        };
        let (sources, profile, inverse) = apply_changes(&self.sources, &self.profile, &forward)?;
        let changed = sources != self.sources || profile != self.profile;
        Ok(Prepared {
            owner: self.owner.clone(),
            base: self.revision,
            sources,
            profile,
            forward,
            inverse,
            changed,
        })
    }

    pub fn commit(
        &mut self,
        prepared: Prepared,
        origin: Origin,
        group: Option<TypingGroup>,
    ) -> EditResult<Revision> {
        if !Arc::ptr_eq(&self.owner, &prepared.owner) {
            return Err(EditError::ForeignSession);
        }
        if prepared.base != self.revision {
            return Err(EditError::StaleRevision);
        }
        if !prepared.changed {
            return Ok(self.revision);
        }
        let revision = self.next_revision()?;
        let before_selection = self.selection.clone();
        let after_selection = self
            .selection
            .clone()
            .filter(|s| validate_selection(&prepared.sources, s).is_ok());
        let bytes = change_bytes(&prepared.forward) + change_bytes(&prepared.inverse);
        let coalesce = origin == Origin::Source
            && group.is_some()
            && self.cursor == self.history.len()
            && self.history.last().is_some_and(|last| {
                last.origin == origin
                    && last.group == group
                    && last.bytes.saturating_add(bytes) <= 1_048_576
                    && last
                        .forwards
                        .last()
                        .is_some_and(|c| same_files(c, &prepared.forward))
            });
        self.history.truncate(self.cursor);
        if coalesce {
            if let Some(last) = self.history.last_mut() {
                last.forwards.push(prepared.forward);
                last.inverses.push(prepared.inverse);
                last.bytes += bytes;
                last.after_selection = after_selection.clone();
                last.last_revision = revision;
            }
        } else {
            self.history.push(HistoryEntry {
                forwards: vec![prepared.forward],
                inverses: vec![prepared.inverse],
                origin,
                group,
                bytes,
                before_selection,
                after_selection: after_selection.clone(),
                last_revision: revision,
            });
            if self.history.len() > 120 {
                self.history.remove(0);
            }
        }
        self.cursor = self.history.len();
        self.sources = prepared.sources;
        self.profile = prepared.profile;
        self.revision = revision;
        self.selection = after_selection;
        while self.history.len() > 1
            && self.history.iter().map(|e| e.bytes).sum::<usize>() > 32 * 1024 * 1024
        {
            self.history.remove(0);
            self.cursor -= 1;
        }
        Ok(revision)
    }

    /// Undo is a new revision. An outstanding visual candidate remains stale even
    /// if undo happens to restore exactly the bytes it originally captured.
    pub fn undo(&mut self) -> EditResult<bool> {
        if self.cursor == 0 {
            return Ok(false);
        }
        let revision = self.next_revision()?;
        let entry = self
            .history
            .get(self.cursor - 1)
            .ok_or(EditError::InvalidRange)?;
        let mut sources = self.sources.clone();
        let mut profile = self.profile.clone();
        for inverse in entry.inverses.iter().rev() {
            (sources, profile, _) = apply_changes(&sources, &profile, inverse)?;
        }
        self.selection = entry.before_selection.clone();
        self.sources = sources;
        self.profile = profile;
        self.cursor -= 1;
        self.revision = revision;
        Ok(true)
    }
    pub fn redo(&mut self) -> EditResult<bool> {
        let Some(entry) = self.history.get(self.cursor) else {
            return Ok(false);
        };
        let revision = self.next_revision()?;
        let mut sources = self.sources.clone();
        let mut profile = self.profile.clone();
        for forward in &entry.forwards {
            (sources, profile, _) = apply_changes(&sources, &profile, forward)?;
        }
        self.selection = entry.after_selection.clone();
        self.sources = sources;
        self.profile = profile;
        self.cursor += 1;
        self.revision = revision;
        Ok(true)
    }
    fn next_revision(&self) -> EditResult<Revision> {
        self.revision
            .0
            .checked_add(1)
            .map(Revision)
            .ok_or(EditError::RevisionExhausted)
    }
}

fn validate_selection(sources: &Sources, selection: &Selection) -> EditResult<()> {
    let source = sources
        .get(&selection.file)
        .ok_or_else(|| EditError::MissingFile(selection.file.clone()))?;
    if selection.secondary.len() > 9999 {
        return Err(EditError::LimitExceeded);
    }
    for (anchor, head) in std::iter::once((selection.anchor, selection.head))
        .chain(selection.secondary.iter().copied())
    {
        if source.get(..anchor).is_none() || source.get(..head).is_none() {
            return Err(EditError::InvalidBoundary);
        }
    }
    Ok(())
}

fn validate_state(sources: &Sources, profile: &Profile) -> EditResult<()> {
    if !sources.contains_key(&profile.main)
        || !["auto", "pdflatex", "lualatex", "xelatex"].contains(&profile.engine.as_str())
    {
        return Err(EditError::InvalidProfile);
    }
    if sources.len() > 600
        || sources.values().any(|v| v.len() > 4_000_000)
        || sources.values().map(String::len).sum::<usize>() > 60 * 1024 * 1024
    {
        return Err(EditError::LimitExceeded);
    }
    Ok(())
}
fn apply_changes(
    sources: &Sources,
    profile: &Profile,
    changes: &Changes,
) -> EditResult<(Sources, Profile, Changes)> {
    let mut candidate = sources.clone();
    let mut seen = BTreeSet::new();
    let mut inverses = Vec::new();
    for change in &changes.files {
        let file = change.file();
        if !seen.insert(file) {
            return Err(EditError::DuplicateFile(file.into()));
        }
        let inverse = match change {
            FileChange::Edit { edits, .. } => {
                let old = candidate
                    .get(file)
                    .ok_or_else(|| EditError::MissingFile(file.into()))?;
                let (next, inverse) = apply_replacements(old, edits)?;
                candidate.insert(file.into(), next);
                FileChange::Edit {
                    file: file.into(),
                    edits: inverse,
                }
            }
            FileChange::Create { text, .. } => {
                if candidate.contains_key(file) {
                    return Err(EditError::ExistingFile(file.into()));
                }
                candidate.insert(file.into(), text.clone());
                FileChange::Remove {
                    file: file.into(),
                    expected: text.clone(),
                }
            }
            FileChange::Remove { expected, .. } => {
                let old = candidate
                    .get(file)
                    .ok_or_else(|| EditError::MissingFile(file.into()))?;
                if old != expected {
                    return Err(EditError::ExpectedTextMismatch);
                }
                let text = candidate
                    .remove(file)
                    .ok_or_else(|| EditError::MissingFile(file.into()))?;
                FileChange::Create {
                    file: file.into(),
                    text,
                }
            }
        };
        inverses.push(inverse);
    }
    let (next_profile, inverse_profile) = if let Some(change) = &changes.profile {
        if &change.expected != profile {
            return Err(EditError::ExpectedTextMismatch);
        }
        (
            change.replacement.clone(),
            Some(ProfileChange {
                expected: change.replacement.clone(),
                replacement: profile.clone(),
            }),
        )
    } else {
        (profile.clone(), None)
    };
    validate_state(&candidate, &next_profile)?;
    Ok((
        candidate,
        next_profile,
        Changes {
            files: inverses,
            profile: inverse_profile,
        },
    ))
}

pub fn apply_replacements(
    source: &str,
    edits: &[Replacement],
) -> EditResult<(String, Vec<Replacement>)> {
    if edits.len() > 100_000 {
        return Err(EditError::LimitExceeded);
    }
    let mut sorted: Vec<_> = edits.iter().collect();
    sorted.sort_by_key(|e| (e.range.start, e.range.end));
    let mut previous: Option<&Replacement> = None;
    for edit in &sorted {
        if edit.range.start > edit.range.end || edit.range.end > source.len() {
            return Err(EditError::InvalidRange);
        }
        if !source.is_char_boundary(edit.range.start) || !source.is_char_boundary(edit.range.end) {
            return Err(EditError::InvalidBoundary);
        }
        if let Some(prior) = previous {
            if edit.range.start < prior.range.end || edit.range.start == prior.range.start {
                return Err(EditError::OverlappingEdits);
            }
        }
        if source.get(edit.range.clone()) != Some(edit.expected.as_str()) {
            return Err(EditError::ExpectedTextMismatch);
        }
        previous = Some(edit);
    }
    let mut next = String::new();
    let mut inverse: Vec<Replacement> = Vec::new();
    let mut copied = 0;
    for edit in sorted {
        next.push_str(
            source
                .get(copied..edit.range.start)
                .ok_or(EditError::InvalidRange)?,
        );
        let start = next.len();
        next.push_str(&edit.text);
        // Adjacent deletions collapse to the same insertion offset. Join their
        // inverses instead of creating ambiguous simultaneous insertions.
        if let Some(last) = inverse.last_mut().filter(|last| last.range.end == start) {
            last.range.end = next.len();
            last.expected.push_str(&edit.text);
            last.text.push_str(&edit.expected);
        } else {
            inverse.push(Replacement {
                range: start..next.len(),
                expected: edit.text.clone(),
                text: edit.expected.clone(),
            });
        }
        copied = edit.range.end;
    }
    next.push_str(source.get(copied..).ok_or(EditError::InvalidRange)?);
    Ok((next, inverse))
}
fn same_files(a: &Changes, b: &Changes) -> bool {
    a.profile.is_none()
        && b.profile.is_none()
        && a.files
            .iter()
            .map(FileChange::file)
            .collect::<BTreeSet<_>>()
            == b.files
                .iter()
                .map(FileChange::file)
                .collect::<BTreeSet<_>>()
}
fn change_bytes(c: &Changes) -> usize {
    c.files
        .iter()
        .map(|f| match f {
            FileChange::Edit { edits, .. } => edits
                .iter()
                .map(|e| e.expected.len().saturating_add(e.text.len()))
                .sum(),
            FileChange::Create { text, .. } => text.len(),
            FileChange::Remove { expected, .. } => expected.len(),
        })
        .sum()
}
