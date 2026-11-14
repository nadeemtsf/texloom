//! Explicit source drafts, never a visual approval or a Save operation.
//! Handles and operation receipts belong to this open native document only.
use super::*;
use texloom_core::document::{intent::SourceIntent, structure::EntityReference};

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
#[derive(Clone, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceDraftReply {
    pub operation_id: String,
    pub outcome: SourceDraftOutcome,
    /// Always current. Cached receipts must never republish an old source/PDF.
    pub snapshot: DocumentSnapshot,
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
        safe_revision(base)?;
        if !valid_id(operation_id) || !valid_id(intent.target()) { return Err("Invalid operation ID or source target".into()); }
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        // Resolve duplicates before freshness: retry after Undo returns the old
        // immutable receipt alongside today's snapshot, without executing again.
        if let Some(entry) = slot.source_intents.operations.get(operation_id) {
            if entry.base != base || entry.intent != intent { return Err("Operation ID already belongs to a different request".into()); }
            return Ok(SourceDraftReply { operation_id: operation_id.into(), outcome: entry.outcome.clone(), snapshot: slot.snapshot() });
        }
        slot.require_resolved_source_intents()?;
        if slot.source_intents.operations.len() >= MAX_OPERATIONS ||
            slot.source_intents.operation_bytes.saturating_add(intent.bytes()) > MAX_OPERATION_BYTES {
            return Err("Source operation receipt capacity is full; reopen only after resolving outstanding operations".into());
        }
        if slot.session.revision() != Revision(base) { return Err("The source revision changed".into()); }
        let target = slot.source_intents.targets.get(intent.target()).ok_or("Source target is unknown in this session")?.clone();
        let plan = slot.session.plan_source_intent(intent.core(target)).map_err(|error| format!("Source intent refused: {error:?}"))?;
        let changed = plan.changed();
        let outcome = if changed {
            // Preflight every fallible core guard on the exact future state.
            // Reconstructing history adds no visual or disk-save authority.
            let patch = plan.source_patch();
            let mut proposed = core::Session::from_checkpoint(slot.session.checkpoint()).map_err(fail)?;
            let candidate = proposed.prepare(patch.clone()).map_err(fail)?;
            proposed.commit(candidate, Origin::Source, None).map_err(fail)?;
            safe_revision(proposed.revision().0)?;
            self.mark_cache_unclean()?;
            match self.protect_source_intent(id, slot, base, patch, &proposed) {
                Ok(()) => {
                    let receipt = slot.session.apply_source_intent_draft(plan).map_err(fail)?;
                    slot.pending.clear();
                    self.scheduler.cancel_project(id);
                    SourceDraftOutcome::SourceDraftApplied { from_revision: receipt.from_revision().0,
                        revision: receipt.revision().0, changed: receipt.changed(), render_verification: "unresolved" }
                },
                Err(error) => SourceDraftOutcome::ProtectionUncertain { from_revision: base, error },
            }
        } else {
            let receipt = slot.session.apply_source_intent_draft(plan).map_err(fail)?;
            SourceDraftOutcome::SourceDraftApplied { from_revision: receipt.from_revision().0,
                revision: receipt.revision().0, changed: false, render_verification: "unresolved" }
        };
        slot.source_intents.operation_bytes += intent.bytes();
        slot.source_intents.operations.insert(operation_id.into(), Entry { base, intent, outcome: outcome.clone() });
        Ok(SourceDraftReply { operation_id: operation_id.into(), outcome, snapshot: slot.snapshot() })
    }
}

#[cfg(test)]
mod tests;
