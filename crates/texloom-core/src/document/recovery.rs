//! Lossless source/history checkpoints. No filesystem or build authority.
//! Deserialize this data only through the bounded infrastructure codec, then
//! call Session::from_checkpoint; Session itself is never deserializable.
use super::*;

const SCHEMA: u32 = 1;
const MAX_STEPS: usize = 1_000_000;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "recovery-serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "recovery-serde", serde(deny_unknown_fields))]
pub struct Checkpoint {
    schema: u32,
    sources: Sources,
    profile: Profile,
    revision: Revision,
    history: Vec<HistoryEntry>,
    cursor: usize,
    selection: Option<Selection>,
}

/// Infrastructure replays a verified journal tail before adopting the session.
/// Keep the recorded typing groups during replay, then break them once at the
/// new UI boundary so a crash does not change the historical undo grouping.
pub struct Replay {
    session: Session,
}
impl Checkpoint {
    pub fn replay(self) -> EditResult<Replay> {
        Session::restore_checkpoint(self).map(|session| Replay { session })
    }
}
impl Replay {
    pub fn session(&self) -> &Session {
        &self.session
    }
    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }
    pub fn finish(mut self) -> Session {
        for entry in &mut self.session.history {
            entry.group = None;
        }
        self.session
    }
}

impl Session {
    /// A checkpoint includes the redo branch and native byte selections, but
    /// never Prepared tokens, owner identities, builds or visual approvals.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            schema: SCHEMA,
            sources: self.sources.clone(),
            profile: self.profile.clone(),
            revision: self.revision,
            history: self.history.clone(),
            cursor: self.cursor,
            selection: self.selection.clone(),
        }
    }

    pub fn from_checkpoint(checkpoint: Checkpoint) -> EditResult<Self> {
        Ok(checkpoint.replay()?.finish())
    }
    fn restore_checkpoint(mut checkpoint: Checkpoint) -> EditResult<Self> {
        if checkpoint.schema != SCHEMA || checkpoint.cursor > checkpoint.history.len()
            || (checkpoint.history.is_empty() && checkpoint.revision.0 != 0)
        {
            return Err(EditError::InvalidRecoveryCheckpoint);
        }
        if checkpoint.history.len() > 120 {
            return Err(EditError::LimitExceeded);
        }
        validate_state(&checkpoint.sources, &checkpoint.profile)?;
        if let Some(selection) = &checkpoint.selection {
            validate_selection(&checkpoint.sources, selection)?;
        }
        let mut steps = 0usize;
        let mut bytes = 0usize;
        let mut last_revision = 0;
        for entry in &mut checkpoint.history {
            if entry.forwards.is_empty() || entry.forwards.len() != entry.inverses.len()
                || entry.last_revision.0 <= last_revision
                || entry.last_revision.0 > checkpoint.revision.0
            {
                return Err(EditError::InvalidRecoveryCheckpoint);
            }
            last_revision = entry.last_revision.0;
            steps = steps.checked_add(entry.forwards.len())
                .ok_or(EditError::LimitExceeded)?;
            if steps > MAX_STEPS {
                return Err(EditError::LimitExceeded);
            }
            // Never trust a serialized accounting field.
            entry.bytes = entry.forwards.iter().chain(&entry.inverses)
                .try_fold(0usize, |total, changes| total.checked_add(change_bytes(changes)))
                .ok_or(EditError::LimitExceeded)?;
            bytes = bytes.checked_add(entry.bytes).ok_or(EditError::LimitExceeded)?;
        }
        if checkpoint.history.len() > 1 && bytes > 32 * 1024 * 1024 {
            return Err(EditError::LimitExceeded);
        }

        // Walk backwards from the checkpoint cursor to the retained base.
        let mut sources = checkpoint.sources.clone();
        let mut profile = checkpoint.profile.clone();
        for entry in checkpoint.history[..checkpoint.cursor].iter().rev() {
            for inverse in entry.inverses.iter().rev() {
                (sources, profile, _) = apply_changes(&sources, &profile, inverse)?;
            }
        }
        if checkpoint.cursor == 0
            && (sources != checkpoint.sources || profile != checkpoint.profile)
        {
            return Err(EditError::InvalidRecoveryCheckpoint);
        }

        // Replay the entire retained timeline, including redo. Each step must
        // change state and its inverse must restore every byte/profile exactly.
        for (index, entry) in checkpoint.history.iter().enumerate() {
            if let Some(selection) = &entry.before_selection {
                validate_selection(&sources, selection)?;
            }
            for (forward, inverse) in entry.forwards.iter().zip(&entry.inverses) {
                let (next_sources, next_profile, _) = apply_changes(&sources, &profile, forward)?;
                if next_sources == sources && next_profile == profile {
                    return Err(EditError::InvalidRecoveryCheckpoint);
                }
                let (previous_sources, previous_profile, _) =
                    apply_changes(&next_sources, &next_profile, inverse)?;
                if previous_sources != sources || previous_profile != profile {
                    return Err(EditError::InvalidRecoveryCheckpoint);
                }
                sources = next_sources;
                profile = next_profile;
            }
            if let Some(selection) = &entry.after_selection {
                validate_selection(&sources, selection)?;
            }
            if index + 1 == checkpoint.cursor
                && (sources != checkpoint.sources || profile != checkpoint.profile)
            {
                return Err(EditError::InvalidRecoveryCheckpoint);
            }
        }

        Ok(Self {
            owner: Arc::new(()),
            sources: checkpoint.sources,
            profile: checkpoint.profile,
            revision: checkpoint.revision,
            history: checkpoint.history,
            cursor: checkpoint.cursor,
            selection: checkpoint.selection,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(text: &str) -> Session {
        Session::new(BTreeMap::from([("main.tex".into(), text.into())]),
            Profile { main: "main.tex".into(), engine: "auto".into() }).unwrap()
    }
    fn replace(session: &mut Session, from: &str, to: &str, group: Option<u64>) {
        let offset = session.sources()["main.tex"].find(from).unwrap();
        let candidate = session.prepare(Patch {
            base: session.revision(), profile: None,
            files: vec![FileChange::Edit { file: "main.tex".into(),
                edits: vec![Replacement { range: offset..offset + from.len(),
                    expected: from.into(), text: to.into() }] }],
        }).unwrap();
        session.commit(candidate, Origin::Source, group.map(TypingGroup)).unwrap();
    }
    fn timeline() -> Session {
        let mut state = session("\u{feff}% keep\r\nName \u{1f600}\n");
        let offset = state.sources()["main.tex"].find("Name").unwrap();
        state.set_selection(Selection { file: "main.tex".into(), anchor: offset,
            head: offset + 4, secondary: vec![(offset,offset)] }).unwrap();
        replace(&mut state, "Name", "Changed", Some(7));
        replace(&mut state, "Changed", "Final", Some(7));
        let created = state.prepare(Patch { base: state.revision(), profile: None,
            files: vec![FileChange::Create { file: "style.sty".into(), text: "% unchanged\r\n".into() }] }).unwrap();
        state.commit(created, Origin::Canvas, None).unwrap();
        let profile = state.prepare(Patch { base: state.revision(), files: vec![],
            profile: Some(ProfileChange { expected: state.profile().clone(),
                replacement: Profile { main: "main.tex".into(), engine: "xelatex".into() } }) }).unwrap();
        state.commit(profile, Origin::Settings, None).unwrap();
        state.undo().unwrap();
        state
    }

    #[test]
    fn checkpoint_restores_sources_redo_profile_and_selections_without_ram_history() {
        let original = "\u{feff}% keep\r\nName \u{1f600}\n";
        let state = timeline();
        let expected_sources = state.sources().clone();
        let expected_selection = state.selection().cloned();
        let revision = state.revision();
        let checkpoint = state.checkpoint();
        drop(state);
        let mut recovered = Session::from_checkpoint(checkpoint).unwrap();
        assert_eq!(recovered.sources(), &expected_sources);
        assert_eq!(recovered.selection(), expected_selection.as_ref());
        assert_eq!(recovered.revision(), revision);
        assert!(recovered.can_redo());
        assert!(recovered.undo().unwrap());
        assert!(!recovered.sources().contains_key("style.sty"));
        assert!(recovered.undo().unwrap());
        assert_eq!(recovered.sources()["main.tex"], original);
        assert!(recovered.redo().unwrap());
        assert!(recovered.redo().unwrap());
        assert!(recovered.redo().unwrap());
        assert_eq!(recovered.sources(), &expected_sources);
        assert_eq!(recovered.profile().engine, "xelatex");
    }

    #[test]
    fn restored_owner_rejects_old_candidates_and_breaks_typing_coalescence() {
        let mut state = session("Name");
        replace(&mut state, "Name", "First", Some(1));
        let candidate = state.prepare(Patch { base: state.revision(), profile: None,
            files: vec![FileChange::Edit { file: "main.tex".into(), edits: vec![
                Replacement { range: 0..5, expected: "First".into(), text: "Second".into() }] }] }).unwrap();
        let mut recovered = Session::from_checkpoint(state.checkpoint()).unwrap();
        assert_eq!(recovered.commit(candidate, Origin::Canvas, None), Err(EditError::ForeignSession));
        replace(&mut recovered, "First", "Second", Some(1));
        assert_eq!(recovered.history_len(), 2);
        recovered.undo().unwrap();
        assert_eq!(recovered.sources()["main.tex"], "First");
    }

    #[test]
    fn journal_replay_preserves_original_typing_groups_until_adoption() {
        let mut state = session("Name");
        replace(&mut state, "Name", "First", Some(7));
        let mut replay = state.checkpoint().replay().unwrap();
        let candidate = state.prepare(Patch {
            base: state.revision(), profile: None,
            files: vec![FileChange::Edit { file: "main.tex".into(), edits: vec![Replacement {
                range: 0..5, expected: "First".into(), text: "Second".into(),
            }] }],
        }).unwrap();
        let patch = candidate.recovery_patch();
        state.commit(candidate, Origin::Source, Some(TypingGroup(7))).unwrap();
        let candidate = replay.session().prepare(patch).unwrap();
        replay.session_mut().commit(candidate, Origin::Source, Some(TypingGroup(7))).unwrap();
        assert_eq!(replay.session().history_len(), state.history_len());
        let mut adopted = replay.finish();
        replace(&mut adopted, "Second", "Third", Some(7));
        assert_eq!(adopted.history_len(), 2);
        adopted.undo().unwrap();
        assert_eq!(adopted.sources()["main.tex"], "Second");
        adopted.undo().unwrap();
        assert_eq!(adopted.sources()["main.tex"], "Name");
    }

    #[test]
    fn corrupt_schema_cursor_revision_inverse_selection_and_accounting_fail_safely() {
        let state = timeline();
        let mut bad = state.checkpoint();
        bad.schema = 2;
        assert!(Session::from_checkpoint(bad).is_err());
        let mut bad = state.checkpoint();
        bad.cursor = usize::MAX;
        assert!(Session::from_checkpoint(bad).is_err());
        let mut bad = state.checkpoint();
        bad.history[0].last_revision = Revision(state.revision().0 + 1);
        assert!(Session::from_checkpoint(bad).is_err());
        let mut bad = state.checkpoint();
        if let FileChange::Edit { edits, .. } = &mut bad.history[0].inverses[0].files[0] {
            edits[0].text = "WRONG".into();
        }
        assert!(Session::from_checkpoint(bad).is_err());
        let mut bad = state.checkpoint();
        bad.selection.as_mut().unwrap().anchor = 1;
        assert!(Session::from_checkpoint(bad).is_err());
        let mut bad = state.checkpoint();
        bad.history[0].inverses.clear();
        assert!(Session::from_checkpoint(bad).is_err());
        let mut accounting = state.checkpoint();
        accounting.history[0].bytes = usize::MAX;
        assert!(Session::from_checkpoint(accounting).is_ok());
        assert_eq!(state.sources()["main.tex"], "\u{feff}% keep\r\nFinal \u{1f600}\n");
    }

    #[test]
    fn merged_deletion_inverses_are_validated_by_state_not_structural_equality() {
        let mut state = session("ABC");
        let candidate = state.prepare(Patch { base: state.revision(), profile: None,
            files: vec![FileChange::Edit { file: "main.tex".into(), edits: vec![
                Replacement { range: 0..1, expected: "A".into(), text: "".into() },
                Replacement { range: 1..2, expected: "B".into(), text: "".into() }] }] }).unwrap();
        state.commit(candidate, Origin::Canvas, None).unwrap();
        let mut recovered = Session::from_checkpoint(state.checkpoint()).unwrap();
        recovered.undo().unwrap();
        assert_eq!(recovered.sources()["main.tex"], "ABC");
    }

    #[test]
    fn retained_history_cursor_roundtrips_after_bounded_eviction() {
        let mut state = session("A");
        for index in 0..160 {
            if index % 2 == 0 { replace(&mut state,"A","B",None); }
            else { replace(&mut state,"B","A",None); }
        }
        for _ in 0..10 { state.undo().unwrap(); }
        let mut recovered = Session::from_checkpoint(state.checkpoint()).unwrap();
        assert_eq!(recovered.history_len(),120);
        for _ in 0..110 {
            assert_eq!(state.undo().unwrap(),recovered.undo().unwrap());
            assert_eq!(state.sources(),recovered.sources());
        }
        for _ in 0..120 {
            assert_eq!(state.redo().unwrap(),recovered.redo().unwrap());
            assert_eq!(state.sources(),recovered.sources());
        }
    }

    #[cfg(feature = "recovery-serde")]
    #[test]
    fn optional_codec_roundtrips_losslessly_and_rejects_unknown_fields() {
        let state = timeline();
        let encoded = serde_json::to_vec(&state.checkpoint()).unwrap();
        let checkpoint: Checkpoint = serde_json::from_slice(&encoded).unwrap();
        let recovered = Session::from_checkpoint(checkpoint).unwrap();
        assert_eq!(recovered.sources(),state.sources());
        assert_eq!(recovered.profile(),state.profile());
        assert_eq!(recovered.revision(),state.revision());
        let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        unknown["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Checkpoint>(unknown).is_err());
    }
}
