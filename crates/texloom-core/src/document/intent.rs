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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    Source(EditError),
    UnsupportedTarget,
    EmptyLiteral,
    ParagraphBreakRequiresStructuralIntent,
    ControlCharacter,
    LiteralTooLarge,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutEffect {
    DocumentReflowRequiringVerification,
}

/// Private construction guarantees one rederived source-range write. The plan
/// does not expose its Prepared object or a geometry-acceptance method.
pub struct IntentPlan {
    kind: OperationKind,
    prepared: Prepared,
    required_proof: RequiredProof,
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
    pub fn candidate_sources(&self) -> &Sources {
        self.prepared.sources()
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
        let revision = self.commit(prepared, Origin::Source, None)?;
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
        let patch = Patch {
            base: self.revision(),
            files: vec![FileChange::Edit {
                file: resolved.file().into(),
                edits: vec![Replacement {
                    range: resolved.range(),
                    expected: resolved.text().into(),
                    text: replacement,
                }],
            }],
            profile: None,
        };
        Ok(IntentPlan {
            kind,
            prepared: self.prepare(patch)?,
            required_proof,
        })
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
