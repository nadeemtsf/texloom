//! Explicit opt-in real-engine acceptance. CI supplies all three TeX engines.
use std::{collections::BTreeMap, fs, path::PathBuf};
use texloom_build::{
    engine::{self, Request},
    scheduler::Cancellation,
};
fn program(name: &str) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| {
            p.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            })
        })
        .find(|p| p.is_file())
        .expect("Required TeX engine missing")
}
#[test]
#[ignore = "requires installed pdfLaTeX, LuaLaTeX and XeLaTeX; run --ignored explicitly"]
fn real_three_engine_corpus() {
    let report_root =
        PathBuf::from(std::env::var_os("TEXLOOM_BUILD_REPORT_DIR").expect("Set report directory"));
    fs::create_dir_all(&report_root).unwrap();
    let cases = [
        (
            "visual-adapter",
            r"\TexloomObject{signature-name}{0}{5}{\textbf{Dr. Example Researcher}}",
        ),
        ("space-path", r"\input{chapter one.tex}"),
        ("asset", r"\includegraphics[width=40mm]{image.pdf}"),
        (
            "references",
            r"\tableofcontents See Section~\ref{sec:first}.\input{chapter.tex}",
        ),
        ("undefined", r"Unknown~\ref{missing}."),
        ("compile-error", r"\DefinitelyUndefinedCommand"),
        (
            "nonconvergent",
            r"\makeatletter\@ifundefined{testcounter}{\def\testcounter{0}}{}\count255=\testcounter\advance\count255 by1\immediate\write\@auxout{\string\gdef\string\testcounter{\the\count255}}\makeatother Changing aux.",
        ),
        (
            "bibliography",
            r"\cite{unknown}\bibliographystyle{plain}\bibliography{references}",
        ),
    ];
    let mut results = Vec::new();
    let mut failed = false;
    for engine in ["pdflatex", "lualatex", "xelatex"] {
        for (case, body) in cases {
            let root = report_root.join(engine).join(case);
            let project = root.join("project");
            fs::create_dir_all(&project).unwrap();
            if case == "asset" {
                fs::write(
                    project.join("image.pdf"),
                    include_bytes!("../../../tests/fixtures/objects/baseline.pdf"),
                )
                .unwrap();
            }
            let main_name = if case == "space-path" {
                "main document.tex"
            } else {
                "main.tex"
            };
            let package = if case == "visual-adapter" {
                "\\usepackage{texloom-objects}\n"
            } else {
                ""
            };
            let support = BTreeMap::from([(
                "texloom-objects.sty".into(),
                include_str!("../../../resources/texloom-objects.sty").into(),
            )]);
            let main = format!(
                "\\documentclass{{article}}\n\\usepackage{{graphicx}}\n{package}\\begin{{document}}\n{body}\n\\end{{document}}\n"
            );
            let sources = BTreeMap::from([
                (main_name.into(), main),
                ("chapter one.tex".into(), r"\section{First}Text.".into()),
                (
                    "chapter.tex".into(),
                    r"\section{First}\label{sec:first}Text.".into(),
                ),
                ("references.bib".into(), "".into()),
            ]);
            let directory = root.join("snapshot");
            let outcome = engine::build(
                Request {
                    project_root: &project,
                    directory: &directory,
                    sources: &sources,
                    main: main_name,
                    support: &support,
                    compiler: &program(engine),
                    engine,
                },
                &Cancellation::default(),
            );
            match outcome {
                Ok(out) => {
                    let accepted = match case {
                        "visual-adapter" => {
                            out.ok
                                && out.eligible
                                && out
                                    .receipt
                                    .used_project_inputs
                                    .contains("texloom-objects.sty")
                        }
                        "space-path" => out.ok && out.eligible,
                        "asset" => {
                            out.ok
                                && out.eligible
                                && out.receipt.used_project_inputs.contains("image.pdf")
                        }
                        "references" => out.ok && out.eligible && out.receipt.passes >= 2,
                        "undefined" => {
                            out.ok
                                && !out.eligible
                                && out
                                    .diagnostics
                                    .iter()
                                    .any(|d| d.code == "unresolved_reference")
                        }
                        "compile-error" => {
                            !out.ok
                                && out
                                    .diagnostics
                                    .iter()
                                    .any(|d| d.file.as_deref() == Some("main.tex"))
                        }
                        "nonconvergent" => {
                            !out.ok
                                && out.receipt.passes == 6
                                && out.diagnostics.iter().any(|d| d.code == "nonconvergence")
                        }
                        "bibliography" => {
                            out.ok&&!out.eligible&&out.receipt.auxiliary_tools.contains_key("bibtex")
                                && out
                                    .diagnostics
                                    .iter()
                                    .any(|d| d.code == "unresolved_reference"&&d.severity==texloom_build::engine::Severity::Error)
                        }
                        _ => false,
                    };
                    println!("{engine}/{case}: accepted={accepted}, ok={}, eligible={}, passes={}, deps={}",out.ok,out.eligible,out.receipt.passes,out.receipt.toolchain_inputs.len());
                    if !accepted {
                        println!("{}", out.log);
                        failed = true;
                    }
                    if out.ok {
                        out.receipt.verify().unwrap();
                        assert!(out.pdf.starts_with(b"%PDF-"));
                    }
                    results.push(serde_json::json!({"engine":engine,"case":case,"accepted":accepted,"ok":out.ok,"eligible":out.eligible,
                    "passes":out.receipt.passes,"version":out.receipt.engine_version,"diagnostics":out.diagnostics,"dependencies":out.receipt.toolchain_inputs.len()}));
                }
                Err(error) => {
                    failed = true;
                    println!("{engine}/{case}: {error}");
                    results.push(serde_json::json!({"engine":engine,"case":case,"accepted":false,"error":error}));
                }
            }
        }
    }
    fs::write(
        report_root.join("report.json"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
    println!("TEXLOOM_BUILD_REPORT_BEGIN\n{}\nTEXLOOM_BUILD_REPORT_END",serde_json::to_string_pretty(&results).unwrap());
    assert!(
        !failed,
        "Real engine corpus failed; inspect report.json and manifests"
    );
}
