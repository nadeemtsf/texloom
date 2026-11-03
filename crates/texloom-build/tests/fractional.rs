//! Real-engine predictions use PDFium object transforms, never pixel interpolation.
#![cfg(windows)]
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use texloom_build::{
    engine::{self, Request},
    scheduler::Cancellation,
    snapshot::Fingerprint,
    verification,
};
mod support;
#[test]
#[ignore = "requires three installed TeX engines and the pinned Windows verifier"]
fn real_fractional_moves_match_independent_object_predictions() {
    let root = PathBuf::from(
        std::env::var_os("TEXLOOM_FRACTIONAL_REPORT_DIR").expect("Set fresh evidence directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let program = PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker"));
    let library =
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL"));
    let worker = support::owned_worker(&root, &program, library);
    let cancel = Cancellation::default();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let support = BTreeMap::from([(
        "texloom-objects.sty".into(),
        fs::read_to_string(repo.join("resources/texloom-objects.sty")).unwrap(),
    )]);
    let source = |dx: f64, dy: f64| {
        format!("\\documentclass{{article}}\n\\usepackage{{texloom-objects}}\n\\begin{{document}}\nBefore the signature.\n\n\\TexloomObject{{sig}}{{{dx}}}{{{dy}}}{{Dr. Example\\\\[2pt]Department\\\\[2pt]University}}\n\n\\vspace{{25mm}}\nUnchanged neighbor text.\n\\end{{document}}\n")
    };
    let mut rows = Vec::new();
    let mut failed = false;
    for name in ["pdflatex", "lualatex", "xelatex"] {
        let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join(format!("{name}.exe")))
            .find(|p| p.is_file())
            .expect("Engine is missing");
        let base = root.join(name);
        let project = base.join("project");
        fs::create_dir_all(&project).unwrap();
        let original = source(0.0, 0.0);
        fs::write(project.join("main.tex"), &original).unwrap();
        let build = |variant: &str, text: String| {
            let buffers = BTreeMap::from([("main.tex".into(), text)]);
            let out = engine::build(
                Request {
                    project_root: &project,
                    directory: &base.join(variant),
                    sources: &buffers,
                    main: "main.tex",
                    support: &support,
                    compiler: &compiler,
                    engine: name,
                },
                &cancel,
            )
            .unwrap();
            assert!(out.ok && out.eligible, "{name}/{variant}: {}", out.log);
            out.receipt.verify().unwrap();
            out.receipt
        };
        let baseline = build("baseline", original.clone());
        let pdf = baseline.directory.join(".out/preview.pdf");
        let sha = Fingerprint::file(&pdf).unwrap().sha256;
        let old = worker
            .inspect(&pdf, &sha, Some("Texloom_sig"), &cancel)
            .unwrap();
        for (index, (dx, dy)) in [(0.13, 4.27), (-1.37, 0.25), (3.0, 8.0)]
            .into_iter()
            .enumerate()
        {
            let candidate = build(&format!("moved-{index}"), source(dx, dy));
            let next_pdf = candidate.directory.join(".out/preview.pdf");
            let next_sha = Fingerprint::file(&next_pdf).unwrap().sha256;
            let next = worker
                .inspect(&next_pdf, &next_sha, Some("Texloom_sig"), &cancel)
                .unwrap();
            let predicted = worker
                .predict_displacement(&pdf, &sha, "Texloom_sig", dx, dy, &cancel)
                .unwrap();
            let result =
                verification::compare_prediction(&old.view, &next.view, dx, dy, &predicted.view);
            if result.is_err() {
                failed = true;
            }
            rows.push(serde_json::json!({"engine":name,"dx":dx,"dy":dy,"accepted":result.is_ok(),"detail":format!("{result:?}"),"baseline_pdf_sha256":sha,"candidate_pdf_sha256":next_sha}));
        }
        assert_eq!(
            fs::read_to_string(project.join("main.tex")).unwrap(),
            original
        );
    }
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    assert!(
        !failed,
        "Fractional prediction mismatch: see {}",
        root.join("report.json").display()
    );
}
