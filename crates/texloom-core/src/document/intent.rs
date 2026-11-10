//! Typed read-only candidate planning. No UI, compilation or visual commit.
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
