//! Opt-in synthetic native source-mapping pilot. No window, Save or commit.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};
use texloom_desktop_lib::benchmark::{visual_plan, Backend, ObjectEditRequest, SelectionPoint};
type Result<T> = std::result::Result<T, String>;
const SOURCE: &str = include_str!("../../resources/templates/multilingual.tex");
#[derive(Deserialize)]
struct Query {
    language: String,
    candidate: usize,
    x: f64,
    y: f64,
}
fn write_new(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(bytes.as_ref())
        .map_err(|e| e.to_string())
}
fn emit(value: Value) -> Result<()> {
    println!("{value}");
    io::stdout().flush().map_err(|e| e.to_string())
}
fn run() -> Result<()> {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Pass a fresh absolute output directory")?,
    );
    if !root.is_absolute() {
        return Err("Output must be absolute".into());
    }
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    let project_root = root.join("project");
    fs::create_dir(&project_root).map_err(|e| e.to_string())?;
    write_new(&project_root.join("main.tex"), SOURCE)?;
    let backend = Backend::new(root.join("cache"))
        .with_verification_resources(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"));
    let project = backend.open_project(Some(
        project_root.join("main.tex").to_string_lossy().into_owned(),
    ))?;
    let current = backend.document_snapshot(&project.id)?;
    let baseline = backend.compile_document(&project.id, 0, None, Some("preview"), 2)?;
    write_new(&root.join("baseline.log"), &baseline.log)?;
    if !baseline.ok || !baseline.verification_eligible || baseline.engine != "xelatex" {
        return Err("Baseline failed, ineligible or not XeLaTeX; see retained log".into());
    }
    let pdf = root.join("baseline.pdf");
    write_new(&pdf, &baseline.pdf)?;
    emit(json!({"status":"ready","baseline_pdf":pdf,"engine":baseline.engine}))?;
    for (index, line) in io::stdin().lock().lines().enumerate() {
        if index >= 48 {
            return Err("Query budget exhausted".into());
        }
        let line = line.map_err(|e| e.to_string())?;
        if line.len() > 8192 {
            return Err("Query exceeds budget".into());
        }
        let query: Query = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if !["arabic", "hebrew", "english"].contains(&query.language.as_str())
            || query.candidate >= 16
            || !query.x.is_finite()
            || !query.y.is_finite()
        {
            return Err("Invalid declared query".into());
        }
        let result = (|| -> Result<Value> {
            let unit = backend.resolve_text_selection(
                &project.id,
                0,
                &baseline.build_id,
                1,
                vec![SelectionPoint {
                    x: query.x,
                    y: query.y,
                }],
                query.candidate,
            )?;
            let id = format!("mapped-{}-{}", query.language, query.candidate);
            let request: ObjectEditRequest = serde_json::from_value(json!({
                "sources":current.sources,"main":project.main,"file":unit.file,"action":"wrap",
                "id":id,"selection":[unit.start,unit.end],"expected":unit.content,"dx":0,"dy":0
            }))
            .map_err(|e| e.to_string())?;
            let plan = visual_plan(request)?;
            let prepared = backend.document_prepare_visual(
                &project.id,
                0,
                plan.sources,
                &baseline.build_id,
                None,
            )?;
            let built = backend.compile_document(&project.id, 0, Some(&prepared.token), None, 2);
            backend.document_discard(&project.id, &prepared.token)?;
            let built = built?;
            write_new(&root.join(format!("{id}.log")), &built.log)?;
            if !built.ok || !built.verification_eligible {
                return Err("Private zero compilation failed/ineligible".into());
            }
            let output = root.join(format!("{id}.pdf"));
            write_new(&output, &built.pdf)?;
            Ok(json!({"status":"probe","id":id,"pdf":output,"unit":unit}))
        })();
        let snapshot = backend.document_snapshot(&project.id)?;
        let unchanged = snapshot.sources == current.sources
            && snapshot.revision == 0
            && !snapshot.can_undo
            && fs::read(project_root.join("main.tex")).map_err(|e| e.to_string())?
                == SOURCE.as_bytes();
        let mut value = result.unwrap_or_else(|reason| json!({"status":"refused","reason":reason}));
        value["canonical_and_disk_unchanged"] = json!(unchanged);
        emit(value)?;
        if !unchanged {
            return Err("Canonical source/history or original disk changed".into());
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        let _ = emit(json!({"status":"fatal","reason":error}));
        std::process::exit(1);
    }
}
