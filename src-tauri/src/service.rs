use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use texloom_core::document::projection::EditorProjection;
use uuid::Uuid;
mod build_cache;
mod document;
mod synctex;
mod layout_gate;
mod visual;
mod adapters;
mod project_workflows;
pub use project_workflows::{ProjectProfile,RecentProject,ImportedProject,EncodingIssue,EncodingReview,EncodingChoice,ImportedImage};
pub use adapters::{AdapterControl, AdapterParameter, AdapterPlan, AdapterRequest};
pub use adapters::{describe as describe_adapters, plan as plan_adapter};
pub use layout_gate::VisualOperation;
pub use document::{
    CloseReply, DocumentSnapshot, EditorChange, EditorRange, PreparedReply, RecoveryReply, SelectionReply, SourceEditReply,
    DraftParagraphPosition, DraftSourceIntent, SourceDraftOutcome, SourceDraftRefusalReason, SourceDraftReply, SourceTarget, SourceTargetsReply,
};
#[cfg(feature = "typegen")]
pub use document::{EditorSelection, RecoveryStatus};
#[cfg(feature = "benchmark")]
pub use document::{SourceDraftCostReply, SourceDraftCostState};
pub use visual::plan_object_edit as visual_plan;
pub use visual::{
    describe_text_edit, preview_edit_parameters, LiteralEditRequest, SpacingEditRequest,
    TextEditModel,
    LiteralRunView, SpacingView,
};
pub use visual::{ObjectEditRequest, ObjectPlan, ResolvedSelection, SelectionPoint};

const EXAMPLE: &str = include_str!("../../resources/main.tex");
const OBJECT_STYLE: &str = include_str!("../../resources/texloom-objects.sty");
const LEGACY_OBJECT_STYLE: &str = include_str!("../../resources/legacy/texloom-objects-v03.sty");
const MAX_FILES: usize = 600;
const MAX_PROJECT_BYTES: u64 = 60 * 1024 * 1024;

type Result<T> = std::result::Result<T, String>;
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn text_file(path: &Path) -> bool {
    texloom_build::snapshot::is_source(path)
}
fn allowed_file(path: &Path) -> bool {
    text_file(path)
        || matches!(
            path.extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str(),
            "png" | "jpg" | "jpeg" | "pdf" | "svg" | "eps" | "otf" | "ttf" | "csv"
        )
}
fn relative(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.chars().any(|c| matches!(c, '\\' | ':' | '\0')) {
        return Err("Invalid project-relative filename".into());
    }
    let path = PathBuf::from(name);
    if path.is_absolute()
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err("Files must remain inside the project".into());
    }
    Ok(path)
}

fn project_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(
        root: &Path,
        dir: &Path,
        out: &mut Vec<PathBuf>,
        total: &mut u64,
        depth: usize,
    ) -> Result<()> {
        if depth > 20 {
            return Err("Project folders are nested too deeply".into());
        }
        for entry in fs::read_dir(dir).map_err(fail)? {
            let entry = entry.map_err(fail)?;
            let kind = entry.file_type().map_err(fail)?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.')
                || matches!(name.as_str(), "node_modules" | "target" | "builds")
                || kind.is_symlink()
            {
                continue;
            }
            texloom_build::snapshot::check_plain(&entry.path())?;
            if kind.is_dir() {
                visit(root, &entry.path(), out, total, depth + 1)?;
            } else if kind.is_file() && allowed_file(&entry.path()) {
                *total += entry.metadata().map_err(fail)?.len();
                if *total > MAX_PROJECT_BYTES || out.len() >= MAX_FILES {
                    return Err("Project exceeds 600 files or 60 MB".into());
                }
                out.push(entry.path().strip_prefix(root).map_err(fail)?.to_path_buf());
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    let mut total = 0;
    visit(root, root, &mut out, &mut total, 0)?;
    out.sort();
    Ok(out)
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature="typegen",derive(ts_rs::TS))]
pub struct Project {
    pub id: String,
    pub name: String,
    pub main: String,
    pub sources: BTreeMap<String, String>,
    #[serde(default)]
    pub recoveries: Vec<texloom_build::persistence::PendingSave>,
    #[serde(default)]
    pub recovery_error: Option<String>,
    #[serde(default)]
    pub restored: bool,
    #[serde(default)]
    pub draft_warning: Option<String>,
    #[serde(default)]
    pub source_only: bool,
}

#[derive(Deserialize)]
pub struct CompileRequest {
    pub project_id: String,
    pub main: String,
    pub sources: BTreeMap<String, String>,
    pub engine: String,
    pub revision: u64,
    pub purpose: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct MappedTextObject {
    pub id: String,
    pub file: String,
    pub dx: f64,
    pub dy: f64,
    pub plain_text: Option<String>,
    pub content: String,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CompileResult {
    pub ok: bool,
    pub build_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    #[cfg_attr(feature = "typegen", ts(type = "number[] | Uint8Array"))]
    pub pdf: Vec<u8>,
    pub objects: Vec<MappedTextObject>,
    pub log: String,
    pub engine: String,
    pub seconds: f64,
    pub verification_eligible: bool,
    pub passes: usize,
    pub diagnostics: Vec<texloom_build::engine::Diagnostic>,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub job_id: u64,
}

#[derive(Clone)]
struct Build {
    directory: PathBuf,
    compiler: PathBuf,
    project_id: String,
    main: String,
    revision: u64,
    receipt: texloom_build::snapshot::Receipt,
    eligible: bool,
    purpose: String,
    cancel: texloom_build::scheduler::Cancellation,
    sequence: u64,
    bytes: u64,
}

pub struct Backend {
    root: PathBuf,
    draft_recovery_enabled: bool,
    projects: Mutex<HashMap<String, PathBuf>>,
    builds: Mutex<HashMap<String, Arc<Build>>>,
    scheduler: texloom_build::scheduler::Scheduler,
    compiler_profiles: texloom_build::engine::CompilerProfiles,
    documents: Mutex<HashMap<String, document::DocumentSlot>>,
    disk_snapshots: Mutex<HashMap<String, texloom_build::persistence::DiskSnapshot>>,
    displayed: Mutex<HashMap<String, String>>,
    cache: std::sync::OnceLock<Result<build_cache::SessionCache>>,
    verifier: Option<Result<texloom_build::inspection_worker::Worker>>,
    recent_lock:Mutex<()>,
}

impl Backend {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            draft_recovery_enabled: true,
            projects: Mutex::new(HashMap::new()),
            builds: Mutex::new(HashMap::new()),
            scheduler: Default::default(),
            compiler_profiles: Default::default(),
            documents: Mutex::new(HashMap::new()),
            disk_snapshots: Mutex::new(HashMap::new()),
            displayed: Mutex::new(HashMap::new()),
            cache: Default::default(),
            verifier: None,
            recent_lock:Mutex::new(()),
        }
    }
    pub(crate) fn with_draft_recovery(mut self, enabled: bool) -> Self {
        self.draft_recovery_enabled = enabled;
        self
    }
    pub fn open_project(&self, path: Option<String>) -> Result<Project> {
        self.open_project_with_profile(path,None)
    }
    pub(super) fn open_project_with_profile(&self,path:Option<String>,hint:Option<ProjectProfile>)->Result<Project>{
        let (root, preferred) = if let Some(path) = path {
            let path = PathBuf::from(path).canonicalize().map_err(fail)?;
            if path.is_file() {
                if path.extension().and_then(|e| e.to_str()) != Some("tex") {
                    return Err("Choose a .tex document".into());
                }
                (
                    path.parent()
                        .ok_or("Missing project directory")?
                        .to_path_buf(),
                    Some(
                        path.file_name()
                            .ok_or("Missing filename")?
                            .to_string_lossy()
                            .to_string(),
                    ),
                )
            } else {
                (path, None)
            }
        } else {
            let root = self.root.join("Examples/LaTeX example");
            fs::create_dir_all(&root).map_err(fail)?;
            if !root.join("main.tex").exists() {
                fs::write(root.join("main.tex"), EXAMPLE).map_err(fail)?;
            }
            (root, Some("main.tex".into()))
        };
        let root = root.canonicalize().map_err(fail)?;
        let profile=if hint.is_some(){hint}else{project_workflows::profile(&root)?};
        let mut sources: BTreeMap<String, String> = BTreeMap::new();
        for file in project_files(&root)? {
            if text_file(&file) {
                let path = root.join(&file);
                if fs::metadata(&path).map_err(fail)?.len() > 4_000_000 {
                    return Err("A source file exceeds 4 MB".into());
                }
                let source = fs::read_to_string(path)
                    .map_err(|e| format!("{} must be UTF-8: {e}", file.display()))?;
                if source.contains('\0'){return Err(format!("{} needs encoding review before editing",file.display()))}
                sources.insert(file.to_string_lossy().replace('\\', "/"), source);
            }
        }
        let candidates: Vec<String> = sources
            .iter()
            .filter(|(name, text)| name.ends_with(".tex") && text.contains("\\documentclass"))
            .map(|(name, _)| name.clone())
            .collect();
        let main = preferred
            .or_else(||profile.as_ref().map(|p|p.main.clone()))
            .or_else(|| {
                candidates
                    .iter()
                    .find(|n| n.as_str() == "main.tex")
                    .cloned()
            })
            .or_else(|| candidates.first().cloned())
            .ok_or("No main LaTeX document found")?;
        let engine=profile.as_ref().filter(|p|p.main==main).map(|p|p.engine.clone()).unwrap_or_else(||"auto".into());
        let (recoveries,recovery_error)=match texloom_build::persistence::pending(&root){Ok(saves)=>(saves,None),Err(error)=>(Vec::new(),Some(error))};
        let mut disk = texloom_build::persistence::DiskSnapshot::capture(&root, &sources)?;
        disk.track_project_profile()?;
        let id = Uuid::new_v4().simple().to_string();
        let mut slot = document::DocumentSlot::new_with_engine(sources.clone(), main.clone(),engine.clone())?;
        if self.draft_recovery_enabled {
            slot.start_typing_recovery(&self.root, &disk);
        }
        self.documents.lock().map_err(fail)?.insert(
            id.clone(),
            slot,
        );
        self.projects
            .lock()
            .map_err(fail)?
            .insert(id.clone(), root.clone());
        self.disk_snapshots.lock().map_err(fail)?.insert(id.clone(), disk);
        let _=self.remember_recent(&root,&main,&engine);
        Ok(Project {
            id,
            name: root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            main,
            sources,
            recoveries,
            recovery_error,
            restored: false,
            draft_warning: None,
            source_only: false,
        })
    }
    fn project_root(&self, id: &str) -> Result<PathBuf> {
        self.projects
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or_else(|| "Reopen this project before continuing".into())
    }
    pub fn save_project(&self, id: &str, sources: &BTreeMap<String, String>,main:&str,engine:&str) -> Result<()> {
        let root = self.project_root(id)?.canonicalize().map_err(fail)?;
        let mut snapshots = self.disk_snapshots.lock().map_err(fail)?;
        let disk = snapshots.get_mut(id).ok_or("Reopen this project before Save")?;
        if disk.root != root { return Err("The project directory changed".into()); }
        let mut desired = sources.clone();
        desired.insert(texloom_build::project_io::PROFILE.into(),serde_json::to_string_pretty(&ProjectProfile{schema:1,main:main.into(),engine:engine.into()}).map_err(fail)?+"\n");
        match disk.save(&desired)? {
            texloom_build::persistence::SaveOutcome::Saved => Ok(()),
            texloom_build::persistence::SaveOutcome::Conflicts(conflicts) => {
                let files = conflicts.iter().take(8).map(|c|format!("{} ({})",c.file,c.kind)).collect::<Vec<_>>().join(", ");
                Err(format!("Save conflict: {files}. Your editor changes are preserved. Reload the disk version or save a project copy."))
            }
        }
    }
    pub fn compile(
        &self,
        request: CompileRequest,
        lease: &texloom_build::scheduler::Lease,
    ) -> Result<CompileResult> {
        self.compile_inner(&request, lease, None)
    }
    fn compile_inner(
        &self,
        request: &CompileRequest,
        lease: &texloom_build::scheduler::Lease,
        preview: Option<&mut dyn FnMut(&str, texloom_build::engine::UnverifiedPreview)>,
    ) -> Result<CompileResult> {
        lease.cancel.check()?;
        let root = self.project_root(&request.project_id)?;
        let main_source = request
            .sources
            .get(&request.main)
            .ok_or("Main source is missing")?;
        let engine = choose_engine(&request.engine, main_source)?;
        let binary = find_program(&engine).ok_or_else(|| {
            format!("{engine} was not found. Install MiKTeX or TeX Live, then reopen Texloom.")
        })?;
        let id = Uuid::new_v4().simple().to_string();
        let build_root = self.build_root()?;
        let directory = build_root.join(&id);
        let mut workspace = build_cache::Workspace::new(&build_root, &directory)?;
        let support = BTreeMap::from([
            ("texloom-objects.sty".into(), OBJECT_STYLE.into()),
        ]);
        let build_request = texloom_build::engine::Request {
                project_root: &root,
                directory: &directory,
                sources: &request.sources,
                main: &request.main,
                support: &support,
                compiler: &binary,
                engine: &engine,
            };
        let output = if let Some(observer) = preview {
            texloom_build::engine::build_with_preview(build_request, &lease.cancel,
                Some(&self.compiler_profiles), &mut |value| observer(&engine, value))?
        } else {
            texloom_build::engine::build_with_profiles(build_request, &lease.cancel, &self.compiler_profiles)?
        };
        lease.cancel.check()?;
        let mut objects = Vec::new();
        let mut ids = std::collections::HashSet::new();
        if output.ok {
            let mut text_preview = None;
            for (name, source) in &request.sources {
                if !name.ends_with(".tex") {
                    continue;
                }
                let mut projection = None;
                for object in texloom_core::text_objects(source)? {
                    if !ids.insert(object.id.clone()) {
                        return Err("Text object IDs must be unique across the project".into());
                    }
                    let projection = projection.get_or_insert_with(|| EditorProjection::new(source));
                    objects.push(MappedTextObject {
                        id: object.id,
                        file: name.clone(),
                        dx: object.dx,
                        dy: object.dy,
                        plain_text: text_preview
                            .get_or_insert_with(|| texloom_core::PlainTextPreview::new(main_source))
                            .render(&object.content),
                        content: object.content,
                        line: object.line,
                        start: projection.editor_offset(object.range.start).map_err(fail)?,
                        end: projection.editor_offset(object.range.end).map_err(fail)?,
                    });
                }
            }
            lease.cancel.check()?;
            self.builds.lock().map_err(fail)?.insert(
                id.clone(),
                Arc::new(Build {
                    directory: output.receipt.directory.clone(),
                    compiler: output.receipt.compiler.path.clone(),
                    project_id: request.project_id.clone(),
                    main: request.main.clone(),
                    revision: request.revision,
                    eligible: output.eligible,
                    purpose: request.purpose.clone(),
                    receipt: output.receipt.clone(),
                    cancel: lease.cancel.clone(),
                    sequence: lease.id,
                    bytes: build_cache::directory_bytes(&output.receipt.directory)?,
                }),
            );
            workspace.keep = true;
            self.prune_builds(&id);
        }
        Ok(CompileResult {
            ok: output.ok,
            build_id: id,
            revision: request.revision,
            pdf: output.pdf,
            objects,
            log: output.log,
            engine,
            seconds: output.seconds,
            verification_eligible: output.eligible,
            passes: output.receipt.passes,
            diagnostics: output.diagnostics,
            job_id: lease.id,
        })
    }
    pub fn cancel_compilation(&self, id: &str, sequence: u64, token: Option<&str>) {
        self.scheduler.cancel_sequence(id, sequence);
        if let Some(token) = token {
            if let Ok(mut docs) = self.documents.lock() {
                if let Some(slot) = docs.get_mut(id) {
                    slot.discard(token);
                }
            }
        }
    }
    pub fn verify_build(&self, id: &str) -> Result<()> {
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Compile the document first")?;
        build.receipt.verify()
    }
    pub fn verify_project_build(
        &self,
        project: &str,
        revision: u64,
        id: &str,
        require_eligible: bool,
    ) -> Result<()> {
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Compile the document first")?;
        if build.project_id != project || build.revision != revision {
            return Err("That PDF belongs to a different source revision".into());
        }
        if require_eligible && !build.eligible {
            return Err(
                "Resolve compiler errors and references before visual editing or export".into(),
            );
        }
        build.receipt.verify()
    }
    pub fn export_document_build(
        &self,
        project: &str,
        revision: u64,
        id: &str,
        path: &str,
    ) -> Result<()> {
        self.verify_project_build(project, revision, id, true)?;
        let snapshot = self.document_snapshot(project)?;
        if snapshot.revision != revision {
            return Err("The source changed while preparing export".into());
        }
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Export build expired")?;
        if build.purpose != "export" {
            return Err("Export requires its own clean build".into());
        }
        build.cancel.check()?;
        let target = PathBuf::from(path);
        if !target.is_absolute()
            || target
                .extension()
                .and_then(|s| s.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
                != Some("pdf")
        {
            return Err("Choose a .pdf output filename".into());
        }
        if let Ok(resolved) = target.canonicalize() {
            if build.receipt.assets.iter().any(|(name, asset)| {
                asset.path == resolved && build.receipt.used_project_inputs.contains(name)
            }) {
                return Err("Export cannot replace a PDF used as an input by this document".into());
            }
        }
        let temp = target
            .parent()
            .ok_or("Missing export folder")?
            .join(format!(".texloom-export-{}.tmp", Uuid::new_v4().simple()));
        let result = (|| {
            fs::copy(build.directory.join(".out/preview.pdf"), &temp).map_err(fail)?;
            build.receipt.verify()?;
            build.cancel.check()?;
            if self.document_snapshot(project)?.revision != revision {
                return Err("The source changed while exporting".into());
            }
            fs::rename(&temp, &target).map_err(fail)
        })();
        if temp.exists() {
            let _ = fs::remove_file(temp);
        }
        result
    }
    pub fn inverse(&self, id: &str, page: usize, x: f64, y: f64) -> Result<SourceLocation> {
        if page == 0 || !x.is_finite() || !y.is_finite() {
            return Err("Invalid page coordinate".into());
        }
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Compile the document first")?;
        let sibling = build.compiler.with_file_name(if cfg!(windows) {
            "synctex.exe"
        } else {
            "synctex"
        });
        self.check_mapping_snapshot(&build)?;
        let binary = if sibling.is_file() {
            sibling
        } else {
            find_program("synctex").ok_or("SyncTeX is not available with this compiler")?
        };
        let pdf = build.directory.join(".out/preview.pdf");
        let mut command = Command::new(binary);
        command
            .current_dir(&build.directory)
            .arg("edit")
            .arg("-o")
            .arg(format!("{page}:{x}:{y}:{}", pdf.display()));
        let output = synctex::run(command, &Default::default())?;
        self.check_mapping_snapshot(&build)?;
        let (input, line) = synctex::inverse(&output)?;
        let file = synctex::project_input(&build.directory, input)?;
        let _project = self.project_root(&build.project_id)?;
        Ok(SourceLocation {
            file,
            line,
        })
    }
    pub fn forward(&self, id: &str, file: &str, line: usize) -> Result<SourceRectangle> {
        if line == 0 {
            return Err("Source lines begin at one".into());
        }
        let file = relative(file)?;
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Compile the document first")?;
        self.check_mapping_snapshot(&build)?;
        let sibling = build.compiler.with_file_name(if cfg!(windows) {
            "synctex.exe"
        } else {
            "synctex"
        });
        let binary = if sibling.is_file() {
            sibling
        } else {
            find_program("synctex").ok_or("SyncTeX is unavailable")?
        };
        let mut command = Command::new(binary);
        command
            .current_dir(&build.directory)
            .arg("view")
            .arg("-i")
            .arg(format!("{line}:0:{}", build.directory.join(file).display()))
            .arg("-o")
            .arg(build.directory.join(".out/preview.pdf"));
        let output = synctex::run(command, &Default::default())?;
        self.check_mapping_snapshot(&build)?;
        self.project_root(&build.project_id)?;
        synctex::forward(&output)
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SourceRectangle {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn choose_engine(request: &str, source: &str) -> Result<String> {
    if request != "auto" {
        return if ["pdflatex", "xelatex", "lualatex"].contains(&request) {
            Ok(request.into())
        } else {
            Err("Unknown LaTeX engine".into())
        };
    }
    for line in source.trim_start_matches('\u{feff}').lines().take(20) {
        let lower = line.to_ascii_lowercase();
        if lower.trim_start().starts_with('%')
            && lower.contains("!tex")
            && lower.contains("program")
        {
            if let Some((_, value)) = lower.split_once('=') {
                let value = value.trim();
                if ["pdflatex", "xelatex", "lualatex"].contains(&value) {
                    return Ok(value.into());
                }
            }
        }
    }
    Ok(if source.contains("\\directlua") {
        "lualatex"
    } else if ["fontspec", "polyglossia", "setmainfont", "arabicfont"]
        .iter()
        .any(|s| source.contains(s))
    {
        "xelatex"
    } else {
        "pdflatex"
    }
    .into())
}

fn find_program(name: &str) -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let mut roots: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|s| std::env::split_paths(&s).collect())
        .unwrap_or_default();
    for variable in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(root) = std::env::var_os(variable) {
            let root = PathBuf::from(root);
            roots.push(root.join("Programs/MiKTeX/miktex/bin/x64"));
            roots.push(root.join("MiKTeX/miktex/bin/x64"));
        }
    }
    if cfg!(windows) {
        for year in 2020..=2030 {
            roots.push(PathBuf::from(format!("C:/texlive/{year}/bin/windows")));
        }
    }
    roots
        .into_iter()
        .map(|root| root.join(&executable))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    #[test]
    fn engine_directive_survives_a_utf8_bom_and_windows_line_endings() {
        assert_eq!(
            choose_engine(
                "auto",
                "\u{feff}% !TeX program = lualatex\r\n\\documentclass{article}"
            )
            .unwrap(),
            "lualatex"
        );
        assert_eq!(
            choose_engine("pdflatex", "% !TeX program = xelatex").unwrap(),
            "pdflatex"
        );
    }
}
