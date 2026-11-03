//! Native document authority. IPC snapshots are replicas; only this service
//! applies edits or history. Compiler jobs consume immutable session snapshots.
use super::*;
use texloom_core::document::{
    self as core, projection::EditorProjection, FileChange, Origin, Patch, Profile, ProfileChange,
    Replacement, Revision, Selection, TypingGroup,
};
mod typing_recovery;
pub use typing_recovery::{CloseReply, RecoveryReply, RecoveryStatus};

pub(super) struct DocumentSlot {
    session: core::Session,
    pub(super) pending: HashMap<String, Pending>,
    protection: typing_recovery::Protection,
}
pub(super) struct Pending {
    candidate: core::Prepared,
    pub(super) plan: Option<Arc<layout_gate::Plan>>,
    pub(super) approval: Option<layout_gate::Approval>,
}
impl DocumentSlot {
    pub(super) fn revision(&self) -> u64 { self.session.revision().0 }
    pub(super) fn discard(&mut self, token: &str) {
        self.pending.remove(token);
    }
    pub(super) fn new(sources: BTreeMap<String, String>, main: String) -> Result<Self> {
        Ok(Self {
            session: core::Session::new(
                sources,
                Profile {
                    main,
                    engine: "auto".into(),
                },
            )
            .map_err(fail)?,
            pending: HashMap::new(),
            protection: Default::default(),
        })
    }
    fn snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            revision: self.session.revision().0,
            sources: self.session.sources().clone(),
            main: self.session.profile().main.clone(),
            engine: self.session.profile().engine.clone(),
            can_undo: self.session.can_undo(),
            can_redo: self.session.can_redo(),
            selection: self.editor_selection(),
            recovery: self.protection.status(),
        }
    }
    fn source_reply(&self, base: u64, file: &str) -> SourceEditReply {
        SourceEditReply {
            kind: "source_delta",
            base_revision: base,
            revision: self.session.revision().0,
            sources: if self.session.revision() != Revision(base) {
                BTreeMap::from([(file.to_string(), self.session.sources()[file].clone())])
            } else {
                BTreeMap::new()
            },
            main: self.session.profile().main.clone(),
            engine: self.session.profile().engine.clone(),
            can_undo: self.session.can_undo(),
            can_redo: self.session.can_redo(),
            selection: self.editor_selection(),
            recovery: self.protection.status(),
        }
    }
    fn editor_selection(&self) -> Option<EditorSelection> {
        let s = self.session.selection()?;
        let p = EditorProjection::new(self.session.sources().get(&s.file)?);
        let secondary = s
            .secondary
            .iter()
            .map(|(a, h)| {
                Some(EditorRange {
                    anchor: p.editor_offset(*a).ok()?,
                    head: p.editor_offset(*h).ok()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(EditorSelection {
            file: s.file.clone(),
            anchor: p.editor_offset(s.anchor).ok()?,
            head: p.editor_offset(s.head).ok()?,
            secondary,
        })
    }
}
#[derive(Clone, Serialize)]
pub struct DocumentSnapshot {
    pub revision: u64,
    pub sources: BTreeMap<String, String>,
    pub main: String,
    pub engine: String,
    pub can_undo: bool,
    pub can_redo: bool,
    pub selection: Option<EditorSelection>,
    pub recovery: RecoveryStatus,
}
/// A source edit cannot create/delete files or change the project profile.
/// Send only its changed file; opening, history and visual commits send full snapshots.
#[derive(Clone, Serialize)]
pub struct SourceEditReply {
    pub kind: &'static str,
    pub base_revision: u64,
    pub revision: u64,
    pub sources: BTreeMap<String, String>,
    pub main: String,
    pub engine: String,
    pub can_undo: bool,
    pub can_redo: bool,
    pub selection: Option<EditorSelection>,
    pub recovery: RecoveryStatus,
}
#[derive(Clone, Serialize)]
pub struct EditorSelection {
    pub file: String,
    pub anchor: usize,
    pub head: usize,
    pub secondary: Vec<EditorRange>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct EditorRange {
    pub anchor: usize,
    pub head: usize,
}
#[derive(Serialize)]
pub struct SelectionReply {
    pub revision: u64,
    pub selection: Option<EditorSelection>,
    pub recovery: RecoveryStatus,
}
#[derive(Deserialize)]
pub struct EditorChange {
    pub offset: usize,
    pub length: usize,
    pub text: String,
}
#[derive(Serialize)]
pub struct PreparedReply {
    pub token: String,
    pub revision: u64,
}

impl Backend {
    pub fn document_selection(
        &self,
        id: &str,
        base: u64,
        file: String,
        anchor: usize,
        head: usize,
        secondary: Vec<EditorRange>,
    ) -> Result<SelectionReply> {
        if secondary.len() > 9999 {
            return Err("Too many cursors".into());
        }
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source revision changed".into());
        }
        let p = EditorProjection::new(
            slot.session
                .sources()
                .get(&file)
                .ok_or("Source file is missing")?,
        );
        let secondary = secondary
            .into_iter()
            .map(|r| {
                Ok((
                    p.source_offset(r.anchor).map_err(fail)?,
                    p.source_offset(r.head).map_err(fail)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let selection = Selection {
            file,
            anchor: p.source_offset(anchor).map_err(fail)?,
            head: p.source_offset(head).map_err(fail)?,
            secondary,
        };
        let changed = slot.session.selection() != Some(&selection);
        slot.session.set_selection(selection.clone()).map_err(fail)?;
        if changed {
            self.record_typing_recovery(id, slot, base, texloom_build::recovery::Mutation::Selection { selection });
        }
        Ok(SelectionReply {
            revision: slot.session.revision().0,
            selection: slot.editor_selection(),
            recovery: slot.protection.status(),
        })
    }
    pub fn document_snapshot(&self, id: &str) -> Result<DocumentSnapshot> {
        self.documents
            .lock()
            .map_err(fail)?
            .get(id)
            .map(DocumentSlot::snapshot)
            .ok_or("Reopen this project".into())
    }
    pub fn document_edit(
        &self,
        id: &str,
        base: u64,
        file: &str,
        changes: Vec<EditorChange>,
        group: Option<u64>,
    ) -> Result<DocumentSnapshot> {
        self.edit_source_with(id, base, file, changes, group, DocumentSlot::snapshot)
    }
    pub fn document_edit_delta(
        &self,
        id: &str,
        base: u64,
        file: &str,
        changes: Vec<EditorChange>,
        group: Option<u64>,
    ) -> Result<SourceEditReply> {
        self.edit_source_with(id, base, file, changes, group, |slot| {
            slot.source_reply(base, file)
        })
    }
    fn edit_source_with<R>(
        &self,
        id: &str,
        base: u64,
        file: &str,
        changes: Vec<EditorChange>,
        group: Option<u64>,
        reply: impl FnOnce(&DocumentSlot) -> R,
    ) -> Result<R> {
        if changes.len() > 10_000 || changes.iter().map(|c| c.text.len()).sum::<usize>() > 4_000_000
        {
            return Err("Edit exceeds source limits".into());
        }
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source revision changed".into());
        }
        let source = slot
            .session
            .sources()
            .get(file)
            .ok_or("Source file is missing")?;
        let projection = EditorProjection::new(source);
        let edits = changes
            .into_iter()
            .map(|c| {
                let end = c
                    .offset
                    .checked_add(c.length)
                    .ok_or("Invalid editor range")?;
                projection.replacement(c.offset, end, &c.text).map_err(fail)
            })
            .collect::<Result<Vec<_>>>()?;
        let candidate = slot
            .session
            .prepare(Patch {
                base: Revision(base),
                files: vec![FileChange::Edit {
                    file: file.into(),
                    edits,
                }],
                profile: None,
            })
            .map_err(fail)?;
        if candidate.changed() {
            self.mark_cache_unclean()?;
        }
        let mutation = candidate.changed().then(|| texloom_build::recovery::Mutation::Patch {
            patch: candidate.recovery_patch(), origin: Origin::Source, group: group.map(TypingGroup),
        });
        slot.session
            .commit(candidate, Origin::Source, group.map(TypingGroup))
            .map_err(fail)?;
        if slot.session.revision() != Revision(base) {
            slot.pending.clear();
            self.scheduler.cancel_project(id);
        }
        if let Some(mutation) = mutation { self.record_typing_recovery(id, slot, base, mutation); }
        Ok(reply(slot))
    }
    pub fn document_profile(
        &self,
        id: &str,
        base: u64,
        main: String,
        engine: String,
    ) -> Result<DocumentSnapshot> {
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        let candidate = slot
            .session
            .prepare(Patch {
                base: Revision(base),
                files: vec![],
                profile: Some(ProfileChange {
                    expected: slot.session.profile().clone(),
                    replacement: Profile { main, engine },
                }),
            })
            .map_err(fail)?;
        if candidate.changed() {
            self.mark_cache_unclean()?;
        }
        let mutation = candidate.changed().then(|| texloom_build::recovery::Mutation::Patch {
            patch: candidate.recovery_patch(), origin: Origin::Settings, group: None,
        });
        slot.session
            .commit(candidate, Origin::Settings, None)
            .map_err(fail)?;
        if slot.session.revision() != Revision(base) {
            slot.pending.clear();
            self.scheduler.cancel_project(id);
        }
        if let Some(mutation) = mutation { self.record_typing_recovery(id, slot, base, mutation); }
        Ok(slot.snapshot())
    }
    pub fn document_history(&self, id: &str, base: u64, redo: bool) -> Result<DocumentSnapshot> {
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source revision changed".into());
        }
        if (redo && slot.session.can_redo()) || (!redo && slot.session.can_undo()) {
            self.mark_cache_unclean()?;
        }
        if redo {
            slot.session.redo().map_err(fail)?;
        } else {
            slot.session.undo().map_err(fail)?;
        }
        if slot.session.revision() != Revision(base) {
            slot.pending.clear();
            self.scheduler.cancel_project(id);
            self.record_typing_recovery(id, slot, base, if redo {texloom_build::recovery::Mutation::Redo} else {texloom_build::recovery::Mutation::Undo});
        }
        Ok(slot.snapshot())
    }
    #[cfg(test)]
    pub fn document_prepare(
        &self,
        id: &str,
        base: u64,
        sources: BTreeMap<String, String>,
    ) -> Result<PreparedReply> {
        self.prepare_candidate(id, base, sources, None)
    }
    pub fn document_prepare_visual(
        &self,
        id: &str,
        base: u64,
        sources: BTreeMap<String, String>,
        baseline: &str,
        operation: Option<VisualOperation>,
    ) -> Result<PreparedReply> {
        self.verify_project_build(id, base, baseline, true)?;
        self.prepare_candidate(id, base, sources, operation.map(|op| (baseline, op)))
    }
    fn prepare_candidate(
        &self,
        id: &str,
        base: u64,
        sources: BTreeMap<String, String>,
        operation: Option<(&str, VisualOperation)>,
    ) -> Result<PreparedReply> {
        for (name, text) in &sources {
            if !text_file(&relative(name)?) || text.len() > 4_000_000 {
                return Err("Invalid source file".into());
            }
        }
        if sources.len() > MAX_FILES
            || sources.values().map(String::len).sum::<usize>() > MAX_PROJECT_BYTES as usize
        {
            return Err("Project exceeds source limits".into());
        }
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source revision changed".into());
        }
        let plan = operation
            .map(|(baseline, op)| {
                self.plan_visual(
                    id,
                    base,
                    baseline,
                    slot.session.sources(),
                    &slot.session.profile().main,
                    &sources,
                    op,
                )
                .map(Arc::new)
            })
            .transpose()?;
        let files = diff_sources(slot.session.sources(), &sources);
        let candidate = slot
            .session
            .prepare(Patch {
                base: Revision(base),
                files,
                profile: None,
            })
            .map_err(fail)?;
        let revision = base
            .checked_add(u64::from(candidate.changed()))
            .ok_or("Revision exhausted")?;
        let token = Uuid::new_v4().simple().to_string();
        slot.pending.clear();
        slot.pending.insert(
            token.clone(),
            Pending {
                candidate,
                plan,
                approval: None,
            },
        );
        Ok(PreparedReply { token, revision })
    }
    pub fn compile_document(
        &self,
        id: &str,
        revision: u64,
        token: Option<&str>,
        purpose: Option<&str>,
        sequence: u64,
    ) -> Result<CompileResult> {
        let purpose = if token.is_some() {
            "visual"
        } else {
            purpose.unwrap_or("preview")
        };
        if !["preview", "visual", "export"].contains(&purpose) {
            return Err("Unknown build purpose".into());
        }
        let lease = self.scheduler.request_sequence(id, sequence)?.acquire()?;
        let (request, plan) = {
            let mut docs = self.documents.lock().map_err(fail)?;
            let slot = docs.get_mut(id).ok_or("Reopen this project")?;
            if slot.session.revision() != Revision(revision) {
                return Err("The source changed before compilation started".into());
            }
            if let Some(token) = token {
                let pending = slot
                    .pending
                    .get_mut(token)
                    .ok_or("The visual candidate is no longer current")?;
                pending.approval = None;
                (
                    CompileRequest {
                        project_id: id.into(),
                        main: pending.candidate.profile().main.clone(),
                        sources: pending.candidate.sources().clone(),
                        engine: pending.candidate.profile().engine.clone(),
                        revision: pending.candidate.base().0
                            + u64::from(pending.candidate.changed()),
                        purpose: purpose.into(),
                    },
                    pending.plan.clone(),
                )
            } else {
                (
                    CompileRequest {
                        project_id: id.into(),
                        main: slot.session.profile().main.clone(),
                        sources: slot.session.sources().clone(),
                        engine: slot.session.profile().engine.clone(),
                        revision: slot.session.revision().0,
                        purpose: purpose.into(),
                    },
                    None,
                )
            }
        };
        let result = self.compile(request.clone(), &lease)?;
        let approval = if result.ok && result.verification_eligible {
            plan.as_ref()
                .map(|p| self.approve_layout(p, &request, &result, &lease))
                .transpose()?
        } else {
            None
        };
        lease.cancel.check()?;
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(revision) {
            return Err("The source changed while compiling".into());
        }
        if let Some(token) = token {
            let pending = slot
                .pending
                .get_mut(token)
                .ok_or("The source changed while compiling the visual candidate")?;
            pending.approval = approval;
        }
        Ok(result)
    }
    /// Requires native source-plan/build approval as well as the UI's completed
    /// PDF preparation. A private selection probe can never be committed.
    pub fn document_commit(
        &self,
        id: &str,
        token: &str,
        build_id: &str,
    ) -> Result<DocumentSnapshot> {
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        let pending = slot
            .pending
            .get(token)
            .ok_or("The visual candidate is no longer current")?;
        let approval = pending
            .approval
            .as_ref()
            .ok_or("This candidate has no native visual-operation approval")?;
        if pending.candidate.base() != slot.session.revision() {
            return Err("The source revision changed".into());
        }
        approval.check(id, slot.session.revision().0, build_id)?;
        if pending.candidate.changed() {
            self.mark_cache_unclean()?;
        }
        let pending = slot
            .pending
            .remove(token)
            .ok_or("The visual candidate is missing")?;
        let base = slot.session.revision().0;
        let mutation = pending.candidate.changed().then(|| texloom_build::recovery::Mutation::Patch {
            patch: pending.candidate.recovery_patch(), origin: Origin::Canvas, group: None,
        });
        slot.session
            .commit(pending.candidate, Origin::Canvas, None)
            .map_err(fail)?;
        if let Some(mutation) = mutation { self.record_typing_recovery(id, slot, base, mutation); }
        Ok(slot.snapshot())
    }
    pub fn document_discard(&self, id: &str, token: &str) -> Result<()> {
        if let Some(slot) = self.documents.lock().map_err(fail)?.get_mut(id) {
            slot.pending.remove(token);
        }
        Ok(())
    }
    pub fn project_recoveries(&self, id:&str) -> Result<Vec<texloom_build::persistence::PendingSave>> {
        texloom_build::persistence::pending(&self.project_root(id)?)
    }
    pub fn recover_document(&self,id:&str,base:u64,transaction:&str,finish:bool)->Result<Project> {
        let (root,profile) = {
            let documents=self.documents.lock().map_err(fail)?;
            let slot=documents.get(id).ok_or("Reopen this project")?;
            if slot.session.revision()!=Revision(base){return Err("The editor changed before save recovery; try again".into());}
            let root=self.project_root(id)?;
            texloom_build::persistence::recover(&root,transaction,finish)?;
            (root,slot.session.profile().clone())
        };
        let preferred=root.join(relative(&profile.main)?);
        let opened=self.open_project(Some(if preferred.is_file(){preferred}else{root}.to_string_lossy().into_owned()))?;
        if self.document_snapshot(id)?.revision!=base {
            self.close_document(&opened.id)?;
            return Err("Files were recovered, but newer editor changes were preserved. Save a copy before reopening.".into());
        }
        self.document_profile(&opened.id,0,opened.main.clone(),profile.engine)?;
        Ok(opened)
    }
    pub fn discard_document_recovery(&self,id:&str,base:u64,transaction:&str)->Result<Vec<texloom_build::persistence::PendingSave>> {
        let documents=self.documents.lock().map_err(fail)?;
        let slot=documents.get(id).ok_or("Reopen this project")?;
        if slot.session.revision()!=Revision(base){return Err("The editor changed before discarding recovery data".into());}
        let root=self.project_root(id)?;
        texloom_build::persistence::discard_recovery(&root,transaction)?;
        texloom_build::persistence::pending(&root)
    }
    pub fn copy_document(&self, id: &str, base: u64, parent: &str) -> Result<String> {
        let documents = self.documents.lock().map_err(fail)?;
        let slot = documents.get(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source changed before the project copy; try again".into());
        }
        let root = self.project_root(id)?;
        let mut sources = slot.session.sources().clone();
        if !sources.contains_key("studio.sty") && sources.values().any(|s|s.contains("\\StudioBlock")||s.contains("\\usepackage{studio}")) {
            sources.insert("studio.sty".into(),STYLE.into());
        }
        if !sources.contains_key("texloom-objects.sty") && sources.values().any(|s|s.contains("\\TexloomObject")||s.contains("\\usepackage{texloom-objects}")) {
            sources.insert("texloom-objects.sty".into(),OBJECT_STYLE.into());
        }
        (if root.exists() {texloom_build::persistence::copy_project(&root,&sources,Path::new(parent))}
        else {texloom_build::persistence::copy_sources(&sources,Path::new(parent))})
            .map(|path|path.to_string_lossy().into_owned())
    }
    pub fn save_document(&self, id: &str, base: u64) -> Result<RecoveryReply> {
        // Serialize the full explicit save with edits/history/closure. No later
        // source revision can be written under an earlier Save acknowledgement.
        let mut documents = self.documents.lock().map_err(fail)?;
        let slot = documents.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source changed before Save; try again".into());
        }
        self.save_project(id, slot.session.sources())?;
        // Save succeeded even if recovery storage is degraded. Never report
        // an accepted disk Save as failed because a separate journal failed.
        self.checkpoint_typing_recovery(id, slot);
        Ok(RecoveryReply { revision: base, recovery: slot.protection.status() })
    }
}

fn diff_sources(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> Vec<FileChange> {
    let mut changes = Vec::new();
    for (file, old) in before {
        match after.get(file) {
            None => changes.push(FileChange::Remove {
                file: file.clone(),
                expected: old.clone(),
            }),
            Some(next) if next != old => {
                let mut start = old
                    .bytes()
                    .zip(next.bytes())
                    .take_while(|(a, b)| a == b)
                    .count();
                while !old.is_char_boundary(start) || !next.is_char_boundary(start) {
                    start -= 1;
                }
                let mut old_end = old.len();
                let mut next_end = next.len();
                while old_end > start
                    && next_end > start
                    && old.as_bytes().get(old_end - 1) == next.as_bytes().get(next_end - 1)
                {
                    old_end -= 1;
                    next_end -= 1;
                }
                while !old.is_char_boundary(old_end) || !next.is_char_boundary(next_end) {
                    old_end += 1;
                    next_end += 1;
                }
                changes.push(FileChange::Edit {
                    file: file.clone(),
                    edits: vec![Replacement {
                        range: start..old_end,
                        expected: old.get(start..old_end).unwrap_or_default().into(),
                        text: next.get(start..next_end).unwrap_or_default().into(),
                    }],
                });
            }
            _ => {}
        }
    }
    for (file, text) in after {
        if !before.contains_key(file) {
            changes.push(FileChange::Create {
                file: file.clone(),
                text: text.clone(),
            });
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_delta_preserves_other_files_and_the_full_shared_history() {
        let original = "\u{feff}A\r\n\u{1f600}B\r\n";
        let b = Backend::new(PathBuf::from("."));
        let sources = BTreeMap::from([
            ("main.tex".into(), original.into()),
            ("chapter.tex".into(), "Untouched chapter\r\n".into()),
        ]);
        b.documents.lock().unwrap().insert(
            "test".into(),
            DocumentSlot::new(sources.clone(), "main.tex".into()).unwrap(),
        );
        b.document_selection("test", 0, "main.tex".into(), 4, 5, vec![])
            .unwrap();
        let reply = b
            .document_edit_delta(
                "test",
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 4,
                    length: 1,
                    text: "New\nName".into(),
                }],
                Some(1),
            )
            .unwrap();
        assert_eq!(reply.kind, "source_delta");
        assert_eq!((reply.base_revision, reply.revision), (0, 1));
        assert_eq!(
            reply.sources,
            BTreeMap::from([(
                "main.tex".into(),
                "\u{feff}A\r\n\u{1f600}New\r\nName\r\n".into()
            )])
        );
        assert!(reply.can_undo && !reply.can_redo);
        assert_eq!(
            serde_json::to_value(&reply).unwrap()["kind"],
            "source_delta"
        );
        let full = b.document_snapshot("test").unwrap();
        assert_eq!(full.sources.len(), 2);
        assert_eq!(full.sources["chapter.tex"], sources["chapter.tex"]);
        let undone = b.document_history("test", 1, false).unwrap();
        assert_eq!(undone.sources, sources);
        assert_eq!(undone.selection.unwrap().anchor, 4);
        assert_eq!(
            b.document_history("test", 2, true).unwrap().sources,
            full.sources
        );
    }
    #[test]
    fn source_delta_no_op_is_empty_and_stale_or_invalid_edits_change_nothing() {
        let b = backend("A");
        let same = b
            .document_edit_delta(
                "test",
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 0,
                    length: 1,
                    text: "A".into(),
                }],
                Some(1),
            )
            .unwrap();
        assert_eq!((same.base_revision, same.revision), (0, 0));
        assert!(same.sources.is_empty());
        assert!(!same.can_undo);
        assert!(b
            .document_edit_delta("test", 1, "main.tex", vec![], None)
            .is_err());
        assert!(b
            .document_edit_delta("test", 0, "missing.tex", vec![], None)
            .is_err());
        assert!(b
            .document_edit_delta(
                "test",
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 2,
                    length: 0,
                    text: "B".into(),
                }],
                None
            )
            .is_err());
        let current = b.document_snapshot("test").unwrap();
        assert_eq!(current.revision, 0);
        assert_eq!(current.sources["main.tex"], "A");
        assert!(!current.can_undo);
    }
    #[test]
    fn no_op_source_profile_and_history_keep_the_current_job_and_candidate() {
        let b = backend("A");
        let lease = b
            .scheduler
            .request_sequence("test", 1)
            .unwrap()
            .acquire()
            .unwrap();
        let p = b
            .document_prepare("test", 0, BTreeMap::from([("main.tex".into(), "B".into())]))
            .unwrap();
        b.document_edit(
            "test",
            0,
            "main.tex",
            vec![EditorChange {
                offset: 0,
                length: 1,
                text: "A".into(),
            }],
            None,
        )
        .unwrap();
        b.document_profile("test", 0, "main.tex".into(), "auto".into())
            .unwrap();
        b.document_history("test", 0, false).unwrap();
        assert!(!lease.cancel.cancelled());
        assert!(b.documents.lock().unwrap()["test"]
            .pending
            .contains_key(&p.token));
        b.document_edit(
            "test",
            0,
            "main.tex",
            vec![EditorChange {
                offset: 0,
                length: 1,
                text: "C".into(),
            }],
            None,
        )
        .unwrap();
        assert!(lease.cancel.cancelled());
        assert!(!b.documents.lock().unwrap()["test"]
            .pending
            .contains_key(&p.token));
    }
    #[test]
    fn native_multi_cursor_history_projects_every_caret() {
        let b = backend("\u{feff}A\r\n\u{1f600}B\r\n");
        b.document_selection(
            "test",
            0,
            "main.tex".into(),
            0,
            1,
            vec![EditorRange { anchor: 4, head: 5 }],
        )
        .unwrap();
        b.document_edit(
            "test",
            0,
            "main.tex",
            vec![
                EditorChange {
                    offset: 0,
                    length: 1,
                    text: "X".into(),
                },
                EditorChange {
                    offset: 4,
                    length: 1,
                    text: "Y".into(),
                },
            ],
            Some(1),
        )
        .unwrap();
        b.document_selection(
            "test",
            1,
            "main.tex".into(),
            1,
            1,
            vec![EditorRange { anchor: 5, head: 5 }],
        )
        .unwrap();
        let undo = b.document_history("test", 1, false).unwrap();
        assert_eq!(undo.sources["main.tex"], "\u{feff}A\r\n\u{1f600}B\r\n");
        let s = undo.selection.unwrap();
        assert_eq!((s.anchor, s.head), (0, 1));
        assert_eq!((s.secondary[0].anchor, s.secondary[0].head), (4, 5));
        let redo = b.document_history("test", 2, true).unwrap();
        assert_eq!(redo.sources["main.tex"], "\u{feff}X\r\n\u{1f600}Y\r\n");
        assert_eq!(redo.selection.unwrap().secondary[0].head, 5);
        assert!(b
            .document_selection("test", 3, "main.tex".into(), 3, 3, vec![])
            .is_err()); // surrogate interior
    }
    #[test]
    fn explicit_save_and_reopen_preserve_canonical_bytes() {
        let root = std::env::temp_dir().join(format!("texloom-document-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let original = "\u{feff}\\documentclass{article}\r\nA\nB\r\n";
        let path = root.join("main.tex");
        fs::write(&path, original).unwrap();
        let b = Backend::new(root.join("app-data"));
        let project = b.open_project(Some(path.to_string_lossy().into())).unwrap();
        assert_eq!(project.sources["main.tex"].as_bytes(), original.as_bytes());
        let offset = EditorProjection::new(original)
            .text()
            .encode_utf16()
            .count()
            - 2;
        let snapshot = b
            .document_edit(
                &project.id,
                0,
                "main.tex",
                vec![EditorChange {
                    offset,
                    length: 1,
                    text: "C".into(),
                }],
                None,
            )
            .unwrap();
        assert_eq!(fs::read(&path).unwrap(), original.as_bytes()); // typing is not Save
        b.save_document(&project.id, snapshot.revision).unwrap();
        assert_eq!(
            fs::read(&path).unwrap(),
            snapshot.sources["main.tex"].as_bytes()
        );
        let reopened = b.open_project(Some(path.to_string_lossy().into())).unwrap();
        assert_eq!(reopened.sources, snapshot.sources);
        drop(b);
        fs::remove_dir_all(root).unwrap();
    }
    fn backend(text: &str) -> Backend {
        let b = Backend::new(PathBuf::from("."));
        b.documents.lock().unwrap().insert(
            "test".into(),
            DocumentSlot::new(
                BTreeMap::from([("main.tex".into(), text.into())]),
                "main.tex".into(),
            )
            .unwrap(),
        );
        b
    }
    #[test]
    fn native_editor_protocol_preserves_windows_bytes_and_shares_history() {
        let b = backend("\u{feff}A\r\n🦀B\r\n");
        let changes =
            serde_json::from_str(r#"[{"offset":4,"length":1,"text":"New\nName"}]"#).unwrap();
        let result = b
            .document_edit("test", 0, "main.tex", changes, Some(1))
            .unwrap();
        assert_eq!(result.sources["main.tex"], "\u{feff}A\r\n🦀New\r\nName\r\n");
        assert_eq!(
            b.document_history("test", 1, false).unwrap().sources["main.tex"],
            "\u{feff}A\r\n🦀B\r\n"
        );
        assert!(b
            .document_edit("test", 0, "main.tex", vec![], None)
            .is_err());
    }
    #[test]
    fn visual_candidate_requires_its_own_successful_build_and_typing_invalidates_it() {
        let b = backend("A");
        let p = b
            .document_prepare("test", 0, BTreeMap::from([("main.tex".into(), "B".into())]))
            .unwrap();
        assert!(b
            .document_commit("test", &p.token, "unrelated-build")
            .is_err());
        b.document_edit(
            "test",
            0,
            "main.tex",
            vec![EditorChange {
                offset: 0,
                length: 1,
                text: "Typed".into(),
            }],
            None,
        )
        .unwrap();
        assert!(b
            .document_commit("test", &p.token, "unrelated-build")
            .is_err());
        assert_eq!(
            b.document_snapshot("test").unwrap().sources["main.tex"],
            "Typed"
        );
    }
    #[test]
    fn minimal_visual_diff_roundtrips_unicode_and_added_dependencies() {
        let before = BTreeMap::from([("main.tex".into(), "\u{feff}🦀 % keep\r\nA".into())]);
        let after = BTreeMap::from([
            ("main.tex".into(), "\u{feff}🦀 % keep\r\nB".into()),
            ("objects.sty".into(), "Style".into()),
        ]);
        let mut s = core::Session::new(
            before.clone(),
            Profile {
                main: "main.tex".into(),
                engine: "auto".into(),
            },
        )
        .unwrap();
        let candidate = s
            .prepare(Patch {
                base: Revision(0),
                files: diff_sources(&before, &after),
                profile: None,
            })
            .unwrap();
        s.commit(candidate, Origin::Canvas, None).unwrap();
        assert_eq!(s.sources(), &after);
        s.undo().unwrap();
        assert_eq!(s.sources(), &before);
    }
    fn saved_project_fixture(source:&str)->(Backend,Project,PathBuf) {
        let root=std::env::temp_dir().join(format!("texloom-native-save-{}",Uuid::new_v4().simple()));
        let project=root.join("project");fs::create_dir_all(&project).unwrap();
        fs::write(project.join("main.tex"),source).unwrap();
        let backend=Backend::new(root.join("app"));
        let opened=backend.open_project(Some(project.join("main.tex").to_string_lossy().into_owned())).unwrap();
        (backend,opened,root)
    }
    #[test]
    fn explicit_save_refuses_external_changes_without_losing_editor_bytes_or_history() {
        let source="\\documentclass{article}\n\\begin{document}Before\\end{document}";
        let (backend,project,root)=saved_project_fixture(source);
        let offset=source.find("Before").unwrap();
        let edited=backend.document_edit(&project.id,0,"main.tex",vec![EditorChange{offset,length:6,text:"Local".into()}],None).unwrap();
        fs::write(root.join("project/main.tex"),b"EXTERNAL SOURCE").unwrap();
        let error=backend.save_document(&project.id,edited.revision).unwrap_err();
        assert!(error.contains("Save conflict"),"{error}");
        assert_eq!(fs::read(root.join("project/main.tex")).unwrap(),b"EXTERNAL SOURCE");
        let current=backend.document_snapshot(&project.id).unwrap();
        assert_eq!(current.sources,edited.sources);assert_eq!(current.revision,edited.revision);assert!(current.can_undo);
        assert_eq!(backend.document_history(&project.id,edited.revision,false).unwrap().sources,project.sources);
        drop(backend);fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn stale_save_and_copy_requests_cannot_write_a_newer_revision() {
        let source="\\documentclass{article}\n\\begin{document}Before\\end{document}";
        let (backend,project,root)=saved_project_fixture(source);
        backend.document_edit(&project.id,0,"main.tex",vec![EditorChange{offset:source.find("Before").unwrap(),length:6,text:"Local".into()}],None).unwrap();
        assert!(backend.save_document(&project.id,0).is_err());
        assert!(backend.copy_document(&project.id,0,&root.to_string_lossy()).is_err());
        assert_eq!(fs::read(root.join("project/main.tex")).unwrap(),source.as_bytes());
        let copy=PathBuf::from(backend.copy_document(&project.id,1,&root.to_string_lossy()).unwrap());
        assert!(fs::read_to_string(copy.join("main.tex")).unwrap().contains("Local"));
        assert_eq!(fs::read(root.join("project/main.tex")).unwrap(),source.as_bytes());
        backend.save_document(&project.id,1).unwrap();
        assert_eq!(fs::read(root.join("project/main.tex")).unwrap(),backend.document_snapshot(&project.id).unwrap().sources["main.tex"].as_bytes());
        drop(backend);fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn legacy_support_save_is_checked_and_external_style_changes_are_preserved() {
        let source="\\documentclass{article}\n\\usepackage{studio}\n\\begin{document}\\StudioBlock{sig}{1}{10}{20}{100}{12}{Name}\\end{document}";
        let (backend,project,root)=saved_project_fixture(source);
        backend.save_document(&project.id,0).unwrap();backend.save_document(&project.id,0).unwrap();
        assert_eq!(fs::read_to_string(root.join("project/studio.sty")).unwrap(),STYLE);
        fs::write(root.join("project/studio.sty"),b"EXTERNAL STYLE").unwrap();
        assert!(backend.save_document(&project.id,0).is_err());
        assert_eq!(fs::read(root.join("project/studio.sty")).unwrap(),b"EXTERNAL STYLE");
        assert_eq!(backend.document_snapshot(&project.id).unwrap().revision,0);
        drop(backend);fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn native_recovery_reopens_checked_files_and_discard_never_changes_editor_source() {
        use std::os::windows::fs::OpenOptionsExt;
        let source="\\documentclass{article}\n\\usepackage{studio}\n\\begin{document}\\StudioBlock{sig}{1}{10}{20}{100}{12}{Before}\\end{document}";
        let (backend,project,root)=saved_project_fixture(source);
        let edited=backend.document_edit(&project.id,0,"main.tex",vec![EditorChange{offset:source.find("Before").unwrap(),length:6,text:"Local".into()}],None).unwrap();
        let lock=fs::OpenOptions::new().read(true).share_mode(1).open(root.join("project/main.tex")).unwrap();
        assert!(backend.save_document(&project.id,edited.revision).is_err());drop(lock);
        let drafts=backend.project_recoveries(&project.id).unwrap();assert_eq!(drafts.len(),1);
        assert!(backend.recover_document(&project.id,0,&drafts[0].id,true).is_err());
        assert_eq!(fs::read(root.join("project/main.tex")).unwrap(),source.as_bytes());
        let reopened=backend.recover_document(&project.id,edited.revision,&drafts[0].id,true).unwrap();
        assert!(reopened.recoveries.is_empty());
        assert_eq!(reopened.sources["main.tex"],edited.sources["main.tex"]);
        assert_eq!(fs::read_to_string(root.join("project/studio.sty")).unwrap(),STYLE);
        assert_eq!(backend.document_snapshot(&project.id).unwrap().sources,edited.sources);
        assert!(backend.project_recoveries(&reopened.id).unwrap().is_empty());
        let loaded=backend.document_snapshot(&reopened.id).unwrap();
        let later=backend.document_edit(&reopened.id,loaded.revision,"main.tex",vec![EditorChange{offset:loaded.sources["main.tex"].find("Local").unwrap(),length:5,text:"Later".into()}],None).unwrap();
        let lock=fs::OpenOptions::new().read(true).share_mode(1).open(root.join("project/main.tex")).unwrap();
        assert!(backend.save_document(&reopened.id,later.revision).is_err());drop(lock);
        let draft=backend.project_recoveries(&reopened.id).unwrap().remove(0);
        assert!(backend.discard_document_recovery(&reopened.id,later.revision,&draft.id).unwrap().is_empty());
        assert_eq!(backend.document_snapshot(&reopened.id).unwrap().sources,later.sources);
        assert_eq!(fs::read_to_string(root.join("project/main.tex")).unwrap(),loaded.sources["main.tex"]);
        drop(backend);fs::remove_dir_all(root).unwrap();
    }

}
