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
#[ignore = "requires three installed TeX engines and the pinned Windows verifier"]
fn real_parameter_adapters_compile_render_and_preserve_originals() {
    let root = PathBuf::from(
        std::env::var_os("TEXLOOM_ADAPTER_REPORT_DIR")
            .expect("Set fresh adapter evidence directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let program = PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker"));
    let library =
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL"));
    let worker = support::owned_worker(&root, &program, library);
    let cancel = Cancellation::default();
    // A complete tiny RGB PNG with its original bytes preserved through builds.
    let png = include_bytes!("../../../tests/fixtures/adapters/image.png");
    let source="\u{feff}\\documentclass{article}\r\n\\usepackage{graphicx,array,float}\r\n\\begin{document}\r\nBefore.\r\n\r\n\\includegraphics[width=20mm,keepaspectratio]{image.png}\r\n\r\n\\begin{tabular}{|p{20mm}|p{20mm}|}First column&Second column\\\\\\end{tabular}\r\n\r\n{\\fontsize{12}{14}\\selectfont Editable heading}\r\n\\vspace{2mm}\r\n\\begin{figure}[htbp]\\rule{10mm}{3mm}\\caption{Fixture}\\end{figure}\r\nNeighbor text.\r\n\\end{document}\r\n";
    let source=&source.replacen("\\begin{document}","\\usepackage{geometry}\r\n\\geometry{margin=25mm}\r\n\\linespread{1.05}\r\n\\begin{document}",1);
    let source = source.as_str();
    let controls = texloom_core::adapters::describe(source);
    let mut variants = vec![("baseline".to_string(), source.to_string())];
    for (kind, key, value) in [
        ("image", "width", "35mm"),
        ("table", "column_1", "30mm"),
        ("typography", "size", "16"),
        ("spacing", "length", "8mm"),
        ("float", "placement", "H"),
        ("layout", "margin", "22mm"),
    ] {
        let control = controls
            .iter()
            .find(|c| c.kind == kind && c.parameters.iter().any(|p| p.key == key))
            .expect("Missing adapter");
        let candidate =
            texloom_core::adapters::apply(source, source, control.id, key, value).unwrap();
        assert_ne!(candidate, source);
        variants.push((kind.into(), candidate));
    }
    let at = source.find("Editable heading").unwrap();
    let spread = controls
        .iter()
        .find(|c| c.parameters.iter().any(|p| p.key == "factor"))
        .unwrap();
    variants.push((
        "line-factor".into(),
        texloom_core::adapters::apply(source, source, spread.id, "factor", "1.15").unwrap(),
    ));
    variants.push((
        "style".into(),
        texloom_core::adapters::style(
            source,
            source,
            source,
            at..at + "Editable heading".len(),
            "bold",
        )
        .unwrap(),
    ));
    let mut rows = Vec::new();
    let mut failed = false;
    for name in ["pdflatex", "lualatex", "xelatex"] {
        let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join(format!("{name}.exe")))
            .find(|p| p.is_file())
            .expect("Engine missing");
        let base = root.join(name);
        let project = base.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("main.tex"), source).unwrap();
        fs::write(project.join("image.png"), png).unwrap();
        for (variant, text) in &variants {
            let sources = BTreeMap::from([("main.tex".into(), text.clone())]);
            let out = engine::build(
                Request {
                    project_root: &project,
                    directory: &base.join(variant),
                    sources: &sources,
                    main: "main.tex",
                    support: &BTreeMap::new(),
                    compiler: &compiler,
                    engine: name,
                },
                &cancel,
            )
            .unwrap();
            let checked = (|| -> Result<(), String> {
                if !out.ok || !out.eligible {
                    return Err("Compiler result is ineligible".into());
                }
                out.receipt.verify()?;
                let pdf = out.receipt.directory.join(".out/preview.pdf");
                let inspected =
                    worker.inspect(&pdf, &Fingerprint::file(&pdf)?.sha256, None, &cancel)?;
                verification::validate(&inspected.view).map_err(str::to_string)?;
                Ok(())
            })();
            let accepted = checked.is_ok();
            failed |= !accepted;
            println!("{name}/{variant}: {checked:?}");
            rows.push(serde_json::json!({"engine":name,"variant":variant,"accepted":accepted,"inspection_error":checked.err(),"passes":out.receipt.passes,"diagnostics":out.diagnostics,"log":out.log}));
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(
            fs::read(project.join("main.tex")).unwrap(),
            source.as_bytes()
        );
        assert_eq!(fs::read(project.join("image.png")).unwrap(), png);
    }
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    assert!(
        !failed,
        "An adapter failed; see {}",
        root.join("report.json").display()
    );
}
