use super::*;
use std::collections::BTreeMap;
use texloom_core::document::{FileChange, Profile, ProfileChange, Replacement};

struct Fixture {
    root: PathBuf,
    store: Store,
    disk: DiskSnapshot,
    session: Session,
}
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!("texloom-typing-recovery-{}", identity()));
        fs::create_dir_all(root.join("project")).unwrap();
        fs::write(root.join("project/main.tex"), source).unwrap();
        let sources = BTreeMap::from([("main.tex".into(), source.into())]);
        let disk = DiskSnapshot::capture(&root.join("project"), &sources).unwrap();
        let session = Session::new(
            sources,
            Profile {
                main: "main.tex".into(),
                engine: "auto".into(),
            },
        )
        .unwrap();
        let store = Store::open(&root.join("drafts")).unwrap();
        Self {
            root,
            store,
            disk,
            session,
        }
    }
    fn change(&mut self, from: &str, to: &str, group: Option<u64>) -> Mutation {
        let start = self.session.sources()["main.tex"].find(from).unwrap();
        self.patch(
            Patch {
                base: self.session.revision(),
                profile: None,
                files: vec![FileChange::Edit {
                    file: "main.tex".into(),
                    edits: vec![Replacement {
                        range: start..start + from.len(),
                        expected: from.into(),
                        text: to.into(),
                    }],
                }],
            },
            Origin::Source,
            group,
        )
    }
    fn patch(&mut self, patch: Patch, origin: Origin, group: Option<u64>) -> Mutation {
        let candidate = self.session.prepare(patch).unwrap();
        let mutation = Mutation::Patch {
            patch: candidate.recovery_patch(),
            origin,
            group: group.map(TypingGroup),
        };
        self.session
            .commit(candidate, origin, group.map(TypingGroup))
            .unwrap();
        mutation
    }
    fn record(&self, journal: &mut Journal, base: u64, mutation: Mutation) -> Result<()> {
        journal.record(&self.store, base, mutation, &self.session, &self.disk)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.root.starts_with(std::env::temp_dir()));
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn flushed_mutations_restore_exact_sources_profile_redo_and_original_typing_groups() {
    let original = "\u{feff}% keep\r\nName \u{1f600}\n";
    let mut f = Fixture::new(original);
    // Checkpoint in the middle of a typing group: replay must not split undo.
    f.change("Name", "First", Some(7));
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let mutation = f.change("First", "Second", Some(7));
    f.record(&mut journal, 1, mutation).unwrap();
    let selection = Selection {
        file: "main.tex".into(),
        anchor: 3,
        head: 3,
        secondary: vec![(4, 4)],
    };
    f.session.set_selection(selection.clone()).unwrap();
    f.record(&mut journal, 2, Mutation::Selection { selection })
        .unwrap();
    let mutation = f.patch(
        Patch {
            base: f.session.revision(),
            profile: None,
            files: vec![FileChange::Create {
                file: "style.sty".into(),
                text: "% style\r\n".into(),
            }],
        },
        Origin::Canvas,
        None,
    );
    f.record(&mut journal, 2, mutation).unwrap();
    let mutation = f.patch(
        Patch {
            base: f.session.revision(),
            files: vec![],
            profile: Some(ProfileChange {
                expected: f.session.profile().clone(),
                replacement: Profile {
                    main: "main.tex".into(),
                    engine: "xelatex".into(),
                },
            }),
        },
        Origin::Settings,
        None,
    );
    f.record(&mut journal, 3, mutation).unwrap();
    f.session.undo().unwrap();
    f.record(&mut journal, 4, Mutation::Undo).unwrap();
    let id = journal.id().to_string();
    assert_eq!(journal.protected_revision(), Some(5));
    drop(journal);
    let mut recovered = f.store.recover(&id, &f.disk.root).unwrap();
    assert!(recovered.warning.is_none());
    assert_eq!(recovered.session.sources(), f.session.sources());
    assert_eq!(recovered.session.selection(), f.session.selection());
    assert!(recovered.session.can_redo());
    assert_eq!(
        fs::read_to_string(f.disk.root.join("main.tex")).unwrap(),
        original
    );
    recovered.session.undo().unwrap();
    recovered.session.undo().unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], original);
    assert!(!recovered.session.can_undo());
    recovered.session.redo().unwrap();
    recovered.session.redo().unwrap();
    recovered.session.redo().unwrap();
    assert_eq!(recovered.session.profile().engine, "xelatex");
    assert_eq!(recovered.session.sources()["style.sty"], "% style\r\n");
}

#[test]
fn recovery_retains_saved_conflict_fingerprints_and_never_writes_originals() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let mutation = f.change("A", "B", None);
    f.record(&mut journal, 0, mutation).unwrap();
    assert!(matches!(
        f.disk.save(f.session.sources()).unwrap(),
        crate::persistence::SaveOutcome::Saved
    ));
    f.record(
        &mut journal,
        1,
        Mutation::Saved {
            disk: f.disk.clone(),
        },
    )
    .unwrap();
    let mutation = f.change("B", "C", None);
    f.record(&mut journal, 1, mutation).unwrap();
    let foreign = f
        .session
        .prepare(Patch {
            base: f.session.revision(),
            files: vec![FileChange::Create {
                file: "new.sty".into(),
                text: "Never approved".into(),
            }],
            profile: None,
        })
        .unwrap();
    let id = journal.id().to_string();
    drop(journal);
    fs::write(f.disk.root.join("main.tex"), b"EXTERNAL").unwrap();
    let mut recovered = f.store.recover(&id, &f.disk.root).unwrap();
    assert_eq!(recovered.disk.files, f.disk.files);
    assert_eq!(recovered.session.sources()["main.tex"], "C");
    assert_eq!(
        recovered
            .disk
            .conflicts(recovered.session.sources())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(fs::read(f.disk.root.join("main.tex")).unwrap(), b"EXTERNAL");
    assert!(recovered
        .session
        .commit(foreign, Origin::Canvas, None)
        .is_err());
    assert!(f.store.recover(&id, &f.root).is_err());
    assert!(f.store.recover("../outside", &f.disk.root).is_err());
}

#[test]
fn active_drafts_are_locked_and_excluded_from_crash_inventory() {
    let f = Fixture::new("A");
    let journal = f.store.begin(&f.disk, &f.session).unwrap();
    assert!(f.store.inventory().unwrap().drafts.is_empty());
    assert!(f.store.recover(journal.id(), &f.disk.root).is_err());
    assert!(f.store.discard(journal.id(), &f.disk.root).is_err());
    let id = journal.id().to_string();
    drop(journal);
    assert_eq!(f.store.inventory().unwrap().drafts[0].id, id);
    f.store.discard(&id, &f.disk.root).unwrap();
    assert!(f.store.inventory().unwrap().drafts.is_empty());
    assert_eq!(fs::read(f.disk.root.join("main.tex")).unwrap(), b"A");
}

#[test]
fn every_final_frame_truncation_restores_only_the_last_complete_verified_prefix() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let mutation = f.change("A", "B", None);
    f.record(&mut journal, 0, mutation).unwrap();
    let prefix = journal.bytes as usize;
    let mutation = f.change("B", "C", None);
    f.record(&mut journal, 1, mutation).unwrap();
    let path = journal.directory.join(generation_name(1));
    let id = journal.id().to_string();
    drop(journal);
    let full = fs::read(&path).unwrap();
    for end in prefix..full.len() {
        fs::write(&path, &full[..end]).unwrap();
        let recovered = f.store.recover(&id, &f.disk.root).unwrap();
        assert_eq!(recovered.session.sources()["main.tex"], "B", "cut {end}");
        assert_eq!(recovered.session.revision(), Revision(1));
        assert_eq!(recovered.warning.is_some(), end != prefix);
        assert_eq!(fs::metadata(&path).unwrap().len(), end as u64);
    }
    fs::write(&path, full).unwrap();
    assert_eq!(
        f.store
            .recover(&id, &f.disk.root)
            .unwrap()
            .session
            .sources()["main.tex"],
        "C"
    );
}

#[test]
fn complete_frame_corruption_or_reordering_is_refused_without_deleting_data() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let first_start = journal.bytes as usize;
    let mutation = f.change("A", "B", None);
    f.record(&mut journal, 0, mutation).unwrap();
    let second_start = journal.bytes as usize;
    let mutation = f.change("B", "C", None);
    f.record(&mut journal, 1, mutation).unwrap();
    let path = journal.directory.join(generation_name(1));
    let id = journal.id().to_string();
    drop(journal);
    let bytes = fs::read(&path).unwrap();
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    fs::write(&path, &corrupt).unwrap();
    assert!(f
        .store
        .recover(&id, &f.disk.root)
        .unwrap_err_text()
        .contains("checksum"));
    assert!(f.store.discard(&id, &f.disk.root).is_err());
    assert_eq!(fs::read(&path).unwrap(), corrupt);
    let reordered = [
        &bytes[..first_start],
        &bytes[second_start..],
        &bytes[first_start..second_start],
    ]
    .concat();
    fs::write(&path, reordered).unwrap();
    assert!(f.store.recover(&id, &f.disk.root).is_err());
    assert_eq!(fs::read(f.disk.root.join("main.tex")).unwrap(), b"A");
}

trait ErrorText {
    fn unwrap_err_text(self) -> String;
}
impl<T> ErrorText for Result<T> {
    fn unwrap_err_text(self) -> String {
        match self {
            Ok(_) => panic!("Expected refusal"),
            Err(error) => error,
        }
    }
}

#[test]
fn compaction_retains_exact_history_and_prunes_only_matching_closed_generation_bytes() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let mutation = f.change("A", "B", Some(1));
    f.record(&mut journal, 0, mutation).unwrap();
    let first = journal.directory.join(generation_name(1));
    journal.events = ROTATE_EVENTS;
    let mutation = f.change("B", "C", Some(1));
    f.record(&mut journal, 1, mutation).unwrap();
    assert_eq!(generations(&journal.directory).unwrap().len(), 2);
    journal.events = ROTATE_EVENTS;
    let mutation = f.change("C", "D", Some(1));
    f.record(&mut journal, 2, mutation).unwrap();
    assert!(!first.exists());
    assert_eq!(generations(&journal.directory).unwrap().len(), 2);
    let retained = journal.directory.join(generation_name(2));
    let mut unknown = fs::read(&retained).unwrap();
    unknown.push(b'?');
    fs::write(&retained, &unknown).unwrap();
    journal.events = ROTATE_EVENTS;
    let mutation = f.change("D", "E", Some(1));
    f.record(&mut journal, 3, mutation).unwrap();
    assert_eq!(fs::read(&retained).unwrap(), unknown);
    let id = journal.id().to_string();
    drop(journal);
    let mut recovered = f.store.recover(&id, &f.disk.root).unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], "E");
    recovered.session.undo().unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], "A");
}

#[test]
fn failed_storage_keeps_accepted_source_and_repairs_protection_with_a_new_full_checkpoint() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    // Inject a real write error without changing the canonical edit or weakening
    // filesystem permissions: replace this test's writer with a read-only handle.
    journal.file = File::open(journal.directory.join(generation_name(1))).unwrap();
    let mutation = f.change("A", "B", Some(2));
    assert!(f.record(&mut journal, 0, mutation).is_err());
    assert_eq!(f.session.sources()["main.tex"], "B");
    assert_eq!(journal.protected_revision(), None);
    let mutation = f.change("B", "C", Some(2));
    f.record(&mut journal, 1, mutation).unwrap();
    assert_eq!(journal.protected_revision(), Some(2));
    assert_eq!(journal.generation, 2);
    let id = journal.id().to_string();
    drop(journal);
    let mut recovered = f.store.recover(&id, &f.disk.root).unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], "C");
    recovered.session.undo().unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], "A");
}

#[test]
fn incomplete_new_checkpoint_falls_back_with_a_warning_and_is_never_silently_removed() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let mutation = f.change("A", "B", None);
    f.record(&mut journal, 0, mutation).unwrap();
    let directory = journal.directory.clone();
    let id = journal.id().to_string();
    drop(journal);
    let partial = directory.join(generation_name(2));
    fs::write(&partial, [8, 0, 0]).unwrap();
    let recovered = f.store.recover(&id, &f.disk.root).unwrap();
    assert_eq!(recovered.session.sources()["main.tex"], "B");
    assert!(recovered.warning.is_some());
    assert!(f.store.discard(&id, &f.disk.root).is_err());
    assert_eq!(fs::read(partial).unwrap(), [8, 0, 0]);
}

#[test]
fn path_escape_unknown_files_and_invalid_record_lengths_are_refused() {
    let f = Fixture::new("A");
    let journal = f.store.begin(&f.disk, &f.session).unwrap();
    let directory = journal.directory.clone();
    let id = journal.id().to_string();
    drop(journal);
    let outside = directory.join("notes.txt");
    fs::write(&outside, b"User file").unwrap();
    assert!(f.store.recover(&id, &f.disk.root).is_err());
    assert!(f.store.discard(&id, &f.disk.root).is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"User file");
    fs::remove_file(outside).unwrap();
    let path = directory.join(generation_name(1));
    let mut file = File::open(&path).unwrap();
    let mut consumed = 0;
    let FrameRead::Complete(payload, _) = read_frame(&mut file, &[0; 32], &mut consumed).unwrap()
    else {
        panic!()
    };
    drop(file);
    let mut header: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    header["checkpoint"]["sources"]["../escape.tex"] = serde_json::json!("Outside");
    let (bytes, _) = frame(&serde_json::to_vec(&header).unwrap(), &[0; 32]).unwrap();
    fs::write(&path, bytes).unwrap();
    assert!(f.store.recover(&id, &f.disk.root).is_err());
    assert!(!f.root.join("escape.tex").exists());
    fs::write(&path, [255; 16]).unwrap();
    assert!(f.store.recover(&id, &f.disk.root).is_err());
}

#[test]
fn store_budget_lock_failure_is_visible_and_does_not_rollback_the_native_edit() {
    let mut f = Fixture::new("A");
    let mut journal = f.store.begin(&f.disk, &f.session).unwrap();
    let storage = f.store.budget_lock().unwrap();
    let mutation = f.change("A", "B", None);
    assert!(f.record(&mut journal, 0, mutation).is_err());
    assert_eq!(f.session.sources()["main.tex"], "B");
    assert_eq!(journal.protected_revision(), None);
    drop(storage);
    let mutation = f.change("B", "C", None);
    f.record(&mut journal, 1, mutation).unwrap();
    assert_eq!(journal.protected_revision(), Some(2));
}

#[test]
fn unknown_store_entries_pause_new_protection_but_do_not_hide_valid_crash_drafts() {
    let f = Fixture::new("A");
    let journal = f.store.begin(&f.disk, &f.session).unwrap();
    let id = journal.id().to_string();
    drop(journal);
    let unknown = f.store.root.join("user-note.txt");
    fs::write(&unknown, b"Keep").unwrap();
    let inventory = f.store.inventory().unwrap();
    assert_eq!(inventory.drafts.len(), 1);
    assert_eq!(inventory.errors.len(), 1);
    assert!(f.store.begin(&f.disk, &f.session).is_err());
    assert_eq!(
        f.store
            .recover(&id, &f.disk.root)
            .unwrap()
            .session
            .sources()["main.tex"],
        "A"
    );
    f.store.discard(&id, &f.disk.root).unwrap();
    assert_eq!(fs::read(unknown).unwrap(), b"Keep");
}
