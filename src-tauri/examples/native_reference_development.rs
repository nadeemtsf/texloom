//! Explicit development harness at the native candidate boundary, never a UI test.
//! Opens only fresh synthetic copies; does not call Save or remove evidence.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use texloom_desktop_lib::benchmark::Backend;

type Result<T> = std::result::Result<T, String>;
type Sources = BTreeMap<String, String>;
#[derive(Deserialize)]
struct Protocol {
    schema: u32,
    kind: String,
    conditions: Vec<Condition>,
}
#[derive(Deserialize)]
struct Condition {
    id: String,
    engine: String,
    language: String,
    baseline: Sources,
    request: Value,
    candidates: Vec<Candidate>,
}
#[derive(Deserialize)]
struct Candidate {
    variant: String,
    sources: Sources,
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    use std::io::Write;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(bytes.as_ref())
        .map_err(|e| e.to_string())
}
fn source_files(root: &Path, sources: &Sources) -> Result<()> {
    fs::create_dir(root).map_err(|e| e.to_string())?;
    let expected = ["main.tex", "notes.sty", "texloom-objects.sty"];
    if sources.len() != expected.len() || expected.iter().any(|name| !sources.contains_key(*name)) {
        return Err("Only the three declared synthetic .tex/.sty files are permitted".into());
    }
    for (name, text) in sources {
        write(&root.join(name), text.as_bytes())?;
    }
    Ok(())
}
fn disk_equals(root: &Path, sources: &Sources) -> bool {
    sources
        .iter()
        .all(|(name, text)| fs::read(root.join(name)).is_ok_and(|bytes| bytes == text.as_bytes()))
}
fn run_case(root: &Path, case: &Condition) -> Result<Value> {
    let allowed = [
        "pdflatex-latin",
        "xelatex-latin",
        "lualatex-latin",
        "xelatex-multilingual",
        "lualatex-multilingual",
    ];
    if !allowed.contains(&case.id.as_str())
        || !["pdflatex", "xelatex", "lualatex"].contains(&case.engine.as_str())
    {
        return Err("Unknown development condition".into());
    }
    let variants = [
        "correct",
        "wrong_first_occurrence",
        "unrelated_text_changed",
        "appearance_changed",
        "duplicate_ink",
        "support_file_changed",
    ];
    let actual: std::collections::BTreeSet<_> =
        case.candidates.iter().map(|c| c.variant.as_str()).collect();
    if case.candidates.len() != 6 || actual != variants.into_iter().collect() {
        return Err("Unexpected candidate set".into());
    }
    let directory = root.join(&case.id);
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let project_root = directory.join("project");
    source_files(&project_root, &case.baseline)?;
    let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
    let backend = Backend::new(directory.join("cache")).with_verification_resources(resources);
    let project =
        backend.open_project(Some(project_root.join("main.tex").to_string_lossy().into()))?;
    let current = backend.document_snapshot(&project.id)?;
    if current.sources != case.baseline || current.revision != 0 {
        return Err("Opened bytes or initial revision differ from protocol".into());
    }
    let baseline = backend.compile_document(&project.id, 0, None, Some("preview"), 1)?;
    write(&directory.join("baseline-log.txt"), &baseline.log)?;
    if !baseline.ok || !baseline.verification_eligible || baseline.engine != case.engine {
        return Err(
            "Baseline build failed, ineligible or used another engine; retained cache/log".into(),
        );
    }
    write(&directory.join("baseline.pdf"), &baseline.pdf)?;
    let mut request = case.request.clone();
    request
        .as_object_mut()
        .ok_or("Request must be an object")?
        .insert("sources".into(), json!(current.sources));
    let operation = json!({"kind":"text","request":request});
    let mut faults = Vec::new();
    // Fault packets are checked before committing the valid case, so the same
    // baseline/revision is current. A stale baseline must not cause their refusal.
    for candidate in case.candidates.iter().filter(|c| c.variant != "correct") {
        let outcome = backend.document_prepare_visual(
            &project.id,
            0,
            candidate.sources.clone(),
            &baseline.build_id,
            Some(serde_json::from_value(operation.clone()).map_err(|e| e.to_string())?),
        );
        let (accepted, reason) = match outcome {
            Ok(prepared) => {
                backend.document_discard(&project.id, &prepared.token)?;
                (true, None)
            }
            Err(error) => (false, Some(error)),
        };
        let after = backend.document_snapshot(&project.id)?;
        faults.push(json!({"variant":candidate.variant,"prepare_accepted":accepted,"reason":reason,
            "compile_called":false,"canonical_source_history_unchanged":after.sources==current.sources&&after.revision==0&&!after.can_undo}));
    }
    let candidate = case
        .candidates
        .iter()
        .find(|c| c.variant == "correct")
        .ok_or("Missing correct candidate")?;
    let mut committed_result = false;
    let mut failure_stage = "native-prepare";
    let valid = (|| -> Result<Value> {
        let prepared = backend.document_prepare_visual(
            &project.id,
            0,
            candidate.sources.clone(),
            &baseline.build_id,
            Some(serde_json::from_value(operation).map_err(|e| e.to_string())?),
        )?;
        failure_stage = "native-build-and-approval";
        let built = backend.compile_document(&project.id, 0, Some(&prepared.token), None, 2)?;
        write(&directory.join("candidate-log.txt"), &built.log)?;
        if !built.ok || !built.verification_eligible {
            return Err("Candidate compilation was unsuccessful/ineligible".into());
        }
        write(&directory.join("native-candidate.pdf"), &built.pdf)?;
        failure_stage = "native-commit";
        let committed = backend.document_commit(&project.id, &prepared.token, &built.build_id)?;
        committed_result = true;
        failure_stage = "postcommit-export-and-undo";
        source_files(&directory.join("committed-sources"), &committed.sources)?;
        let undone = backend.document_history(&project.id, committed.revision, false)?;
        source_files(&directory.join("undone-sources"), &undone.sources)?;
        Ok(
            json!({"status":"committed","commit_performed":true,"revision":committed.revision,"committed_source_equals_declared_candidate":committed.sources==candidate.sources,
            "undo_exact":undone.sources==current.sources,"undo_revision":undone.revision,
            "disk_unchanged":disk_equals(&project_root,&current.sources),"build_id":built.build_id,"engine":built.engine}),
        )
    })();
    let valid = match valid {
        Ok(result) => result,
        Err(reason) => {
            json!({"status":"failed_or_refused","commit_performed":committed_result,"stage":failure_stage,"reason":reason})
        }
    };
    let after = backend.document_snapshot(&project.id)?;
    source_files(&directory.join("after-run-sources"), &after.sources)?;
    Ok(
        json!({"id":case.id,"engine":case.engine,"language":case.language,"fault_packets":faults,"intended_move":valid,
        "disk_unchanged_after_run":disk_equals(&project_root,&current.sources),"baseline_build_id":baseline.build_id}),
    )
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "Usage: native_reference_development ABSOLUTE_PROTOCOL ABSOLUTE_FRESH_NATIVE_OUTPUT"
                .into(),
        );
    }
    let protocol_path = PathBuf::from(&args[0])
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let output = PathBuf::from(&args[1]);
    if !output.is_absolute()
        || output.file_name() != Some(std::ffi::OsStr::new("native-output"))
        || output
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .as_deref()
            != protocol_path.parent()
    {
        return Err(
            "Output must be a new native-output directory beside the declared protocol".into(),
        );
    }
    let output = protocol_path.parent().unwrap().join("native-output");
    let raw = fs::read(&protocol_path).map_err(|e| e.to_string())?;
    if raw.len() > 2_000_000 {
        return Err("Development protocol exceeds its budget".into());
    }
    let protocol: Protocol = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if protocol.schema != 1
        || protocol.kind != "native_candidate_boundary_development_v1"
        || protocol.conditions.len() != 5
    {
        return Err("Not the declared five-condition development protocol".into());
    }
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for case in protocol.conditions {
        let row = match run_case(&output, &case) {
            Ok(row) => row,
            Err(reason) => {
                json!({"id":case.id,"engine":case.engine,"status":"unresolved","reason":reason})
            }
        };
        println!(
            "{}: {}",
            case.id,
            row.get("intended_move")
                .and_then(|v| v.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("unresolved")
        );
        rows.push(row);
        write(
            &output.join(format!("{}.json", case.id)),
            serde_json::to_vec_pretty(rows.last().unwrap()).map_err(|e| e.to_string())?,
        )?;
    }
    let report = json!({"schema":1,"kind":"native_candidate_boundary_development_not_pdf_mouse_selection","conditions":rows,
        "ui_opened":false,"save_called":false,"final_evaluation_executed":false,
        "limits":["Proposed-source/operation entry, not PDF.js mapping or zero-offset frontend selection proof","Fault packets are not compiled after source-plan refusal","No independent human labels, latency result or native UI claim"]});
    write(
        &output.join("native-report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1)
    }
}
