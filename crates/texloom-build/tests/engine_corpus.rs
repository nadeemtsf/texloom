//! Explicit opt-in real-engine acceptance. CI supplies all three TeX engines.
use std::{collections::BTreeMap, fs, path::PathBuf};
use texloom_build::{
    engine::{self, CompilerProfiles, Request},
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
fn first_pass_preview_is_unverified_and_cannot_hide_later_failure_or_cancellation() {
    let root = PathBuf::from(std::env::var_os("TEXLOOM_BUILD_REPORT_DIR").expect("Set report directory"))
        .join("first-pass-preview");
    fs::create_dir_all(&root).unwrap();
    let mut rows = Vec::new();
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let compiler = program(engine);
        for (case, body) in [
            ("plain", "A small synthetic compile."),
            ("references", r"\section{Example}\label{sec:example} Reference to section \ref{sec:example}."),
            ("contents", r"\tableofcontents\section{First section}First paragraph.\section{Second section}Second paragraph."),
            ("late-failure", r"\makeatletter\@ifundefined{texloompreviewseen}{}{\errmessage{Synthetic deliberate later-pass failure}}\immediate\write\@auxout{\string\gdef\string\texloompreviewseen{1}}\makeatother The first pass produced a readable PDF."),
            ("compile-error", r"\DefinitelyUndefinedCommand"),
            ("cancel-at-preview", r"\makeatletter\@ifundefined{previewcounter}{\def\previewcounter{0}}{}\count255=\previewcounter\advance\count255 by1\immediate\write\@auxout{\string\gdef\string\previewcounter{\the\count255}}\makeatother Cancellation after a readable first pass."),
            ("mutate-after-preview", "Frozen input remains protected after a readable first pass."),
        ] {
            let directory = root.join(engine).join(case);
            let project = directory.join("project");
            fs::create_dir_all(&project).unwrap();
            let source = format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n");
            fs::write(project.join("main.tex"), &source).unwrap();
            let sources = BTreeMap::from([("main.tex".into(), source)]);
            let support = BTreeMap::new();
            let baseline_path = directory.join("baseline");
            let observed_path = directory.join("observed");
            let request = |path| Request {
                project_root: &project, directory: path, sources: &sources, main: "main.tex",
                support: &support, compiler: &compiler, engine,
            };
            let profiles = engine::CompilerProfiles::default();
            let baseline = if case == "cancel-at-preview" || case == "mutate-after-preview" { None } else {
                Some(engine::build_with_profiles(request(&baseline_path), &Cancellation::default(), &profiles).unwrap())
            };
            let cancel = Cancellation::default();
            let mut events = Vec::new();
            let outcome = engine::build_with_preview(request(&observed_path), &cancel, Some(&profiles), &mut |preview| {
                assert!(events.is_empty(), "Only one first-pass event is allowed");
                assert!(preview.pdf.starts_with(b"%PDF-"));
                fs::write(directory.join("first-pass.pdf"), &preview.pdf).unwrap();
                events.push(serde_json::json!({"seconds":preview.seconds,
                    "pdf":texloom_build::snapshot::Fingerprint::bytes(&preview.pdf),
                    "diagnostics":preview.diagnostics}));
                if case == "cancel-at-preview" { cancel.cancel(); }
                if case == "mutate-after-preview" { fs::write(observed_path.join("main.tex"), "changed frozen input").unwrap(); }
            });
            assert_eq!(fs::read_to_string(project.join("main.tex")).unwrap(), sources["main.tex"]);
            if case == "cancel-at-preview" {
                assert_eq!(events.len(), 1);
                let error = outcome.err().expect("Callback cancellation must prevent final output");
                assert!(error.to_ascii_lowercase().contains("cancel"));
                let auxiliary = fs::read_to_string(observed_path.join(".out/preview.aux")).unwrap();
                assert!(auxiliary.contains(r"\gdef\previewcounter{1}"));
                assert!(!auxiliary.contains(r"\gdef\previewcounter{2}"));
                rows.push(serde_json::json!({"engine":engine,"case":case,"events":events,"error":error}));
                continue;
            }
            if case == "mutate-after-preview" {
                assert_eq!(events.len(), 1);
                let error = outcome.err().expect("Changed frozen input must prevent a final build");
                assert!(error.contains("Frozen build input changed"));
                rows.push(serde_json::json!({"engine":engine,"case":case,"events":events,"error":error}));
                continue;
            }
            let baseline = baseline.unwrap();
            let observed = outcome.unwrap();
            assert!(!baseline.timings.version_reused);
            assert_eq!(observed.timings.version_reused, baseline.eligible);
            assert_eq!(observed.ok, baseline.ok);
            assert_eq!(observed.eligible, baseline.eligible);
            assert_eq!(observed.receipt.passes, baseline.receipt.passes);
            assert_eq!(observed.receipt.arguments, baseline.receipt.arguments);
            assert_eq!(observed.receipt.used_project_inputs, baseline.receipt.used_project_inputs);
            assert_eq!(observed.converged, baseline.converged);
            assert_eq!(serde_json::to_value(&observed.diagnostics).unwrap(), serde_json::to_value(&baseline.diagnostics).unwrap());
            let expected_events = if case == "compile-error" { 0 } else { 1 };
            assert_eq!(events.len(), expected_events);
            if case == "late-failure" || case == "compile-error" {
                assert!(!observed.ok && !observed.eligible && observed.pdf.is_empty());
                assert_eq!(observed.receipt.passes, if case == "late-failure" {2} else {1});
            } else {
                assert!(observed.ok && observed.eligible && observed.converged);
                assert!(observed.receipt.passes >= 2);
                assert!(events[0]["seconds"].as_f64().unwrap() < observed.seconds);
                observed.receipt.verify().unwrap(); baseline.receipt.verify().unwrap();
                fs::write(directory.join("final.pdf"), &observed.pdf).unwrap();
                fs::write(directory.join("baseline.pdf"), &baseline.pdf).unwrap();
            }
            rows.push(serde_json::json!({"engine":engine,"case":case,"events":events,
                "ok":observed.ok,"eligible":observed.eligible,"passes":observed.receipt.passes,
                "arguments":observed.receipt.arguments,"version":observed.receipt.engine_version,
                "compiler":observed.receipt.compiler,"source":observed.receipt.inputs,
                "seconds":observed.seconds,"baseline_seconds":baseline.seconds,
                "version_reused":observed.timings.version_reused,
                "diagnostics":observed.diagnostics}));
        }
    }
    fs::write(root.join("report.json"), serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    println!("First-pass observer: {} engine/condition controls passed", rows.len());
}

#[test]
#[ignore = "requires installed pdfLaTeX, LuaLaTeX and XeLaTeX; run --ignored explicitly"]
fn a_confirmed_profile_survives_a_source_error_and_repair() {
    let root = PathBuf::from(std::env::var_os("TEXLOOM_BUILD_REPORT_DIR").expect("Set report directory"))
        .join("repair-profile");
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let folder = root.join(engine);
        let project = folder.join("project");
        fs::create_dir_all(&project).unwrap();
        let original = r"\documentclass{article}\begin{document}Fixture\end{document}";
        fs::write(project.join("main.tex"), original).unwrap();
        let compiler = program(engine);
        let profiles = CompilerProfiles::default();
        let support = BTreeMap::new();
        let build = |stage: &str, body: &str| {
            let directory = folder.join(stage);
            let source = format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n");
            let sources = BTreeMap::from([("main.tex".into(), source)]);
            engine::build_with_profiles(Request {
                project_root: &project, directory: &directory, sources: &sources, main: "main.tex",
                support: &support, compiler: &compiler, engine,
            }, &Cancellation::default(), &profiles).unwrap()
        };

        let cold_error = build("cold-error", r"\DefinitelyUndefinedCommand");
        assert!(!cold_error.ok && !cold_error.timings.version_reused);
        let first_valid = build("valid", "Valid source.");
        assert!(first_valid.eligible && !first_valid.timings.version_reused);
        let warm_error = build("warm-error", r"\DefinitelyUndefinedCommand");
        assert!(!warm_error.ok && !warm_error.eligible && warm_error.pdf.is_empty());
        assert!(warm_error.timings.version_reused);
        assert!(warm_error.diagnostics.iter().any(|d| d.code == "latex_error" && d.file.as_deref() == Some("main.tex")));
        let repaired = build("repair", "Valid repaired source.");
        assert!(repaired.eligible && repaired.timings.version_reused && repaired.pdf.starts_with(b"%PDF-"));
        repaired.receipt.verify().unwrap();
        assert_eq!(fs::read_to_string(project.join("main.tex")).unwrap(), original);
    }
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
