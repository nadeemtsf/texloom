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
#[ignore = "requires three TeX engines, bundled-template packages and pinned Windows PDFium"]
fn templates_and_document_classes_compile_without_changing_originals() {
    let root = PathBuf::from(
        std::env::var_os("TEXLOOM_TEMPLATE_REPORT_DIR")
            .expect("Set fresh template evidence directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let worker = support::owned_worker(
        &root,
        &PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker")),
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL")),
    );
    let cancel = Cancellation::default();
    let mut cases = Vec::new();
    for id in ["article", "letter", "cv", "multilingual"] {
        cases.push((
            id,
            texloom_core::templates::get(id).unwrap().source.to_string(),
        ));
    }
    cases.extend([
        ("letter-class","\\documentclass{letter}\n\\begin{document}\n\\begin{letter}{Example Recipient\\\\Example address}\n\\opening{Dear Recipient,}\nA synthetic letter paragraph with ordinary text.\n\\closing{Best regards,}\n\\end{letter}\n\\end{document}\n".into()),
        ("koma-class","\\documentclass{scrartcl}\n\\begin{document}\n\\section*{Synthetic KOMA-Script example}\nAn ordinary paragraph using this document class.\n\\end{document}\n".into()),
        ("ams-journal-class","\\documentclass{amsart}\n\\title{Synthetic journal article}\n\\author{Example Author}\n\\begin{document}\n\\maketitle\n\\section{Introduction}\nAn ordinary paragraph using the AMS article class.\n\\end{document}\n".into()),
    ]);
    let mut rows = Vec::new();
    let mut failed = false;
    for engine in ["pdflatex", "lualatex", "xelatex"] {
        let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join(format!("{engine}.exe")))
            .find(|p| p.is_file())
            .expect("Engine missing");
        for (id, text) in &cases {
            if *id == "multilingual" && engine != "xelatex" {
                continue;
            }
            let base = root.join(engine).join(id);
            let project = base.join("project");
            fs::create_dir_all(&project).unwrap();
            fs::write(project.join("main.tex"), text).unwrap();
            let sources = BTreeMap::from([("main.tex".into(), text.clone())]);
            let outcome = engine::build(
                Request {
                    project_root: &project,
                    directory: &base.join("build"),
                    sources: &sources,
                    main: "main.tex",
                    support: &BTreeMap::new(),
                    compiler: &compiler,
                    engine,
                },
                &cancel,
            );
            let row = match outcome {
                Ok(out) => {
                    let inspection =
                        (|| -> Result<usize, String> {
                            if !out.ok || !out.eligible {
                                return Err("Build is ineligible".into());
                            }
                            out.receipt.verify()?;
                            let pdf = out.receipt.directory.join(".out/preview.pdf");
                            let view = worker.inspect(
                                &pdf,
                                &Fingerprint::file(&pdf)?.sha256,
                                None,
                                &cancel,
                            )?;
                            verification::validate(&view.view).map_err(str::to_string)?;
                            if *id == "multilingual" {
                                for script in [('א', 'ת'), ('ا', 'ي')] {
                                    if !view.view.glyphs.iter().any(|g| {
                                        g.text.chars().any(|c| c >= script.0 && c <= script.1)
                                    }) {
                                        return Err(
                                        "A language script has no independently readable glyphs"
                                            .into(),
                                    );
                                    }
                                }
                            }
                            Ok(view.view.glyphs.len())
                        })();
                    serde_json::json!({"engine":engine,"case":id,"accepted":inspection.is_ok(),"inspection":format!("{inspection:?}"),"diagnostics":out.diagnostics,"log":out.log})
                }
                Err(e) => serde_json::json!({"engine":engine,"case":id,"accepted":false,"error":e}),
            };
            failed |= row["accepted"] != true;
            println!("{engine}/{id}: {}", row["accepted"]);
            rows.push(row);
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
            assert_eq!(fs::read_to_string(project.join("main.tex")).unwrap(), *text);
        }
    }
    assert!(
        !failed,
        "A template/class failed: see {}",
        root.join("report.json").display()
    );
}
