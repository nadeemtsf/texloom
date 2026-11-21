//! Three-engine native source-error recovery control without a desktop window.
use serde_json::json;
use std::{env, fs, path::PathBuf};
use texloom_desktop_lib::benchmark::{Backend, EditorChange};

const GOOD: &str = "\\documentclass{article}\n\\pagestyle{empty}\n\\begin{document}\nOriginal settled text.\n\\end{document}\n";
const BAD: &str = "\\documentclass{article}\n\\pagestyle{empty}\n\\begin{document}\n\\MissingFixtureCommand\n\\end{document}\n";
const FIXED: &str = "\\documentclass{article}\n\\pagestyle{empty}\n\\begin{document}\nCorrected settled text.\n\\end{document}\n";

fn change(old: &str, new: &str) -> Vec<EditorChange> {
    vec![EditorChange {
        offset: 0,
        length: old.encode_utf16().count(),
        text: new.into(),
    }]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        env::args_os()
            .nth(1)
            .ok_or("Provide a fresh absolute run directory")?,
    );
    if !root.is_absolute() {
        return Err("Use an absolute path".into());
    }
    fs::create_dir(&root)?;
    let backend = Backend::new(root.join("cache"));
    let mut rows = Vec::new();
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let project_root = root.join(engine);
        fs::create_dir(&project_root)?;
        let input = project_root.join("main.tex");
        fs::write(&input, GOOD)?;
        let project = backend.open_project(Some(input.to_string_lossy().into_owned()))?;
        let original =
            backend.document_profile(&project.id, 0, "main.tex".into(), engine.into())?;
        let good = backend.compile_document_observed(
            &project.id,
            original.revision,
            None,
            Some("preview"),
            1,
            None,
        )?;
        assert!(good.ok && good.verification_eligible);
        backend.verify_project_build(&project.id, original.revision, &good.build_id, true)?;
        backend.pin_displayed_build(&project.id, &good.build_id)?;
        fs::write(project_root.join("before.pdf"), &good.pdf)?;

        let broken = backend.document_edit(
            &project.id,
            original.revision,
            "main.tex",
            change(GOOD, BAD),
            None,
        )?;
        let failed = backend.compile_document_observed(
            &project.id,
            broken.revision,
            None,
            Some("preview"),
            2,
            None,
        )?;
        assert!(!failed.ok && !failed.verification_eligible && failed.pdf.is_empty());
        assert!(failed.log.contains("Undefined control sequence"));
        assert!(backend.verify_build(&failed.build_id).is_err());
        assert!(backend
            .verify_project_build(&project.id, broken.revision, &good.build_id, true)
            .is_err());
        assert!(backend
            .pin_displayed_build(&project.id, &failed.build_id)
            .is_err());
        backend.pin_displayed_build(&project.id, &good.build_id)?;

        let repaired = backend.document_edit(
            &project.id,
            broken.revision,
            "main.tex",
            change(BAD, FIXED),
            None,
        )?;
        let settled = backend.compile_document_observed(
            &project.id,
            repaired.revision,
            None,
            Some("preview"),
            3,
            None,
        )?;
        assert!(settled.ok && settled.verification_eligible);
        assert!(settled.diagnostics.is_empty());
        backend.verify_project_build(&project.id, repaired.revision, &settled.build_id, true)?;
        backend.pin_displayed_build(&project.id, &settled.build_id)?;
        fs::write(project_root.join("after.pdf"), &settled.pdf)?;
        fs::write(project_root.join("failed.log"), &failed.log)?;
        assert_eq!(repaired.sources["main.tex"], FIXED);
        assert_eq!(fs::read_to_string(&input)?, GOOD);
        rows.push(json!({
            "engine": engine,
            "revisions": [original.revision, broken.revision, repaired.revision],
            "builds": [&good.build_id, &failed.build_id, &settled.build_id],
            "elapsed_seconds": [good.seconds, failed.seconds, settled.seconds],
            "failed_diagnostics": failed.diagnostics,
            "good_revision_verified": true,
            "failed_build_refused": true,
            "previous_build_refused_for_broken_revision": true,
            "repaired_revision_verified": true,
        }));
        backend.close_document(&project.id)?;
    }
    let report = json!({"schema": 1, "runs": rows, "scope": "Native compile service and display-pin commands without physical input or a renderer."});
    fs::write(
        root.join("evidence.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("3 engines completed settled build, source failure and repair");
    Ok(())
}
