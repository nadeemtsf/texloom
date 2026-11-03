use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use texloom_core::{Block, Change, NewBlock};
use uuid::Uuid;
mod build_cache;
mod document;
mod layout_gate;
mod visual;
pub use layout_gate::VisualOperation;
pub use document::{
    DocumentSnapshot, EditorChange, EditorRange, PreparedReply, SelectionReply, SourceEditReply,
};
pub use visual::plan_object_edit as visual_plan;
pub use visual::{
    describe_text_edit, preview_edit_parameters, LiteralEditRequest, SpacingEditRequest,
    TextEditModel,
};
pub use visual::{ObjectEditRequest, ObjectPlan, ResolvedSelection, SelectionPoint};

const STYLE: &str = include_str!("../../resources/studio.sty");
const EXAMPLE: &str = include_str!("../../resources/main.tex");
const OBJECT_STYLE: &str = include_str!("../../resources/texloom-objects.sty");
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
pub struct Project {
    pub id: String,
    pub name: String,
    pub main: String,
    pub sources: BTreeMap<String, String>,
    #[serde(default)]
    pub recoveries: Vec<texloom_build::persistence::PendingSave>,
    #[serde(default)]
    pub recovery_error: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct CompileRequest {
    pub project_id: String,
    pub main: String,
    pub sources: BTreeMap<String, String>,
    pub engine: String,
    pub revision: u64,
    pub purpose: String,
}

#[derive(Serialize)]
pub struct CompileResult {
    pub ok: bool,
    pub build_id: String,
    pub revision: u64,
    pub pdf: Vec<u8>,
    pub blocks: Vec<Value>,
    pub objects: Vec<Value>,
    pub log: String,
    pub engine: String,
    pub seconds: f64,
    pub verification_eligible: bool,
    pub passes: usize,
    pub diagnostics: Vec<texloom_build::engine::Diagnostic>,
    pub job_id: u64,
}

#[derive(Clone, Deserialize)]
pub struct TransformRequest {
    pub source: String,
    pub action: String,
    pub id: String,
    #[serde(default)]
    pub changes: BTreeMap<String, Value>,
    #[serde(default)]
    pub selection: Option<[usize; 2]>,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub plain: bool,
    #[serde(default = "one")]
    pub page: usize,
    #[serde(default = "x_default")]
    pub x: f64,
    #[serde(default = "y_default")]
    pub y: f64,
    #[serde(default = "width_default")]
    pub width: f64,
    #[serde(default = "size_default")]
    pub size: f64,
}
fn one() -> usize {
    1
}
fn x_default() -> f64 {
    62.0
}
fn y_default() -> f64 {
    100.0
}
fn width_default() -> f64 {
    270.0
}
fn size_default() -> f64 {
    12.0
}

pub fn transform(request: TransformRequest) -> Result<String> {
    if request.source.len() > 4_000_000 {
        return Err("Source file exceeds 4 MB".into());
    }
    if request.action == "patch" {
        let mut changes = Vec::new();
        for (name, value) in request.changes {
            let number = || {
                value
                    .as_f64()
                    .ok_or_else(|| "Enter a numeric dimension".to_string())
            };
            changes.push(match name.as_str() {
                "x" => Change::X(number()?),
                "y" => Change::Y(number()?),
                "width" => Change::Width(number()?),
                "size" => Change::Size(number()?),
                "page" => {
                    Change::Page(value.as_u64().ok_or("Page must be a positive integer")? as usize)
                }
                "content" => Change::Content(value.as_str().ok_or("Content must be text")?.into()),
                _ => return Err("Unknown block property".into()),
            });
        }
        texloom_core::patch_block(&request.source, &request.id, &changes)
    } else if request.action == "delete" {
        let matches: Vec<_> = texloom_core::blocks(&request.source).into_iter()
            .filter(|b| b.id == request.id).collect();
        if matches.len() != 1 { return Err("Select one unique positioned block".into()); }
        let mut source = request.source;
        source.replace_range(matches[0].range.clone(), "");
        Ok(source)
    } else if request.action == "insert" {
        let projection = texloom_core::document::projection::EditorProjection::new(&request.source);
        let selection = request
            .selection
            .map(|[a, b]| {
                Ok::<_, String>(
                    projection.source_offset(a).map_err(fail)?
                        ..projection.source_offset(b).map_err(fail)?,
                )
            })
            .transpose()?;
        let content = if request.plain {
            texloom_core::escape_text(&request.content)
        } else {
            request.content
        };
        let block = NewBlock {
            id: request.id,
            page: request.page,
            x: request.x,
            y: request.y,
            width: request.width,
            size: request.size,
            content,
        };
        texloom_core::insert_block(&request.source, &block, selection)
    } else {
        Err("Unknown visual edit".into())
    }
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
    projects: Mutex<HashMap<String, PathBuf>>,
    builds: Mutex<HashMap<String, Arc<Build>>>,
    scheduler: texloom_build::scheduler::Scheduler,
    documents: Mutex<HashMap<String, document::DocumentSlot>>,
    disk_snapshots: Mutex<HashMap<String, texloom_build::persistence::DiskSnapshot>>,
    displayed: Mutex<HashMap<String, String>>,
    cache: std::sync::OnceLock<Result<build_cache::SessionCache>>,
    verifier: Option<Result<texloom_build::inspection_worker::Worker>>,
}

impl Backend {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            projects: Mutex::new(HashMap::new()),
            builds: Mutex::new(HashMap::new()),
            scheduler: Default::default(),
            documents: Mutex::new(HashMap::new()),
            disk_snapshots: Mutex::new(HashMap::new()),
            displayed: Mutex::new(HashMap::new()),
            cache: Default::default(),
            verifier: None,
        }
    }
    pub fn open_project(&self, path: Option<String>) -> Result<Project> {
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
            let root = self.root.join("Examples/Visual text 0.3");
            fs::create_dir_all(&root).map_err(fail)?;
            if !root.join("main.tex").exists() {
                fs::write(root.join("main.tex"), EXAMPLE).map_err(fail)?;
            }
            if !root.join("studio.sty").exists() {
                fs::write(root.join("studio.sty"), STYLE).map_err(fail)?;
            }
            (root, Some("main.tex".into()))
        };
        let mut sources: BTreeMap<String, String> = BTreeMap::new();
        for file in project_files(&root)? {
            if text_file(&file) {
                let path = root.join(&file);
                if fs::metadata(&path).map_err(fail)?.len() > 4_000_000 {
                    return Err("A source file exceeds 4 MB".into());
                }
                let source = fs::read_to_string(path)
                    .map_err(|e| format!("{} must be UTF-8: {e}", file.display()))?;
                sources.insert(file.to_string_lossy().replace('\\', "/"), source);
            }
        }
        let candidates: Vec<String> = sources
            .iter()
            .filter(|(name, text)| name.ends_with(".tex") && text.contains("\\documentclass"))
            .map(|(name, _)| name.clone())
            .collect();
        let main = preferred
            .or_else(|| {
                candidates
                    .iter()
                    .find(|n| n.as_str() == "main.tex")
                    .cloned()
            })
            .or_else(|| candidates.first().cloned())
            .ok_or("No main LaTeX document found")?;
        let (recoveries,recovery_error)=match texloom_build::persistence::pending(&root){Ok(saves)=>(saves,None),Err(error)=>(Vec::new(),Some(error))};
        let disk = texloom_build::persistence::DiskSnapshot::capture(&root, &sources)?;
        let id = Uuid::new_v4().simple().to_string();
        self.documents.lock().map_err(fail)?.insert(
            id.clone(),
            document::DocumentSlot::new(sources.clone(), main.clone())?,
        );
        self.projects
            .lock()
            .map_err(fail)?
            .insert(id.clone(), root.clone());
        self.disk_snapshots.lock().map_err(fail)?.insert(id.clone(), disk);
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
    pub fn save_project(&self, id: &str, sources: &BTreeMap<String, String>) -> Result<()> {
        let root = self.project_root(id)?.canonicalize().map_err(fail)?;
        let mut snapshots = self.disk_snapshots.lock().map_err(fail)?;
        let disk = snapshots.get_mut(id).ok_or("Reopen this project before Save")?;
        if disk.root != root { return Err("The project directory changed".into()); }
        let mut desired = sources.clone();
        // Preserve the existing isolated StudioBlock compatibility dependency.
        // Normal text-object dependencies already belong to canonical source.
        if !desired.contains_key("studio.sty")
            && (sources.values().any(|s|s.contains("\\StudioBlock"))
                || disk.files.get("studio.sty").is_some_and(Option::is_some))
        {
            desired.insert("studio.sty".into(), STYLE.into());
        }
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
        lease.cancel.check()?;
        let root = self.project_root(&request.project_id)?;
        let source = request
            .sources
            .get(&request.main)
            .ok_or("Main source is missing")?;
        let engine = choose_engine(&request.engine, source)?;
        let binary = find_program(&engine).ok_or_else(|| {
            format!("{engine} was not found. Install MiKTeX or TeX Live, then reopen Texloom.")
        })?;
        let id = Uuid::new_v4().simple().to_string();
        let build_root = self.build_root()?;
        let directory = build_root.join(&id);
        let mut workspace = build_cache::Workspace::new(&build_root, &directory)?;
        let support = BTreeMap::from([
            ("studio.sty".into(), STYLE.into()),
            ("texloom-objects.sty".into(), OBJECT_STYLE.into()),
        ]);
        let output = texloom_build::engine::build(
            texloom_build::engine::Request {
                project_root: &root,
                directory: &directory,
                sources: &request.sources,
                main: &request.main,
                support: &support,
                compiler: &binary,
                engine: &engine,
            },
            &lease.cancel,
        )?;
        lease.cancel.check()?;
        let metrics = measurements(&output.receipt.directory.join(".out/preview.studio"));
        let mut mapped = Vec::new();
        let mut objects = Vec::new();
        let mut ids = std::collections::HashSet::new();
        if output.ok {
            for (name, source) in &request.sources {
                if !name.ends_with(".tex") {
                    continue;
                }
                for block in texloom_core::blocks(source) {
                    mapped.push(ui_block(&block, name, source, &metrics)?);
                }
                for object in texloom_core::text_objects(source)? {
                    if !ids.insert(object.id.clone()) {
                        return Err("Text object IDs must be unique across the project".into());
                    }
                    let projection =
                        texloom_core::document::projection::EditorProjection::new(source);
                    objects.push(json!({"id":object.id,"file":name,"dx":object.dx,"dy":object.dy,
                        "plain_text":texloom_core::plain_text_preview(&object.content,request.sources.get(&request.main).unwrap_or(source)),
                        "content":object.content,"line":object.line,"start":projection.editor_offset(object.range.start).map_err(fail)?,
                        "end":projection.editor_offset(object.range.end).map_err(fail)?}));
                }
            }
            lease.cancel.check()?;
            self.builds.lock().map_err(fail)?.insert(
                id.clone(),
                Arc::new(Build {
                    directory: output.receipt.directory.clone(),
                    compiler: output.receipt.compiler.path.clone(),
                    project_id: request.project_id,
                    main: request.main,
                    revision: request.revision,
                    eligible: output.eligible,
                    purpose: request.purpose,
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
            blocks: mapped,
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
        let (_, output) = run_command(command, Duration::from_secs(10))?;
        let mut input = None;
        let mut line = None;
        for record in output.lines() {
            if let Some(value) = record.strip_prefix("Input:") {
                if input.is_none() {
                    input = Some(value.trim().to_string());
                }
            }
            if let Some(value) = record.strip_prefix("Line:") {
                if line.is_none() {
                    line = value.trim().parse::<usize>().ok();
                }
            }
        }
        let input = PathBuf::from(input.ok_or("No source was mapped at this point")?);
        let input = if input.is_absolute() {
            input
        } else {
            build.directory.join(input)
        };
        let input = input.canonicalize().map_err(fail)?;
        let folder = build.directory.canonicalize().map_err(fail)?;
        let file = input
            .strip_prefix(folder)
            .map_err(|_| "The mapped source is outside this project")?
            .to_string_lossy()
            .replace('\\', "/");
        let _project = self.project_root(&build.project_id)?;
        Ok(SourceLocation {
            file,
            line: line.ok_or("No source line was mapped")?,
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
        let (_, output) = run_command(command, Duration::from_secs(10))?;
        let mut numbers: HashMap<&str, f64> = HashMap::new();
        for record in output.lines() {
            if let Some((key, value)) = record.split_once(':') {
                if let Ok(number) = value.trim().parse::<f64>() {
                    numbers.entry(key.trim()).or_insert(number);
                }
            }
        }
        let page = numbers
            .get("Page")
            .copied()
            .ok_or("No PDF position was mapped for this source")? as usize;
        let height = numbers.get("H").copied().unwrap_or(12.0).max(1.0);
        Ok(SourceRectangle {
            page,
            x: numbers
                .get("h")
                .or_else(|| numbers.get("x"))
                .copied()
                .unwrap_or(62.0),
            y: (numbers
                .get("v")
                .or_else(|| numbers.get("y"))
                .copied()
                .unwrap_or(112.0)
                - height)
                .max(0.0),
            width: numbers.get("W").copied().unwrap_or(300.0).max(30.0),
            height,
        })
    }
}

#[derive(Serialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Serialize)]
pub struct SourceRectangle {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn ui_block(
    block: &Block,
    file: &str,
    source: &str,
    metrics: &HashMap<(String, usize), f64>,
) -> Result<Value> {
    Ok(
        json!({ "id": block.id, "file": file, "page": block.page, "x": block.x, "y": block.y,
        "width": block.width, "size": block.size, "content": block.content,
        "height": metrics.get(&(block.id.clone(), block.page)).copied().unwrap_or(block.size * 1.3),
        "start": texloom_core::document::projection::EditorProjection::new(source).editor_offset(block.range.start).map_err(fail)?,
        "end": texloom_core::document::projection::EditorProjection::new(source).editor_offset(block.range.end).map_err(fail)?, "line": block.line }),
    )
}

fn measurements(path: &Path) -> HashMap<(String, usize), f64> {
    let mut out = HashMap::new();
    if let Ok(text) = fs::read_to_string(path) {
        for line in text.lines() {
            let p: Vec<_> = line.split('|').collect();
            if p.len() != 5 {
                continue;
            }
            if let (Ok(page), Ok(h), Ok(d)) = (
                p[1].parse::<usize>(),
                p[3].trim_end_matches("pt").parse::<f64>(),
                p[4].trim_end_matches("pt").parse::<f64>(),
            ) {
                out.insert((p[0].to_string(), page), (h + d) * 72.0 / 72.27);
            }
        }
    }
    out
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

fn run_command(command: Command, timeout: Duration) -> Result<(bool, String)> {
    let args: Vec<String> = command
        .get_args()
        .map(|x| x.to_string_lossy().into())
        .collect();
    let env: BTreeMap<String, String> = command
        .get_envs()
        .filter_map(|(k, v)| v.map(|v| (k.to_string_lossy().into(), v.to_string_lossy().into())))
        .collect();
    let cwd = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir().map_err(fail)?);
    let output = texloom_build::process::run(
        &texloom_build::process::CommandSpec {
            program: Path::new(command.get_program()),
            args: &args,
            directory: &cwd,
            env: &env,
            timeout,
            memory_bytes: 2 * 1024 * 1024 * 1024,
        },
        &Default::default(),
    )
    .map_err(fail)?;
    Ok((output.success, output.text))
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
