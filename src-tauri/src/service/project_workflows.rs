use super::*;
use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};
use texloom_build::{
    project_io::{self, Files},
    snapshot::{Dependency, Fingerprint},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ProjectProfile {
    pub schema: u32,
    pub main: String,
    pub engine: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub main: String,
    pub engine: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub opened: u64,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportedProject {
    pub path: String,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct EncodingIssue {
    pub file: String,
    pub detected: Option<String>,
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct EncodingReview {
    pub root: String,
    pub fingerprint: String,
    pub files: Vec<EncodingIssue>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct EncodingChoice {
    pub file: String,
    pub encoding: String,
    pub sha256: String,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportedImage {
    pub asset: String,
    pub sha256: String,
}

fn bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    texloom_build::snapshot::check_plain(path)?;
    let before = Dependency::capture(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(fail)?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(fail)?;
    if bytes.len() as u64 > max || Fingerprint::bytes(&bytes) != before.fingerprint {
        return Err("A project file changed or exceeds its size limit".into());
    }
    before.verify()?;
    Ok(bytes)
}
fn root(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path).canonicalize().map_err(fail)?;
    let root = if path.is_file() {
        path.parent()
            .ok_or("The source folder is missing")?
            .to_path_buf()
    } else {
        path
    };
    texloom_build::snapshot::check_plain(&root)?;
    if !root.is_dir() {
        return Err("Choose a project file or folder".into());
    }
    Ok(root)
}
fn read_files(root: &Path) -> Result<Files> {
    let mut files = Files::new();
    let mut total = 0usize;
    for file in project_files(root)? {
        let name = file.to_string_lossy().replace('\\', "/");
        let bytes = bounded(
            &root.join(file),
            if text_file(Path::new(&name)) {
                4_000_000
            } else {
                MAX_PROJECT_BYTES
            },
        )?;
        total = total
            .checked_add(bytes.len())
            .ok_or("Project size overflow")?;
        if total > MAX_PROJECT_BYTES as usize {
            return Err("Project exceeds 60 MiB".into());
        }
        files.insert(name, bytes);
    }
    let profile = root.join(project_io::PROFILE);
    if profile.is_file() {
        files.insert(project_io::PROFILE.into(), bounded(&profile, 65536)?);
    }
    Ok(files)
}
fn profile_valid(profile: &ProjectProfile) -> Result<()> {
    relative(&profile.main)?;
    if profile.schema != 1
        || !profile.main.ends_with(".tex")
        || !["auto", "pdflatex", "xelatex", "lualatex"].contains(&profile.engine.as_str())
    {
        return Err("Invalid project main source or engine".into());
    }
    Ok(())
}
pub(super) fn profile(root: &Path) -> Result<Option<ProjectProfile>> {
    let path = root.join(project_io::PROFILE);
    if !path.exists() {
        return Ok(None);
    }
    let profile: ProjectProfile = serde_json::from_slice(&bounded(&path, 65536)?)
        .map_err(|e| format!("Invalid Texloom project metadata: {e}"))?;
    profile_valid(&profile)?;
    Ok(Some(profile))
}

impl Backend {
    pub fn create_project(
        &self,
        template: &str,
        name: &str,
        parent: Option<&str>,
    ) -> Result<ImportedProject> {
        let template =
            texloom_core::templates::get(template).ok_or("Choose a supported project template")?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 80 || name.contains(['/', '\\']) {
            return Err("Use a project name of 1–80 characters without folder separators".into());
        }
        project_io::portable_name(name)?;
        let parent = if let Some(path) = parent.filter(|p| !p.trim().is_empty()) {
            let path = PathBuf::from(path.trim());
            texloom_build::snapshot::check_plain(&path)?;
            path.canonicalize().map_err(fail)?
        } else {
            let path = self.root.join("Projects");
            fs::create_dir_all(&path).map_err(fail)?;
            path
        };
        texloom_build::snapshot::check_plain(&parent)?;
        if !parent.is_dir() {
            return Err("Choose an existing parent folder".into());
        }
        let target = parent.join(format!(
            "{name}-{}",
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let files = Files::from([
            ("main.tex".into(), template.source.as_bytes().to_vec()),
            (
                project_io::PROFILE.into(),
                serde_json::to_vec_pretty(&ProjectProfile {
                    schema: 1,
                    main: "main.tex".into(),
                    engine: template.engine.into(),
                })
                .map_err(fail)?,
            ),
        ]);
        project_io::write_new_tree(&target, &files)?;
        Ok(ImportedProject {
            path: target.to_string_lossy().into_owned(),
        })
    }
    fn read_recents(&self) -> Result<Vec<RecentProject>> {
        let path = self.root.join("recent-projects.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        let entries: Vec<RecentProject> =
            serde_json::from_slice(&bounded(&path, 65536)?).map_err(|_| {
                "Recent-project metadata is unreadable; original projects are unaffected"
            })?;
        if entries.len() > 20 {
            return Err("Recent-project metadata exceeds its limit".into());
        }
        for entry in &entries {
            profile_valid(&ProjectProfile {
                schema: 1,
                main: entry.main.clone(),
                engine: entry.engine.clone(),
            })?;
        }
        Ok(entries)
    }
    pub fn recent_projects(&self) -> Result<Vec<RecentProject>> {
        let _guard = self.recent_lock.lock().map_err(fail)?;
        self.read_recents()
    }
    pub(super) fn remember_recent(&self, root: &Path, main: &str, engine: &str) -> Result<()> {
        let _guard = self.recent_lock.lock().map_err(fail)?;
        fs::create_dir_all(&self.root).map_err(fail)?;
        texloom_build::snapshot::check_plain(&self.root)?;
        let mut entries = self.read_recents()?;
        let path = root.to_string_lossy().to_string();
        entries.retain(|e| e.path != path);
        entries.insert(
            0,
            RecentProject {
                path,
                name: root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                main: main.into(),
                engine: engine.into(),
                opened: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(fail)?
                    .as_secs(),
            },
        );
        entries.truncate(20);
        let target = self.root.join("recent-projects.json");
        let temp = self
            .root
            .join(format!(".recent-{}.tmp", Uuid::new_v4().simple()));
        let bytes = serde_json::to_vec_pretty(&entries).map_err(fail)?;
        let mut f = fs::File::create_new(&temp).map_err(fail)?;
        f.write_all(&bytes).map_err(fail)?;
        f.sync_all().map_err(fail)?;
        drop(f);
        fs::rename(&temp, &target).map_err(fail)
    }
    pub fn open_recent(&self, path: &str) -> Result<Project> {
        let recent = self
            .recent_projects()?
            .into_iter()
            .find(|p| p.path == path)
            .ok_or("That recent project is no longer listed")?;
        self.open_project_with_profile(
            Some(recent.path),
            Some(ProjectProfile {
                schema: 1,
                main: recent.main,
                engine: recent.engine,
            }),
        )
    }
    pub fn import_project_archive(&self, path: &str) -> Result<ImportedProject> {
        let files = project_io::read_archive(Path::new(path))?;
        let base = self.root.join("Imports");
        fs::create_dir_all(&base).map_err(fail)?;
        texloom_build::snapshot::check_plain(&base)?;
        let target = base.join(format!(
            "Project-{}",
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        project_io::write_new_tree(&target, &files)?;
        Ok(ImportedProject {
            path: target.to_string_lossy().into_owned(),
        })
    }
    pub fn export_project_archive(&self, id: &str, revision: u64, path: &str) -> Result<()> {
        let root = self.project_root(id)?;
        let snapshot = self.document_snapshot(id)?;
        if snapshot.revision != revision {
            return Err("The source changed before project export".into());
        }
        let originals = read_files(&root)?;
        let original_hash = project_io::fingerprint(&originals);
        let mut files = originals
            .into_iter()
            .filter(|(name, _)| texloom_build::snapshot::is_asset(Path::new(name)))
            .collect::<Files>();
        files.extend(
            snapshot
                .sources
                .iter()
                .map(|(name, text)| (name.clone(), text.as_bytes().to_vec())),
        );
        for (name, style, command) in [
            ("studio.sty", STYLE, "\\StudioBlock"),
            ("texloom-objects.sty", OBJECT_STYLE, "\\TexloomObject"),
        ] {
            if !files.contains_key(name)
                && snapshot.sources.values().any(|s| {
                    s.contains(command)
                        || s.contains(&format!(
                            "\\usepackage{{{}}}",
                            name.trim_end_matches(".sty")
                        ))
                })
            {
                files.insert(name.into(), style.as_bytes().to_vec());
            }
        }
        files.insert(
            project_io::PROFILE.into(),
            serde_json::to_vec_pretty(&ProjectProfile {
                schema: 1,
                main: snapshot.main,
                engine: snapshot.engine,
            })
            .map_err(fail)?,
        );
        let bytes = project_io::archive_bytes(&files)?;
        let destination = PathBuf::from(path);
        if destination
            .extension()
            .and_then(|s| s.to_str())
            .is_none_or(|s| !s.eq_ignore_ascii_case("zip"))
        {
            return Err("Choose a .zip filename for the project".into());
        }
        let parent = destination
            .parent()
            .ok_or("Choose an export folder")?
            .canonicalize()
            .map_err(fail)?;
        texloom_build::snapshot::check_plain(&parent)?;
        let destination = parent.join(destination.file_name().ok_or("Choose a ZIP filename")?);
        if destination.exists() {
            texloom_build::snapshot::check_plain(&destination)?;
        }
        let before = if destination.exists() {
            Some(Fingerprint::file(&destination)?)
        } else {
            None
        };
        let temp = parent.join(format!(".texloom-export-{}.tmp", Uuid::new_v4().simple()));
        let mut f = fs::File::create_new(&temp).map_err(fail)?;
        f.write_all(&bytes).map_err(fail)?;
        f.sync_all().map_err(fail)?;
        drop(f);
        let publish = (|| {
            if project_io::fingerprint(&read_files(&root)?) != original_hash {
                return Err("Project files changed during export; retry".into());
            }
            let docs = self.documents.lock().map_err(fail)?;
            if docs.get(id).is_none_or(|slot| slot.revision() != revision) {
                return Err("The source changed during project export".into());
            }
            let current = if destination.exists() {
                Some(Fingerprint::file(&destination)?)
            } else {
                None
            };
            if current != before {
                return Err("The destination changed during export".into());
            }
            fs::rename(&temp, &destination).map_err(fail)
        })();
        if temp.exists() {
            let _ = fs::remove_file(&temp);
        }
        publish
    }
    pub fn review_encoding(&self, path: &str) -> Result<EncodingReview> {
        let root = root(path)?;
        let files = read_files(&root)?;
        let issues = files
            .iter()
            .filter(|(name, bytes)| text_file(Path::new(name)) && !project_io::utf8(bytes))
            .map(|(name, bytes)| EncodingIssue {
                file: name.clone(),
                detected: project_io::detected(bytes).map(str::to_string),
                bytes: bytes.len(),
                sha256: Fingerprint::bytes(bytes).sha256,
            })
            .collect();
        Ok(EncodingReview {
            root: root.to_string_lossy().into_owned(),
            fingerprint: project_io::fingerprint(&files),
            files: issues,
        })
    }
    pub fn preview_encoding(&self, path: &str, file: &str, encoding: &str) -> Result<String> {
        let root = root(path)?;
        let files = read_files(&root)?;
        let bytes = files
            .get(file)
            .ok_or("That source file is outside the selected project")?;
        if !text_file(Path::new(file)) {
            return Err("Choose a source file to preview".into());
        };
        project_io::decode(bytes, encoding)
    }
    pub fn convert_encoding(
        &self,
        path: &str,
        parent: &str,
        fingerprint: &str,
        choices: Vec<EncodingChoice>,
    ) -> Result<ImportedProject> {
        let root = root(path)?;
        let mut files = read_files(&root)?;
        if project_io::fingerprint(&files) != fingerprint {
            return Err("The project changed after encoding review".into());
        }
        let mut chosen = std::collections::BTreeSet::new();
        for choice in choices {
            if !chosen.insert(choice.file.clone()) {
                return Err("Duplicate source-encoding choice".into());
            }
            let bytes = files
                .get(&choice.file)
                .ok_or("Encoding choice is outside the project")?;
            if !text_file(Path::new(&choice.file))
                || Fingerprint::bytes(bytes).sha256 != choice.sha256
            {
                return Err("The reviewed source bytes changed".into());
            }
            let mut text = project_io::decode(bytes, &choice.encoding)?;
            if project_io::detected(bytes).is_some() {
                text.insert(0, '\u{feff}')
            }
            files.insert(choice.file, text.into_bytes());
        }
        for (name, bytes) in files.iter_mut() {
            if text_file(Path::new(name)) {
                if !project_io::utf8(bytes) {
                    return Err(format!("Choose the original encoding for {name}"));
                }
                if ["tex", "sty", "cls"].contains(
                    &Path::new(name)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or(""),
                ) {
                    let text = std::str::from_utf8(bytes).map_err(fail)?;
                    *bytes = texloom_core::adapters::utf8_declarations(text)?.into_bytes();
                }
            }
        }
        let parent = PathBuf::from(parent).canonicalize().map_err(fail)?;
        texloom_build::snapshot::check_plain(&parent)?;
        if parent.starts_with(&root) {
            return Err("Choose a destination outside the original project".into());
        }
        let target = parent.join(format!(
            "Project-UTF8-{}",
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        project_io::write_new_tree(&target, &files)?;
        if project_io::fingerprint(&read_files(&root)?) != fingerprint {
            return Err(
                "Original project changed during conversion; review the new copy before opening it"
                    .into(),
            );
        }
        Ok(ImportedProject {
            path: target.to_string_lossy().into_owned(),
        })
    }
    pub fn import_image(&self, id: &str, revision: u64, path: &str) -> Result<ImportedImage> {
        let root = self.project_root(id)?;
        let snapshot = self.document_snapshot(id)?;
        if snapshot.revision != revision {
            return Err("The source changed before image import".into());
        }
        let original = PathBuf::from(path).canonicalize().map_err(fail)?;
        let extension = original
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !["png", "jpg", "jpeg", "pdf"].contains(&extension.as_str()) {
            return Err("Choose a PNG, JPEG or PDF image".into());
        }
        let bytes = bounded(&original, 20 * 1024 * 1024)?;
        let valid = match extension.as_str() {
            "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "pdf" => bytes.starts_with(b"%PDF-"),
            _ => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        };
        if !valid {
            return Err("The image header does not match its extension".into());
        }
        let members = read_files(&root)?;
        let assets = members
            .iter()
            .filter(|(n, _)| texloom_build::snapshot::is_asset(Path::new(n)))
            .collect::<Vec<_>>();
        let total = assets.iter().map(|(_, b)| b.len()).sum::<usize>()
            + snapshot.sources.values().map(String::len).sum::<usize>()
            + bytes.len();
        if total > MAX_PROJECT_BYTES as usize || assets.len() + snapshot.sources.len() >= MAX_FILES
        {
            return Err("Image import would exceed the project limits".into());
        }
        let images = root.join("images");
        if !images.exists() {
            fs::create_dir(&images).map_err(fail)?
        }
        texloom_build::snapshot::check_plain(&images)?;
        let asset = format!("images/image-{}.{}", Uuid::new_v4().simple(), extension);
        let target = root.join(&asset);
        let temp = images.join(format!(".image-{}.tmp", Uuid::new_v4().simple()));
        let result = (|| {
            let mut f = fs::File::create_new(&temp).map_err(fail)?;
            f.write_all(&bytes).map_err(fail)?;
            f.sync_all().map_err(fail)?;
            drop(f);
            let docs = self.documents.lock().map_err(fail)?;
            if docs.get(id).is_none_or(|slot| slot.revision() != revision) {
                return Err("The source changed during image import".into());
            }
            fs::rename(&temp, &target).map_err(fail)?;
            Ok(ImportedImage {
                asset,
                sha256: Fingerprint::bytes(&bytes).sha256,
            })
        })();
        if temp.exists() {
            let _ = fs::remove_file(temp);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn templates_create_new_portable_projects_with_explicit_engine_choices() {
        let f = Fixture::new(b"Original");
        for id in ["article", "letter", "cv", "multilingual"] {
            let made = f
                .backend
                .create_project(id, "Example project", None)
                .unwrap();
            let project = f.backend.open_project(Some(made.path)).unwrap();
            assert_eq!(
                project.sources["main.tex"],
                texloom_core::templates::get(id).unwrap().source
            );
            assert_eq!(
                f.backend.document_snapshot(&project.id).unwrap().engine,
                texloom_core::templates::get(id).unwrap().engine
            );
        }
        for name in ["../outside", "CON", "a/b", ""] {
            assert!(f.backend.create_project("article", name, None).is_err());
        }
        assert!(f
            .backend
            .create_project("unknown", "Example", None)
            .is_err());
        assert_eq!(fs::read(f.project.join("main.tex")).unwrap(), b"Original");
    }
    #[test]
    fn explicit_save_and_copy_preserve_the_selected_main_and_engine() {
        let f = Fixture::new(b"Original\r\n");
        fs::write(f.project.join("other.tex"), b"Other").unwrap();
        let project = f.backend.open_project(Some(f.path())).unwrap();
        let changed = f
            .backend
            .document_profile(&project.id, 0, "other.tex".into(), "lualatex".into())
            .unwrap();
        f.backend
            .save_document(&project.id, changed.revision)
            .unwrap();
        assert_eq!(
            fs::read(f.project.join("main.tex")).unwrap(),
            b"Original\r\n"
        );
        let opened = f
            .backend
            .open_project(Some(f.project.to_string_lossy().into_owned()))
            .unwrap();
        assert_eq!(opened.main, "other.tex");
        assert_eq!(
            f.backend.document_snapshot(&opened.id).unwrap().engine,
            "lualatex"
        );
        let copied = f
            .backend
            .copy_document(&project.id, changed.revision, &f.root.to_string_lossy())
            .unwrap();
        let copy = f.backend.open_project(Some(copied)).unwrap();
        assert_eq!(copy.main, "other.tex");
        assert_eq!(
            f.backend.document_snapshot(&copy.id).unwrap().engine,
            "lualatex"
        );
        assert!(f
            .backend
            .document_snapshot(&project.id)
            .unwrap()
            .sources
            .keys()
            .all(|name| name != project_io::PROFILE));
    }
    #[test]
    fn save_profile_checks_external_metadata_before_writing_any_source() {
        let f = Fixture::new(b"Original\r\n");
        let project = f.backend.open_project(Some(f.path())).unwrap();
        let external = b"{\"schema\":1,\"main\":\"main.tex\",\"engine\":\"xelatex\"}";
        fs::write(f.project.join(project_io::PROFILE), external).unwrap();
        let changed = f
            .backend
            .document_edit_delta(
                &project.id,
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 0,
                    length: 8,
                    text: "Unsaved".into(),
                }],
                None,
            )
            .unwrap();
        assert!(f
            .backend
            .save_document(&project.id, changed.revision)
            .err()
            .unwrap()
            .contains("Save conflict"));
        assert_eq!(
            fs::read(f.project.join("main.tex")).unwrap(),
            b"Original\r\n"
        );
        assert_eq!(
            fs::read(f.project.join(project_io::PROFILE)).unwrap(),
            external
        );
    }
    struct Fixture {
        root: PathBuf,
        project: PathBuf,
        backend: Backend,
    }
    impl Fixture {
        fn new(bytes: &[u8]) -> Self {
            let root = std::env::temp_dir()
                .join(format!("texloom-project-tools-{}", Uuid::new_v4().simple()));
            let project = root.join("original");
            fs::create_dir_all(&project).unwrap();
            fs::write(project.join("main.tex"), bytes).unwrap();
            let backend = Backend::new(root.join("app"));
            Self {
                root,
                project,
                backend,
            }
        }
        fn path(&self) -> String {
            self.project.join("main.tex").to_string_lossy().into_owned()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn project_zip_roundtrip_binds_unsaved_bytes_assets_main_engine_and_originals() {
        let original = b"\xef\xbb\xbfOriginal\r\n";
        let f = Fixture::new(original);
        fs::write(f.project.join("second.tex"), b"Second\r\n").unwrap();
        fs::write(f.project.join("image.png"), [0, 1, 255]).unwrap();
        let project = f.backend.open_project(Some(f.path())).unwrap();
        f.backend
            .document_edit_delta(
                &project.id,
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 0,
                    length: 8,
                    text: "Unsaved".into(),
                }],
                None,
            )
            .unwrap();
        let profile = f
            .backend
            .document_profile(&project.id, 1, "second.tex".into(), "xelatex".into())
            .unwrap();
        let zip = f.root.join("project.zip");
        f.backend
            .export_project_archive(&project.id, profile.revision, &zip.to_string_lossy())
            .unwrap();
        assert_eq!(fs::read(f.project.join("main.tex")).unwrap(), original);
        let imported = f
            .backend
            .import_project_archive(&zip.to_string_lossy())
            .unwrap();
        let reopened = f.backend.open_project(Some(imported.path.clone())).unwrap();
        let snapshot = f.backend.document_snapshot(&reopened.id).unwrap();
        assert_eq!(reopened.main, "second.tex");
        assert_eq!(snapshot.engine, "xelatex");
        assert_eq!(snapshot.sources["main.tex"], "\u{feff}Unsaved\r\n");
        assert_eq!(
            fs::read(Path::new(&imported.path).join("image.png")).unwrap(),
            [0, 1, 255]
        );
        assert!(f
            .backend
            .export_project_archive(&project.id, 0, &zip.to_string_lossy())
            .is_err());
        let recent = f.backend.recent_projects().unwrap();
        let canonical = Path::new(&imported.path)
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(recent.iter().any(|r| r.path == canonical));
        let recent_open = f.backend.open_recent(&canonical).unwrap();
        assert_eq!(
            f.backend.document_snapshot(&recent_open.id).unwrap().engine,
            "xelatex"
        );
    }
    #[test]
    fn encoding_review_conversion_preserves_originals_and_rejects_stale_or_wrong_choices() {
        let bytes=b"\\documentclass{article}\r\n\\usepackage[latin1]{inputenc}\r\n\\begin{document}caf\xe9\\end{document}\r\n";
        let f = Fixture::new(bytes);
        assert!(f.backend.open_project(Some(f.path())).is_err());
        let review = f.backend.review_encoding(&f.path()).unwrap();
        assert_eq!(review.files.len(), 1);
        assert!(f
            .backend
            .preview_encoding(&f.path(), "../secret.tex", "windows-1252")
            .is_err());
        assert!(f
            .backend
            .preview_encoding(&f.path(), "main.tex", "utf-8")
            .is_err());
        assert!(f
            .backend
            .preview_encoding(&f.path(), "main.tex", "windows-1252")
            .unwrap()
            .contains("café"));
        let choices = vec![EncodingChoice {
            file: "main.tex".into(),
            encoding: "windows-1252".into(),
            sha256: review.files[0].sha256.clone(),
        }];
        assert!(f
            .backend
            .convert_encoding(&f.path(), &f.path(), &review.fingerprint, choices.clone())
            .is_err());
        let converted = f
            .backend
            .convert_encoding(
                &f.path(),
                &f.root.to_string_lossy(),
                &review.fingerprint,
                choices.clone(),
            )
            .unwrap();
        assert_eq!(fs::read(f.project.join("main.tex")).unwrap(), bytes);
        let copy = f.backend.open_project(Some(converted.path)).unwrap();
        assert!(copy.sources["main.tex"].contains("[utf8]"));
        assert!(copy.sources["main.tex"].contains("café"));
        assert!(copy.sources["main.tex"].ends_with("\r\n"));
        fs::write(f.project.join("main.tex"), b"External change").unwrap();
        assert!(f
            .backend
            .convert_encoding(
                &f.path(),
                &f.root.to_string_lossy(),
                &review.fingerprint,
                choices
            )
            .is_err());
    }
    #[test]
    fn image_import_is_revision_bound_preserves_source_and_refuses_forged_file_types() {
        let f = Fixture::new(b"Source\r\n");
        let project = f.backend.open_project(Some(f.path())).unwrap();
        let image = f.root.join("image.png");
        fs::write(&image, b"not an image").unwrap();
        assert!(f
            .backend
            .import_image(&project.id, 0, &image.to_string_lossy())
            .is_err());
        let png = b"\x89PNG\r\n\x1a\nFixture";
        fs::write(&image, png).unwrap();
        assert!(f
            .backend
            .import_image(&project.id, 1, &image.to_string_lossy())
            .is_err());
        let imported = f
            .backend
            .import_image(&project.id, 0, &image.to_string_lossy())
            .unwrap();
        assert_eq!(fs::read(f.project.join(imported.asset)).unwrap(), png);
        assert_eq!(fs::read(f.project.join("main.tex")).unwrap(), b"Source\r\n");
        assert_eq!(
            f.backend.document_snapshot(&project.id).unwrap().revision,
            0
        );
    }
    #[test]
    fn corrupt_recent_metadata_never_blocks_opening_or_overwrites_its_evidence() {
        let f = Fixture::new(b"Source");
        fs::create_dir_all(f.root.join("app")).unwrap();
        let path = f.root.join("app/recent-projects.json");
        fs::write(&path, b"broken metadata").unwrap();
        assert!(f.backend.open_project(Some(f.path())).is_ok());
        assert!(f.backend.recent_projects().is_err());
        assert_eq!(fs::read(path).unwrap(), b"broken metadata");
    }
}
