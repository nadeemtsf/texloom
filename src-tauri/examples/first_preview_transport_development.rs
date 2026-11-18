//! Disposable native service routing control; no window, input, original Save or installation.
use std::{env, fs, path::PathBuf};
use serde_json::json;
use texloom_desktop_lib::benchmark::Backend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env::args_os().nth(1).ok_or("Provide a fresh absolute run directory")?);
    if !root.is_absolute() { return Err("Use an absolute path".into()); }
    fs::create_dir(&root)?;
    let project_root = root.join("project");fs::create_dir(&project_root)?;
    let source = "\\documentclass{article}\n\\begin{document}\nPreliminary transport fixture.\n\\end{document}\n";
    fs::write(project_root.join("main.tex"), source)?;
    let backend = Backend::new(root.join("cache"));
    let project = backend.open_project(Some(project_root.join("main.tex").to_string_lossy().into()))?;
    let before = backend.document_snapshot(&project.id)?;
    let mut events = Vec::new();
    let mut observe = |engine: &str, preview: texloom_build::engine::UnverifiedPreview| {
        let current = backend.document_snapshot(&project.id).unwrap();
        assert_eq!(current.sources, before.sources);assert_eq!(current.revision, before.revision);
        assert!(!current.can_undo && !current.can_redo);
        assert!(events.is_empty());
        fs::write(root.join("first-pass.pdf"), &preview.pdf).unwrap();
        events.push(json!({"engine":engine,"seconds":preview.seconds,"bytes":preview.pdf.len()}));
    };
    let settled = backend.compile_document_observed(&project.id, before.revision, None,
        Some("preview"), 1, Some(&mut observe))?;
    assert_eq!(events.len(),1);assert!(settled.ok && settled.verification_eligible);
    fs::write(root.join("final.pdf"), &settled.pdf)?;
    let mut forbidden = 0;
    let mut reject = |_engine: &str, _preview| { forbidden += 1; };
    let exported = backend.compile_document_observed(&project.id, before.revision, None,
        Some("export"), 2, Some(&mut reject))?;
    assert!(exported.ok && exported.verification_eligible);
    let prepared = backend.document_prepare_visual(&project.id, before.revision, before.sources.clone(),
        &settled.build_id, None)?;
    let visual = backend.compile_document_observed(&project.id, before.revision, Some(&prepared.token),
        Some("preview"), 3, Some(&mut reject))?;
    assert!(visual.ok && visual.verification_eligible);assert_eq!(forbidden,0);
    let after = backend.document_snapshot(&project.id)?;
    assert_eq!(after.sources,before.sources);assert_eq!(after.revision,before.revision);
    assert!(!after.can_undo && !after.can_redo);
    assert_eq!(fs::read_to_string(project_root.join("main.tex"))?,source);
    let report = json!({"schema":1,"kind":"headless_native_preview_routing","events":events,
        "export_events":0,"visual_events":0,"source_history_and_original_preserved":true,
        "final_build_id":settled.build_id,"engine":settled.engine,"passes":settled.passes,
        "scope":"Native service calls; no Tauri dispatch or physical/native UI acceptance"});
    fs::write(root.join("report.json"),serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");Ok(())
}
