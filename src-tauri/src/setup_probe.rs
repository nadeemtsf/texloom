//! Disposable compiler check shared by the pre- and post-install setup calls.
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use texloom_build::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
};
use uuid::Uuid;

const READY: i32 = 0;
const MISSING: i32 = 10;
const UNAVAILABLE: i32 = 11;
const ARTICLE: &str = include_str!("../../resources/templates/article.tex");
const MULTILINGUAL: &str = include_str!("../../resources/templates/multilingual.tex");

pub(crate) fn check(install_packages: bool) -> i32 {
    if install_packages {
        // Setup has permission to install dependencies for these bundled templates.
        for (engine, source) in [("pdflatex", ARTICLE), ("xelatex", MULTILINGUAL)] {
            let result = check_compilers(&compilers(&[engine]), source, true);
            if result != READY {
                return result;
            }
        }
        READY
    } else {
        check_compilers(
            &compilers(&["pdflatex", "xelatex", "lualatex"]),
            ARTICLE,
            false,
        )
    }
}

fn compilers(engines: &[&str]) -> Vec<PathBuf> {
    engines
        .iter()
        .filter_map(|engine| crate::service::find_program(engine))
        .filter(|path| path.is_absolute())
        .collect()
}

fn check_compilers(compilers: &[PathBuf], source: &str, install_packages: bool) -> i32 {
    if compilers.is_empty() {
        return MISSING;
    }

    let directory = std::env::temp_dir().join(format!("texloom-latex-check-{}", Uuid::new_v4()));
    if fs::create_dir(&directory).is_err() {
        return UNAVAILABLE;
    }
    let ready = fs::write(directory.join("sample.tex"), source).is_ok()
        && compilers
            .iter()
            .any(|compiler| compile_sample(compiler, &directory, install_packages));
    let _ = fs::remove_dir_all(&directory);
    if ready {
        READY
    } else {
        UNAVAILABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_and_unusable_compilers_have_distinct_setup_outcomes() {
        assert_eq!(check_compilers(&[], ARTICLE, false), MISSING);
        let vanished = std::env::temp_dir()
            .join(Uuid::new_v4().to_string())
            .join("pdflatex.exe");
        assert_eq!(check_compilers(&[vanished], ARTICLE, false), UNAVAILABLE);
    }
}

fn compile_sample(compiler: &Path, directory: &Path, install_packages: bool) -> bool {
    let run = |arguments: &[&str], seconds| {
        let args: Vec<_> = arguments.iter().map(|arg| (*arg).into()).collect();
        process::run(
            &CommandSpec {
                program: compiler,
                args: &args,
                directory,
                env: &BTreeMap::new(),
                timeout: Some(Duration::from_secs(seconds)),
                memory_bytes: 2 * 1024 * 1024 * 1024,
            },
            &Cancellation::default(),
        )
        .ok()
        .filter(|result| result.success)
    };
    let Some(version) = run(&["--version"], 5) else {
        return false;
    };
    let banner = version.text.to_ascii_lowercase();
    let mut arguments = vec![
        "-no-shell-escape",
        "-interaction=nonstopmode",
        "-halt-on-error",
    ];
    let miktex = banner.contains("miktex");
    if miktex {
        arguments.push(if install_packages {
            "--enable-installer"
        } else {
            "--disable-installer"
        });
    } else if !banner.contains("tex live") {
        return false;
    }
    // A failed attempt must never satisfy a later engine's sample check.
    let output = directory.join("sample.pdf");
    if output.exists() && fs::remove_file(&output).is_err() {
        return false;
    }
    arguments.push("sample.tex");
    let timeout = if miktex && install_packages { 180 } else { 20 };
    if run(&arguments, timeout).is_none() {
        return false;
    }
    let mut header = [0; 5];
    File::open(output)
        .and_then(|mut pdf| pdf.read_exact(&mut header))
        .is_ok()
        && &header == b"%PDF-"
}
