//! Typed candidate planning and explicit source drafts. No UI, compilation or visual commit.
use super::{
    structure::{EntityReference, OperationKind, RegionKind},
    *,
};

const MAX_LITERAL_BYTES: usize = 256_000;

pub enum SourceIntent {
    ReplaceLiteralParagraph {
        target: EntityReference,
        text: String,
    },
    DeleteParagraphContent {
        target: EntityReference,
    },
    MoveAdjacentParagraph {
        target: EntityReference,
        neighbor: EntityReference,
        position: ParagraphPosition,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParagraphPosition {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    Source(EditError),
    UnsupportedTarget,
    EmptyLiteral,
    ParagraphBreakRequiresStructuralIntent,
    ControlCharacter,
    LiteralTooLarge,
    ExpectedAdjacentParagraphs,
    ExpectedContinuousParagraphs,
}
impl From<EditError> for PlanError {
    fn from(value: EditError) -> Self {
        Self::Source(value)
    }
}

/// Obligations to be discharged later. Constructing this is not a proof pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiredProof {
    LiteralParagraphReplacement { requested_text: String },
    ParagraphContentDeletion,
    ParagraphReorder {
        original_target: Range<usize>,
        original_neighbor: Range<usize>,
        position: ParagraphPosition,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutEffect {
    DocumentReflowRequiringVerification,
}

/// Private construction guarantees a rederived edit of one source file. The plan
/// does not expose its Prepared object or a geometry-acceptance method.
pub struct IntentPlan {
    kind: OperationKind,
    prepared: PreparedSourceEdit,
    required_proof: RequiredProof,
}

/// Exact source ranges in visual order before and after a proposed move.
pub struct ParagraphOrderPlan {
    pub intent: IntentPlan,
    pub original: Vec<Range<usize>>,
    pub reordered: Vec<(usize, Range<usize>)>,
}

/// A source draft was applied through shared history. This does not attest to
/// compilation, rendered identity, layout preservation or visual acceptance.
/// RequiredProof stays unresolved and belongs to this exact source revision.
pub struct DraftIntentReceipt {
    kind: OperationKind,
    from_revision: Revision,
    revision: Revision,
    changed: bool,
    required_proof: RequiredProof,
}
impl DraftIntentReceipt {
    pub fn kind(&self) -> OperationKind {
        self.kind
    }
    pub fn from_revision(&self) -> Revision {
        self.from_revision
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn required_proof(&self) -> &RequiredProof {
        &self.required_proof
    }
    pub fn layout_effect(&self) -> LayoutEffect {
        LayoutEffect::DocumentReflowRequiringVerification
    }
}
impl IntentPlan {
    pub fn kind(&self) -> OperationKind {
        self.kind
    }
    pub fn base_revision(&self) -> Revision {
        self.prepared.base()
    }
    pub fn changed(&self) -> bool {
        self.prepared.changed()
    }
    /// Materialize a complete candidate only when a caller needs all files.
    pub fn candidate_sources(&self, session: &Session) -> EditResult<Sources> {
        self.prepared.validate_for(session)?;
        let mut sources = session.sources().clone();
        sources.insert(self.prepared.file().into(), self.prepared.text().into());
        Ok(sources)
    }
    pub fn source_patch(&self) -> Patch {
        self.prepared.recovery_patch()
    }
    pub fn required_proof(&self) -> &RequiredProof {
        &self.required_proof
    }
    pub fn layout_effect(&self) -> LayoutEffect {
        LayoutEffect::DocumentReflowRequiringVerification
    }
}

impl Session {
    /// Explicit source-draft lane, analogous to typing in the source editor.
    /// Callers must invalidate rendered editing capabilities, retain the last
    /// good PDF, and journal the plan's source_patch before applying it. A
    /// verified visual transaction must use its separate native approval lane.
    /// This consumes a privately constructed plan, never a raw replacement.
    pub fn apply_source_intent_draft(
        &mut self,
        plan: IntentPlan,
    ) -> EditResult<DraftIntentReceipt> {
        let IntentPlan {
            kind,
            prepared,
            required_proof,
        } = plan;
        let from_revision = prepared.base();
        let changed = prepared.changed();
        // commit rechecks private session ownership and base revision before
        // any mutation; no-op plans must pass the same provenance guards.
        let revision = self.commit_source_edit(prepared, None)?;
        Ok(DraftIntentReceipt {
            kind,
            from_revision,
            revision,
            changed,
            required_proof,
        })
    }

    pub fn plan_source_intent(&self, intent: SourceIntent) -> Result<IntentPlan, PlanError> {
        let (target, text) = match &intent {
            SourceIntent::ReplaceLiteralParagraph { target, text } => (target, Some(text)),
            SourceIntent::DeleteParagraphContent { target } => (target, None),
            SourceIntent::MoveAdjacentParagraph {
                target,
                neighbor,
                position,
            } => {
                return self.plan_adjacent_paragraph_move(target, neighbor, *position);
            }
        };
        // Resolve ownership/revision first. No nearest equal-text lookup or
        // frontend-supplied raw range can authorize another occurrence.
        let resolved = self.resolve_entity_reference(target)?;
        let projection = self.source_projection(resolved.file())?;
        if !projection.regions().iter().any(|region| {
            region.kind() == RegionKind::PlainParagraph && region.range() == resolved.range()
        }) {
            return Err(PlanError::UnsupportedTarget);
        }
        let (kind, replacement, required_proof) = if let Some(text) = text {
            if text.len() > MAX_LITERAL_BYTES {
                return Err(PlanError::LiteralTooLarge);
            }
            if text
                .chars()
                .any(|c| matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}'))
            {
                return Err(PlanError::ParagraphBreakRequiresStructuralIntent);
            }
            if text.chars().any(char::is_control) {
                return Err(PlanError::ControlCharacter);
            }
            if text.trim().is_empty() {
                return Err(PlanError::EmptyLiteral);
            }
            (
                OperationKind::ReplaceText,
                crate::escape_text(text),
                RequiredProof::LiteralParagraphReplacement {
                    requested_text: text.clone(),
                },
            )
        } else {
            (
                OperationKind::DeleteContent,
                String::new(),
                RequiredProof::ParagraphContentDeletion,
            )
        };
        let prepared = self.prepare_source_edit(
            self.revision(),
            resolved.file(),
            vec![Replacement {
                range: resolved.range(),
                expected: resolved.text().into(),
                text: replacement,
            }],
        )?;
        Ok(IntentPlan {
            kind,
            prepared,
            required_proof,
        })
    }

    fn plan_adjacent_paragraph_move(
        &self,
        target: &EntityReference,
        neighbor: &EntityReference,
        position: ParagraphPosition,
    ) -> Result<IntentPlan, PlanError> {
        self.plan_paragraph_order(target, neighbor, position, 2)
            .map(|plan| plan.intent)
            .map_err(|error| match error {
                PlanError::ExpectedContinuousParagraphs => PlanError::ExpectedAdjacentParagraphs,
                other => other,
            })
    }

    pub fn plan_visual_paragraph_order(
        &self,
        target: &EntityReference,
        neighbor: &EntityReference,
        position: ParagraphPosition,
    ) -> Result<ParagraphOrderPlan, PlanError> {
        self.plan_paragraph_order(target, neighbor, position, 8)
    }

    fn plan_paragraph_order(
        &self,
        target: &EntityReference,
        neighbor: &EntityReference,
        position: ParagraphPosition,
        maximum_paragraphs: usize,
    ) -> Result<ParagraphOrderPlan, PlanError> {
        let target = self.resolve_entity_reference(target)?;
        let neighbor = self.resolve_entity_reference(neighbor)?;
        let target_range = target.range();
        let neighbor_range = neighbor.range();
        if target.file() != neighbor.file() || target_range == neighbor_range {
            return Err(PlanError::ExpectedContinuousParagraphs);
        }
        let projection = self.source_projection(target.file())?;
        let regions = projection.regions();
        let index = |range: &Range<usize>| {
            regions.iter().position(|region| region.kind() == RegionKind::PlainParagraph && region.range() == *range)
                .ok_or(PlanError::ExpectedContinuousParagraphs)
        };
        let target_index = index(&target_range)?;
        let neighbor_index = index(&neighbor_range)?;
        let first = target_index.min(neighbor_index);
        let last = target_index.max(neighbor_index);
        let run = &regions[first..=last];
        if run.len() > maximum_paragraphs * 2 - 1 || run.iter().enumerate().any(|(i, region)| {
            if i % 2 == 0 {
                region.kind() != RegionKind::PlainParagraph
            } else {
                !matches!(region.kind(), RegionKind::Trivia | RegionKind::ParagraphBreak)
            }
        }) {
            return Err(PlanError::ExpectedContinuousParagraphs);
        }
        let original: Vec<_> = run.iter().step_by(2).map(|region| region.range()).collect();
        let target_slot = (target_index - first) / 2;
        let neighbor_slot = (neighbor_index - first) / 2;
        let mut order: Vec<_> = (0..original.len()).collect();
        let moved = order.remove(target_slot);
        let destination = neighbor_slot - usize::from(target_slot < neighbor_slot)
            + usize::from(position == ParagraphPosition::After);
        order.insert(destination, moved);

        let source = projection.source();
        let mut edits = Vec::new();
        let mut reordered = Vec::with_capacity(original.len());
        let mut start = original[0].start;
        for (slot, identity) in order.into_iter().enumerate() {
            let old_range = &original[slot];
            let new_range = &original[identity];
            let previous = &source[old_range.clone()];
            let replacement = &source[new_range.clone()];
            if previous != replacement {
                edits.push(Replacement {
                    range: old_range.clone(),
                    expected: previous.into(),
                    text: replacement.into(),
                });
            }
            let end = start + replacement.len();
            reordered.push((identity, start..end));
            if let Some(next) = original.get(slot + 1) {
                start = end + next.start - old_range.end;
            }
        }
        let intent = IntentPlan {
            kind: OperationKind::ReorderContent,
            prepared: self.prepare_source_edit(self.revision(), target.file(), edits)?,
            required_proof: RequiredProof::ParagraphReorder {
                original_target: target_range,
                original_neighbor: neighbor_range,
                position,
            },
        };
        Ok(ParagraphOrderPlan { intent, original, reordered })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_exhaustion_leaves_a_typed_draft_and_shared_history_unchanged() {
        let source = "\\documentclass{article}\n\\begin{document}\nTarget.\n\\end{document}\n";
        let mut session = Session::new(
            Sources::from([("main.tex".into(), source.into())]),
            Profile {
                main: "main.tex".into(),
                engine: "auto".into(),
            },
        )
        .unwrap();
        session.revision = Revision(u64::MAX);
        let projection = session.source_projection("main.tex").unwrap();
        let target = projection
            .regions()
            .iter()
            .enumerate()
            .find_map(|(index, _)| projection.entity_reference(index))
            .unwrap();
        let plan = session
            .plan_source_intent(SourceIntent::DeleteParagraphContent { target })
            .unwrap();
        assert!(matches!(
            session.apply_source_intent_draft(plan),
            Err(EditError::RevisionExhausted)
        ));
        assert_eq!(session.sources()["main.tex"], source);
        assert_eq!(session.revision(), Revision(u64::MAX));
        assert_eq!(session.history_len(), 0);
        assert!(!session.can_undo());
    }
}
