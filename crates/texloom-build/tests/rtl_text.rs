//! Real XeLaTeX Arabic/Hebrew glyph ownership and independent displacement.
#![cfg(windows)]
use std::{collections::BTreeMap, fs, path::PathBuf};
use texloom_build::{
    engine::{self, Request},
    scheduler::Cancellation,
    snapshot::Fingerprint,
    verification,
};
mod support;
#[test]
#[ignore = "requires XeLaTeX and the pinned Windows PDFium worker"]
fn rtl_paragraphs_keep_identity_and_match_independent_movement() {
    let root = PathBuf::from(
        std::env::var_os("TEXLOOM_RTL_REPORT_DIR").expect("Set fresh RTL report directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| p.join("xelatex.exe"))
        .find(|p| p.is_file())
        .unwrap();
    let worker = support::owned_worker(
        &root,
        &PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker")),
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").unwrap()),
    );
    let template = fs::read_to_string(repo.join("resources/templates/multilingual.tex")).unwrap();
    let styles = BTreeMap::from([(
        "texloom-objects.sty".into(),
        fs::read_to_string(repo.join("resources/texloom-objects.sty")).unwrap(),
    )]);
    let cancel = Cancellation::default();
    let mut rows = Vec::new();
    for (name, text) in [
        ("arabic", "مرحبا بالعالم. هذا نص عربي للتجربة."),
        ("hebrew", "שלום עולם. זהו טקסט בעברית לבדיקה."),
    ] {
        let base = root.join(name);
        let project = base.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("main.tex"), &template).unwrap();
        let build = |variant: &str, dx: f64, dy: f64, marked: bool| {
            let source = if marked {
                template
                    .replace(
                        "\\begin{document}",
                        "\\usepackage{texloom-objects}\n\\begin{document}",
                    )
                    .replacen(
                        text,
                        &format!("\\TexloomObject{{rtl-{name}}}{{{dx}}}{{{dy}}}{{{text}}}"),
                        1,
                    )
            } else {
                template.clone()
            };
            let sources = BTreeMap::from([("main.tex".into(), source)]);
            let output = engine::build(
                Request {
                    project_root: &project,
                    directory: &base.join(variant),
                    sources: &sources,
                    main: "main.tex",
                    support: &styles,
                    compiler: &compiler,
                    engine: "xelatex",
                },
                &cancel,
            )
            .unwrap();
            assert!(
                output.ok && output.eligible,
                "{name}/{variant}: {}",
                output.log
            );
            output.receipt.verify().unwrap();
            output.receipt.directory.join(".out/preview.pdf")
        };
        let original = build("unmarked", 0.0, 0.0, false);
        let zero = build("zero", 0.0, 0.0, true);
        let marker = format!("Texloom_rtl-{name}");
        let inspect = |pdf: &std::path::Path, tag: Option<&str>| {
            worker
                .inspect(pdf, &Fingerprint::file(pdf).unwrap().sha256, tag, &cancel)
                .unwrap()
        };
        let a = inspect(&original, None);
        let b = inspect(&zero, Some(&marker));
        assert_eq!(a.view.rasters.len(), b.view.rasters.len());
        for (p, q) in a.view.rasters.iter().zip(&b.view.rasters) {
            assert_eq!(
                p.pixels, q.pixels,
                "Zero-offset probe changed independent page paint"
            );
        }
        for (index, (dx, dy)) in [(0.0, 8.0), (1.25, 0.5)].into_iter().enumerate() {
            let moved = build(&format!("moved-{index}"), dx, dy, true);
            let next = inspect(&moved, Some(&marker));
            let prediction = worker
                .predict_displacement(
                    &zero,
                    &Fingerprint::file(&zero).unwrap().sha256,
                    &marker,
                    dx,
                    dy,
                    &cancel,
                )
                .unwrap();
            let result =
                verification::compare_prediction(&b.view, &next.view, dx, dy, &prediction.view);
            rows.push(serde_json::json!({"language":name,"engine":"xelatex","dx":dx,"dy":dy,"accepted":result.is_ok(),"detail":format!("{result:?}")}));
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
            assert!(result.is_ok(), "{name}/{dx}/{dy}: {result:?}");
        }
        assert_eq!(
            fs::read_to_string(project.join("main.tex")).unwrap(),
            template
        );
    }
}
