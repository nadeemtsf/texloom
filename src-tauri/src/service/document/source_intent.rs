//! Explicit source drafts, never a visual approval or a Save operation.
//! Handles and operation receipts belong to this open native document only.
use super::*;
use texloom_core::document::{intent::{IntentPlan, PlanError, SourceIntent}, structure::EntityReference};

const MAX_OPERATIONS: usize = 128;
const MAX_OPERATION_BYTES: usize = 4_000_000;
const MAX_TARGETS: usize = 4096;
const MAX_TARGET_BYTES: usize = 8_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum DraftSourceIntent {
    ReplaceLiteralParagraph { target: String, text: String },
    DeleteParagraphContent { target: String },
}
impl DraftSourceIntent {
    fn target(&self) -> &str {
        match self { Self::ReplaceLiteralParagraph { target, .. } | Self::DeleteParagraphContent { target } => target }
    }
    fn bytes(&self) -> usize {
        self.target().len() + match self { Self::ReplaceLiteralParagraph { text, .. } => text.len(), _ => 0 }
    }
    fn core(&self, target: EntityReference) -> SourceIntent {
        match self {
            Self::ReplaceLiteralParagraph { text, .. } => SourceIntent::ReplaceLiteralParagraph { target, text: text.clone() },
            Self::DeleteParagraphContent { .. } => SourceIntent::DeleteParagraphContent { target },
        }
    }
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceTarget {
    pub handle: String,
    pub text: String,
    /// Informational editor UTF-16 coordinates; requests cannot supply ranges.
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceTargetsReply {
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub file: String,
    pub targets: Vec<SourceTarget>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum SourceDraftOutcome {
    SourceDraftApplied {
        #[cfg_attr(feature = "typegen", ts(type = "number"))]
        from_revision: u64,
        #[cfg_attr(feature = "typegen", ts(type = "number"))]
        revision: u64,
        changed: bool,
        #[cfg_attr(feature = "typegen", ts(type = "'unresolved'"))]
        render_verification: &'static str,
    },
    /// Admitted request refused before canonical or recovery mutation.
    SourceDraftRefused {
        #[cfg_attr(feature = "typegen", ts(type = "number"))]
        requested_revision: u64,
        #[cfg_attr(feature = "typegen", ts(type = "number"))]
        observed_revision: u64,
        reason: SourceDraftRefusalReason,
    },
    /// The canonical buffer stayed unchanged, but an interrupted journal write
    /// may have recorded the proposed draft. Resolve rather than retrying it.
    ProtectionUncertain {
        #[cfg_attr(feature = "typegen", ts(type = "number"))]
        from_revision: u64,
        error: String,
    },
    /// No receipt in this session; this never proves an old session did not act.
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum SourceDraftRefusalReason {
    RevisionChanged,
    TargetUnavailable,
    UnsupportedTarget,
    EmptyLiteral,
    ParagraphBreak,
    ControlCharacter,
    LiteralTooLarge,
    SourceGuardRejected,
}
impl From<PlanError> for SourceDraftRefusalReason {
    fn from(error: PlanError) -> Self {
        match error {
            PlanError::UnsupportedTarget => Self::UnsupportedTarget,
            PlanError::EmptyLiteral => Self::EmptyLiteral,
            PlanError::ParagraphBreakRequiresStructuralIntent => Self::ParagraphBreak,
            PlanError::ControlCharacter => Self::ControlCharacter,
            PlanError::LiteralTooLarge => Self::LiteralTooLarge,
            PlanError::Source(core::EditError::StaleRevision | core::EditError::ForeignSession) => Self::TargetUnavailable,
            PlanError::Source(_) => Self::SourceGuardRejected,
        }
    }
}
#[derive(Clone, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceDraftReply {
    pub operation_id: String,
    pub outcome: SourceDraftOutcome,
    /// Always current. Cached receipts must never republish an old source/PDF.
    pub snapshot: DocumentSnapshot,
}
/// A development-only reply-shape ablation, never registered as Tauri IPC.
#[cfg(feature = "benchmark")]
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceDraftCostState {
    Delta { delta: SourceEditReply },
    Snapshot { snapshot: DocumentSnapshot },
}
#[cfg(feature = "benchmark")]
#[derive(Serialize)]
pub struct SourceDraftCostReply {
    pub operation_id: String,
    pub outcome: SourceDraftOutcome,
    pub state: SourceDraftCostState,
}
struct Entry { base: u64, intent: DraftSourceIntent, outcome: SourceDraftOutcome }
#[derive(Default)]
pub(super) struct IntentState {
    target_revision: Option<u64>,
    files: BTreeMap<String, Vec<SourceTarget>>,
    targets: HashMap<String, EntityReference>,
    target_bytes: usize,
    operations: HashMap<String, Entry>,
    operation_bytes: usize,
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe_revision(value: u64) -> Result<()> {
    if value > 9_007_199_254_740_991 { Err("Revision exceeds the IPC integer range".into()) } else { Ok(()) }
}
impl DocumentSlot {
    fn prepare_source_intent(&self, base: u64, intent: &DraftSourceIntent) -> std::result::Result<IntentPlan, SourceDraftRefusalReason> {
        if self.session.revision() != Revision(base) { return Err(SourceDraftRefusalReason::RevisionChanged); }
        if self.source_intents.target_revision != Some(base) { return Err(SourceDraftRefusalReason::TargetUnavailable); }
        let target = self.source_intents.targets.get(intent.target()).ok_or(SourceDraftRefusalReason::TargetUnavailable)?.clone();
        self.session.plan_source_intent(intent.core(target)).map_err(Into::into)
    }
    fn finish_source_intent(&mut self, base: u64, operation_id: &str, intent: DraftSourceIntent, outcome: SourceDraftOutcome) -> SourceDraftOutcome {
        self.source_intents.operation_bytes += intent.bytes();
        self.source_intents.operations.insert(operation_id.into(), Entry { base, intent, outcome: outcome.clone() });
        outcome
    }
    fn refuse_source_intent(&mut self, base: u64, operation_id: &str, intent: DraftSourceIntent, reason: SourceDraftRefusalReason) -> SourceDraftOutcome {
        let outcome = SourceDraftOutcome::SourceDraftRefused { requested_revision: base, observed_revision: self.revision(), reason };
        self.finish_source_intent(base, operation_id, intent, outcome)
    }
    pub(super) fn require_resolved_source_intents(&self) -> Result<()> {
        if self.source_intents.operations.values().any(|entry| matches!(entry.outcome, SourceDraftOutcome::ProtectionUncertain { .. })) {
            Err("Source draft protection is unresolved. Inspect the status and reopen/recover the project before editing; no automatic retry is allowed.".into())
        } else { Ok(()) }
    }
}
impl Backend {
    pub fn document_source_targets(&self, id: &str, base: u64, file: &str) -> Result<SourceTargetsReply> {
        safe_revision(base)?;
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) { return Err("The source revision changed".into()); }
        if slot.source_intents.target_revision != Some(base) {
            slot.source_intents.target_revision = Some(base);
            slot.source_intents.files.clear();
            slot.source_intents.targets.clear();
            slot.source_intents.target_bytes = 0;
        }
        if let Some(targets) = slot.source_intents.files.get(file) {
            return Ok(SourceTargetsReply { revision: base, file: file.into(), targets: targets.clone() });
        }
        let projection = slot.session.source_projection(file).map_err(fail)?;
        let editor = EditorProjection::new(projection.source());
        let mut targets = Vec::new();
        let mut references = Vec::new();
        let mut bytes = 0usize;
        for (index, region) in projection.regions().iter().enumerate() {
            if let Some(reference) = projection.entity_reference(index) {
                let text = projection.text(index).ok_or("Missing source target")?;
                bytes += text.len();
                if slot.source_intents.targets.len() + targets.len() >= MAX_TARGETS ||
                    slot.source_intents.target_bytes + bytes > MAX_TARGET_BYTES {
                    return Err("Source target capacity is full; no targets were issued".into());
                }
                let handle = Uuid::new_v4().simple().to_string();
                let range = region.range();
                targets.push(SourceTarget { handle: handle.clone(), text: text.into(),
                    start: editor.editor_offset(range.start).map_err(fail)?,
                    end: editor.editor_offset(range.end).map_err(fail)? });
                references.push((handle, reference));
            }
        }
        slot.source_intents.targets.extend(references);
        slot.source_intents.target_bytes += bytes;
        slot.source_intents.files.insert(file.into(), targets.clone());
        Ok(SourceTargetsReply { revision: base, file: file.into(), targets })
    }

    pub fn document_source_intent_status(&self, id: &str, operation_id: &str) -> Result<SourceDraftReply> {
        if !valid_id(operation_id) { return Err("Invalid operation ID".into()); }
        let docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get(id).ok_or("Reopen this project")?;
        Ok(SourceDraftReply { operation_id: operation_id.into(),
            outcome: slot.source_intents.operations.get(operation_id).map(|entry| entry.outcome.clone()).unwrap_or(SourceDraftOutcome::Unknown),
            snapshot: slot.snapshot() })
    }

    pub fn document_source_intent(&self, id: &str, base: u64, operation_id: &str, intent: DraftSourceIntent) -> Result<SourceDraftReply> {
        self.source_intent_with(id, base, operation_id, intent, |slot,outcome,_| SourceDraftReply {
            operation_id: operation_id.into(), outcome, snapshot: slot.snapshot(),
        })
    }
    #[cfg(feature = "benchmark")]
    pub fn benchmark_source_intent_delta(&self, id: &str, base: u64, operation_id: &str, intent: DraftSourceIntent) -> Result<SourceDraftCostReply> {
        self.source_intent_with(id, base, operation_id, intent, |slot,outcome,changed_file| {
            let current=slot.revision();
            let state=if current==base {
                SourceDraftCostState::Delta { delta: slot.source_reply(base, "") }
            } else if changed_file.is_some() && matches!(&outcome,SourceDraftOutcome::SourceDraftApplied {
                from_revision,revision,changed:true,..} if *from_revision==base && *revision==current && current==base+1) {
                SourceDraftCostState::Delta { delta: slot.source_reply(base, changed_file.unwrap()) }
            } else { SourceDraftCostState::Snapshot { snapshot: slot.snapshot() } };
            SourceDraftCostReply { operation_id:operation_id.into(),outcome,state }
        })
    }
    fn source_intent_with<R>(&self, id: &str, base: u64, operation_id: &str, intent: DraftSourceIntent,
        reply: impl Fn(&DocumentSlot,SourceDraftOutcome,Option<&str>)->R) -> Result<R> {
        safe_revision(base)?;
        if !valid_id(operation_id) || !valid_id(intent.target()) { return Err("Invalid operation ID or source target".into()); }
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        // Resolve duplicates before freshness: retry after Undo returns the old
        // immutable receipt alongside today's snapshot, without executing again.
        if let Some(entry) = slot.source_intents.operations.get(operation_id) {
            if entry.base != base || entry.intent != intent { return Err("Operation ID already belongs to a different request".into()); }
            return Ok(reply(slot,entry.outcome.clone(),None));
        }
        slot.require_resolved_source_intents()?;
        if slot.source_intents.operations.len() >= MAX_OPERATIONS ||
            slot.source_intents.operation_bytes.saturating_add(intent.bytes()) > MAX_OPERATION_BYTES {
            return Err("Source operation receipt capacity is full; reopen only after resolving outstanding operations".into());
        }
        let plan = match slot.prepare_source_intent(base, &intent) {
            Ok(plan) => plan,
            Err(reason) => {let outcome=slot.refuse_source_intent(base,operation_id,intent,reason);return Ok(reply(slot,outcome,None));},
        };
        let changed = plan.changed();
        let patch = plan.source_patch();
        let (file, edits) = match patch.files.as_slice() {
            [FileChange::Edit {file,edits}] => (file, edits),
            _ => {let outcome=slot.refuse_source_intent(base,operation_id,intent,SourceDraftRefusalReason::SourceGuardRejected);return Ok(reply(slot,outcome,None));},
        };
        let changed_file = changed.then(|| file.clone());
        let outcome = if changed {
            // Preflight every fallible core guard on the exact future state.
            let proposed = (|| -> Result<core::Session> {
                let mut proposed = slot.session.fork_for_preflight();
                let candidate = proposed.prepare_source_edit(Revision(base), file, edits.clone()).map_err(fail)?;
                proposed.commit_source_edit(candidate, None).map_err(fail)?;
                safe_revision(proposed.revision().0)?;
                self.mark_cache_unclean()?;
                Ok(proposed)
            })();
            let proposed = match proposed {
                Ok(proposed) => proposed,
                Err(_) => {let outcome=slot.refuse_source_intent(base,operation_id,intent,SourceDraftRefusalReason::SourceGuardRejected);return Ok(reply(slot,outcome,None));},
            };
            match self.protect_source_intent(id, slot, base, patch, &proposed) {
                Ok(()) => {
                    match slot.session.apply_source_intent_draft(plan) {
                        Ok(receipt) => {
                            slot.pending.clear();
                            self.scheduler.cancel_project(id);
                            SourceDraftOutcome::SourceDraftApplied { from_revision: receipt.from_revision().0,
                                revision: receipt.revision().0, changed: receipt.changed(), render_verification: "unresolved" }
                        },
                        // Protection already ran: never describe this as a
                        // clean refusal, even if an unexpected core guard fails.
                        Err(error) => SourceDraftOutcome::ProtectionUncertain { from_revision: base, error: format!("Protected draft requires reconciliation: {error}") },
                    }
                },
                Err(error) => SourceDraftOutcome::ProtectionUncertain { from_revision: base, error },
            }
        } else {
            let receipt = match slot.session.apply_source_intent_draft(plan) {
                Ok(receipt) => receipt,
                Err(_) => {let outcome=slot.refuse_source_intent(base,operation_id,intent,SourceDraftRefusalReason::SourceGuardRejected);return Ok(reply(slot,outcome,None));},
            };
            SourceDraftOutcome::SourceDraftApplied { from_revision: receipt.from_revision().0,
                revision: receipt.revision().0, changed: false, render_verification: "unresolved" }
        };
        let outcome=slot.finish_source_intent(base,operation_id,intent,outcome);
        Ok(reply(slot,outcome,changed_file.as_deref()))
    }
}

#[cfg(test)]
mod tests;
