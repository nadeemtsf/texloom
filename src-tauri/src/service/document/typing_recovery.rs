use super::*;
use texloom_build::{
    persistence::DiskSnapshot,
    recovery::{Inventory, Journal, Mutation, Store},
};

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct RecoveryStatus {
    pub draft_id: Option<String>,
    #[cfg_attr(feature = "typegen", ts(type = "number | null"))]
    pub protected_revision: Option<u64>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct RecoveryReply {
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub recovery: RecoveryStatus,
}
#[derive(Default, Debug, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CloseReply {
    pub warning: Option<String>,
}
#[derive(Default)]
pub(super) struct Protection {
    journal: Option<Journal>,
    error: Option<String>,
    attempted: bool,
}
impl Protection {
    pub(super) fn status(&self) -> RecoveryStatus {
        RecoveryStatus {
            draft_id: self.journal.as_ref().map(|j| j.id().to_string()),
            protected_revision: self.journal.as_ref().and_then(Journal::protected_revision),
            error: self.error.clone(),
        }
    }
}
impl DocumentSlot {
    pub(in crate::service) fn start_typing_recovery(&mut self, root: &Path, disk: &DiskSnapshot) {
        self.protection.attempted = true;
        match Store::open(&root.join(".source-recovery"))
            .and_then(|store| store.begin(disk, &self.session))
        {
            Ok(journal) => {
                self.protection.journal = Some(journal);
                self.protection.error = None;
            }
            Err(error) => {
                self.protection.error = Some(error);
            }
        }
    }
}
impl Backend {
    /// Durably accept a proposed source draft before publishing canonical RAM.
    /// Failure may follow partial OS writes, so the caller caches an uncertain
    /// outcome and must not claim this was a clean transactional refusal.
    pub(super) fn protect_source_intent(&self, id: &str, slot: &mut DocumentSlot,
        base: u64, patch: Patch, proposed: &core::Session) -> Result<()> {
        let result = (|| {
            let snapshots = self.disk_snapshots.lock().map_err(fail)?;
            let disk = snapshots.get(id).ok_or("The saved project baseline is missing")?;
            let store = Store::open(&self.root.join(".source-recovery"))?;
            if slot.protection.journal.is_none() {
                slot.protection.attempted = true;
                slot.protection.journal = Some(store.begin(disk, &slot.session)?);
            }
            slot.protection.journal.as_mut().ok_or("Recovery protection is unavailable")?.record(
                &store, base, Mutation::Patch { patch, origin: Origin::Source, group: None }, proposed, disk)
        })();
        slot.protection.error = result.as_ref().err().cloned();
        result
    }
    pub(super) fn record_typing_recovery(
        &self,
        id: &str,
        slot: &mut DocumentSlot,
        base: u64,
        mutation: Mutation,
    ) {
        let result = (|| {
            let snapshots = self.disk_snapshots.lock().map_err(fail)?;
            let Some(disk) = snapshots.get(id) else {
                // Direct in-memory unit-test sessions have no disk project.
                return Ok(());
            };
            if let Some(journal) = slot.protection.journal.as_mut() {
                let store = Store::open(&self.root.join(".source-recovery"))?;
                journal.record(&store, base, mutation, &slot.session, disk)
            } else if !slot.protection.attempted {
                slot.start_typing_recovery(&self.root, disk);
                Ok(())
            } else {
                return Ok(());
            }
        })();
        match result {
            Ok(()) if slot.protection.journal.is_some() => slot.protection.error = None,
            Ok(()) => {}
            Err(error) => slot.protection.error = Some(error),
        }
    }
    pub(super) fn checkpoint_typing_recovery(&self, id: &str, slot: &mut DocumentSlot) {
        let result = (|| {
            let snapshots = self.disk_snapshots.lock().map_err(fail)?;
            let disk = snapshots
                .get(id)
                .ok_or("The saved project baseline is missing")?;
            if let Some(journal) = slot.protection.journal.as_mut() {
                let store = Store::open(&self.root.join(".source-recovery"))?;
                journal.checkpoint(&store, &slot.session, disk)
            } else {
                slot.start_typing_recovery(&self.root, disk);
                Ok(())
            }
        })();
        match result {
            Ok(()) if slot.protection.journal.is_some() => slot.protection.error = None,
            Ok(()) => {}
            Err(error) => slot.protection.error = Some(error),
        }
    }
    pub fn retry_typing_recovery(&self, id: &str, base: u64) -> Result<RecoveryReply> {
        let mut documents = self.documents.lock().map_err(fail)?;
        let slot = documents.get_mut(id).ok_or("Reopen this project")?;
        slot.require_resolved_source_intents()?;
        if slot.session.revision() != Revision(base) {
            return Err("The source changed before retrying recovery protection".into());
        }
        self.checkpoint_typing_recovery(id, slot);
        Ok(RecoveryReply {
            revision: base,
            recovery: slot.protection.status(),
        })
    }
    pub fn typing_drafts(&self) -> Result<Inventory> {
        Store::open(&self.root.join(".source-recovery"))?.inventory()
    }
    pub fn open_typing_draft(&self, draft_id: &str) -> Result<Project> {
        let store = Store::open(&self.root.join(".source-recovery"))?;
        let inventory = store.inventory()?;
        let draft = inventory
            .drafts
            .iter()
            .find(|d| d.id == draft_id)
            .ok_or("This draft is unavailable or open in another Texloom instance")?;
        let root = PathBuf::from(&draft.project);
        let source_only = !root.exists();
        if !source_only && (root.canonicalize().map_err(fail)? != root || !root.is_dir()) {
            return Err(
                "The recovery project folder changed or is missing. The draft is retained.".into(),
            );
        }
        let recovered = store.recover(draft_id, &root)?;
        let id = Uuid::new_v4().simple().to_string();
        let main = recovered.session.profile().main.clone();
        let sources = recovered.session.sources().clone();
        let mut slot = DocumentSlot {
            session: recovered.session,
            pending: HashMap::new(),
            protection: Default::default(),
            source_intents: Default::default(),
        };
        slot.start_typing_recovery(&self.root, &recovered.disk);
        let (recoveries, recovery_error) = match if source_only {
            Ok(Vec::new())
        } else {
            texloom_build::persistence::pending(&root)
        } {
            Ok(recoveries) => (recoveries, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        self.documents
            .lock()
            .map_err(fail)?
            .insert(id.clone(), slot);
        self.projects
            .lock()
            .map_err(fail)?
            .insert(id.clone(), root.clone());
        self.disk_snapshots
            .lock()
            .map_err(fail)?
            .insert(id.clone(), recovered.disk);
        Ok(Project {
            id,
            name: root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            main,
            sources,
            recoveries,
            recovery_error,
            restored: true,
            draft_warning: [recovered.warning, source_only.then(||"The original project folder is missing. Source and history were recovered; use Save a copy to write source files to a new folder. Images and other missing assets cannot be recovered from the typing journal.".into())]
                .into_iter().flatten().reduce(|a,b|a+" "+&b),
            source_only,
        })
    }
    pub fn discard_typing_draft(&self, draft_id: &str) -> Result<Inventory> {
        let store = Store::open(&self.root.join(".source-recovery"))?;
        let inventory = store.inventory()?;
        let draft = inventory
            .drafts
            .iter()
            .find(|d| d.id == draft_id)
            .ok_or("This draft is unavailable or open in another Texloom instance")?;
        store.discard(draft_id, Path::new(&draft.project))?;
        store.inventory()
    }
    /// Only an explicit close/discard decision may remove a live typing draft.
    /// Unexpected termination simply drops the lock and leaves the draft.
    pub fn close_document_checked(&self, id: &str, base: u64, discard: bool) -> Result<CloseReply> {
        let mut documents = self.documents.lock().map_err(fail)?;
        let Some(slot) = documents.get(id) else {
            return Ok(CloseReply::default());
        };
        if slot.session.revision() != Revision(base) {
            return Err("The source changed before closing; it was retained".into());
        }
        if discard { slot.require_resolved_source_intents()?; }
        let root = self.project_root(id)?;
        let journal = documents
            .remove(id)
            .and_then(|slot| slot.protection.journal);
        self.scheduler.cancel_project(id);
        self.projects.lock().map_err(fail)?.remove(id);
        self.disk_snapshots.lock().map_err(fail)?.remove(id);
        self.displayed.lock().map_err(fail)?.remove(id);
        let draft_id = journal.as_ref().map(|j| j.id().to_string());
        drop(journal);
        drop(documents);
        self.prune_builds("");
        let warning = if discard {
            draft_id.and_then(|draft_id| {
                Store::open(&self.root.join(".source-recovery"))
                    .and_then(|store| store.discard(&draft_id, &root))
                    .err()
            })
        } else {
            None
        };
        Ok(CloseReply {
            warning: warning.map(|error| {
                format!("The project closed, but its recovery draft was retained: {error}")
            }),
        })
    }
}

#[cfg(test)]
mod tests;
