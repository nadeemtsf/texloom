use super::*;

fn fixture() -> (Backend, Project, PathBuf) {
    let root =
        std::env::temp_dir().join(format!("texloom-native-typing-{}", Uuid::new_v4().simple()));
    fs::create_dir_all(root.join("project")).unwrap();
    fs::write(
        root.join("project/main.tex"),
        "\u{feff}\\documentclass{article}\r\nName\r\n",
    )
    .unwrap();
    let backend = Backend::new(root.join("app"));
    let project = backend
        .open_project(Some(
            root.join("project/main.tex").to_string_lossy().into_owned(),
        ))
        .unwrap();
    (backend, project, root)
}
fn change(
    backend: &Backend,
    id: &str,
    from: &str,
    to: &str,
    group: Option<u64>,
) -> SourceEditReply {
    let snapshot = backend.document_snapshot(id).unwrap();
    let source = &snapshot.sources["main.tex"];
    let byte = source.find(from).unwrap();
    let offset = EditorProjection::new(source).editor_offset(byte).unwrap();
    backend
        .document_edit_delta(
            id,
            snapshot.revision,
            "main.tex",
            vec![EditorChange {
                offset,
                length: from.encode_utf16().count(),
                text: to.into(),
            }],
            group,
        )
        .unwrap()
}

#[test]
fn native_typing_survives_a_new_backend_and_recovers_one_exact_history_without_pdf_authority() {
    let (backend, project, root) = fixture();
    let original = project.sources.clone();
    let first = change(&backend, &project.id, "Name", "First", Some(7));
    assert_eq!(first.recovery.protected_revision, Some(first.revision));
    assert!(first.recovery.error.is_none());
    let second = change(&backend, &project.id, "First", "Second\nName", Some(7));
    let full = backend.document_snapshot(&project.id).unwrap();
    assert!(backend.typing_drafts().unwrap().drafts.is_empty());
    let old_candidate = backend
        .document_prepare(
            &project.id,
            full.revision,
            BTreeMap::from([("main.tex".into(), "Unverified visual source".into())]),
        )
        .unwrap();
    drop(backend); // no clean-close marker; recovery reads disk, not old RAM
    let backend = Backend::new(root.join("app"));
    let inventory = backend.typing_drafts().unwrap();
    assert_eq!(inventory.drafts.len(), 1);
    let recovered = backend.open_typing_draft(&inventory.drafts[0].id).unwrap();
    assert!(recovered.restored);
    assert_eq!(recovered.sources, full.sources);
    assert_eq!(
        backend.document_snapshot(&recovered.id).unwrap().revision,
        second.revision
    );
    assert!(backend
        .document_commit(&recovered.id, &old_candidate.token, "old-build")
        .is_err());
    let undone = backend
        .document_history(&recovered.id, second.revision, false)
        .unwrap();
    assert_eq!(undone.sources, original);
    let redone = backend
        .document_history(&recovered.id, undone.revision, true)
        .unwrap();
    assert_eq!(redone.sources, recovered.sources);
    assert_eq!(
        fs::read_to_string(root.join("project/main.tex")).unwrap(),
        original["main.tex"]
    );
    assert_eq!(
        backend.typing_drafts().unwrap().drafts.len(),
        1,
        "Earlier crash draft is retained for review"
    );
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovered_save_keeps_the_original_external_edit_precondition_and_project_copy_escape() {
    let (backend, project, root) = fixture();
    let edited = change(&backend, &project.id, "Name", "Local", None);
    drop(backend);
    fs::write(root.join("project/main.tex"), b"EXTERNAL").unwrap();
    let backend = Backend::new(root.join("app"));
    let draft = backend.typing_drafts().unwrap().drafts.remove(0);
    let recovered = backend.open_typing_draft(&draft.id).unwrap();
    let error = backend
        .save_document(&recovered.id, edited.revision)
        .unwrap_err();
    assert!(error.contains("Save conflict"), "{error}");
    assert_eq!(
        fs::read(root.join("project/main.tex")).unwrap(),
        b"EXTERNAL"
    );
    let copy = PathBuf::from(
        backend
            .copy_document(&recovered.id, edited.revision, &root.to_string_lossy())
            .unwrap(),
    );
    assert_eq!(
        fs::read_to_string(copy.join("main.tex")).unwrap(),
        recovered.sources["main.tex"]
    );
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_io_failure_does_not_reject_typing_or_misreport_an_accepted_explicit_save() {
    let (backend, project, root) = fixture();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join("app/.source-recovery/store.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let edited = change(&backend, &project.id, "Name", "Local", Some(1));
    assert_eq!(edited.revision, 1);
    assert!(edited.recovery.error.is_some());
    assert!(edited.recovery.protected_revision.is_none());
    let saved = backend.save_document(&project.id, 1).unwrap();
    assert!(saved.recovery.error.is_some());
    assert!(fs::read_to_string(root.join("project/main.tex"))
        .unwrap()
        .contains("Local"));
    assert_eq!(
        backend.document_snapshot(&project.id).unwrap().sources["main.tex"],
        edited.sources["main.tex"]
    );
    drop(lock);
    let repaired = backend.retry_typing_recovery(&project.id, 1).unwrap();
    assert_eq!(repaired.recovery.protected_revision, Some(1));
    assert!(repaired.recovery.error.is_none());
    assert!(backend.retry_typing_recovery(&project.id, 0).is_err());
    let current = backend.document_snapshot(&project.id).unwrap();
    assert_eq!(current.revision, 1);
    assert!(current.can_undo);
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn close_is_revision_guarded_and_explicit_discard_removes_only_the_current_live_draft() {
    let (backend, project, root) = fixture();
    change(&backend, &project.id, "Name", "Earlier", None);
    drop(backend);
    let backend = Backend::new(root.join("app"));
    let original_id = backend.typing_drafts().unwrap().drafts[0].id.clone();
    let recovered = backend.open_typing_draft(&original_id).unwrap();
    let edited = change(&backend, &recovered.id, "Earlier", "Later", None);
    assert!(backend
        .close_document_checked(&recovered.id, edited.revision - 1, true)
        .is_err());
    assert_eq!(
        backend.document_snapshot(&recovered.id).unwrap().revision,
        edited.revision
    );
    let closed = backend
        .close_document_checked(&recovered.id, edited.revision, true)
        .unwrap();
    assert!(closed.warning.is_none());
    assert!(backend.document_snapshot(&recovered.id).is_err());
    let inventory = backend.typing_drafts().unwrap();
    assert_eq!(inventory.drafts.len(), 1);
    assert_eq!(inventory.drafts[0].id, original_id);
    backend.discard_typing_draft(&original_id).unwrap();
    assert!(backend.typing_drafts().unwrap().drafts.is_empty());
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_cleanup_reports_retained_data_without_leaving_a_half_closed_native_session() {
    let (backend, project, root) = fixture();
    let snapshot = backend.document_snapshot(&project.id).unwrap();
    let id = snapshot.recovery.draft_id.unwrap();
    fs::write(
        root.join("app/.source-recovery")
            .join(&id)
            .join("user-note.txt"),
        b"Keep",
    )
    .unwrap();
    let reply = backend
        .close_document_checked(&project.id, 0, true)
        .unwrap();
    assert!(reply.warning.is_some());
    assert!(backend.document_snapshot(&project.id).is_err());
    assert_eq!(
        fs::read(
            root.join("app/.source-recovery")
                .join(id)
                .join("user-note.txt")
        )
        .unwrap(),
        b"Keep"
    );
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_project_folder_still_allows_source_history_recovery_and_a_new_source_copy() {
    let (backend, project, root) = fixture();
    change(&backend, &project.id, "Name", "Recovered", None);
    drop(backend);
    let original_project = root.join("project");
    assert!(original_project.starts_with(&root));
    fs::remove_dir_all(&original_project).unwrap();
    let backend = Backend::new(root.join("app"));
    let draft = backend.typing_drafts().unwrap().drafts.remove(0);
    let recovered = backend.open_typing_draft(&draft.id).unwrap();
    assert!(recovered.source_only);
    assert!(recovered
        .draft_warning
        .as_ref()
        .unwrap()
        .contains("missing"));
    let snapshot = backend.document_snapshot(&recovered.id).unwrap();
    assert_eq!(
        snapshot.recovery.protected_revision,
        Some(snapshot.revision)
    );
    assert!(snapshot.can_undo);
    assert!(backend
        .save_document(&recovered.id, snapshot.revision)
        .is_err());
    assert!(!original_project.exists());
    let copy = PathBuf::from(
        backend
            .copy_document(&recovered.id, snapshot.revision, &root.to_string_lossy())
            .unwrap(),
    );
    assert_eq!(
        fs::read_to_string(copy.join("main.tex")).unwrap(),
        recovered.sources["main.tex"]
    );
    assert!(!original_project.exists());
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}
