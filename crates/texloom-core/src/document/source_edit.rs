//! A guarded edit of one existing source file. No compiler/visual authority.
use super::*;

/// The owner and base revision are checked again at commit. This plan contains
/// only the affected file, never another copy of the untouched project.
pub struct PreparedSourceEdit {
    owner: Arc<()>,
    base: Revision,
    file: String,
    text: String,
    forward: Changes,
    inverse: Changes,
    changed: bool,
}
impl PreparedSourceEdit {
    pub(super) fn validate_for(&self, session: &Session) -> EditResult<()> {
        if !Arc::ptr_eq(&session.owner, &self.owner) { return Err(EditError::ForeignSession); }
        if self.base != session.revision { return Err(EditError::StaleRevision); }
        Ok(())
    }
    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn base(&self) -> Revision {
        self.base
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn recovery_patch(&self) -> Patch {
        Patch {
            base: self.base,
            files: self.forward.files.clone(),
            profile: None,
        }
    }
}
impl Session {
    pub fn prepare_source_edit(
        &self,
        base: Revision,
        file: &str,
        edits: Vec<Replacement>,
    ) -> EditResult<PreparedSourceEdit> {
        if base != self.revision {
            return Err(EditError::StaleRevision);
        }
        let source = self
            .sources
            .get(file)
            .ok_or_else(|| EditError::MissingFile(file.into()))?;
        let (text, inverse) = apply_replacements(source, &edits)?;
        // File membership/profile are unchanged. Count byte lengths without
        // cloning unrelated source strings; all other limits already hold.
        let bytes = self.sources.values().map(String::len).sum::<usize>() - source.len();
        if text.len() > 4_000_000
            || bytes
                .checked_add(text.len())
                .is_none_or(|n| n > 60 * 1024 * 1024)
        {
            return Err(EditError::LimitExceeded);
        }
        let changed = text != *source;
        Ok(PreparedSourceEdit {
            owner: self.owner.clone(),
            base,
            file: file.into(),
            text,
            forward: Changes {
                files: vec![FileChange::Edit {
                    file: file.into(),
                    edits,
                }],
                profile: None,
            },
            inverse: Changes {
                files: vec![FileChange::Edit {
                    file: file.into(),
                    edits: inverse,
                }],
                profile: None,
            },
            changed,
        })
    }
    pub fn commit_source_edit(
        &mut self,
        prepared: PreparedSourceEdit,
        group: Option<TypingGroup>,
    ) -> EditResult<Revision> {
        prepared.validate_for(self)?;
        if !prepared.changed {
            return Ok(self.revision);
        }
        let revision = self.next_revision()?;
        let after_selection = self.selection.as_ref().and_then(|selection| {
            if selection.file == prepared.file {
                let FileChange::Edit { edits, .. } = &prepared.forward.files[0] else { return None; };
                position::ChangeMap::from_prepared(self.sources.get(&prepared.file)?, &prepared.text, edits)
                    .map_selection(selection)
            } else {
                validate_selection(&self.sources, selection).ok().map(|_| selection.clone())
            }
        });
        self.record_history(
            prepared.forward,
            prepared.inverse,
            Origin::Source,
            group,
            revision,
            after_selection.clone(),
        );
        self.sources.insert(prepared.file, prepared.text);
        self.revision = revision;
        self.selection = after_selection;
        Ok(revision)
    }
}

#[cfg(test)]
mod tests;
