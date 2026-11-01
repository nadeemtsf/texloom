use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};
pub type Result<T> = std::result::Result<T, String>;
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn is_source(path: &Path) -> bool {
    matches!(
        extension(path).as_str(),
        "tex"
            | "sty"
            | "cls"
            | "bib"
            | "bst"
            | "cfg"
            | "def"
            | "clo"
            | "txt"
            | "bbl"
            | "bbx"
            | "cbx"
            | "lbx"
    )
}
pub fn is_asset(path: &Path) -> bool {
    matches!(
        extension(path).as_str(),
        "png" | "jpg" | "jpeg" | "pdf" | "svg" | "eps" | "otf" | "ttf" | "csv"
    )
}
fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}
pub fn relative(name: &str) -> Result<PathBuf> {
    let path = PathBuf::from(name);
    if name.is_empty()
        || name.chars().any(|x| matches!(x, '\\' | ':' | '\0'))
        || path.is_absolute()
        || path
            .components()
            .any(|x| !matches!(x, Component::Normal(_)))
        || path
            .components()
            .next()
            .is_some_and(|x| x.as_os_str() == ".out")
    {
        return Err("Invalid snapshot-relative filename".into());
    }
    if path
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
        != name
    {
        return Err("Aliased snapshot filename".into());
    }
    Ok(path)
}
fn no_reparse(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(fail)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(format!(
                "Reparse-point dependency requires an explicit import: {}",
                path.display()
            ));
        }
    }
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "Symlink dependency requires an explicit import: {}",
            path.display()
        ));
    }
    Ok(())
}
fn files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(root: &Path, dir: &Path, depth: usize, out: &mut Vec<PathBuf>) -> Result<()> {
        if depth > 20 {
            return Err("Project folder nesting exceeds 20 levels".into());
        }
        no_reparse(dir)?;
        for e in fs::read_dir(dir).map_err(fail)? {
            let e = e.map_err(fail)?;
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.')
                || matches!(name.as_str(), "node_modules" | "target" | "builds")
            {
                continue;
            }
            let path = e.path();
            no_reparse(&path)?;
            if path.is_dir() {
                visit(root, &path, depth + 1, out)?;
            } else if is_source(&path) || is_asset(&path) {
                out.push(path.strip_prefix(root).map_err(fail)?.to_path_buf());
                if out.len() > 600 {
                    return Err("Project exceeds 600 files".into());
                }
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    visit(root, root, 0, &mut result)?;
    result.sort();
    Ok(result)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fingerprint {
    pub bytes: u64,
    pub sha256: String,
}
impl Fingerprint {
    pub fn bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        }
    }
    pub fn file(path: &Path) -> Result<Self> {
        let mut file = fs::File::open(path).map_err(fail)?;
        let mut hash = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0; 64 * 1024];
        loop {
            let n = file.read(&mut buffer).map_err(fail)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > 256 * 1024 * 1024 {
                return Err(format!("Dependency exceeds 256 MiB: {}", path.display()));
            }
            hash.update(&buffer[..n]);
        }
        Ok(Self {
            bytes: total,
            sha256: format!("{:x}", hash.finalize()),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub path: PathBuf,
    pub fingerprint: Fingerprint,
}
impl Dependency {
    pub fn capture(path: &Path) -> Result<Self> {
        let path = path.canonicalize().map_err(fail)?;
        let fingerprint = Fingerprint::file(&path)?;
        Ok(Self { path, fingerprint })
    }
    pub fn verify(&self) -> Result<()> {
        if self.path.canonicalize().map_err(fail)? != self.path
            || Fingerprint::file(&self.path)? != self.fingerprint
        {
            return Err(format!("Build dependency changed: {}", self.path.display()));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub schema: u32,
    pub directory: PathBuf,
    pub project_root: PathBuf,
    pub main: String,
    pub inputs: BTreeMap<String, Fingerprint>,
    pub assets: BTreeMap<String, Dependency>,
    pub toolchain_inputs: BTreeMap<PathBuf, Dependency>,
    pub compiler: Dependency,
    pub invocation: PathBuf,
    pub engine_version: String,
    pub environment: BTreeMap<String, String>,
    pub arguments: Vec<String>,
    pub passes: usize,
    pub outputs: BTreeMap<String, Fingerprint>,
    pub used_project_inputs: BTreeSet<String>,
    pub generated_inputs: BTreeSet<String>,
}
impl Receipt {
    pub fn capture(
        root: &Path,
        directory: &Path,
        main: &str,
        sources: &BTreeMap<String, String>,
        support: &BTreeMap<String, String>,
        compiler: &Path,
    ) -> Result<Self> {
        if !sources.contains_key(main) || !main.ends_with(".tex") {
            return Err("Main .tex source is missing".into());
        }
        if sources.len() > 600 || sources.values().any(|x| x.len() > 4_000_000) {
            return Err("Source limits exceeded".into());
        }
        no_reparse(root)?;
        let root = root.canonicalize().map_err(fail)?;
        if directory.exists() {
            return Err("Build directory must be new and private".into());
        }
        fs::create_dir_all(directory).map_err(fail)?;
        let directory = directory.canonicalize().map_err(fail)?;
        let mut r = Self {
            schema: 1,
            directory,
            project_root: root.clone(),
            main: main.into(),
            inputs: BTreeMap::new(),
            assets: BTreeMap::new(),
            toolchain_inputs: BTreeMap::new(),
            compiler: Dependency::capture(compiler)?,
            invocation: if compiler.is_absolute() {
                compiler.into()
            } else {
                std::env::current_dir().map_err(fail)?.join(compiler)
            },
            engine_version: String::new(),
            environment: BTreeMap::new(),
            arguments: Vec::new(),
            passes: 0,
            outputs: BTreeMap::new(),
            used_project_inputs: BTreeSet::new(),
            generated_inputs: BTreeSet::new(),
        };
        let mut names = BTreeSet::new();
        let mut bytes = 0u64;
        for (name, text) in sources
            .iter()
            .chain(support.iter().filter(|(n, _)| !sources.contains_key(*n)))
        {
            let path = relative(name)?;
            if !is_source(&path) || !names.insert(name.to_ascii_lowercase()) {
                return Err("Invalid or case-colliding source filename".into());
            }
            bytes += text.len() as u64;
            if bytes > 60 * 1024 * 1024 {
                return Err("Snapshot exceeds 60 MiB".into());
            }
            r.write_input(name, text.as_bytes())?;
        }
        for file in files(&root)? {
            // The canonical source map defines text membership. Deleted files on disk
            // cannot silently reappear; external source conflict handling is separate.
            if !is_asset(&file) {
                continue;
            }
            let name = file.to_string_lossy().replace('\\', "/");
            if !names.insert(name.to_ascii_lowercase()) {
                return Err("Case-colliding asset filename".into());
            }
            let original = root.join(&file);
            let dependency = Dependency::capture(&original)?;
            bytes += dependency.fingerprint.bytes;
            if bytes > 60 * 1024 * 1024 {
                return Err("Snapshot exceeds 60 MiB".into());
            }
            let target = r.directory.join(&file);
            fs::create_dir_all(target.parent().ok_or("Missing parent")?).map_err(fail)?;
            fs::copy(&original, &target).map_err(fail)?;
            if Fingerprint::file(&target)? != dependency.fingerprint {
                return Err("Asset changed while taking snapshot".into());
            }
            no_reparse(&original)?;
            dependency.verify()?;
            r.inputs
                .insert(name.clone(), dependency.fingerprint.clone());
            r.assets.insert(name, dependency);
        }
        if r.inputs.len() > 600 {
            return Err("Snapshot exceeds 600 inputs".into());
        }
        fs::create_dir_all(r.directory.join(".out")).map_err(fail)?;
        r.verify()?;
        Ok(r)
    }
    fn write_input(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        let path = self.directory.join(relative(name)?);
        fs::create_dir_all(path.parent().ok_or("Missing parent")?).map_err(fail)?;
        fs::write(&path, bytes).map_err(fail)?;
        self.inputs.insert(name.into(), Fingerprint::bytes(bytes));
        Ok(())
    }
    pub fn verify(&self) -> Result<()> {
        self.compiler.verify()?;
        if self.invocation.canonicalize().map_err(fail)? != self.compiler.path {
            return Err("Compiler alias changed".into());
        }
        for (name, expected) in &self.inputs {
            let path = self.directory.join(relative(name)?);
            no_reparse(&path)?;
            if Fingerprint::file(&path)? != *expected {
                return Err(format!("Frozen build input changed: {name}"));
            }
        }
        let actual: BTreeSet<_> = files(&self.project_root)?
            .into_iter()
            .filter(|p| is_asset(p))
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        if actual != self.assets.keys().cloned().collect() {
            return Err("Project assets were added, removed or renamed during this build".into());
        }
        for (name, asset) in &self.assets {
            let original = self.project_root.join(relative(name)?);
            no_reparse(&original)?;
            if original.canonicalize().map_err(fail)? != asset.path {
                return Err(format!("Asset path changed: {name}"));
            }
            asset.verify()?;
        }
        for dependency in self.toolchain_inputs.values() {
            dependency.verify()?;
        }
        for (name, expected) in &self.outputs {
            if Fingerprint::file(&self.directory.join(name))? != *expected {
                return Err(format!("Build output changed: {name}"));
            }
        }
        Ok(())
    }
    pub fn seal_outputs(&mut self) -> Result<()> {
        for name in &self.generated_inputs {
            self.outputs
                .insert(name.clone(), Fingerprint::file(&self.directory.join(name))?);
        }
        for name in [
            "preview.pdf",
            "preview.synctex.gz",
            "preview.studio",
            "preview.fls",
        ] {
            let name = format!(".out/{name}");
            let path = self.directory.join(&name);
            if path.is_file() {
                self.outputs.insert(name, Fingerprint::file(&path)?);
            }
        }
        Ok(())
    }
    /// Recorder discovery is followed by another stable full pass. External inputs
    /// must live in the declared toolchain/font roots, not arbitrary user folders.
    pub fn record_inputs(&mut self, fls: &Path, roots: &[PathBuf]) -> Result<bool> {
        let text = fs::read_to_string(fls).map_err(|_| {
            "The engine did not produce its required dependency recorder".to_string()
        })?;
        if text.len() > 4 * 1024 * 1024 {
            return Err("Dependency recorder exceeds 4 MiB".into());
        }
        let roots: Vec<_> = roots.iter().filter_map(|p| p.canonicalize().ok()).collect();
        let mut added = false;
        for line in text.lines() {
            let Some(name) = line.strip_prefix("INPUT ") else {
                continue;
            };
            let name = name.trim().trim_matches('"');
            if matches!(name, "/dev/null" | "NUL" | "nul") {
                continue;
            }
            let p = PathBuf::from(name);
            let path = if p.is_absolute() {
                p
            } else {
                self.directory.join(p)
            };
            let path = path
                .canonicalize()
                .map_err(|e| format!("Recorded input unavailable: {name}: {e}"))?;
            if path.starts_with(self.directory.join(".out")) {
                self.generated_inputs.insert(
                    path.strip_prefix(&self.directory)
                        .map_err(fail)?
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
                continue;
            }
            if let Ok(file) = path.strip_prefix(&self.directory) {
                let name = file.to_string_lossy().replace('\\', "/");
                if !self.inputs.contains_key(&name) {
                    return Err(format!(
                        "Untracked input created inside the snapshot: {name}"
                    ));
                }
                self.used_project_inputs.insert(name);
                continue;
            }
            if !roots.iter().any(|root| path.starts_with(root)) {
                return Err(format!(
                    "External dependency must be imported into the project: {}",
                    path.display()
                ));
            }
            if !self.toolchain_inputs.contains_key(&path) {
                self.toolchain_inputs
                    .insert(path.clone(), Dependency::capture(&path)?);
                added = true;
                if self.toolchain_inputs.len() > 5000 {
                    return Err("Too many toolchain dependencies".into());
                }
            }
        }
        Ok(added)
    }
    pub fn persist(&self) -> Result<()> {
        fs::write(
            self.directory.join("texloom-build-manifest.json"),
            serde_json::to_vec_pretty(self).map_err(fail)?,
        )
        .map_err(fail)
    }
}
pub fn toolchain_roots(compiler: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = compiler.parent() {
        roots.push(parent.into());
    }
    for ancestor in compiler.ancestors().skip(1) {
        if ancestor.file_name().is_some_and(|x| {
            ["miktex", "texlive"].contains(&x.to_string_lossy().to_ascii_lowercase().as_str())
        }) {
            roots.push(ancestor.into());
        }
    }
    #[cfg(windows)]
    {
        for var in ["APPDATA", "LOCALAPPDATA", "ProgramData"] {
            if let Some(root) = std::env::var_os(var) {
                roots.push(PathBuf::from(root).join("MiKTeX"));
            }
        }
        if let Some(root) = std::env::var_os("WINDIR") {
            roots.push(PathBuf::from(root).join("Fonts"));
        }
        if let Some(root) = std::env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(root).join("Microsoft/Windows/Fonts"));
        }
    }
    #[cfg(unix)]
    {
        roots.extend(
            [
                "/usr/share/texlive",
                "/usr/share/texmf",
                "/var/lib/texmf",
                "/etc/texmf",
                "/usr/share/fonts",
                "/usr/local/share/fonts",
                "/etc/fonts",
                "/usr/share/fontconfig",
                "/var/cache/fontconfig",
            ]
            .map(PathBuf::from),
        );
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            roots.extend([
                home.join("texmf"),
                home.join(".cache/fontconfig"),
                home.join(".cache/texlive"),
            ]);
            if let Ok(entries) = fs::read_dir(&home) {
                for e in entries.flatten() {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if n.strip_prefix(".texlive").is_some_and(|year| {
                        year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit())
                    }) {
                        roots.push(e.path());
                    }
                }
            }
        }
    }
    roots
}
#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn miktex_installation_root_includes_packages_beside_the_inner_miktex_folder() {
        let roots = toolchain_roots(Path::new("C:/Programs/MiKTeX/miktex/bin/x64/pdflatex.exe"));
        assert!(roots.contains(&PathBuf::from("C:/Programs/MiKTeX")));
    }
}
