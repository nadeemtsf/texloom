use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use texloom_build::snapshot::{Fingerprint, Receipt};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "texloom-snapshot-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("project")).unwrap();
        Self(path)
    }
    fn capture(&self) -> Receipt {
        let sources = BTreeMap::from([("main.tex".into(), "\u{feff}Canonical\r\nText\n".into())]);
        Receipt::capture(
            &self.0.join("project"),
            &self.0.join("snapshot"),
            "main.tex",
            &sources,
            &BTreeMap::from([("objects.sty".into(), "Portable".into())]),
            &std::env::current_exe().unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn canonical_sources_and_assets_are_real_copies_and_deleted_disk_sources_stay_deleted() {
    let w = Workspace::new();
    fs::write(w.0.join("project/main.tex"), "Old disk source").unwrap();
    fs::write(w.0.join("project/removed.tex"), "Must not reappear").unwrap();
    fs::write(w.0.join("project/image.png"), b"ORIGINAL").unwrap();
    let r = w.capture();
    assert_eq!(
        fs::read(r.directory.join("main.tex")).unwrap(),
        "\u{feff}Canonical\r\nText\n".as_bytes()
    );
    assert!(!r.directory.join("removed.tex").exists());
    assert!(r.directory.join("objects.sty").is_file());
    fs::write(r.directory.join("image.png"), b"WRITE IN SNAPSHOT").unwrap();
    assert_eq!(
        fs::read(w.0.join("project/image.png")).unwrap(),
        b"ORIGINAL"
    );
    assert!(r.verify().is_err());
}
#[test]
fn asset_edits_renames_and_new_assets_invalidate_receipts() {
    for action in 0..3 {
        let w = Workspace::new();
        let image = w.0.join("project/image.png");
        fs::write(&image, b"A").unwrap();
        let r = w.capture();
        match action {
            0 => fs::write(image, b"B").unwrap(),
            1 => fs::rename(image, w.0.join("project/renamed.png")).unwrap(),
            _ => fs::write(w.0.join("project/new.png"), b"C").unwrap(),
        };
        assert!(r.verify().is_err());
    }
}
#[test]
fn recorder_requires_declared_inputs_and_checks_toolchain_changes() {
    let w = Workspace::new();
    let mut r = w.capture();
    let toolchain = w.0.join("toolchain");
    fs::create_dir(&toolchain).unwrap();
    let package = toolchain.join("package.sty");
    fs::write(&package, b"PACKAGE").unwrap();
    let fls = r.directory.join(".out/preview.fls");
    fs::write(
        &fls,
        format!("INPUT main.tex\nINPUT {}\n", package.display()),
    )
    .unwrap();
    assert!(r.record_inputs(&fls, &[]).is_err());
    assert!(r.record_inputs(&fls, &[toolchain]).unwrap());
    assert!(!r.record_inputs(&fls, &[w.0.join("toolchain")]).unwrap());
    r.verify().unwrap();
    fs::write(package, b"CHANGED").unwrap();
    assert!(r.verify().is_err());
}
#[test]
fn unknown_generated_inputs_and_source_aliases_are_refused() {
    let w = Workspace::new();
    let mut r = w.capture();
    fs::write(r.directory.join("untracked.tex"), "Hidden input").unwrap();
    let fls = r.directory.join(".out/preview.fls");
    fs::write(&fls, "INPUT untracked.tex\n").unwrap();
    assert!(r.record_inputs(&fls, &[]).is_err());
    let sources = BTreeMap::from([
        ("main.tex".into(), "A".into()),
        ("Main.tex".into(), "B".into()),
    ]);
    assert!(Receipt::capture(
        &w.0.join("project"),
        &w.0.join("other"),
        "main.tex",
        &sources,
        &BTreeMap::new(),
        &std::env::current_exe().unwrap()
    )
    .is_err());
}
#[test]
fn hash_matches_known_sha256_vector() {
    assert_eq!(
        Fingerprint::bytes(b"abc").sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
