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
fn typed_draft_intent_uses_shared_history_and_keeps_render_proof_unresolved() {
    let mut s = state("Same.\r\n\r\nSame.\r\n\r\nTail.");
    let original = s.sources().clone();
    let offset = original["main.tex"].rfind("Same.").unwrap();
    s.set_selection(Selection {
        file: "main.tex".into(),
        anchor: offset,
        head: offset + 5,
        secondary: vec![],
    })
    .unwrap();
    let plan = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 1),
            text: "Revised & שלום.".into(),
        })
        .unwrap();
    let expected = plan.candidate_sources(&s).unwrap();
    let receipt = s.apply_source_intent_draft(plan).unwrap();
    assert_eq!(receipt.from_revision(), Revision(0));
    assert_eq!(receipt.revision(), Revision(1));
    assert!(receipt.changed());
    assert_eq!(receipt.kind(), OperationKind::ReplaceText);
    assert_eq!(
        receipt.required_proof(),
        &RequiredProof::LiteralParagraphReplacement {
            requested_text: "Revised & שלום.".into(),
        }
    );
    assert_eq!(
        receipt.layout_effect(),
        LayoutEffect::DocumentReflowRequiringVerification
    );
    assert_eq!(s.sources(), &expected);
    assert_eq!(s.history_len(), 1);
    assert!(
        s.selection().is_none(),
        "changed selection does not guess equal-text identity"
    );
    assert!(s.undo().unwrap());
    assert_eq!(s.sources(), &original);
    assert_eq!(s.selection().unwrap().anchor, offset);
    assert!(s.redo().unwrap());
    assert_eq!(s.sources(), &expected);
    assert_eq!(s.sources()["keep.sty"], original["keep.sty"]);
}

#[test]
fn draft_intents_reject_foreign_stale_and_restored_equal_source_plans() {
    let mut s = state("Same.");
    let mut foreign = state("Same.");
    let plan = s
        .plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&s, 0),
        })
        .unwrap();
    assert!(matches!(
        plan.candidate_sources(&foreign),
        Err(EditError::ForeignSession)
    ));
    assert!(matches!(
        foreign.apply_source_intent_draft(plan),
        Err(EditError::ForeignSession)
    ));
    assert!(!foreign.can_undo());
    let stale = s
        .plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&s, 0),
        })
        .unwrap();
    let newer = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Other.".into(),
        })
        .unwrap();
    s.apply_source_intent_draft(newer).unwrap();
    s.undo().unwrap();
    let before = s.sources().clone();
    let revision = s.revision();
    let history = s.history_len();
    assert!(matches!(
        stale.candidate_sources(&s),
        Err(EditError::StaleRevision)
    ));
    assert!(matches!(
        s.apply_source_intent_draft(stale),
        Err(EditError::StaleRevision)
    ));
    assert_eq!(s.sources(), &before);
    assert_eq!(s.revision(), revision);
    assert_eq!(s.history_len(), history);
    let external = state("Same.");
    let old_plan = external
        .plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&external, 0),
        })
        .unwrap();
    let mut restored = Session::from_checkpoint(external.checkpoint()).unwrap();
    assert!(matches!(
        restored.apply_source_intent_draft(old_plan),
        Err(EditError::ForeignSession)
    ));
}

#[test]
fn draft_operations_are_distinct_undo_steps_and_recover_the_redo_branch() {
    let mut s = state("Same.\r\n\r\nSame.\r\n\r\nTail.");
    let original = s.sources().clone();
    let replace = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Changed.".into(),
        })
        .unwrap();
    s.apply_source_intent_draft(replace).unwrap();
    let replaced = s.sources().clone();
    let delete = s
        .plan_source_intent(SourceIntent::DeleteParagraphContent {
            target: target(&s, 1),
        })
        .unwrap();
    s.apply_source_intent_draft(delete).unwrap();
    let deleted = s.sources().clone();
    assert_eq!(s.history_len(), 2);
    s.undo().unwrap();
    assert_eq!(s.sources(), &replaced);
    let mut restored = Session::from_checkpoint(s.checkpoint()).unwrap();
    assert!(restored.can_redo());
    restored.redo().unwrap();
    assert_eq!(restored.sources(), &deleted);
    restored.undo().unwrap();
    restored.undo().unwrap();
    assert_eq!(restored.sources(), &original);
    assert_eq!(restored.profile().engine, "auto");
}

#[test]
fn no_op_draft_preserves_revision_history_redo_and_selection_but_checks_owner() {
    let mut s = state("Same.");
    let other = state("Same.");
    let foreign_noop = other
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&other, 0),
            text: "Same.".into(),
        })
        .unwrap();
    assert!(matches!(
        s.apply_source_intent_draft(foreign_noop),
        Err(EditError::ForeignSession)
    ));
    let offset = s.sources()["main.tex"].find("Same.").unwrap();
    s.set_selection(Selection {
        file: "main.tex".into(),
        anchor: offset,
        head: offset,
        secondary: vec![],
    })
    .unwrap();
    let selection = s.selection().cloned();
    let plan = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Same.".into(),
        })
        .unwrap();
    let receipt = s.apply_source_intent_draft(plan).unwrap();
    assert!(!receipt.changed());
    assert_eq!(receipt.revision(), Revision(0));
    assert_eq!(s.history_len(), 0);
    assert_eq!(s.selection(), selection.as_ref());
    assert_eq!(
        receipt.required_proof(),
        &RequiredProof::LiteralParagraphReplacement {
            requested_text: "Same.".into()
        }
    );
    let changed = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Other.".into(),
        })
        .unwrap();
    s.apply_source_intent_draft(changed).unwrap();
    s.undo().unwrap();
    let revision = s.revision();
    let history = s.history_len();
    let noop = s
        .plan_source_intent(SourceIntent::ReplaceLiteralParagraph {
            target: target(&s, 0),
            text: "Same.".into(),
        })
        .unwrap();
    s.apply_source_intent_draft(noop).unwrap();
    assert_eq!(s.revision(), revision);
    assert_eq!(s.history_len(), history);
    assert!(s.can_redo());
    s.redo().unwrap();
    assert!(s.sources()["main.tex"].contains("Other."));
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
    let candidate = plan.candidate_sources(&s).unwrap();
    assert_eq!(
        candidate["main.tex"],
        original["main.tex"].replace("Target.", escaped)
    );
    assert_eq!(candidate["keep.sty"], original["keep.sty"]);
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
    let candidate = plan.candidate_sources(&s).unwrap();
    assert_eq!(candidate["main.tex"], expected);
    assert_eq!(candidate.len(), original.len());
    assert_eq!(candidate["keep.sty"], original["keep.sty"]);
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
    assert_eq!(&plan.candidate_sources(&s).unwrap(), s.sources());
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
    let candidate = plan.candidate_sources(&s).unwrap();
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
        next.candidate_sources(&s).unwrap()["main.tex"],
        original.replace("Target.", "Second update.")
    );
}
