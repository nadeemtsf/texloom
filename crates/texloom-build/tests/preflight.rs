use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use texloom_build::{
    engine::{self, Request},
    scheduler::Cancellation,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "texloom-preflight-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check_failure(mode: &str, code: &str) {
    let directory = Directory::new();
    let project = directory.0.join("project");
    fs::create_dir(&project).unwrap();
    let source =
        "\u{feff}\\documentclass{article}\r\n\\begin{document}Unchanged\r\n\\end{document}\r\n";
    fs::write(project.join("main.tex"), source).unwrap();
    let sources = BTreeMap::from([
        ("main.tex".into(), source.into()),
        ("compiler-fixture.sty".into(), mode.into()),
    ]);
    let output = engine::build(
        Request {
            project_root: &project,
            directory: &directory.0.join("snapshot"),
            sources: &sources,
            main: "main.tex",
            support: &BTreeMap::new(),
            compiler: Path::new(env!("CARGO_BIN_EXE_process_fixture")),
            engine: "pdflatex",
        },
        &Cancellation::default(),
    )
    .unwrap();
    assert!(!output.ok);
    assert!(!output.eligible);
    assert!(!output.converged);
    assert!(output.pdf.is_empty());
    assert_eq!(output.receipt.passes, 0);
    assert!(output.receipt.engine_version.is_empty());
    assert_eq!(output.receipt.arguments, ["--version"]);
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].code, code);
    assert!(output.diagnostics[0].message.len() < 200);
    assert!(output.log.contains("fixture-first"));
    assert!(output.log.contains("fixture-last"));
    assert!(output.log.len() < 300_000);
    assert!(output.receipt.outputs.is_empty());
    if mode == "denied" {
        assert!(output.receipt.directory.join(".out/preview.pdf").is_file());
    }
    output.receipt.verify().unwrap();
    assert!(output
        .receipt
        .directory
        .join("texloom-build-manifest.json")
        .is_file());
    assert_eq!(
        fs::read(project.join("main.tex")).unwrap(),
        source.as_bytes()
    );
    assert!(!project.join(".out").exists());
}

#[test]
fn nonzero_version_check_retains_trace_without_compiling_or_touching_the_project() {
    check_failure("denied", "compiler_preflight_access_denied");
}

#[test]
fn a_successful_exit_with_truncated_version_output_cannot_start_a_document_build() {
    check_failure("truncated", "compiler_preflight_output_truncated");
}
