//! Bounded, declared-file BibTeX/Biber recipes. No shell or project config execution.
use crate::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
    snapshot::{self, AuxiliaryTool, Dependency, Fingerprint, Receipt},
};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
#[derive(Debug)]
pub struct Recipe {
    pub tool: &'static str,
    pub digest: Fingerprint,
    pub styles: Vec<String>,
}
fn read(path: &Path) -> Result<String> {
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(4_000_001)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 4_000_000 {
        return Err("Bibliography metadata exceeds 4 MB".into());
    }
    String::from_utf8(data).map_err(|_| "Bibliography metadata is not UTF-8".into())
}
fn declared(receipt: &Receipt, name: &str, extension: &str) -> Result<()> {
    let name = if Path::new(name).extension().is_some() {
        name.to_string()
    } else {
        format!("{name}.{extension}")
    };
    snapshot::relative(&name)?;
    if !receipt.inputs.contains_key(&name) || !name.ends_with(&format!(".{extension}")) {
        return Err(format!(
            "Bibliography input must be a declared project .{extension} file: {name}"
        ));
    }
    Ok(())
}
pub fn requested(receipt: &Receipt) -> Result<Option<Recipe>> {
    let out = receipt.directory.join(".out");
    let bcf = out.join("preview.bcf");
    if bcf.is_file() {
        let text = read(&bcf)?;
        let mut reader = quick_xml::Reader::from_str(&text);
        reader.config_mut().trim_text(true);
        let mut count = 0;
        loop {
            use quick_xml::events::Event;
            match reader.read_event().map_err(|e| e.to_string())? {
                Event::Start(element) if element.local_name().as_ref() == "datasource" => {
                    for attribute in element.attributes() {
                        let attribute = attribute.map_err(|e| e.to_string())?;
                        match attribute.key.as_ref() {
                            "type" if attribute.value.as_ref() != "file" => {
                                return Err(
                                    "Remote bibliography data sources are not supported".into()
                                )
                            }
                            "glob" if attribute.value.as_ref() != "false" => {
                                return Err(
                                    "Bibliography globs require explicit source files".into()
                                )
                            }
                            "datatype" if attribute.value.as_ref() != "bibtex" => {
                                return Err("This recipe supports BibTeX data sources".into())
                            }
                            _ => (),
                        }
                    }
                    let raw = reader
                        .read_text(element.name())
                        .map_err(|e| e.to_string())?;
                    let name = quick_xml::escape::unescape(&raw).map_err(|e| e.to_string())?;
                    declared(receipt, name.trim(), "bib")?;
                    count += 1;
                    if count > 100 {
                        return Err("Too many bibliography data sources".into());
                    }
                }
                Event::DocType(_) => {
                    return Err("Bibliography document types are not supported".into())
                }
                Event::Eof => break,
                _ => (),
            }
        }
        return Ok(Some(Recipe {
            tool: "biber",
            digest: Fingerprint::bytes(text.as_bytes()),
            styles: vec![],
        }));
    }
    let mut pending = vec!["preview.aux".to_string()];
    let mut seen = BTreeSet::new();
    let mut records = Vec::new();
    let mut data = Vec::new();
    let mut styles = Vec::new();
    while let Some(name) = pending.pop() {
        snapshot::relative(&name)?;
        if !name.ends_with(".aux") {
            return Err("Unsupported bibliography auxiliary input".into());
        }
        if !seen.insert(name.clone()) {
            continue;
        }
        if seen.len() > 600 {
            return Err("Too many bibliography auxiliary files".into());
        }
        let path = out.join(&name);
        if !path.is_file() {
            continue;
        }
        let text = read(&path)?;
        for line in text.lines().map(str::trim) {
            for command in ["\\bibdata", "\\bibstyle", "\\citation", "\\@input"] {
                if let Some(value) = line
                    .strip_prefix(&format!("{command}{{"))
                    .and_then(|v| v.strip_suffix('}'))
                {
                    if value.contains(['{', '}']) {
                        return Err("Unsupported bibliography metadata expression".into());
                    }
                    if command == "\\@input" {
                        pending.push(value.into());
                        continue;
                    }
                    records.push(format!("{name}:{line}"));
                    if command == "\\bibdata" {
                        for file in value.split(',').map(str::trim) {
                            declared(receipt, file, "bib")?;
                            data.push(file.to_string())
                        }
                    }
                    if command == "\\bibstyle" {
                        snapshot::relative(value)?;
                        styles.push(value.to_string())
                    }
                }
            }
        }
    }
    if data.is_empty() {
        return Ok(None);
    }
    styles.sort();
    styles.dedup();
    if styles.len() != 1 {
        return Err("The BibTeX recipe requires one explicit bibliography style".into());
    }
    records.sort();
    Ok(Some(Recipe {
        tool: "bibtex",
        digest: Fingerprint::bytes(records.join("\n").as_bytes()),
        styles,
    }))
}
pub fn search_path(root: &Path) -> String {
    format!(
        "{}{}",
        root.display(),
        if cfg!(windows) { ';' } else { ':' }
    )
}
fn program(compiler: &Path, name: &str) -> Result<PathBuf> {
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let sibling = compiler
        .parent()
        .ok_or("Compiler folder is missing")?
        .join(&executable);
    if sibling.is_file() {
        return Ok(sibling);
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|p| p.join(&executable))
        .find(|p| p.is_file())
        .ok_or_else(|| format!("Install {name} with MiKTeX or TeX Live to build this bibliography"))
}
fn run(
    receipt: &Receipt,
    program: &Path,
    args: &[String],
    directory: &Path,
    cancel: &Cancellation,
    budget: Duration,
) -> Result<process::Output> {
    let result = process::run(
        &CommandSpec {
            program,
            args,
            directory,
            env: &receipt.environment,
            timeout: budget.min(Duration::from_secs(60)),
            memory_bytes: 2 * 1024 * 1024 * 1024,
        },
        cancel,
    )
    .map_err(|e| e.to_string())?;
    cancel.check()?;
    if result.timed_out || result.truncated || !result.success {
        return Err(format!(
            "Bibliography tool failed or exceeded its limits: {}",
            result.text.chars().take(1600).collect::<String>()
        ));
    }
    Ok(result)
}
fn tool(
    receipt: &mut Receipt,
    name: &str,
    cancel: &Cancellation,
    budget: Duration,
) -> Result<PathBuf> {
    let invocation = program(&receipt.invocation, name)?;
    if let Some(previous) = receipt.auxiliary_tools.get(name) {
        previous.verify()?;
        if invocation.canonicalize().map_err(|e| e.to_string())? != previous.binary.path {
            return Err("Bibliography tool changed".into());
        }
        return Ok(invocation);
    }
    let binary = Dependency::capture(&invocation)?;
    let result = run(
        receipt,
        &invocation,
        &["--version".into()],
        &receipt.directory,
        cancel,
        budget,
    )?;
    binary.verify()?;
    if invocation.canonicalize().map_err(|e| e.to_string())? != binary.path {
        return Err("Bibliography tool changed during preflight".into());
    }
    receipt.auxiliary_tools.insert(
        name.into(),
        AuxiliaryTool {
            invocation: invocation.clone(),
            binary,
            version: result.text.chars().take(1600).collect(),
        },
    );
    Ok(invocation)
}
pub fn execute(
    receipt: &mut Receipt,
    recipe: &Recipe,
    cancel: &Cancellation,
    budget: Duration,
) -> Result<String> {
    let started = Instant::now();
    let remaining = || {
        budget
            .checked_sub(started.elapsed())
            .ok_or_else(|| "Bibliography exceeded the build time limit".to_string())
    };
    receipt.verify()?;
    for style in &recipe.styles {
        let name = if style.ends_with(".bst") {
            style.clone()
        } else {
            format!("{style}.bst")
        };
        if receipt.inputs.contains_key(&name) {
            continue;
        }
        let lookup = tool(receipt, "kpsewhich", cancel, remaining()?)?;
        let output = run(
            receipt,
            &lookup,
            &["--format=bst".into(), name.clone()],
            &receipt.directory,
            cancel,
            remaining()?,
        )?;
        let paths = output
            .text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err("The bibliography style lookup was ambiguous".into());
        }
        let path = PathBuf::from(paths[0])
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !snapshot::toolchain_roots(&receipt.compiler.path)
            .iter()
            .filter_map(|p| p.canonicalize().ok())
            .any(|root| path.starts_with(root))
        {
            return Err("The bibliography style is outside declared toolchain roots".into());
        }
        receipt
            .toolchain_inputs
            .insert(path.clone(), Dependency::capture(&path)?);
    }
    let executable = tool(receipt, recipe.tool, cancel, remaining()?)?;
    let (args, directory) = if recipe.tool == "biber" {
        (
            vec![
                "--noconf".into(),
                "--input-directory".into(),
                receipt.directory.to_string_lossy().into_owned(),
                "--output-directory".into(),
                receipt
                    .directory
                    .join(".out")
                    .to_string_lossy()
                    .into_owned(),
                receipt
                    .directory
                    .join(".out/preview.bcf")
                    .to_string_lossy()
                    .into_owned(),
            ],
            receipt.directory.clone(),
        )
    } else {
        (vec!["preview".into()], receipt.directory.join(".out"))
    };
    let result = run(
        receipt,
        &executable,
        &args,
        &directory,
        cancel,
        remaining()?,
    )?;
    receipt.verify()?;
    let bbl = receipt.directory.join(".out/preview.bbl");
    if !bbl.is_file() || fs::metadata(&bbl).map_err(|e| e.to_string())?.len() > 4_000_000 {
        return Err("Bibliography did not produce a bounded preview.bbl".into());
    }
    receipt.generated_inputs.insert(".out/preview.bbl".into());
    Ok(format!("\n--- {} ---\n{}", recipe.tool, result.text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    fn fixture() -> Receipt {
        let root = std::env::temp_dir().join(format!(
            "texloom-bib-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("project")).unwrap();
        fs::write(root.join("compiler"), b"fixture").unwrap();
        Receipt::capture(
            &root.join("project"),
            &root.join("build"),
            "main.tex",
            &BTreeMap::from([
                ("main.tex".into(), "main".into()),
                ("refs.bib".into(), "@book{x,title={Test}}".into()),
            ]),
            &BTreeMap::new(),
            &root.join("compiler"),
        )
        .unwrap()
    }
    #[test]
    fn bibliography_requests_are_bound_to_declared_files_and_citation_state() {
        let r = fixture();
        let path = r.directory.join(".out/preview.aux");
        fs::write(&path, "\\citation{x}\n\\bibdata{refs}\n\\bibstyle{plain}\n").unwrap();
        let first = requested(&r).unwrap().unwrap();
        assert_eq!(first.tool, "bibtex");
        fs::write(&path, "\\citation{y}\n\\bibdata{refs}\n\\bibstyle{plain}\n").unwrap();
        assert_ne!(first.digest, requested(&r).unwrap().unwrap().digest);
        fs::write(&path, "\\bibdata{../outside}\n\\bibstyle{plain}\n").unwrap();
        assert!(requested(&r).is_err());
        fs::remove_dir_all(r.directory.parent().unwrap()).unwrap();
    }
    #[test]
    fn biber_rejects_remote_glob_and_external_data_but_accepts_declared_xml_sources() {
        let r = fixture();
        let path = r.directory.join(".out/preview.bcf");
        for value in ["<bcf:controlfile xmlns:bcf=\"urn:test\"><bcf:datasource type=\"file\" datatype=\"bibtex\" glob=\"false\">refs.bib</bcf:datasource></bcf:controlfile>"]{fs::write(&path,value).unwrap();assert_eq!(requested(&r).unwrap().unwrap().tool,"biber");for bad in [value.replace("refs.bib","https://example.com/refs.bib"),value.replace("glob=\"false\"","glob=\"true\""),value.replace("type=\"file\"","type=\"url\"")]{fs::write(&path,bad).unwrap();assert!(requested(&r).is_err());}}
        fs::remove_dir_all(r.directory.parent().unwrap()).unwrap();
    }
}
