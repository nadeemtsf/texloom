use std::collections::BTreeMap;
use texloom_core::document::{intent::*, structure::*, *};

fn document(body: &str) -> String {
    format!("\u{feff}\\documentclass{{article}}\r\n\\begin{{document}}\r\n{body}\r\n\\end{{document}}\r\n")
}
fn state(body: &str) -> Session {
    Session::new(
        BTreeMap::from([
            ("main.tex".into(), document(body)),
            ("keep.sty".into(), "% unrelated\r\n".into()),
        ]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .unwrap()
}
fn target(s: &Session, occurrence: usize) -> EntityReference {
    let p = s.source_projection("main.tex").unwrap();
    let index = p
        .regions()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind() == RegionKind::PlainParagraph)
        .nth(occurrence)
        .unwrap()
        .0;
    p.entity_reference(index).unwrap()
}

#[test]
fn literal_replacement_escapes_raw_tex_and_keeps_all_unrelated_source() {
    let s = state("Before.\r\n\r\nTarget.\r\n\r\nAfter.");
    let original = s.sources().clone();
    let input = "\\input{evil} & % # _ $ ~ ^ שלום 😀";
    let plan = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 1),
            text: input.into(),
        })
        .unwrap();
    let escaped = "\\textbackslash{}input\\{evil\\} \\& \\% \\# \\_ \\$ \\textasciitilde{} \\textasciicircum{} שלום 😀";
    assert_eq!(
        plan.candidate_sources()["main.tex"],
        original["main.tex"].replace("Target.", escaped)
    );
    assert_eq!(plan.candidate_sources()["keep.sty"], original["keep.sty"]);
    assert_eq!(plan.kind(), OperationKind::ReplaceText);
    assert_eq!(
        plan.required_proof(),
        &RequiredProof::LiteralParagraphReplacement {
            requested_text: input.into()
        }
    );
    assert_eq!(
        plan.layout_effect(),
        LayoutEffect::DocumentReflowRequiringVerification
    );
    assert_eq!(s.sources(), &original);
    assert_eq!(s.history_len(), 0);
    assert_eq!(s.revision(), Revision(0));
}

#[test]
fn deletion_targets_only_the_second_equal_paragraph_and_retains_separators() {
    let s = state("Same.\r\n\r\nSame.\r\n\r\nTail.");
    let original = s.sources().clone();
    let offset = original["main.tex"].rfind("Same.").unwrap();
    let plan = s
        .plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&s, 1),
        })
        .unwrap();
    let mut expected = original["main.tex"].clone();
    expected.replace_range(offset..offset + 5, "");
    assert_eq!(plan.candidate_sources()["main.tex"], expected);
    assert_eq!(plan.candidate_sources().len(), original.len());
    assert_eq!(plan.candidate_sources()["keep.sty"], original["keep.sty"]);
    assert_eq!(plan.kind(), OperationKind::DeleteContent);
    assert_eq!(
        plan.required_proof(),
        &RequiredProof::ParagraphContentDeletion
    );
    let patch = plan.source_patch();
    assert!(patch.profile.is_none());
    let [FileChange::Edit { file, edits }] = patch.files.as_slice() else {
        panic!("not a one-file edit");
    };
    assert_eq!(file, "main.tex");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].range, offset..offset + 5);
    assert_eq!(edits[0].expected, "Same.");
    assert_eq!(edits[0].text, "");
    assert_eq!(s.sources(), &original);
    assert!(!s.can_undo());
}

#[test]
fn foreign_and_stale_targets_refuse_before_candidate_preparation() {
    let mut s = state("Same.");
    let other = state("Same.");
    assert!(matches!(
        s.plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&other, 0)
        }),
        Err(PlanError::Source(EditError::ForeignSession))
    ));
    let stale = target(&s, 0);
    let offset = s.sources()["main.tex"].find("Same.").unwrap();
    let change = s
        .prepare_source_edit(
            s.revision(),
            "main.tex",
            vec![Replacement {
                range: offset..offset + 5,
                expected: "Same.".into(),
                text: "Other.".into(),
            }],
        )
        .unwrap();
    s.commit_source_edit(change, None).unwrap();
    s.undo().unwrap();
    assert!(matches!(
        s.plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: stale,
            text: "New.".into()
        }),
        Err(PlanError::Source(EditError::StaleRevision))
    ));
}

#[test]
fn empty_control_and_structural_text_require_a_different_intent() {
    let s = state("Target.");
    for (input, expected) in [
        ("", PlanError::EmptyLiteral),
        ("  ", PlanError::EmptyLiteral),
        ("a\nb", PlanError::ParagraphBreakRequiresStructuralIntent),
        ("a\rb", PlanError::ParagraphBreakRequiresStructuralIntent),
        (
            "a\u{2029}b",
            PlanError::ParagraphBreakRequiresStructuralIntent,
        ),
        ("a\0b", PlanError::ControlCharacter),
        ("a\tb", PlanError::ControlCharacter),
    ] {
        let result = s.plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: input.into(),
        });
        assert!(matches!(result, Err(error) if error == expected));
    }
    assert!(matches!(
        s.plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "x".repeat(256_001)
        }),
        Err(PlanError::LiteralTooLarge)
    ));
    assert_eq!(s.revision(), Revision(0));
    assert_eq!(s.history_len(), 0);
}

#[test]
fn same_literal_candidate_is_a_no_op_and_does_not_grant_render_authority() {
    let s = state("Target.");
    let plan = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Target.".into(),
        })
        .unwrap();
    assert!(!plan.changed());
    assert_eq!(plan.base_revision(), Revision(0));
    assert_eq!(plan.candidate_sources(), s.sources());
    assert_eq!(
        plan.required_proof(),
        &RequiredProof::LiteralParagraphReplacement {
            requested_text: "Target.".into()
        }
    );
}

#[test]
fn draft_lane_can_reuse_the_exact_patch_and_history_without_claiming_visual_acceptance() {
    let mut s = state("مَرْحَبًا שלום 😀");
    let original = s.sources().clone();
    let plan = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "New & literal".into(),
        })
        .unwrap();
    let candidate = plan.candidate_sources().clone();
    // Explicit canonical source editing, not a verified geometric operation.
    let prepared = s.prepare(plan.source_patch()).unwrap();
    s.commit(prepared, Origin::Source, None).unwrap();
    assert_eq!(s.sources(), &candidate);
    assert_eq!(s.history_len(), 1);
    s.undo().unwrap();
    assert_eq!(s.sources(), &original);
    s.redo().unwrap();
    assert_eq!(s.sources(), &candidate);
}

#[test]
fn unsupported_formatted_or_dynamic_source_cannot_mint_a_literal_target() {
    for source in [
        "\\textbf{Formatted}",
        "\\Unknown{Dynamic}",
        "\\begin{quote}Nested\\end{quote}",
    ] {
        let s = state(source);
        let projection = s.source_projection("main.tex").unwrap();
        assert!((0..projection.regions().len()).all(|i| projection.entity_reference(i).is_none()));
    }
}

#[test]
fn escaped_literal_candidates_can_be_projected_and_replaced_again() {
    let mut s = state("Before.\r\n\r\nTarget.\r\n\r\nAfter.");
    let original = s.sources()["main.tex"].clone();
    let first = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 1),
            text: "\\literal {x} ~ ^ & # _ $ %".into(),
        })
        .unwrap();
    let draft = s.prepare(first.source_patch()).unwrap();
    s.commit(draft, Origin::Source, None).unwrap();
    let projection = s.source_projection("main.tex").unwrap();
    assert_eq!(
        projection
            .regions()
            .iter()
            .filter(|r| r.kind() == RegionKind::PlainParagraph)
            .count(),
        3
    );
    let next = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 1),
            text: "Second update.".into(),
        })
        .unwrap();
    assert_eq!(
        next.candidate_sources()["main.tex"],
        original.replace("Target.", "Second update.")
    );
}
