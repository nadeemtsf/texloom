use std::collections::BTreeMap;
use texloom_core::document::{projection::EditorProjection, *};

fn sources(text: &str) -> Sources {
    BTreeMap::from([("main.tex".into(), text.into())])
}
fn session(text: &str) -> Session {
    Session::new(
        sources(text),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap()
}
fn patch(s: &Session, range: std::ops::Range<usize>, expected: &str, text: &str) -> Patch {
    Patch {
        base: s.revision(),
        profile: None,
        files: vec![FileChange::Edit {
            file: "main.tex".into(),
            edits: vec![Replacement {
                range,
                expected: expected.into(),
                text: text.into(),
            }],
        }],
    }
}
fn edit(
    s: &mut Session,
    range: std::ops::Range<usize>,
    old: &str,
    new: &str,
    origin: Origin,
    group: Option<TypingGroup>,
) {
    let candidate = s.prepare(patch(s, range, old, new)).unwrap();
    s.commit(candidate, origin, group).unwrap();
}

#[test]
fn multiple_patches_preserve_every_unrelated_byte_and_reverse_exactly() {
    let original = "\u{feff}% keep\r\nName: Nadim 🦀\nالقدس\r\n";
    let a = original.find("Nadim").unwrap();
    let b = original.find("القدس").unwrap();
    let edits = vec![
        Replacement {
            range: b..b + "القدس".len(),
            expected: "القدس".into(),
            text: "Bethlehem".into(),
        },
        Replacement {
            range: a..a + 5,
            expected: "Nadim".into(),
            text: "Example".into(),
        },
    ];
    let (changed, inverse) = apply_replacements(original, &edits).unwrap();
    assert_eq!(changed, "\u{feff}% keep\r\nName: Example 🦀\nBethlehem\r\n");
    assert_eq!(
        apply_replacements(&changed, &inverse).unwrap().0.as_bytes(),
        original.as_bytes()
    );
}

#[test]
fn adjacent_deletions_have_an_applicable_inverse() {
    let (changed, inverse) = apply_replacements(
        "ABC",
        &[
            Replacement {
                range: 0..1,
                expected: "A".into(),
                text: "".into(),
            },
            Replacement {
                range: 1..2,
                expected: "B".into(),
                text: "".into(),
            },
        ],
    )
    .unwrap();
    assert_eq!(changed, "C");
    assert_eq!(apply_replacements(&changed, &inverse).unwrap().0, "ABC");
}

#[test]
fn invalid_ranges_offsets_preconditions_and_overlaps_are_refused() {
    let e = |range, expected: &str| Replacement {
        range,
        expected: expected.into(),
        text: "x".into(),
    };
    assert_eq!(
        apply_replacements("🦀", &[e(0..1, "")]),
        Err(EditError::InvalidBoundary)
    );
    assert_eq!(
        apply_replacements("abc", &[e(0..usize::MAX, "")]),
        Err(EditError::InvalidRange)
    );
    assert_eq!(
        apply_replacements("abc", &[e(0..1, "z")]),
        Err(EditError::ExpectedTextMismatch)
    );
    assert_eq!(
        apply_replacements("abc", &[e(0..2, "ab"), e(1..3, "bc")]),
        Err(EditError::OverlappingEdits)
    );
    assert_eq!(
        apply_replacements("abc", &[e(1..1, ""), e(1..1, "")]),
        Err(EditError::OverlappingEdits)
    );
}

#[test]
fn multi_file_failure_does_not_apply_the_successful_prefix() {
    let s = session("A");
    let original = s.sources().clone();
    let p = Patch {
        base: s.revision(),
        profile: None,
        files: vec![
            FileChange::Edit {
                file: "main.tex".into(),
                edits: vec![Replacement {
                    range: 0..1,
                    expected: "A".into(),
                    text: "B".into(),
                }],
            },
            FileChange::Remove {
                file: "missing.sty".into(),
                expected: "".into(),
            },
        ],
    };
    assert!(matches!(s.prepare(p), Err(EditError::MissingFile(_))));
    assert_eq!(s.sources(), &original);
    assert_eq!(s.revision(), Revision(0));
    assert!(!s.can_undo());
}

#[test]
fn creating_a_support_file_and_changing_the_build_profile_share_one_undo() {
    let mut s = session("Main");
    let original = s.sources().clone();
    let profile = s.profile().clone();
    let p = Patch {
        base: s.revision(),
        files: vec![
            FileChange::Create {
                file: "other.tex".into(),
                text: "Other".into(),
            },
            FileChange::Create {
                file: "objects.sty".into(),
                text: "Portable style\r\n".into(),
            },
        ],
        profile: Some(ProfileChange {
            expected: profile.clone(),
            replacement: Profile {
                main: "other.tex".into(),
                engine: "lualatex".into(),
            },
        }),
    };
    let candidate = s.prepare(p).unwrap();
    assert_eq!(s.sources(), &original);
    s.commit(candidate, Origin::Canvas, None).unwrap();
    let accepted = s.sources().clone();
    assert_eq!(s.history_len(), 1);
    assert!(s.undo().unwrap());
    assert_eq!(s.sources(), &original);
    assert_eq!(s.profile(), &profile);
    assert!(s.redo().unwrap());
    assert_eq!(s.sources(), &accepted);
    assert_eq!(s.profile().engine, "lualatex");
}

#[test]
fn discarded_preparation_is_read_only() {
    let s = session("Original");
    let candidate = s.prepare(patch(&s, 0..8, "Original", "Candidate")).unwrap();
    assert_eq!(candidate.sources()["main.tex"], "Candidate");
    drop(candidate);
    assert_eq!(s.sources()["main.tex"], "Original");
    assert_eq!(s.revision(), Revision(0));
    assert!(!s.can_undo());
}

#[test]
fn newer_typing_prevents_a_prepared_visual_candidate_from_committing() {
    let mut s = session("A");
    let visual = s.prepare(patch(&s, 0..1, "A", "Moved A")).unwrap();
    edit(&mut s, 0..1, "A", "Typed B", Origin::Source, None);
    assert_eq!(
        s.commit(visual, Origin::Canvas, None),
        Err(EditError::StaleRevision)
    );
    assert_eq!(s.sources()["main.tex"], "Typed B");
    assert_eq!(s.history_len(), 1);
}

#[test]
fn candidate_cannot_cross_projects_even_with_identical_sources_and_revisions() {
    let a = session("A");
    let mut b = session("A");
    let candidate = a.prepare(patch(&a, 0..1, "A", "B")).unwrap();
    assert_eq!(
        b.commit(candidate, Origin::Canvas, None),
        Err(EditError::ForeignSession)
    );
    assert_eq!(b.sources()["main.tex"], "A");
}

#[test]
fn undo_restoring_identical_bytes_does_not_revive_an_old_candidate() {
    let mut s = session("A");
    let old = s.prepare(patch(&s, 0..1, "A", "Old move")).unwrap();
    edit(&mut s, 0..1, "A", "B", Origin::Source, None);
    s.undo().unwrap();
    assert_eq!(s.sources()["main.tex"], "A");
    assert_eq!(s.revision(), Revision(2));
    assert_eq!(
        s.commit(old, Origin::Canvas, None),
        Err(EditError::StaleRevision)
    );
    s.redo().unwrap();
    assert_eq!(s.revision(), Revision(3));
}

#[test]
fn typing_can_coalesce_but_canvas_edits_always_have_their_own_entry() {
    let mut s = session("");
    let g = Some(TypingGroup(9));
    edit(&mut s, 0..0, "", "A", Origin::Source, g);
    edit(&mut s, 1..1, "", "B", Origin::Source, g);
    assert_eq!(s.history_len(), 1);
    edit(&mut s, 0..2, "AB", "Moved", Origin::Canvas, g);
    assert_eq!(s.history_len(), 2);
    s.undo().unwrap();
    assert_eq!(s.sources()["main.tex"], "AB");
    s.undo().unwrap();
    assert_eq!(s.sources()["main.tex"], "");
    s.redo().unwrap();
    assert_eq!(s.sources()["main.tex"], "AB");
}

#[test]
fn editing_after_undo_discards_redo_and_noop_does_not_change_revision() {
    let mut s = session("A");
    edit(&mut s, 0..1, "A", "B", Origin::Source, None);
    s.undo().unwrap();
    let unchanged = s.prepare(patch(&s, 0..1, "A", "A")).unwrap();
    let rev = s.revision();
    s.commit(unchanged, Origin::Source, None).unwrap();
    assert_eq!(s.revision(), rev);
    assert!(s.can_redo());
    edit(&mut s, 0..1, "A", "C", Origin::Source, None);
    assert!(!s.can_redo());
    assert!(!s.redo().unwrap());
}

#[test]
fn deleting_the_main_file_or_conflicting_profile_fails_without_mutation() {
    let s = session("A");
    assert!(matches!(
        s.prepare(Patch {
            base: s.revision(),
            profile: None,
            files: vec![FileChange::Remove {
                file: "main.tex".into(),
                expected: "A".into()
            }]
        }),
        Err(EditError::InvalidProfile)
    ));
    let mut p = patch(&s, 0..1, "A", "B");
    p.profile = Some(ProfileChange {
        expected: Profile {
            main: "wrong.tex".into(),
            engine: "auto".into(),
        },
        replacement: s.profile().clone(),
    });
    assert!(matches!(s.prepare(p), Err(EditError::ExpectedTextMismatch)));
    assert_eq!(s.sources()["main.tex"], "A");
}

#[test]
fn editor_projection_preserves_bom_mixed_eols_and_supplementary_unicode() {
    let original = "\u{feff}A\r\n🦀B\nC\rD";
    let p = EditorProjection::new(original);
    assert_eq!(p.text(), "A\n🦀B\nC\nD");
    assert_eq!(p.source_offset(0).unwrap(), 3);
    assert_eq!(p.source_offset(3), Err(EditError::InvalidEditorOffset));
    assert_eq!(p.editor_offset(5), Err(EditError::InvalidBoundary));
    let replacement = p.replacement(4, 5, "Name\nTitle").unwrap();
    let (changed, inverse) = apply_replacements(original, &[replacement]).unwrap();
    assert_eq!(changed, "\u{feff}A\r\n🦀Name\nTitle\nC\rD");
    assert_eq!(apply_replacements(&changed, &inverse).unwrap().0, original);
}

#[test]
fn inserted_lines_use_the_local_eol_without_rewriting_other_lines() {
    let original = "A\r\nB\nC";
    let p = EditorProjection::new(original);
    let (changed, _) =
        apply_replacements(original, &[p.replacement(0, 1, "X\nY").unwrap()]).unwrap();
    assert_eq!(changed, "X\r\nY\r\nB\nC");
    let eof = EditorProjection::new("A\r\nB");
    assert_eq!(eof.replacement(3, 3, "\nC").unwrap().text, "\r\nC");
}

#[test]
fn exhaustive_unicode_patch_roundtrips_use_original_bytes_as_the_oracle() {
    let samples = ["", "A\r\nB", "\u{feff}Nadim", "🦀éالقدس\n", "A\nB\rC\r\n"];
    for original in samples {
        let boundaries: Vec<_> = (0..=original.len())
            .filter(|&i| original.is_char_boundary(i))
            .collect();
        for &a in &boundaries {
            for &b in &boundaries {
                if a > b {
                    continue;
                }
                for inserted in ["", "X", "🦀\r\nالقدس"] {
                    let e = Replacement {
                        range: a..b,
                        expected: original[a..b].into(),
                        text: inserted.into(),
                    };
                    let (changed, inverse) = apply_replacements(original, &[e]).unwrap();
                    assert_eq!(
                        apply_replacements(&changed, &inverse).unwrap().0.as_bytes(),
                        original.as_bytes()
                    );
                    assert!(changed.starts_with(&original[..a]));
                    assert!(changed.ends_with(&original[b..]));
                }
            }
        }
    }
}

#[test]
fn undo_and_redo_restore_the_editors_source_selection() {
    let mut s = session("AB");
    s.set_selection(Selection {
        file: "main.tex".into(),
        anchor: 1,
        head: 2,
        secondary: vec![(0, 0)],
    })
    .unwrap();
    edit(
        &mut s,
        1..2,
        "B",
        "Name",
        Origin::Source,
        Some(TypingGroup(1)),
    );
    s.set_selection(Selection {
        file: "main.tex".into(),
        anchor: 5,
        head: 5,
        secondary: vec![(0, 0)],
    })
    .unwrap();
    s.undo().unwrap();
    assert_eq!(s.selection().unwrap().anchor, 1);
    assert_eq!(s.selection().unwrap().head, 2);
    assert_eq!(s.selection().unwrap().secondary, vec![(0, 0)]);
    s.redo().unwrap();
    assert_eq!(s.selection().unwrap().head, 5);
    assert_eq!(s.selection().unwrap().secondary, vec![(0, 0)]);
}

#[test]
fn selection_updates_do_not_change_source_revisions_or_invalidate_candidates() {
    let mut s = session("A");
    let candidate = s.prepare(patch(&s, 0..1, "A", "B")).unwrap();
    s.set_selection(Selection {
        file: "main.tex".into(),
        anchor: 1,
        head: 1,
        secondary: vec![],
    })
    .unwrap();
    assert_eq!(s.revision(), Revision(0));
    s.commit(candidate, Origin::Canvas, None).unwrap();
    assert_eq!(s.sources()["main.tex"], "B");
}

#[test]
fn history_is_bounded_without_losing_the_most_recent_undo() {
    let mut s = session("0");
    for n in 1..=150 {
        let old = (n - 1).to_string();
        edit(
            &mut s,
            0..old.len(),
            &old,
            &n.to_string(),
            Origin::Source,
            None,
        );
    }
    assert_eq!(s.history_len(), 120);
    for _ in 0..120 {
        assert!(s.undo().unwrap());
    }
    assert_eq!(s.sources()["main.tex"], "30");
    assert!(!s.undo().unwrap());
}
