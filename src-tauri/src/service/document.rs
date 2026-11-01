//! Native document authority. IPC snapshots are replicas; only this service
//! applies edits or history. Compiler jobs consume immutable session snapshots.
use super::*;
use texloom_core::document::{
    self as core, projection::EditorProjection, FileChange, Origin, Patch, Profile, ProfileChange,
    Replacement, Revision, Selection, TypingGroup,
};

pub(super) struct DocumentSlot {
    session: core::Session,
    pending: HashMap<String, Pending>,
}
struct Pending {
    candidate: core::Prepared,
    build_id: Option<String>,
}
impl DocumentSlot {
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
        slot.session.set_selection(selection).map_err(fail)?;
        Ok(SelectionReply {
            revision: slot.session.revision().0,
            selection: slot.editor_selection(),
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
        slot.session
            .commit(candidate, Origin::Source, group.map(TypingGroup))
            .map_err(fail)?;
        slot.pending.clear();
        Ok(slot.snapshot())
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
        slot.session
            .commit(candidate, Origin::Settings, None)
            .map_err(fail)?;
        slot.pending.clear();
        Ok(slot.snapshot())
    }
    pub fn document_history(&self, id: &str, base: u64, redo: bool) -> Result<DocumentSnapshot> {
        let mut docs = self.documents.lock().map_err(fail)?;
        let slot = docs.get_mut(id).ok_or("Reopen this project")?;
        if slot.session.revision() != Revision(base) {
            return Err("The source revision changed".into());
        }
        if redo {
            slot.session.redo().map_err(fail)?;
        } else {
            slot.session.undo().map_err(fail)?;
        }
        slot.pending.clear();
        Ok(slot.snapshot())
    }
    pub fn document_prepare(
        &self,
        id: &str,
        base: u64,
        sources: BTreeMap<String, String>,
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
                build_id: None,
            },
        );
        Ok(PreparedReply { token, revision })
    }
    pub fn compile_document(
        &self,
        id: &str,
        revision: u64,
        token: Option<&str>,
    ) -> Result<CompileResult> {
        let request = {
            let docs = self.documents.lock().map_err(fail)?;
            let slot = docs.get(id).ok_or("Reopen this project")?;
            if slot.session.revision() != Revision(revision) {
                return Err("The source changed before compilation started".into());
            }
            if let Some(token) = token {
                let pending = slot
                    .pending
                    .get(token)
                    .ok_or("The visual candidate is no longer current")?;
                CompileRequest {
                    project_id: id.into(),
                    main: pending.candidate.profile().main.clone(),
                    sources: pending.candidate.sources().clone(),
                    engine: pending.candidate.profile().engine.clone(),
                    revision: pending.candidate.base().0 + u64::from(pending.candidate.changed()),
                }
            } else {
                CompileRequest {
                    project_id: id.into(),
                    main: slot.session.profile().main.clone(),
                    sources: slot.session.sources().clone(),
                    engine: slot.session.profile().engine.clone(),
                    revision: slot.session.revision().0,
                }
            }
        };
        let result = self.compile(request)?;
        if let Some(token) = token {
            let mut docs = self.documents.lock().map_err(fail)?;
            let slot = docs.get_mut(id).ok_or("Reopen this project")?;
            let pending = slot
                .pending
                .get_mut(token)
                .ok_or("The source changed while compiling the visual candidate")?;
            if result.ok {
                pending.build_id = Some(result.build_id.clone());
            }
        }
        Ok(result)
    }
    /// The frontend calls this only after PDF preparation and layout validation.
    /// Runtime independent PDF verification remains a separate P6 gate.
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
        if pending.build_id.as_deref() != Some(build_id) {
            return Err(
                "Candidate must finish its own successful native compilation before commit".into(),
            );
        }
        let pending = slot
            .pending
            .remove(token)
            .ok_or("The visual candidate is missing")?;
        slot.session
            .commit(pending.candidate, Origin::Canvas, None)
            .map_err(fail)?;
        Ok(slot.snapshot())
    }
    pub fn document_discard(&self, id: &str, token: &str) -> Result<()> {
        if let Some(slot) = self.documents.lock().map_err(fail)?.get_mut(id) {
            slot.pending.remove(token);
        }
        Ok(())
    }
    pub fn save_document(&self, id: &str, base: u64) -> Result<()> {
        let snapshot = self.document_snapshot(id)?;
        if snapshot.revision != base {
            return Err("The source changed before Save; try again".into());
        }
        self.save_project(id, &snapshot.sources)
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
}
