//! Run the production build service against disposable synthetic projects.
use serde_json::json;
use std::{collections::BTreeMap, env, fs, path::PathBuf, time::Duration};
use texloom_build::{
    engine::{self, CompilerProfiles, Request},
    scheduler::Cancellation,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let output = PathBuf::from(args.next().ok_or("Provide a new output directory")?);
    let compiler = PathBuf::from(args.next().ok_or("Provide the compiler executable")?);
    let repeats: usize = args.next().ok_or("Provide repeat count")?.to_string_lossy().parse()?;
    let mode = args.next();
    let scaling = mode.as_deref().is_some_and(|option| option == "--pages");
    let (long, warm, paired) = match mode.as_deref() {
        None => (false, false, false),
        Some(option) if option == "--long" => (true, false, false),
        Some(option) if option == "--warm" => (false, true, false),
        Some(option) if option == "--paired" || option == "--pages" => (false, false, true),
        Some(option) if option == "--alternating-projects" => (false, false, false),
        _ => return Err("Unknown argument".into()),
    };
    if !compiler.is_absolute() || !compiler.is_file() || !(1..=30).contains(&repeats) || args.next().is_some() {
        return Err("Expected absolute compiler path and 1–30 repeats".into());
    }
    let engine_name = compiler.file_stem().and_then(|s| s.to_str()).ok_or("Missing engine name")?;
    if long && engine_name != "lualatex" {
        return Err("The long control requires LuaLaTeX".into());
    }
    fs::create_dir(&output)?;
    if mode.as_deref().is_some_and(|option| option == "--alternating-projects") {
        return alternating_projects(&output, &compiler, engine_name, repeats);
    }
    let fixtures: &[&str] = if long {
        &["long"]
    } else if scaling {
        &["pages-1", "pages-20", "pages-100"]
    } else {
        &["plain", "references", "large-project"]
    };
    for &case in fixtures {
        let profiles = CompilerProfiles::default();
        let case_dir = output.join(case);
        let project = case_dir.join("project");
        fs::create_dir_all(&project)?;
        let pages = case.strip_prefix("pages-").map(str::parse::<usize>).transpose()?;
        let body = match (pages, case) {
            (Some(pages), _) => (1..=pages)
                .map(|page| format!("Page {page:03}. A repeatable compiler workload."))
                .collect::<Vec<_>>()
                .join("\n\\newpage\n"),
            (_, "long") => r"\makeatletter
\@ifundefined{texloomprofileseen}{\directlua{os.sleep(121)}}{}
\immediate\write\@auxout{\string\gdef\string\texloomprofileseen{1}}
\makeatother
Long compile completed.".into(),
            (_, "references") => r"\section{Example}\label{sec:example}
Reference to section \ref{sec:example}.".into(),
            _ => "A small synthetic compile.".into(),
        };
        let source = format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n");
        fs::write(project.join("main.tex"), &source)?;
        let mut sources = BTreeMap::from([("main.tex".into(), source)]);
        if case == "large-project" {
            for i in 0..24 {
                let name = format!("unused-{i}.tex");
                let comment = format!("%{}\n", "a".repeat(1024 * 1024 - 2));
                sources.insert(name, comment);
            }
        }
        let count = if long { 1 } else { repeats + 1 };
        for trial in 0..count {
            let conditions: &[bool] = if paired {
                if trial % 2 == 0 { &[false, true] } else { &[true, false] }
            } else if warm {
                &[true]
            } else {
                &[false]
            };
            for &use_profile in conditions {
                let mode = if use_profile { "warm" } else { "fresh" };
                let name = if paired { format!("run-{trial:02}-{mode}") } else { format!("run-{trial:02}") };
                let directory = case_dir.join(name);
                let request = Request {
                    project_root: &project,
                    directory: &directory,
                    sources: &sources,
                    main: "main.tex",
                    support: &BTreeMap::new(),
                    compiler: &compiler,
                    engine: engine_name,
                };
                let result = measure(request, use_profile.then_some(&profiles))?;
                if result.timings.version_reused != (use_profile && trial > 0) {
                    return Err("Unexpected version probe reuse".into());
                }
                if !result.ok || !result.eligible || !result.converged {
                    return Err(format!("{case} trial {trial} refused: {:?}", result.diagnostics).into());
                }
                result.receipt.verify()?;
                if fs::read_to_string(project.join("main.tex"))? != sources["main.tex"] {
                    return Err("Original source changed".into());
                }
                if long && result.timings.first_pass_seconds <= 120.0 {
                    return Err("The first TeX pass did not exceed 120 seconds".into());
                }
                println!("{}", json!({
                    "case": case, "trial": trial, "warmup": !long && trial == 0, "profile": mode,
                    "engine": engine_name, "version": result.receipt.engine_version,
                    "passes": result.receipt.passes, "toolchain_inputs": result.receipt.toolchain_inputs.len(),
                    "pdf_bytes": result.pdf.len(), "seconds": result.seconds, "timings": result.timings,
                }));
            }
        }
    }
    Ok(())
}

fn alternating_projects(
    output: &std::path::Path,
    compiler: &std::path::Path,
    engine_name: &str,
    repeats: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = CompilerProfiles::default();
    for name in ["alpha", "beta"] {
        let project = output.join(name).join("project");
        fs::create_dir_all(&project)?;
        let source = format!("\\documentclass{{article}}\n\\begin{{document}}\nProject {name}.\n\\end{{document}}\n");
        fs::write(project.join("main.tex"), source)?;
    }
    for trial in 0..=repeats {
        for name in ["alpha", "beta"] {
            let project = output.join(name).join("project");
            let path = project.join("main.tex");
            let source = fs::read_to_string(&path)?;
            let sources = BTreeMap::from([("main.tex".into(), source.clone())]);
            let arms = if trial % 2 == 0 { [false, true] } else { [true, false] };
            for use_cache in arms {
                let arm = if use_cache { "retained" } else { "fresh" };
                let directory = output.join(name).join(format!("run-{trial:02}-{arm}"));
                let result = measure(Request {
                    project_root: &project,
                    directory: &directory,
                    sources: &sources,
                    main: "main.tex",
                    support: &BTreeMap::new(),
                    compiler,
                    engine: engine_name,
                }, use_cache.then_some(&profiles))?;
                if !result.eligible || result.timings.version_reused != (use_cache && trial > 0) {
                    return Err(format!("{name}/{trial}/{arm}: unexpected build result or probe reuse").into());
                }
                result.receipt.verify()?;
                if fs::read_to_string(&path)? != source {
                    return Err("Original source changed".into());
                }
                println!("{}", json!({
                    "project": name, "trial": trial, "warmup": trial == 0, "arm": arm,
                    "engine": engine_name, "version": result.receipt.engine_version,
                    "seconds": result.seconds, "timings": result.timings,
                    "pdf": directory.join(".out/preview.pdf"),
                    "receipt": directory.join("texloom-build-manifest.json"),
                }));
            }
        }
    }
    Ok(())
}

fn measure(request: Request<'_>, profiles: Option<&CompilerProfiles>) -> Result<engine::Outcome, String> {
    let cancel = Cancellation::default();
    let watchdog_cancel = cancel.clone();
    let (done, wait) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if matches!(wait.recv_timeout(Duration::from_secs(180)), Err(std::sync::mpsc::RecvTimeoutError::Timeout)) {
            watchdog_cancel.cancel();
        }
    });
    let result = match profiles {
        Some(profiles) => engine::build_with_profiles(request, &cancel, profiles),
        None => engine::build(request, &cancel),
    };
    let _ = done.send(());
    watchdog.join().map_err(|_| "Benchmark watchdog panicked")?;
    result
}
