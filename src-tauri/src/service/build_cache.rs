use super::*;
const OWNER: &[u8] = b"Texloom P3 cache owner\n";
pub(super) struct SessionCache {
    pub root: PathBuf,
    _owner: fs::File,
}
fn session_name(name: &str) -> bool {
    name.strip_prefix("p3-")
        .is_some_and(|n| n.len() == 32 && n.bytes().all(|b| b.is_ascii_hexdigit()))
}
impl SessionCache {
    pub fn new(base: &Path) -> Result<Self> {
        fs::create_dir_all(base).map_err(fail)?;
        let base = base.canonicalize().map_err(fail)?;
        // Recognized, unlocked P3 sessions belong to exited processes. Other
        // instances and historical/unrecognized directories are untouched.
        for e in fs::read_dir(&base).map_err(fail)?.flatten() {
            let path = e.path();
            if !session_name(&e.file_name().to_string_lossy()) {
                continue;
            }
            if !e.file_type().map_err(fail)?.is_dir()
                || path.canonicalize().map_err(fail)?.parent() != Some(base.as_path())
            {
                continue;
            }
            let Ok(mut owner) = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path.join("owner.lock"))
            else {
                continue;
            };
            if owner.try_lock().is_err() {
                continue;
            }
            let mut marker = vec![0; OWNER.len()];
            if owner.metadata().map_err(fail)?.len() != OWNER.len() as u64
                || std::io::Read::read_exact(&mut owner, &mut marker).is_err()
                || marker != OWNER
            {
                continue;
            }
            if inspect(&base, &path).is_err() {
                continue;
            }
            drop(owner);
            let _ = fs::remove_dir_all(path);
        }
        let root = base.join(format!("p3-{}", Uuid::new_v4().simple()));
        fs::create_dir(&root).map_err(fail)?;
        let mut owner = fs::File::create_new(root.join("owner.lock")).map_err(fail)?;
        owner.try_lock().map_err(fail)?;
        std::io::Write::write_all(&mut owner, OWNER).map_err(fail)?;
        Ok(Self {
            root,
            _owner: owner,
        })
    }
}
/// Only private, generated, direct children of the build root can be retired.
pub(super) struct Workspace {
    root: PathBuf,
    pub directory: PathBuf,
    pub keep: bool,
}
impl Workspace {
    pub fn new(root: &Path, directory: &Path) -> Result<Self> {
        fs::create_dir_all(root).map_err(fail)?;
        let root = root.canonicalize().map_err(fail)?;
        if directory
            .parent()
            .ok_or("Missing private build parent")?
            .canonicalize()
            .map_err(fail)?
            != root
        {
            return Err("Build folder must be inside its root".into());
        }
        let name = directory
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Missing build name")?;
        if name.len() != 32 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Invalid private build name".into());
        }
        let directory = root.join(name);
        if directory.exists() {
            return Err("Private build already exists".into());
        }
        Ok(Self {
            root,
            directory,
            keep: false,
        })
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        if !self.keep && self.directory.exists() {
            let _ = remove(&self.root, &self.directory);
        }
    }
}
fn inspect(root: &Path, directory: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(directory).map_err(fail)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err("Reparse point in build cache".into());
        }
    }
    if metadata.file_type().is_symlink()
        || !directory.canonicalize().map_err(fail)?.starts_with(root)
    {
        return Err("Build cache path escaped its root".into());
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut bytes = 0u64;
    for entry in fs::read_dir(directory).map_err(fail)? {
        bytes = bytes.saturating_add(inspect(root, &entry.map_err(fail)?.path())?);
    }
    Ok(bytes)
}
pub(super) fn directory_bytes(directory: &Path) -> Result<u64> {
    inspect(&directory.canonicalize().map_err(fail)?, directory)
}
fn remove(root: &Path, directory: &Path) -> Result<()> {
    let root = root.canonicalize().map_err(fail)?;
    let resolved = directory.canonicalize().map_err(fail)?;
    if resolved.parent() != Some(root.as_path()) {
        return Err("Only direct private build folders can be removed".into());
    }
    let name = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid cache entry")?;
    if name.len() != 32 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Unrecognized cache entry".into());
    }
    inspect(&root, &resolved)?;
    fs::remove_dir_all(resolved).map_err(fail)
}
impl Backend {
    pub(super) fn build_root(&self) -> Result<PathBuf> {
        self.cache
            .get_or_init(|| SessionCache::new(&self.root.join("builds")))
            .as_ref()
            .map(|c| c.root.clone())
            .map_err(Clone::clone)
    }
    pub fn pin_displayed_build(&self, project: &str, id: &str) -> Result<()> {
        let builds = self.builds.lock().map_err(fail)?;
        let build = builds.get(id).ok_or("Build expired")?;
        if build.project_id != project {
            return Err("Build belongs to a different project".into());
        }
        let mut displayed = self.displayed.lock().map_err(fail)?;
        if displayed
            .get(project)
            .and_then(|old| builds.get(old))
            .is_none_or(|old| old.sequence <= build.sequence)
        {
            displayed.insert(project.into(), id.into());
        }
        Ok(())
    }
    pub fn close_document(&self, project: &str) -> Result<()> {
        self.scheduler.cancel_project(project);
        self.documents.lock().map_err(fail)?.remove(project);
        self.projects.lock().map_err(fail)?.remove(project);
        self.displayed.lock().map_err(fail)?.remove(project);
        self.prune_builds("");
        Ok(())
    }
    pub(super) fn prune_builds(&self, protected: &str) {
        let Some(Ok(cache)) = self.cache.get() else {
            return;
        };
        // Never evict the displayed PDF, the just-returned result, or an artifact
        // borrowed by an in-flight SyncTeX/export operation.
        let mut retired = Vec::new();
        if let Ok(mut builds) = self.builds.lock() {
            let pins = match self.displayed.lock() {
                Ok(p) => p
                    .values()
                    .cloned()
                    .collect::<std::collections::HashSet<_>>(),
                Err(_) => return,
            };
            let mut candidates: Vec<_> = builds
                .iter()
                .filter(|(id, b)| {
                    id.as_str() != protected && !pins.contains(*id) && Arc::strong_count(b) == 1
                })
                .map(|(id, b)| (b.sequence, id.clone()))
                .collect();
            candidates.sort();
            let mut bytes = builds.values().map(|b| b.bytes).sum::<u64>();
            for (_, id) in candidates {
                if builds.len() <= 12 && bytes <= 512 * 1024 * 1024 {
                    break;
                }
                if let Some(build) = builds.remove(&id) {
                    bytes = bytes.saturating_sub(build.bytes);
                    retired.push(build.directory.clone());
                }
            }
        }
        for directory in retired {
            let _ = remove(&cache.root, &directory);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exited_cache_sessions_are_reaped_without_touching_live_instances_or_unknown_folders() {
        let base = std::env::temp_dir().join(format!("texloom-cache-lock-test-{}", Uuid::new_v4()));
        let a = SessionCache::new(&base).unwrap();
        let a_path = a.root.clone();
        fs::create_dir_all(base.join("keep-user-folder")).unwrap();
        fs::write(base.join("keep-user-folder/file"), "KEEP").unwrap();
        let b = SessionCache::new(&base).unwrap();
        assert!(a_path.exists());
        drop(a);
        let c = SessionCache::new(&base).unwrap();
        assert!(!a_path.exists());
        assert!(b.root.exists());
        assert!(base.join("keep-user-folder/file").is_file());
        drop(b);
        drop(c);
        fs::remove_dir_all(base).unwrap();
    }
    fn export_fixture() -> (Backend, PathBuf, String) {
        let temp = std::env::temp_dir().join(format!("texloom-export-test-{}", Uuid::new_v4()));
        let project_root = temp.join("project");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(project_root.join("main.tex"), "\\documentclass{article}\nA").unwrap();
        let b = Backend::new(temp.join("app"));
        let project = b
            .open_project(Some(project_root.join("main.tex").to_string_lossy().into()))
            .unwrap();
        let directory = b.root.join("builds").join("b".repeat(32));
        let mut receipt = texloom_build::snapshot::Receipt::capture(
            &project_root,
            &directory,
            "main.tex",
            &project.sources,
            &BTreeMap::new(),
            &std::env::current_exe().unwrap(),
        )
        .unwrap();
        fs::write(
            receipt.directory.join(".out/preview.pdf"),
            b"%PDF-export-fixture",
        )
        .unwrap();
        receipt.seal_outputs().unwrap();
        let lease = b
            .scheduler
            .request_sequence(&project.id, 1)
            .unwrap()
            .acquire()
            .unwrap();
        b.builds.lock().unwrap().insert(
            "export".into(),
            Arc::new(Build {
                directory: receipt.directory.clone(),
                compiler: receipt.compiler.path.clone(),
                project_id: project.id.clone(),
                main: project.main,
                revision: 0,
                receipt,
                eligible: true,
                purpose: "export".into(),
                cancel: lease.cancel.clone(),
                sequence: lease.id,
                bytes: 1,
            }),
        );
        drop(lease);
        (b, temp, project.id)
    }
    #[test]
    fn clean_export_atomically_replaces_output_without_saving_source() {
        let (b, temp, id) = export_fixture();
        let target = temp.join("result.pdf");
        fs::write(&target, b"OLD").unwrap();
        b.export_document_build(&id, 0, "export", &target.to_string_lossy())
            .unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"%PDF-export-fixture");
        assert_eq!(
            fs::read(temp.join("project/main.tex")).unwrap(),
            b"\\documentclass{article}\nA"
        );
        assert!(!fs::read_dir(&temp).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".texloom-export")));
        fs::remove_dir_all(temp).unwrap();
    }
    #[test]
    fn cancelled_or_stale_export_cannot_overwrite_an_existing_pdf() {
        for cancelled in [false, true] {
            let (b, temp, id) = export_fixture();
            let target = temp.join("result.pdf");
            fs::write(&target, b"OLD").unwrap();
            if cancelled {
                b.cancel_compilation(&id, 1, None);
            } else {
                b.document_edit(
                    &id,
                    0,
                    "main.tex",
                    vec![EditorChange {
                        offset: 0,
                        length: 0,
                        text: "% edit\n".into(),
                    }],
                    None,
                )
                .unwrap();
            }
            assert!(b
                .export_document_build(&id, 0, "export", &target.to_string_lossy())
                .is_err());
            assert_eq!(fs::read(&target).unwrap(), b"OLD");
            fs::remove_dir_all(temp).unwrap();
        }
    }
    #[test]
    fn failed_workspace_cleanup_never_deletes_outside_the_build_root() {
        let temp = std::env::temp_dir().join(format!("texloom-cache-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let root = temp.join("builds");
        let name = "a".repeat(32);
        let directory = root.join(&name);
        {
            let w = Workspace::new(&root, &directory).unwrap();
            fs::create_dir_all(&w.directory).unwrap();
            fs::write(w.directory.join("private.tex"), "A").unwrap();
        }
        assert!(!directory.exists());
        assert!(remove(&root, &temp).is_err());
        assert!(temp.exists());
        fs::remove_dir_all(temp).unwrap();
    }
}
