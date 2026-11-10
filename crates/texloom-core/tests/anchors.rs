use std::collections::BTreeMap;
use texloom_core::document::*;
use texloom_core::document::{position::*, projection::EditorProjection};

fn state(text: &str) -> Session {
    Session::new(
        BTreeMap::from([("main.tex".into(), text.into())]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap()
}
fn selected(start: usize, end: usize) -> Selection {
    Selection {
        file: "main.tex".into(),
        anchor: start,
        head: end,
        secondary: vec![],
    }
}
fn apply(s: &mut Session, edits: Vec<Replacement>, fast: bool) {
    if fast {
        let p = s
            .prepare_source_edit(s.revision(), "main.tex", edits)
            .unwrap();
        s.commit_source_edit(p, None).unwrap();
    } else {
        let p = s
            .prepare(Patch {
                base: s.revision(),
                files: vec![FileChange::Edit {
                    file: "main.tex".into(),
                    edits,
                }],
                profile: None,
            })
            .unwrap();
        s.commit(p, Origin::Source, None).unwrap();
    }
}

#[test]
fn prefix_edit_keeps_the_selected_repeated_occurrence_and_history() {
    for fast in [false, true] {
        let mut s = state("same same tail");
        let original = selected(9, 5); // reversed selection of the second occurrence
        s.set_selection(original.clone()).unwrap();
        apply(
            &mut s,
            vec![Replacement {
                range: 0..0,
                expected: "".into(),
                text: "X ".into(),
            }],
            fast,
        );
        assert_eq!(s.selection(), Some(&selected(11, 7)));
        s.undo().unwrap();
        assert_eq!(s.selection(), Some(&original));
        s.redo().unwrap();
        assert_eq!(s.selection(), Some(&selected(11, 7)));
    }
}

#[test]
fn destroying_selected_content_clears_identity_even_when_offsets_still_fit() {
    for fast in [false, true] {
        let mut s = state("target unrelated content remains");
        s.set_selection(selected(0, 6)).unwrap();
        apply(
            &mut s,
            vec![Replacement {
                range: 0..6,
                expected: "target".into(),
                text: "".into(),
            }],
            fast,
        );
        assert!(
            s.selection().is_none(),
            "old offsets must not select unrelated text"
        );
        s.undo().unwrap();
        assert_eq!(s.selection(), Some(&selected(0, 6)));
        s.redo().unwrap();
        assert!(s.selection().is_none());
    }
}

#[test]
fn losing_one_secondary_range_clears_the_whole_selection() {
    for fast in [false, true] {
        let mut s = state("first second tail remains");
        let mut selection = selected(0, 5);
        selection.secondary.push((6, 12));
        s.set_selection(selection).unwrap();
        apply(
            &mut s,
            vec![Replacement {
                range: 6..12,
                expected: "second".into(),
                text: "other!".into(),
            }],
            fast,
        );
        assert!(s.selection().is_none());
    }
}

fn point(source: &str, offset: usize) -> SourceByteOffset {
    SourceByteOffset::checked(source, offset).unwrap()
}
fn logical(source: &str, anchor: usize, head: usize) -> LogicalRange {
    LogicalRange {
        anchor: point(source, anchor),
        head: point(source, head),
    }
}
fn replacement(source: &str, range: std::ops::Range<usize>, text: &str) -> Replacement {
    Replacement {
        expected: source[range.clone()].into(),
        range,
        text: text.into(),
    }
}

#[test]
fn typed_editor_coordinates_reject_surrogates_bom_and_crlf_interior() {
    let source = "\u{feff}😀\r\nאבe\u{301}";
    let p = EditorProjection::new(source);
    assert_eq!(
        p.source_position(EditorUtf16Offset::new(0)).unwrap().get(),
        3
    );
    assert_eq!(
        p.source_position(EditorUtf16Offset::new(2)).unwrap().get(),
        7
    );
    assert_eq!(
        p.source_position(EditorUtf16Offset::new(3)).unwrap().get(),
        9
    );
    assert_eq!(
        p.source_position(EditorUtf16Offset::new(1)),
        Err(EditError::InvalidEditorOffset)
    );
    assert_eq!(
        p.source_position(EditorUtf16Offset::new(usize::MAX)),
        Err(EditError::InvalidEditorOffset)
    );
    assert_eq!(
        p.editor_position(point(source, 0)),
        Err(EditError::InvalidBoundary)
    );
    assert_eq!(
        p.editor_position(point(source, 8)),
        Err(EditError::InvalidBoundary)
    );
    assert!(SourceByteOffset::checked(source, 4).is_err());
    assert!(SourceByteOffset::checked(source, usize::MAX).is_err());
    for byte in source
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(source.len()))
    {
        if let Ok(editor) = p.editor_position(point(source, byte)) {
            assert_eq!(p.source_position(editor).unwrap().get(), byte);
        }
    }
}

#[test]
fn insertion_affinity_and_boundary_insertions_keep_the_selected_bytes() {
    let source = "abc";
    let map = ChangeMap::new(
        source,
        &[
            replacement(source, 1..1, "שלום"),
            replacement(source, 2..2, "😀"),
        ],
    )
    .unwrap();
    assert_eq!(
        map.map_position(point(source, 1), Affinity::Before)
            .unwrap()
            .unwrap()
            .get(),
        1
    );
    assert_eq!(
        map.map_position(point(source, 1), Affinity::After)
            .unwrap()
            .unwrap()
            .get(),
        9
    );
    let range = map.map_range(logical(source, 2, 1)).unwrap().unwrap();
    assert_eq!((range.anchor.get(), range.head.get()), (10, 9));
    assert_eq!(&map.after()[range.head.get()..range.anchor.get()], "b");
    let caret = map.map_range(logical(source, 1, 1)).unwrap().unwrap();
    assert_eq!((caret.anchor.get(), caret.head.get()), (9, 9));
    assert!(map.map_range(logical(source, 0, 3)).unwrap().is_none());
}

#[test]
fn replacement_interiors_and_edited_ranges_invalidate_instead_of_guessing() {
    let source = "prefix target suffix";
    let map = ChangeMap::new(source, &[replacement(source, 7..13, "new")]).unwrap();
    assert!(map
        .map_position(point(source, 9), Affinity::After)
        .unwrap()
        .is_none());
    assert!(map.map_range(logical(source, 7, 13)).unwrap().is_none());
    assert_eq!(
        map.map_position(point(source, 7), Affinity::After)
            .unwrap()
            .unwrap()
            .get(),
        7
    );
    assert_eq!(
        map.map_position(point(source, 13), Affinity::Before)
            .unwrap()
            .unwrap()
            .get(),
        10
    );
    let before = logical(source, 14, source.len());
    let after = map.map_range(before).unwrap().unwrap();
    assert_eq!(&map.after()[after.anchor.get()..after.head.get()], "suffix");
    assert_eq!(map.inverse().map_range(after).unwrap(), Some(before));
}

#[test]
fn adjacent_deletion_inverse_affinity_crosses_the_whole_restored_interval() {
    let source = "abcdef";
    let map = ChangeMap::new(
        source,
        &[replacement(source, 1..2, ""), replacement(source, 2..3, "")],
    )
    .unwrap();
    assert_eq!(map.after(), "adef");
    let inverse = map.inverse();
    let edge = point(map.after(), 1);
    assert_eq!(
        inverse
            .map_position(edge, Affinity::Before)
            .unwrap()
            .unwrap()
            .get(),
        1
    );
    assert_eq!(
        inverse
            .map_position(edge, Affinity::After)
            .unwrap()
            .unwrap()
            .get(),
        3
    );
    let selected = logical(source, 6, 3);
    let mapped = map.map_range(selected).unwrap().unwrap();
    assert_eq!(inverse.map_range(mapped).unwrap(), Some(selected));
}

#[test]
fn no_op_replacements_do_not_destroy_range_identity() {
    let source = "same same";
    let map = ChangeMap::new(
        source,
        &[
            replacement(source, 5..9, "same"),
            replacement(source, 0..0, "X "),
        ],
    )
    .unwrap();
    assert_eq!(
        map.map_range(logical(source, 5, 9)).unwrap().unwrap(),
        logical(map.after(), 7, 11)
    );
}

#[test]
fn change_map_construction_validates_expected_bytes_overlaps_and_boundaries() {
    let source = "😀ab";
    assert!(matches!(
        ChangeMap::new(
            source,
            &[Replacement {
                range: 0..1,
                expected: "?".into(),
                text: "".into()
            }]
        ),
        Err(EditError::InvalidBoundary)
    ));
    assert!(matches!(
        ChangeMap::new(
            source,
            &[Replacement {
                range: 4..5,
                expected: "z".into(),
                text: "".into()
            }]
        ),
        Err(EditError::ExpectedTextMismatch)
    ));
    assert!(matches!(
        ChangeMap::new(
            source,
            &[
                replacement(source, 4..5, ""),
                replacement(source, 4..4, "x")
            ]
        ),
        Err(EditError::OverlappingEdits)
    ));
}

#[test]
fn unrelated_file_change_preserves_selection_and_selected_file_removal_clears_it() {
    let mut s = Session::new(
        BTreeMap::from([
            ("main.tex".into(), "main".into()),
            ("part.tex".into(), "same same".into()),
        ]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap();
    let selection = Selection {
        file: "part.tex".into(),
        anchor: 5,
        head: 9,
        secondary: vec![],
    };
    s.set_selection(selection.clone()).unwrap();
    apply(&mut s, vec![replacement("main", 0..0, "prefix ")], true);
    assert_eq!(s.selection(), Some(&selection));
    let p = s
        .prepare(Patch {
            base: s.revision(),
            files: vec![FileChange::Remove {
                file: "part.tex".into(),
                expected: "same same".into(),
            }],
            profile: None,
        })
        .unwrap();
    s.commit(p, Origin::Source, None).unwrap();
    assert!(s.selection().is_none());
    s.undo().unwrap();
    assert_eq!(s.selection(), Some(&selection));
}

#[test]
fn mapped_selection_checkpoint_restores_exact_history_and_continues_mapping() {
    let mut s = state("מَرْحَبًا same same");
    let end = s.sources()["main.tex"].len();
    s.set_selection(selected(end, end - 4)).unwrap();
    apply(&mut s, vec![replacement("", 0..0, "😀\r\n")], true);
    let expected = s.selection().cloned().unwrap();
    let mut recovered = Session::from_checkpoint(s.checkpoint()).unwrap();
    assert_eq!(recovered.selection(), Some(&expected));
    recovered.undo().unwrap();
    assert_eq!(recovered.selection(), Some(&selected(end, end - 4)));
    recovered.redo().unwrap();
    assert_eq!(recovered.selection(), Some(&expected));
    apply(&mut recovered, vec![replacement("😀\r\n", 0..6, "")], true);
    assert_eq!(recovered.selection(), Some(&selected(end, end - 4)));
}

#[test]
fn adjacent_replacement_end_still_observes_following_insertion_affinity() {
    let source = "abcd";
    let map = ChangeMap::new(
        source,
        &[
            replacement(source, 0..1, "XX"),
            replacement(source, 1..1, "Z"),
        ],
    )
    .unwrap();
    assert_eq!(map.after(), "XXZbcd");
    assert_eq!(
        map.map_position(point(source, 1), Affinity::Before)
            .unwrap()
            .unwrap()
            .get(),
        2
    );
    assert_eq!(
        map.map_position(point(source, 1), Affinity::After)
            .unwrap()
            .unwrap()
            .get(),
        3
    );
    assert_eq!(
        map.map_range(logical(source, 1, 4)).unwrap().unwrap(),
        logical(map.after(), 3, 6)
    );
}

#[test]
fn range_maps_match_an_independent_tagged_character_edit_oracle() {
    // Splice tagged characters instead of computing coordinate deltas. Tags
    // distinguish equal characters and inserted text from original identity.
    let source = "\u{feff}abאבمَرْ😀\r\nxyz";
    let boundaries = source
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(source.len()))
        .collect::<Vec<_>>();
    let cases = [
        vec![(0, 0, "P"), (4, 4, "😀")],
        vec![(1, 3, "Q"), (6, 8, "")],
        vec![(1, 2, ""), (2, 3, ""), (3, 3, "XY")],
        vec![(1, 2, "b"), (5, 6, "א")],
    ];
    for case in cases {
        let edits = case
            .iter()
            .map(|&(a, b, text)| replacement(source, boundaries[a]..boundaries[b], text))
            .collect::<Vec<_>>();
        let map = ChangeMap::new(source, &edits).unwrap();
        let mut tags = source
            .chars()
            .enumerate()
            .map(|(i, c)| (Some(i), c))
            .collect::<Vec<_>>();
        for &(a, b, text) in case.iter().rev() {
            if source[boundaries[a]..boundaries[b]] != *text {
                tags.splice(a..b, text.chars().map(|c| (None, c)));
            }
        }
        assert_eq!(
            map.after(),
            tags.iter().map(|(_, c)| *c).collect::<String>()
        );
        for a in 0..boundaries.len() - 1 {
            for b in a + 1..boundaries.len() {
                let expected = tags
                    .iter()
                    .position(|(tag, _)| *tag == Some(a))
                    .and_then(|start| {
                        let length = b - a;
                        let intact = tags
                            .get(start..start + length)?
                            .iter()
                            .enumerate()
                            .all(|(i, (tag, _))| *tag == Some(a + i));
                        if !intact {
                            return None;
                        }
                        let lo = tags[..start].iter().map(|(_, c)| c.len_utf8()).sum();
                        let hi = tags[..start + length]
                            .iter()
                            .map(|(_, c)| c.len_utf8())
                            .sum();
                        Some(logical(map.after(), lo, hi))
                    });
                assert_eq!(
                    map.map_range(logical(source, boundaries[a], boundaries[b]))
                        .unwrap(),
                    expected
                );
                let reversed = expected.map(|range| LogicalRange {
                    anchor: range.head,
                    head: range.anchor,
                });
                assert_eq!(
                    map.map_range(logical(source, boundaries[b], boundaries[a]))
                        .unwrap(),
                    reversed
                );
                if let Some(mapped) = expected {
                    assert_eq!(
                        map.inverse().map_range(mapped).unwrap(),
                        Some(logical(source, boundaries[a], boundaries[b]))
                    );
                }
            }
        }
    }
}
