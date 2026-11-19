use crate::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
    snapshot::{self, Fingerprint, Receipt},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

const MAX_PDF_BYTES: u64 = 60 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum Severity {
    Error,
    Warning,
    Info,
}
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
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
    pub timings: BuildTimings,
}
/// First-pass PDF bytes, without convergence, native approval or mapping
/// authority. Consumers must parse them before presentation. A later pass
/// can still change the PDF or fail the entire build.
pub struct UnverifiedPreview {
    pub pdf: Vec<u8>,
    pub diagnostics: Vec<Diagnostic>,
    pub seconds: f64,
}
/// Native build timings. Includes child startup and exit in process intervals.
#[derive(Default, Serialize)]
pub struct BuildTimings {
    pub snapshot_seconds: f64,
    pub version_seconds: f64,
    pub version_reused: bool,
    pub engine_seconds: f64,
    pub first_pass_seconds: f64,
    pub verify_seconds: f64,
    pub recorder_seconds: f64,
    pub bibliography_seconds: f64,
    pub finalize_seconds: f64,
}

/// Recent successful identifications for distinct projects and engine commands.
/// Every build verifies its inputs and runs TeX on a fresh snapshot.
#[derive(Default)]
pub struct CompilerProfiles(Mutex<VecDeque<CompilerProfile>>);

const COMPILER_PROFILE_LIMIT: usize = 8;

struct CompilerProfile {
    project_root: PathBuf,
    invocation: PathBuf,
    compiler: snapshot::Dependency,
    environment: BTreeMap<String, String>,
    version: String,
    miktex: bool,
    runtime: String,
}

fn probe_environment(environment: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    // These values are constructed from the fresh snapshot directory on each
    // build. They affect compilation but don't identify the engine version.
    environment.iter()
        .filter(|(key, _)| !matches!(key.as_str(), "TEXMFOUTPUT" | "BIBINPUTS" | "BSTINPUTS"))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

impl CompilerProfile {
    fn same_project_and_command(&self, receipt: &Receipt) -> bool {
        self.project_root == receipt.project_root && self.invocation == receipt.invocation
    }

    fn matches_inputs(&self, receipt: &Receipt) -> bool {
        self.compiler.path == receipt.compiler.path
            && self.compiler.fingerprint == receipt.compiler.fingerprint
            && self.environment == probe_environment(&receipt.environment)
    }
}

impl CompilerProfiles {
    fn take(&self, receipt: &Receipt) -> Option<CompilerProfile> {
        let mut cached = self.0.lock().ok()?;
        let index = cached.iter().position(|profile| profile.same_project_and_command(receipt))?;
        // Remove mismatched entries too, so restoring an older environment
        // after a failed build cannot bring back its old identification.
        // Restoring a matched profile requires a confirmed runtime banner.
        cached.remove(index).filter(|profile| profile.matches_inputs(receipt))
    }

    fn remember(&self, receipt: &Receipt, miktex: bool, runtime: String) {
        let identified = receipt.engine_version.lines().next()
            .is_some_and(|line| line.contains("MiKTeX") || line.contains("TeX Live"));
        if !identified {
            return;
        }
        if let Ok(mut cached) = self.0.lock() {
            cached.retain(|profile| !profile.same_project_and_command(receipt));
            cached.push_front(CompilerProfile {
                project_root: receipt.project_root.clone(),
                invocation: receipt.invocation.clone(),
                compiler: receipt.compiler.clone(),
                environment: probe_environment(&receipt.environment),
                version: receipt.engine_version.clone(),
                miktex,
                runtime,
            });
            cached.truncate(COMPILER_PROFILE_LIMIT);
        }
    }
}

fn stable_runtime_banner(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("This is ") || !(line.contains("MiKTeX") || line.contains("TeX Live")) {
        return None;
    }
    // The first log line includes a timestamp after the engine/format banner.
    // Keep the format name and version, including the format's own date.
    Some(match line.rsplit_once(")  ") {
        Some((head, _)) => format!("{head})"),
        None => line.into(),
    })
}

fn runtime_banner(directory: &Path) -> Option<String> {
    let path = directory.join(".out/preview.log");
    snapshot::check_plain(&path).ok()?;
    let mut reader = BufReader::new(fs::File::open(path).ok()?.take(4096));
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    stable_runtime_banner(&line)
}
const MEMORY: usize = 2 * 1024 * 1024 * 1024;
const MAX_PASSES: usize = 6;
pub fn bibliography_search_path(root:&Path)->String { crate::bibliography::search_path(root) }
pub fn build(request: Request<'_>, cancel: &Cancellation) -> Result<Outcome, String> {
    build_inner(request, cancel, None, None)
}
pub fn build_with_profiles(
    request: Request<'_>,
    cancel: &Cancellation,
    profiles: &CompilerProfiles,
) -> Result<Outcome, String> {
    build_inner(request, cancel, Some(profiles), None)
}
/// Development observer for a future provisional view. Ordinary builds do
/// not read the first PDF and retain the existing final-result contract.
pub fn build_with_preview(
    request: Request<'_>,
    cancel: &Cancellation,
    profiles: Option<&CompilerProfiles>,
    preview: &mut dyn FnMut(UnverifiedPreview),
) -> Result<Outcome, String> {
    build_inner(request, cancel, profiles, Some(preview))
}
fn build_inner(
    request: Request<'_>,
    cancel: &Cancellation,
    profiles: Option<&CompilerProfiles>,
    mut preview: Option<&mut dyn FnMut(UnverifiedPreview)>,
) -> Result<Outcome, String> {
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
    let mut timings = BuildTimings {
        snapshot_seconds: started.elapsed().as_secs_f64(),
        ..BuildTimings::default()
    };
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
    for key in ["BIBINPUTS","BSTINPUTS"] {environment.insert(key.into(),bibliography_search_path(&receipt.directory));}
    receipt.environment = environment.clone();
    let version_started = Instant::now();
    let mut expected_runtime = None;
    let (version, miktex) = if let Some(profile) = profiles.and_then(|p| p.take(&receipt)) {
        timings.version_reused = true;
        expected_runtime = Some(profile.runtime);
        (profile.version, profile.miktex)
    } else {
        let version_args = vec!["--version".into()];
        let version = process::run(
            &CommandSpec {
                program: &receipt.invocation,
                args: &version_args,
                directory: &receipt.directory,
                env: &environment,
                timeout: Some(Duration::from_secs(10)),
                memory_bytes: MEMORY,
            },
            cancel,
        )
        .map_err(|e| format!("Compiler preflight failed: {e}"))?;
        cancel.check()?;
        if !version.success || version.timed_out || version.truncated {
            timings.version_seconds = version_started.elapsed().as_secs_f64();
            let diagnostic = preflight_diagnostic(&version);
            receipt.arguments = version_args;
            receipt.verify()?;
            receipt.persist()?;
            cancel.check()?;
            return Ok(Outcome {
                ok: false,
                eligible: false,
                receipt,
                diagnostics: vec![diagnostic],
                log: version.text,
                pdf: Vec::new(),
                seconds: started.elapsed().as_secs_f64(),
                converged: false,
                timings,
            });
        }
        let banner = version.text.lines().take(3).collect::<Vec<_>>().join("\n");
        (banner, version.text.to_ascii_lowercase().contains("miktex"))
    };
    receipt.engine_version = version;
    timings.version_seconds = version_started.elapsed().as_secs_f64();
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
    let mut bibliography_digest = None;
    let mut bibliography_disabled = false;
    let mut bibliography_errors = Vec::new();
    let mut matched_runtime_after_source_error = false;
    for pass in 1..=MAX_PASSES {
        cancel.check()?;
        let verify_started = Instant::now();
        receipt.verify()?;
        timings.verify_seconds += verify_started.elapsed().as_secs_f64();
        let engine_started = Instant::now();
        let output = process::run(
            &CommandSpec {
                program: &receipt.invocation,
                args: &args,
                directory: &receipt.directory,
                env: &environment,
                timeout: None,
                memory_bytes: MEMORY,
            },
            cancel,
        )
        .map_err(|e| format!("Compiler process failed: {e}"))?;
        let pass_seconds = engine_started.elapsed().as_secs_f64();
        timings.engine_seconds += pass_seconds;
        if pass == 1 {
            timings.first_pass_seconds = pass_seconds;
        }
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
        if !output.success {
            // An attributed TeX error can confirm an existing identity. Only a
            // successful, eligible build can add a new compiler profile.
            matched_runtime_after_source_error = timings.version_reused
                && !truncated
                && diagnostics.iter().any(|diagnostic| diagnostic.code == "latex_error" && diagnostic.file.is_some())
                && expected_runtime.as_ref().is_some_and(|expected| {
                    runtime_banner(&receipt.directory).as_ref() == Some(expected)
                });
            break;
        }
        if profiles.is_some() {
            let runtime = runtime_banner(&receipt.directory);
            if expected_runtime.is_some() && expected_runtime != runtime {
                return Err("The TeX engine or loaded format changed. Compile again to refresh the engine check.".into());
            }
            expected_runtime = runtime;
        }
        // Lua/font tools can create their standard versioned cache on first use.
        // Discover those declared roots after the pass, then hash every input.
        let recorder_started = Instant::now();
        let roots = snapshot::toolchain_roots(&receipt.compiler.path);
        let discovered =
            receipt.record_inputs(&receipt.directory.join(".out/preview.fls"), &roots)?;
        timings.recorder_seconds += recorder_started.elapsed().as_secs_f64();
        let verify_started = Instant::now();
        receipt.verify()?;
        timings.verify_seconds += verify_started.elapsed().as_secs_f64();
        if pass == 1 && !truncated {
            if let Some(observer) = preview.as_deref_mut() {
                cancel.check()?;
                if let Some(pdf) = unverified_pdf(&receipt.directory.join(".out/preview.pdf")) {
                    observer(UnverifiedPreview {
                        pdf,
                        diagnostics: diagnostics.clone(),
                        seconds: started.elapsed().as_secs_f64(),
                    });
                }
                cancel.check()?;
            }
        }
        let mut bibliography_ran=false;
        if !bibliography_disabled {
            match crate::bibliography::requested(&receipt) {
                Ok(Some(recipe)) if bibliography_digest.as_ref()!=Some(&recipe.digest) => {
                    bibliography_digest=Some(recipe.digest.clone());
                    let bibliography_started = Instant::now();
                    let result = crate::bibliography::execute(&mut receipt, &recipe, cancel);
                    timings.bibliography_seconds += bibliography_started.elapsed().as_secs_f64();
                    match result {
                        Ok(output)=>{log.push_str(&output);bibliography_ran=true;if log.len()>1_000_000 {let mut start=log.len()-800_000;while !log.is_char_boundary(start){start+=1}log.drain(..start);truncated=true;}},
                        Err(error)=>{cancel.check()?;bibliography_errors.push(Diagnostic::new("bibliography_tool_failed",Severity::Error,format!("Bibliography: {error}")));bibliography_disabled=true;}
                    }
                },
                Err(error)=>{bibliography_errors.push(Diagnostic::new("bibliography_input_refused",Severity::Error,error));bibliography_disabled=true;},_=>()
            }
        }
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
            && !bibliography_ran
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
    diagnostics.extend(bibliography_errors);
    if truncated {
        diagnostics.push(Diagnostic::new(
            "log_truncated",
            Severity::Error,
            "Compiler output was truncated or interrupted; verification is unavailable",
        ));
    }
    let finalize_started = Instant::now();
    let pdf_path = receipt.directory.join(".out/preview.pdf");
    let delivered_pdf = if success && pdf_path.is_file() {
        Some(read_completed_pdf(&pdf_path)?)
    } else {
        None
    };
    receipt.seal_outputs_with_pdf(delivered_pdf.as_deref())?;
    let pdf = delivered_pdf.unwrap_or_default();
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
    receipt.persist()?;
    cancel.check()?;
    if eligible || matched_runtime_after_source_error {
        if let (Some(profiles), Some(runtime)) = (profiles, expected_runtime) {
            profiles.remember(&receipt, miktex, runtime);
        }
    }
    timings.finalize_seconds = finalize_started.elapsed().as_secs_f64();
    Ok(Outcome {
        ok: success,
        eligible,
        receipt,
        diagnostics,
        log,
        pdf,
        seconds: started.elapsed().as_secs_f64(),
        converged,
        timings,
    })
}
fn read_completed_pdf(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let len = file.metadata().map_err(|error| error.to_string())?.len();
    if len > MAX_PDF_BYTES {
        return Err("Compiled PDF exceeds 60 MiB".into());
    }
    let mut bytes = Vec::with_capacity(len as usize);
    file.take(MAX_PDF_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_PDF_BYTES {
        return Err("Compiled PDF exceeds 60 MiB".into());
    }
    Ok(bytes)
}

fn unverified_pdf(path: &Path) -> Option<Vec<u8>> {
    // A preview failure must not change an otherwise successful final build.
    // Source/dependency failures are handled by the mandatory checks above.
    snapshot::check_plain(path).ok()?;
    if fs::metadata(path).ok()?.len() > MAX_PDF_BYTES {
        return None;
    }
    let mut pdf = Vec::new();
    fs::File::open(path).ok()?.take(MAX_PDF_BYTES + 1).read_to_end(&mut pdf).ok()?;
    (pdf.len() as u64 <= MAX_PDF_BYTES && pdf.starts_with(b"%PDF-")).then_some(pdf)
}
fn preflight_diagnostic(output: &process::Output) -> Diagnostic {
    let lower = output.text.to_ascii_lowercase();
    let (code, message) = if output.timed_out {
        (
            "compiler_preflight_timeout",
            "Compiler version check timed out. Open Compiler output for details.",
        )
    } else if output.truncated {
        (
            "compiler_preflight_output_truncated",
            "Compiler version output exceeded its limit. Open Compiler output for details.",
        )
    } else if lower.contains("access is denied") || lower.contains("permission denied") {
        ("compiler_preflight_access_denied", "The compiler could not initialize because access was denied. Open Compiler output for details.")
    } else {
        (
            "compiler_preflight_failed",
            "The compiler could not initialize. Open Compiler output for details.",
        )
    };
    Diagnostic::new(code, Severity::Error, message)
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
            } else if ["aux", "toc", "out", "lof", "lot", "nav", "snm", "bcf", "bbl"]
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
    let markers: Vec<_> = sources.keys().map(|name| (name, format!("{name}:"))).collect();
    for raw in log.lines() {
        let lower = raw.to_ascii_lowercase();
        let mut location = None;
        // A source location requires a numeric field between two colons.
        let fields = raw.split(':').skip(1);
        let has_line_field = fields.clone().zip(fields.skip(1))
            .any(|(field, _)| field.parse::<usize>().is_ok());
        if has_line_field {
            for (name, marker) in &markers {
                if let Some(pos) = raw.find(marker) {
                    let tail = &raw[pos + marker.len()..];
                    if let Some((number, message)) = tail.split_once(':') {
                        if let Ok(line) = number.parse::<usize>() {
                            location = Some(((*name).clone(), line, message.trim().to_string()));
                            break;
                        }
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
            if lower.contains("not found") && [".sty",".cls",".ldf"].iter().any(|extension|lower.contains(extension)) {
                d.code = "missing_package".into();
            }
        }
        if let Some(d) = d {
            result.push(d);
            if result.len() == 200 {
                break;
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;

    fn profile_receipt(project: &str, invocation: &str) -> Receipt {
        Receipt {
            schema: 1,
            directory: PathBuf::from("snapshot"),
            project_root: PathBuf::from(project),
            main: "main.tex".into(),
            inputs: Default::default(),
            assets: Default::default(),
            toolchain_inputs: Default::default(),
            compiler: snapshot::Dependency {
                path: PathBuf::from(invocation),
                fingerprint: Fingerprint::bytes(b"compiler"),
            },
            auxiliary_tools: Default::default(),
            invocation: PathBuf::from(invocation),
            engine_version: "MiKTeX-pdfTeX fixture".into(),
            environment: Default::default(),
            arguments: Vec::new(),
            passes: 2,
            outputs: Default::default(),
            used_project_inputs: Default::default(),
            generated_inputs: Default::default(),
        }
    }

    #[test]
    fn profiles_survive_switching_projects_and_engines_but_failed_reuse_is_consumed() {
        let profiles = CompilerProfiles::default();
        let a = profile_receipt("a", "pdflatex");
        let b = profile_receipt("b", "pdflatex");
        let c = profile_receipt("a", "lualatex");
        for receipt in [&a, &b, &c] {
            assert!(profiles.take(receipt).is_none());
            profiles.remember(receipt, true, "runtime".into());
        }
        assert!(profiles.take(&a).is_some());
        assert!(profiles.take(&a).is_none());
        assert!(profiles.take(&b).is_some());
        assert!(profiles.take(&c).is_some());
    }

    #[test]
    fn profile_cache_evicts_oldest_and_discards_changed_input_identity() {
        let profiles = CompilerProfiles::default();
        let receipts: Vec<_> = (0..=COMPILER_PROFILE_LIMIT)
            .map(|i| profile_receipt(&format!("project-{i}"), "pdflatex"))
            .collect();
        for receipt in &receipts {
            profiles.remember(receipt, true, "runtime".into());
        }
        assert!(profiles.take(&receipts[0]).is_none());
        assert!(profiles.take(&receipts[1]).is_some());

        let original = receipts.last().unwrap();
        profiles.remember(original, true, "runtime".into());
        let mut new_snapshot = original.clone();
        new_snapshot.directory = PathBuf::from("other-snapshot");
        new_snapshot.environment.insert("TEXMFOUTPUT".into(), "other-output".into());
        assert!(profiles.take(&new_snapshot).is_some());

        let mut changed_environment = original.clone();
        changed_environment.environment.insert("TEXINPUTS".into(), "different".into());
        let mut changed_binary = original.clone();
        changed_binary.compiler.fingerprint = Fingerprint::bytes(b"new compiler");
        let mut changed_alias_target = original.clone();
        changed_alias_target.compiler.path = PathBuf::from("different-target");
        for changed in [changed_environment, changed_binary, changed_alias_target] {
            profiles.remember(original, true, "runtime".into());
            assert!(profiles.take(&changed).is_none());
            assert!(profiles.take(original).is_none());
        }
    }

    #[test]
    fn completed_pdf_receipt_tracks_the_delivered_bytes_and_enforces_the_boundary() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let root = std::env::temp_dir().join(format!(
            "texloom-completed-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        ));
        fs::create_dir(&root).unwrap();
        let project = root.join("project");
        fs::create_dir(&project).unwrap();
        let compiler = root.join("engine");
        fs::write(&compiler, b"synthetic engine").unwrap();
        let mut receipt = Receipt::capture(
            &project,
            &root.join("build"),
            "main.tex",
            &BTreeMap::from([("main.tex".into(), "fixture".into())]),
            &BTreeMap::new(),
            &compiler,
        )
        .unwrap();
        let pdf = receipt.directory.join(".out/preview.pdf");
        let map = receipt.directory.join(".out/preview.synctex.gz");
        fs::write(&pdf, b"%PDF-1.7\nfixture A").unwrap();
        fs::write(&map, b"fixture map").unwrap();

        let delivered = read_completed_pdf(&pdf).unwrap();
        fs::write(&pdf, b"%PDF-1.7\nfixture B").unwrap();
        receipt.generated_inputs.insert(".out/preview.pdf".into());
        receipt.seal_outputs_with_pdf(Some(&delivered)).unwrap();
        assert_eq!(receipt.outputs[".out/preview.pdf"], Fingerprint::bytes(&delivered));
        assert_eq!(receipt.outputs[".out/preview.synctex.gz"], Fingerprint::bytes(b"fixture map"));
        assert_eq!(receipt.verify().unwrap_err(), "Build output changed: .out/preview.pdf");
        fs::write(&pdf, &delivered).unwrap();
        receipt.verify().unwrap();

        let file = fs::OpenOptions::new().write(true).open(&pdf).unwrap();
        file.set_len(MAX_PDF_BYTES).unwrap();
        assert_eq!(read_completed_pdf(&pdf).unwrap().len() as u64, MAX_PDF_BYTES);
        file.set_len(MAX_PDF_BYTES + 1).unwrap();
        assert_eq!(read_completed_pdf(&pdf).unwrap_err(), "Compiled PDF exceeds 60 MiB");
        drop(file);
        fs::remove_file(pdf).unwrap();
        fs::remove_file(map).unwrap();
        fs::remove_file(receipt.directory.join("main.tex")).unwrap();
        fs::remove_dir(receipt.directory.join(".out")).unwrap();
        fs::remove_dir(&receipt.directory).unwrap();
        fs::remove_file(compiler).unwrap();
        fs::remove_dir(project).unwrap();
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn optional_preview_refuses_missing_non_pdf_and_oversized_output() {
        use std::{io::{Seek, Write}, time::SystemTime};
        let name = format!("texloom-preview-{}-{}.pdf", std::process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos());
        let path = std::env::temp_dir().join(name);
        assert!(unverified_pdf(&path).is_none());
        assert!(unverified_pdf(&std::env::temp_dir()).is_none());
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        file.write_all(b"other").unwrap(); file.flush().unwrap();
        assert!(unverified_pdf(&path).is_none());
        file.rewind().unwrap(); file.write_all(b"%PDF-").unwrap(); file.flush().unwrap();
        assert_eq!(unverified_pdf(&path).unwrap(), b"%PDF-");
        file.set_len(60 * 1024 * 1024 + 1).unwrap();
        assert!(unverified_pdf(&path).is_none());
        drop(file); fs::remove_file(path).unwrap();
    }
    #[test]
    fn preflight_failures_use_bounded_messages_and_preserve_timeout_priority() {
        let mut output = process::Output {
            success: false,
            cancelled: false,
            timed_out: false,
            truncated: false,
            text: "unrecognized initialization failure".repeat(1000),
        };
        assert_eq!(
            preflight_diagnostic(&output).code,
            "compiler_preflight_failed"
        );
        assert!(preflight_diagnostic(&output).message.len() < 200);
        output.text = "Access is denied. Permission denied.".into();
        assert_eq!(
            preflight_diagnostic(&output).code,
            "compiler_preflight_access_denied"
        );
        output.truncated = true;
        assert_eq!(
            preflight_diagnostic(&output).code,
            "compiler_preflight_output_truncated"
        );
        output.timed_out = true;
        let diagnostic = preflight_diagnostic(&output);
        assert_eq!(diagnostic.code, "compiler_preflight_timeout");
        assert_eq!(diagnostic.severity, Severity::Error);
    }
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
    #[test]
    fn runtime_matching_keeps_engine_and_format_identity_but_ignores_the_build_clock() {
        let banner = "This is pdfTeX, Version 1.40.28 (MiKTeX 25.12) (preloaded format=pdflatex 2026.10.27)";
        let first = stable_runtime_banner(&format!("{banner}  15 NOV 2026 22:40"));
        let later = stable_runtime_banner(&format!("{banner}  16 NOV 2026 00:02"));
        assert_eq!(first.as_deref(), Some(banner));
        assert_eq!(first, later);
        assert_ne!(first, stable_runtime_banner(&banner.replace("2026.10.27", "2026.10.28")));
        assert_eq!(stable_runtime_banner("Unexpected engine output"), None);
        let spaced = "This is LuaHBTeX, Version 1.22.0 (TeX Live 2026)  (format=lualatex 2026.10.26)";
        assert_eq!(stable_runtime_banner(&format!("{spaced}  15 NOV 2026 22:40")).as_deref(), Some(spaced));
        assert_ne!(stable_runtime_banner(&format!("{spaced}  15 NOV 2026 22:40")),
            stable_runtime_banner(&format!("{}  15 NOV 2026 22:40", spaced.replace("2026.10.26", "2026.10.27"))));
    }
}
