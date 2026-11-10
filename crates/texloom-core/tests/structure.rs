use std::collections::BTreeMap;
use texloom_core::document::{structure::*, *};

fn document(body: &str) -> String {
    format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n")
}
fn session(source: &str) -> Session {
    Session::new(
        BTreeMap::from([("main.tex".into(), source.into())]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap()
}
fn candidates(p: &SourceProjection<'_>) -> Vec<usize> {
    p.regions()
        .iter()
        .enumerate()
        .filter_map(|(i, r)| (r.kind() == RegionKind::PlainParagraph).then_some(i))
        .collect()
}
fn exact_partition(p: &SourceProjection<'_>, expected: &str) {
    let mut cursor = 0;
    let mut copied = String::new();
    for (i, r) in p.regions().iter().enumerate() {
        let range = r.range();
        assert_eq!(range.start, cursor);
        assert!(range.end > range.start);
        assert!(expected.is_char_boundary(range.start) && expected.is_char_boundary(range.end));
        copied.push_str(p.text(i).unwrap());
        cursor = range.end;
    }
    assert_eq!(cursor, expected.len());
    assert_eq!(copied, expected);
    assert_eq!(p.source(), expected);
}

#[test]
fn exact_multilingual_bom_crlf_partition_does_not_change_history() {
    let source="\u{feff}% preamble stays\r\n\\documentclass{article}\r\n\\begin{document}\r\nHello 😀\r\n\r\nمَرْحَبًا שלום\r\n\\end{document}\r\n";
    let s = session(source);
    let p = s.source_projection("main.tex").unwrap();
    exact_partition(&p, source);
    assert_eq!(candidates(&p).len(), 2);
    assert_eq!(s.history_len(), 0);
    assert!(!s.can_undo());
    assert_eq!(s.sources()["main.tex"], source);
}

#[test]
fn opaque_markup_does_not_become_an_editable_paragraph() {
    let source = document("Before.\n\n\\textbf{Keep {nested} markup}.\n\nAfter.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    exact_partition(&p, &source);
    assert_eq!(
        candidates(&p)
            .iter()
            .map(|&i| p.text(i).unwrap())
            .collect::<Vec<_>>(),
        vec!["Before.", "After."]
    );
}

#[test]
fn environment_and_literal_contents_are_never_mistaken_for_top_level_prose() {
    let source=document("Before.\n\n\\begin{quote}\nInner.\n\nStill inner.\n\\end{quote}\n\n\\begin{verbatim}\n\\begin{document}\nFake body.\n\\end{document}\n\\end{verbatim}\n\nAfter.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    exact_partition(&p, &source);
    assert_eq!(
        candidates(&p)
            .iter()
            .map(|&i| p.text(i).unwrap())
            .collect::<Vec<_>>(),
        vec!["Before.", "After."]
    );
}

#[test]
fn incomplete_and_mismatched_source_stays_available_as_opaque_bytes() {
    for source in [
        "",
        "Partial text",
        "\\begin{document}Partial",
        "\\begin{document}{Bad\\end{document}",
        "\\begin{document}\\begin{quote}Text\\end{center}\\end{document}",
        "\\verb|\\begin{document}",
    ] {
        let s = session(source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, source);
        assert!(candidates(&p).is_empty());
    }
}

#[test]
fn definitions_and_conditionals_decline_projection_instead_of_interpreting_tex() {
    for control in [
        "\\newcommand{\\Name}{Name}",
        "\\def\\Name{Name}",
        "\\catcode`x=0",
        "\\iftrue",
        "\\ExplSyntaxOn",
    ] {
        let source = document(&format!("{control}\n\nOrdinary looking text."));
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        assert!(candidates(&p).is_empty());
        assert_eq!(
            p.regions()[0].kind(),
            RegionKind::Opaque(OpaqueReason::DynamicSyntax)
        );
    }
}

#[test]
fn duplicate_document_envelopes_do_not_choose_the_last_occurrence() {
    let source = document("First.") + &document("Second.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    exact_partition(&p, &source);
    assert!(candidates(&p).is_empty());
}

#[test]
fn comments_cannot_create_an_envelope_and_escaped_symbols_stay_literal() {
    let source = "% \\begin{document}Fake\\end{document}\n".to_owned()
        + &document("A \\% \\{brace\\} \\& B.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    exact_partition(&p, &source);
    let ids = candidates(&p);
    assert_eq!(ids.len(), 1);
    assert_eq!(p.text(ids[0]), Some("A \\% \\{brace\\} \\& B."));
}

#[test]
fn repeated_text_references_retain_the_exact_selected_source_range() {
    let source = document("Same.\n\nSame.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    let ids = candidates(&p);
    assert_eq!(ids.len(), 2);
    let first = p.entity_reference(ids[0]).unwrap();
    let second = p.entity_reference(ids[1]).unwrap();
    let a = s.resolve_entity_reference(&first).unwrap();
    let b = s.resolve_entity_reference(&second).unwrap();
    assert_eq!(a.text(), b.text());
    assert!(a.range().end < b.range().start);
    assert_eq!(b.file(), "main.tex");
}

#[test]
fn changing_source_and_undoing_does_not_revive_an_old_reference() {
    let source = document("Target.");
    let mut s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    let reference = p.entity_reference(candidates(&p)[0]).unwrap();
    drop(p);
    let edit = s
        .prepare_source_edit(
            s.revision(),
            "main.tex",
            vec![Replacement {
                range: 0..0,
                expected: String::new(),
                text: "% new\n".into(),
            }],
        )
        .unwrap();
    s.commit_source_edit(edit, None).unwrap();
    assert!(matches!(
        s.resolve_entity_reference(&reference),
        Err(EditError::StaleRevision)
    ));
    s.undo().unwrap();
    assert_eq!(s.sources()["main.tex"], source);
    assert!(matches!(
        s.resolve_entity_reference(&reference),
        Err(EditError::StaleRevision)
    ));
}

#[test]
fn identical_other_session_cannot_use_the_reference() {
    let source = document("Target.");
    let a = session(&source);
    let b = session(&source);
    let p = a.source_projection("main.tex").unwrap();
    let reference = p.entity_reference(candidates(&p)[0]).unwrap();
    assert!(matches!(
        b.resolve_entity_reference(&reference),
        Err(EditError::ForeignSession)
    ));
}

#[test]
fn opaque_regions_and_invalid_indices_cannot_mint_entity_references() {
    let source = document("\\Unknown{target}");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    assert!(p.entity_reference(usize::MAX).is_none());
    assert!(p.text(usize::MAX).is_none());
    for i in 0..p.regions().len() {
        assert!(p.entity_reference(i).is_none());
    }
    assert!(matches!(
        s.source_projection("missing.tex"),
        Err(EditError::MissingFile(_))
    ));
}

#[test]
fn readiness_is_not_edit_permission_or_a_guessed_spacing_or_asset_operation() {
    let source = document("Plain.");
    let s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    let region = &p.regions()[candidates(&p)[0]];
    assert_eq!(
        region.readiness(OperationKind::DeleteContent),
        Readiness::CandidateRequiringPlanAndProof
    );
    assert_eq!(
        region.readiness(OperationKind::AdjustSpacing),
        Readiness::Unavailable(UnavailableReason::NoLocalParameter)
    );
    assert_eq!(
        region.readiness(OperationKind::RemovePositioning),
        Readiness::Unavailable(UnavailableReason::NotPositioned)
    );
    assert_eq!(
        region.readiness(OperationKind::DeleteAsset),
        Readiness::Unavailable(UnavailableReason::AssetLifecycleRequired)
    );
    assert_eq!(s.history_len(), 0);
}

#[test]
fn no_op_draft_edit_retains_a_current_reference() {
    let source = document("Target.");
    let mut s = session(&source);
    let p = s.source_projection("main.tex").unwrap();
    let reference = p.entity_reference(candidates(&p)[0]).unwrap();
    drop(p);
    let edit = s
        .prepare_source_edit(s.revision(), "main.tex", vec![])
        .unwrap();
    s.commit_source_edit(edit, None).unwrap();
    assert_eq!(
        s.resolve_entity_reference(&reference).unwrap().text(),
        "Target."
    );
    assert_eq!(s.revision(), Revision(0));
}

#[test]
fn work_and_region_limits_preserve_source_without_partial_candidates() {
    for body in [
        "x".repeat(200_001),
        "p\n\n".repeat(6_000),
        "p\n\n".repeat(4_999),
    ] {
        let source = document(&body);
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        assert!(candidates(&p).is_empty());
        assert_eq!(
            p.regions()[0].kind(),
            RegionKind::Opaque(OpaqueReason::BudgetExceeded)
        );
    }
}

#[test]
fn source_fragments_wait_for_included_file_context() {
    let s = Session::new(
        BTreeMap::from([
            ("main.tex".into(), document("Main.")),
            ("part.tex".into(), "Included prose.".into()),
        ]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap();
    let p = s.source_projection("part.tex").unwrap();
    exact_partition(&p, "Included prose.");
    assert!(candidates(&p).is_empty());
}

#[test]
fn multiline_math_cannot_expose_prose_candidates_between_delimiters() {
    for body in [
        "$\n\nlooks like prose\n\n$",
        "\\[\n\nlooks like prose\n\n\\]",
    ] {
        let source = document(body);
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        assert!(candidates(&p).is_empty());
    }
}

#[test]
fn unknown_control_flow_cannot_expose_apparently_plain_following_text() {
    for control in [
        "\\begingroup",
        "\\input{unknown}",
        "\\unless\\iftrue",
        "\\Unknown",
        "^^5cdef^^5cName{Name}",
        "\\begin{\\Choose}Unknown context\\end{\\Choose}",
    ] {
        let source = document(&format!("{control}\n\nLooks plain."));
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        assert!(candidates(&p).is_empty());
    }
}

#[test]
fn mixed_unicode_and_structural_fragments_preserve_every_source_boundary() {
    // Deterministic development corpus, not a held-out accuracy sample or a
    // rendering oracle. Exercise interactions beyond the named syntax cases.
    let fragments = [
        "Plain",
        "مَرْحَبًا",
        "שָׁלוֹם",
        "😀",
        "\u{200d}",
        "\u{feff}",
        " ",
        "\t",
        "\r\n",
        "\n\n",
        "{",
        "}",
        "% comment\r\n",
        "\\%",
        "\\{",
        "\\}",
        "\\textbf{",
        "\\Unknown",
        "\\",
        "\\begin{quote}",
        "\\end{quote}",
        "\\begin{document}",
        "\\end{document}",
        "\\verb|",
        "|",
        "\\verb*😀",
        "$",
        "^^5c",
    ];
    let mut state = 0x7465786c6f6f6d_u64;
    for case in 0..512 {
        let mut body = String::new();
        for _ in 0..1 + case % 97 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            body.push_str(fragments[(state >> 32) as usize % fragments.len()]);
        }
        let source = if case % 2 == 0 { document(&body) } else { body };
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        for index in candidates(&p) {
            let reference = p.entity_reference(index).unwrap();
            let resolved = s.resolve_entity_reference(&reference).unwrap();
            assert_eq!(resolved.range(), p.regions()[index].range());
            assert_eq!(resolved.text(), p.text(index).unwrap());
        }
        assert_eq!(s.sources()["main.tex"], source);
        assert_eq!(s.history_len(), 0);
        assert_eq!(s.revision(), Revision(0));
    }
}

#[test]
fn known_literal_command_spellings_require_exact_empty_terminators() {
    for name in ["\\textbackslash", "\\textasciitilde", "\\textasciicircum"] {
        let source = document(&format!("Before.\n\n{name}{{}} literal.\n\nAfter."));
        let s = session(&source);
        let p = s.source_projection("main.tex").unwrap();
        exact_partition(&p, &source);
        assert_eq!(candidates(&p).len(), 3);
        for suffix in ["{not empty}", "{", "", " {}"] {
            let source = document(&format!("{name}{suffix}\n\nLooks plain."));
            let s = session(&source);
            let p = s.source_projection("main.tex").unwrap();
            exact_partition(&p, &source);
            assert!(candidates(&p).is_empty());
        }
    }
}
