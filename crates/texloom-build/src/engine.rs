use crate::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
    snapshot::{self, Fingerprint, Receipt},
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}
impl Diagnostic {
    fn new(code: &str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            file: None,
            line: None,
        }
    }
}
pub struct Request<'a> {
    pub project_root: &'a Path,
    pub directory: &'a Path,
    pub sources: &'a BTreeMap<String, String>,
    pub main: &'a str,
    pub support: &'a BTreeMap<String, String>,
    pub compiler: &'a Path,
    pub engine: &'a str,
}
pub struct Outcome {
    pub ok: bool,
    pub eligible: bool,
    pub receipt: Receipt,
    pub diagnostics: Vec<Diagnostic>,
    pub log: String,
    pub pdf: Vec<u8>,
    pub seconds: f64,
    pub converged: bool,
}
const MEMORY: usize = 2 * 1024 * 1024 * 1024;
const MAX_PASSES: usize = 6;
const MAX_TIME: Duration = Duration::from_secs(120);
pub fn build(request: Request<'_>, cancel: &Cancellation) -> Result<Outcome, String> {
    let started = Instant::now();
    cancel.check()?;
    let mut receipt = Receipt::capture(
        request.project_root,
        request.directory,
        request.main,
        request.sources,
        request.support,
        request.compiler,
    )?;
    let mut environment = BTreeMap::from([
        ("openin_any".into(), "p".into()),
        ("openout_any".into(), "p".into()),
    ]);
    for key in [
        "SOURCE_DATE_EPOCH",
        "FORCE_SOURCE_DATE",
        "TZ",
        "TEXINPUTS",
        "BIBINPUTS",
        "TEXMFHOME",
        "TEXMFVAR",
        "TEXMFCNF",
        "OSFONTDIR",
    ] {
        if let Ok(value) = std::env::var(key) {
            environment.insert(key.into(), value);
        }
    }
    // These flags reduce accidental writes; they are not an untrusted-TeX sandbox.
    environment.insert(
        "TEXMFOUTPUT".into(),
        receipt.directory.join(".out").to_string_lossy().into(),
    );
    receipt.environment = environment.clone();
    let version_args = vec!["--version".into()];
    let version = process::run(
        &CommandSpec {
            program: &receipt.invocation,
            args: &version_args,
            directory: &receipt.directory,
            env: &environment,
            timeout: Duration::from_secs(10),
            memory_bytes: MEMORY,
        },
        cancel,
    )
    .map_err(|e| format!("Compiler preflight failed: {e}"))?;
    cancel.check()?;
    if !version.success {
        return Err(format!("Compiler preflight failed: {}", version.text));
    }
    receipt.engine_version = version.text.lines().take(3).collect::<Vec<_>>().join("\n");
    let miktex = version.text.to_ascii_lowercase().contains("miktex");
    let mut args: Vec<String> = [
        "-interaction=nonstopmode",
        "-halt-on-error",
        "-file-line-error",
        "-synctex=1",
        "-recorder",
        "-no-shell-escape",
        "-jobname=preview",
        "-output-directory=.out",
    ]
    .map(str::to_owned)
    .into();
    if miktex {
        args.push("--disable-installer".into());
    }
    args.push(request.main.into());
    receipt.arguments = args.clone();
    let mut previous = None;
    let mut converged = false;
    let mut success = false;
    let mut log = String::new();
    let mut diagnostics = Vec::new();
    let mut truncated = false;
    for pass in 1..=MAX_PASSES {
        cancel.check()?;
        receipt.verify()?;
        let remaining = MAX_TIME
            .checked_sub(started.elapsed())
            .ok_or("Compilation exceeded its 120-second budget")?;
        let output = process::run(
            &CommandSpec {
                program: &receipt.invocation,
                args: &args,
                directory: &receipt.directory,
                env: &environment,
                timeout: remaining.min(Duration::from_secs(90)),
                memory_bytes: MEMORY,
            },
            cancel,
        )
        .map_err(|e| format!("Compiler process failed: {e}"))?;
        cancel.check()?;
        receipt.passes = pass;
        // Per-process capture is bounded; keep this build's aggregate bounded too.
        log.push_str(&format!(
            "\n--- {} pass {pass} ---\n{}",
            request.engine, output.text
        ));
        if log.len() > 1_000_000 {
            let mut start = log.len() - 800_000;
            while !log.is_char_boundary(start) {
                start += 1;
            }
            log.drain(..start);
            truncated = true;
        }
        truncated |= output.truncated;
        diagnostics = parse_diagnostics(&output.text, request.sources);
        if output.timed_out {
            diagnostics.push(Diagnostic::new(
                "timeout",
                Severity::Error,
                "Compiler exceeded the time limit",
            ));
            break;
        }
        if !output.success {
            break;
        }
        receipt.verify()?;
        // Lua/font tools can create their standard versioned cache on first use.
        // Discover those declared roots after the pass, then hash every input.
        let roots = snapshot::toolchain_roots(&receipt.compiler.path);
        let discovered =
            receipt.record_inputs(&receipt.directory.join(".out/preview.fls"), &roots)?;
        receipt.verify()?;
        let mut state = auxiliary_state(&receipt.directory.join(".out"))?;
        for name in &receipt.generated_inputs {
            state.insert(
                PathBuf::from(name),
                Fingerprint::file(&receipt.directory.join(name))?,
            );
        }
        if pass >= 2
            && previous.as_ref() == Some(&state)
            && !discovered
            && !rerun_requested(&output.text)
        {
            converged = true;
            success = true;
            break;
        }
        previous = Some(state);
    }
    if !converged && receipt.passes == MAX_PASSES {
        diagnostics.push(Diagnostic::new(
            "nonconvergence",
            Severity::Error,
            format!("Document did not settle after {MAX_PASSES} passes"),
        ));
    }
    let aux = fs::read_to_string(receipt.directory.join(".out/preview.aux")).unwrap_or_default();
    if aux.contains("\\bibdata") || receipt.directory.join(".out/preview.bcf").exists() {
        diagnostics.push(Diagnostic::new("bibliography_tool_required",Severity::Error,
            "This direct-engine recipe does not run BibTeX/Biber. Build the bibliography externally or use a supported prebuilt bibliography input."));
    }
    if truncated {
        diagnostics.push(Diagnostic::new(
            "log_truncated",
            Severity::Error,
            "Compiler output was truncated or interrupted; verification is unavailable",
        ));
    }
    let pdf_path = receipt.directory.join(".out/preview.pdf");
    let pdf = if success && pdf_path.is_file() {
        if fs::metadata(&pdf_path).map_err(|e| e.to_string())?.len() > 60 * 1024 * 1024 {
            return Err("Compiled PDF exceeds 60 MiB".into());
        }
        fs::read(pdf_path).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    success &= !pdf.is_empty();
    if !success && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new(
            "compiler_exit",
            Severity::Error,
            "Compiler did not finish successfully; see its output",
        ));
    }
    let eligible =
        success && converged && !diagnostics.iter().any(|d| d.severity == Severity::Error);
    receipt.seal_outputs()?;
    receipt.persist()?;
    cancel.check()?;
    if started.elapsed() >= MAX_TIME {
        return Err("Compilation exceeded its 120-second budget".into());
    }
    Ok(Outcome {
        ok: success,
        eligible,
        receipt,
        diagnostics,
        log,
        pdf,
        seconds: started.elapsed().as_secs_f64(),
        converged,
    })
}
fn auxiliary_state(root: &Path) -> Result<BTreeMap<PathBuf, Fingerprint>, String> {
    fn visit(
        root: &Path,
        dir: &Path,
        out: &mut BTreeMap<PathBuf, Fingerprint>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
                return Err("Symlink in compiler outputs".into());
            }
            if path.is_dir() {
                visit(root, &path, out)?;
            } else if ["aux", "toc", "out", "lof", "lot", "nav", "snm", "bcf"]
                .contains(&path.extension().and_then(|s| s.to_str()).unwrap_or(""))
            {
                out.insert(
                    path.strip_prefix(root).map_err(|e| e.to_string())?.into(),
                    Fingerprint::file(&path)?,
                );
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out)?;
    Ok(out)
}
fn rerun_requested(log: &str) -> bool {
    let log = log.to_ascii_lowercase();
    [
        "rerun to get",
        "label(s) may have changed",
        "please (re)run latex",
        "please rerun latex",
        "rerunfilecheck warning",
    ]
    .iter()
    .any(|s| log.contains(s))
}
pub fn parse_diagnostics(log: &str, sources: &BTreeMap<String, String>) -> Vec<Diagnostic> {
    let mut result = Vec::new();
    for raw in log.lines() {
        let lower = raw.to_ascii_lowercase();
        let mut location = None;
        for name in sources.keys() {
            if let Some(pos) = raw.find(&format!("{name}:")) {
                let tail = &raw[pos + name.len() + 1..];
                if let Some((number, message)) = tail.split_once(':') {
                    if let Ok(line) = number.parse::<usize>() {
                        location = Some((name.clone(), line, message.trim().to_string()));
                        break;
                    }
                }
            }
        }
        let mut d = if let Some((file, line, message)) = location {
            let mut d = Diagnostic::new("latex_error", Severity::Error, message);
            d.file = Some(file);
            d.line = Some(line);
            Some(d)
        } else if lower.contains("undefined references")
            || lower.contains("undefined citations")
            || lower.contains("citation") && lower.contains("undefined")
            || lower.contains("reference") && lower.contains("undefined")
        {
            Some(Diagnostic::new(
                "unresolved_reference",
                Severity::Error,
                raw.trim(),
            ))
        } else if lower.contains("missing character:") {
            Some(Diagnostic::new(
                "missing_glyph",
                Severity::Error,
                raw.trim(),
            ))
        } else if raw.starts_with('!') {
            Some(Diagnostic::new("latex_error", Severity::Error, raw.trim()))
        } else if lower.contains("overfull") {
            Some(Diagnostic::new(
                "overfull_box",
                Severity::Warning,
                raw.trim(),
            ))
        } else if lower.contains("warning:") {
            Some(Diagnostic::new(
                "latex_warning",
                Severity::Warning,
                raw.trim(),
            ))
        } else {
            None
        };
        if let Some(ref mut d) = d {
            if lower.contains("not found") && lower.contains(".sty") {
                d.code = "missing_package".into();
            }
        }
        if let Some(d) = d {
            if result.len() < 200 {
                result.push(d);
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn errors_have_source_locations_and_ordinary_warnings_are_not_errors() {
        let sources = BTreeMap::from([("chapter one.tex".into(), "".into())]);
        let d=parse_diagnostics("C:/private/build/chapter one.tex:17: Undefined control sequence.\nOverfull \\hbox in paragraph\nLaTeX Warning: There were undefined references.\nMissing character: There is no X\n", &sources);
        assert_eq!(d[0].file.as_deref(), Some("chapter one.tex"));
        assert_eq!(d[0].line, Some(17));
        assert_eq!(d[1].severity, Severity::Warning);
        assert_eq!(d[2].code, "unresolved_reference");
        assert_eq!(d[3].code, "missing_glyph");
    }
    #[test]
    fn known_rerun_messages_prevent_early_convergence() {
        assert!(rerun_requested(
            "LaTeX Warning: Label(s) may have changed. Rerun to get cross-references right."
        ));
        assert!(!rerun_requested("Overfull \\hbox (1pt too wide)"));
    }
}
