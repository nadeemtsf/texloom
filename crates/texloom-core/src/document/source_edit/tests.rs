use super::*;

fn state(main: &str) -> Session {
    Session::new(
        BTreeMap::from([
            ("main.tex".into(), main.into()),
            ("chapter.tex".into(), "Unchanged chapter\r\n".repeat(64)),
        ]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap()
}
fn edit(session: &Session, from: &str, to: &str) -> Replacement {
    let offset = session.sources()["main.tex"].find(from).unwrap();
    Replacement {
        range: offset..offset + from.len(),
        expected: from.into(),
        text: to.into(),
    }
}
fn source(session: &mut Session, change: Replacement, group: Option<u64>) {
    let prepared = session
        .prepare_source_edit(session.revision(), "main.tex", vec![change])
        .unwrap();
    session
        .commit_source_edit(prepared, group.map(TypingGroup))
        .unwrap();
}
fn same(a: &Session, b: &Session) {
    assert_eq!(a.sources(), b.sources());
    assert_eq!(a.profile(), b.profile());
    assert_eq!(a.revision(), b.revision());
    assert_eq!(a.selection(), b.selection());
    assert_eq!(
        (a.can_undo(), a.can_redo(), a.history_len()),
        (b.can_undo(), b.can_redo(), b.history_len())
    );
    #[cfg(feature = "recovery-serde")]
    assert_eq!(
        serde_json::to_value(a.checkpoint()).unwrap(),
        serde_json::to_value(b.checkpoint()).unwrap()
    );
}

#[test]
fn typing_changes_only_its_buffer_and_keeps_existing_visual_candidates_frozen() {
    let original = "\u{feff}A\r\n\u{1f600}B\r\n";
    let mut session = state(original);
    let pointer = session.sources()["chapter.tex"].as_ptr();
    let old = session
        .prepare(Patch {
            base: session.revision(),
            files: vec![FileChange::Create {
                file: "style.sty".into(),
                text: "Unapproved candidate".into(),
            }],
            profile: None,
        })
        .unwrap();
    let plan = session
        .prepare_source_edit(
            session.revision(),
            "main.tex",
            vec![edit(&session, "B", "New\r\nName")],
        )
        .unwrap();
    assert_eq!(session.sources()["chapter.tex"].as_ptr(), pointer);
    assert!(plan.changed());
    assert_eq!(plan.base(), Revision(0));
    assert_eq!(plan.text(), "\u{feff}A\r\n\u{1f600}New\r\nName\r\n");
    assert_eq!(plan.recovery_patch().files.len(), 1);
    session
        .commit_source_edit(plan, Some(TypingGroup(1)))
        .unwrap();
    assert_eq!(
        session.sources()["chapter.tex"].as_ptr(),
        pointer,
        "Typing must not replace unrelated source allocations"
    );
    assert_eq!(old.sources()["main.tex"], original);
    assert_eq!(
        session.commit(old, Origin::Canvas, None),
        Err(EditError::StaleRevision)
    );
    session.undo().unwrap();
    assert_eq!(session.sources()["main.tex"], original);
    session.redo().unwrap();
    assert_eq!(
        session.sources()["main.tex"],
        "\u{feff}A\r\n\u{1f600}New\r\nName\r\n"
    );
}

#[test]
fn foreign_and_stale_source_plans_cannot_mutate_canonical_buffers() {
    let mut first = state("A");
    let mut other = state("A");
    let plan = first
        .prepare_source_edit(Revision(0), "main.tex", vec![edit(&first, "A", "B")])
        .unwrap();
    assert_eq!(
        other.commit_source_edit(plan, None),
        Err(EditError::ForeignSession)
    );
    let stale = first
        .prepare_source_edit(Revision(0), "main.tex", vec![edit(&first, "A", "C")])
        .unwrap();
    let change = edit(&first, "A", "B");
    source(&mut first, change, None);
    assert_eq!(
        first.commit_source_edit(stale, None),
        Err(EditError::StaleRevision)
    );
    assert_eq!(first.sources()["main.tex"], "B");
    assert_eq!(other.sources()["main.tex"], "A");
}

#[test]
fn no_op_after_undo_preserves_revision_redo_and_unrelated_buffer_identity() {
    let mut session = state("A");
    let change = edit(&session, "A", "B");
    source(&mut session, change, Some(1));
    session.undo().unwrap();
    let revision = session.revision();
    let pointer = session.sources()["chapter.tex"].as_ptr();
    let plan = session
        .prepare_source_edit(revision, "main.tex", vec![edit(&session, "A", "A")])
        .unwrap();
    assert!(!plan.changed());
    assert_eq!(
        session
            .commit_source_edit(plan, Some(TypingGroup(1)))
            .unwrap(),
        revision
    );
    assert_eq!(session.revision(), revision);
    assert!(session.can_redo());
    assert_eq!(session.sources()["chapter.tex"].as_ptr(), pointer);
    session.redo().unwrap();
    assert_eq!(session.sources()["main.tex"], "B");
}

#[test]
fn validation_refuses_bad_boundaries_expected_text_overlap_missing_files_and_exhausted_revisions() {
    let mut session = state("\u{feff}A\u{1f600}B");
    for edits in [
        vec![Replacement {
            range: 1..2,
            expected: "?".into(),
            text: "X".into(),
        }],
        vec![Replacement {
            range: 3..4,
            expected: "Z".into(),
            text: "X".into(),
        }],
        vec![
            Replacement {
                range: 3..4,
                expected: "A".into(),
                text: "X".into(),
            },
            Replacement {
                range: 3..3,
                expected: "".into(),
                text: "Y".into(),
            },
        ],
    ] {
        assert!(session
            .prepare_source_edit(Revision(0), "main.tex", edits)
            .is_err());
    }
    assert!(session
        .prepare_source_edit(Revision(1), "main.tex", vec![])
        .is_err());
    assert!(session
        .prepare_source_edit(Revision(0), "missing.tex", vec![])
        .is_err());
    assert_eq!(session.revision(), Revision(0));
    assert!(!session.can_undo());
    session.revision = Revision(u64::MAX);
    let plan = session
        .prepare_source_edit(
            session.revision(),
            "main.tex",
            vec![edit(&session, "A", "X")],
        )
        .unwrap();
    assert_eq!(
        session.commit_source_edit(plan, None),
        Err(EditError::RevisionExhausted)
    );
    assert_eq!(session.sources()["main.tex"], "\u{feff}A\u{1f600}B");
}

#[test]
fn file_and_total_project_byte_limits_are_preserved_without_committing_partial_source() {
    let session = state("A");
    assert!(session
        .prepare_source_edit(
            Revision(0),
            "main.tex",
            vec![Replacement {
                range: 0..1,
                expected: "A".into(),
                text: "X".repeat(4_000_001)
            }]
        )
        .is_err());
    let sources = (0..31)
        .map(|i| (format!("file-{i}.tex"), "A".repeat(2_000_000)))
        .collect();
    let session = Session::new(
        sources,
        Profile {
            main: "file-0.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap();
    assert!(session
        .prepare_source_edit(
            Revision(0),
            "file-0.tex",
            vec![Replacement {
                range: 0..0,
                expected: "".into(),
                text: "X".repeat(1_000_000)
            }]
        )
        .is_err());
    assert_eq!(session.sources()["file-0.tex"].len(), 2_000_000);
    assert_eq!(session.revision(), Revision(0));
}

#[test]
fn source_visual_and_profile_edits_keep_one_shared_exact_undo_timeline() {
    let mut session = state("Name");
    let original = session.sources().clone();
    for (from, to) in [("Name", "First"), ("First", "Second")] {
        let change = edit(&session, from, to);
        source(&mut session, change, Some(7));
    }
    let visual = session
        .prepare(Patch {
            base: session.revision(),
            files: vec![FileChange::Create {
                file: "objects.sty".into(),
                text: "% portable\r\n".into(),
            }],
            profile: None,
        })
        .unwrap();
    session.commit(visual, Origin::Canvas, None).unwrap();
    let settings = session
        .prepare(Patch {
            base: session.revision(),
            files: vec![],
            profile: Some(ProfileChange {
                expected: session.profile().clone(),
                replacement: Profile {
                    main: "main.tex".into(),
                    engine: "xelatex".into(),
                },
            }),
        })
        .unwrap();
    session.commit(settings, Origin::Settings, None).unwrap();
    let change = edit(&session, "Second", "Final");
    source(&mut session, change, Some(7));
    let final_sources = session.sources().clone();
    assert_eq!(session.history_len(), 4);
    for _ in 0..4 {
        assert!(session.undo().unwrap());
    }
    assert_eq!(session.sources(), &original);
    assert_eq!(session.profile().engine, "auto");
    assert!(!session.can_undo());
    for _ in 0..4 {
        assert!(session.redo().unwrap());
    }
    assert_eq!(session.sources(), &final_sources);
    assert_eq!(session.profile().engine, "xelatex");
}

#[test]
fn private_history_draft_is_discarded_when_a_later_coalesced_step_fails() {
    let mut session = state("A");
    let change = edit(&session, "A", "B");
    source(&mut session, change, Some(1));
    let change = edit(&session, "B", "C");
    source(&mut session, change, Some(1));
    let FileChange::Edit { edits, .. } = &mut session.history[0].inverses[0].files[0] else {
        panic!()
    };
    edits[0].expected = "WRONG".into();
    let before = session.sources().clone();
    let revision = session.revision();
    assert!(session.undo().is_err());
    assert_eq!(session.sources(), &before);
    assert_eq!(session.revision(), revision);
    assert!(session.can_undo());
    assert!(!session.can_redo());
}

#[test]
fn deterministic_unicode_edit_sequence_matches_full_snapshot_planning_and_recovery_data() {
    let mut fast = state("\u{feff}A\r\n\u{1f600}e\u{301} Name\n");
    let mut full = state("\u{feff}A\r\n\u{1f600}e\u{301} Name\n");
    let mut random = 0x5a17_u64;
    for step in 0..800 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        if step % 19 == 0 && fast.can_undo() {
            fast.undo().unwrap();
            full.undo().unwrap();
        } else if step % 23 == 0 && fast.can_redo() {
            fast.redo().unwrap();
            full.redo().unwrap();
        } else {
            let text = &fast.sources()["main.tex"];
            let boundaries = text
                .char_indices()
                .map(|(n, _)| n)
                .chain(std::iter::once(text.len()))
                .collect::<Vec<_>>();
            let index = random as usize % boundaries.len();
            let start = boundaries[index];
            let end = boundaries[(index + 1).min(boundaries.len() - 1)];
            let replacement =
                ["X", "\u{1f980}", "\r\n", "\u{301}", ""][((random >> 32) % 5) as usize];
            let selection = Selection {
                file: "main.tex".into(),
                anchor: start,
                head: end,
                secondary: vec![(0, 0)],
            };
            let change = Replacement {
                range: start..end,
                expected: text[start..end].into(),
                text: replacement.into(),
            };
            fast.set_selection(selection.clone()).unwrap();
            full.set_selection(selection).unwrap();
            let candidate = full
                .prepare(Patch {
                    base: full.revision(),
                    files: vec![FileChange::Edit {
                        file: "main.tex".into(),
                        edits: vec![change.clone()],
                    }],
                    profile: None,
                })
                .unwrap();
            let group = Some(TypingGroup(step / 3));
            full.commit(candidate, Origin::Source, group).unwrap();
            let candidate = fast
                .prepare_source_edit(fast.revision(), "main.tex", vec![change])
                .unwrap();
            fast.commit_source_edit(candidate, group).unwrap();
        }
        same(&fast, &full);
    }
    while fast.can_undo() {
        fast.undo().unwrap();
        full.undo().unwrap();
        same(&fast, &full);
    }
    while fast.can_redo() {
        fast.redo().unwrap();
        full.redo().unwrap();
        same(&fast, &full);
    }
}
