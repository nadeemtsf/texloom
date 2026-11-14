use std::{collections::BTreeMap, fs, path::PathBuf};
use texloom_build::{
    engine::{self, Request},
    scheduler::Cancellation,
};

#[test]
#[ignore = "requires installed pdfLaTeX, BibTeX, Biber and biblatex"]
fn real_bibtex_and_biber_recipes_resolve_citations_and_seal_their_tools() {
    let root = PathBuf::from(
        std::env::var_os("TEXLOOM_BIBLIOGRAPHY_REPORT_DIR").expect("Set a fresh report directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| {
            p.join(if cfg!(windows) {
                "pdflatex.exe"
            } else {
                "pdflatex"
            })
        })
        .find(|p| p.is_file())
        .expect("pdfLaTeX missing");
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    for (tool, preamble, end) in [
        (
            "bibtex",
            "",
            "\\bibliographystyle{plain}\\bibliography{refs}",
        ),
        (
            "biber",
            "\\usepackage[backend=biber,style=numeric]{biblatex}\n\\addbibresource{refs.bib}",
            "\\printbibliography",
        ),
    ] {
        let project = root.join(tool).join("project");
        fs::create_dir_all(&project).unwrap();
        let main=format!("\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\nA verified citation~\\cite{{example}}.\n{end}\n\\end{{document}}\n");
        let bib="@book{example,author={Example, Alice},title={Verified Bibliography},year={2026},publisher={Fixture Press}}\n";
        let sources = BTreeMap::from([
            ("main.tex".into(), main.clone()),
            ("refs.bib".into(), bib.into()),
        ]);
        for (name, text) in &sources {
            fs::write(project.join(name), text).unwrap()
        }
        let outcome = engine::build(
            Request {
                project_root: &project,
                directory: &root.join(tool).join("build"),
                sources: &sources,
                main: "main.tex",
                support: &BTreeMap::new(),
                compiler: &compiler,
                engine: "pdflatex",
            },
            &Cancellation::default(),
        );
        match outcome {
            Ok(out) => {
                let accepted = out.ok
                    && out.eligible
                    && out.receipt.auxiliary_tools.contains_key(tool)
                    && out.pdf.starts_with(b"%PDF-");
                if !accepted {
                    failures.push(tool.to_string());
                }
                if out.ok {
                    out.receipt.verify().unwrap();
                }
                assert_eq!(fs::read_to_string(project.join("main.tex")).unwrap(), main);
                assert_eq!(fs::read_to_string(project.join("refs.bib")).unwrap(), bib);
                reports.push(serde_json::json!({"tool":tool,"accepted":accepted,"ok":out.ok,"eligible":out.eligible,"passes":out.receipt.passes,"diagnostics":out.diagnostics,"tools":out.receipt.auxiliary_tools,"log":out.log}));
            }
            Err(error) => {
                failures.push(tool.to_string());
                reports.push(serde_json::json!({"tool":tool,"accepted":false,"error":error}));
            }
        }
    }
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&reports).unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "Bibliography recipe failures: {failures:?}; see {}",
        root.join("report.json").display()
    );
}
