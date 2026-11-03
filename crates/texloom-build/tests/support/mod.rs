use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use texloom_build::{inspection_worker::Worker, snapshot::Fingerprint};

/// A long corpus must inspect with the same executable throughout, even when a
/// separate Cargo invocation rebuilds its target directory in the meantime.
pub fn owned_worker(root: &Path, program: &Path, library: PathBuf) -> Worker {
    let directory = root.join("inspector");
    fs::create_dir(&directory).unwrap();
    let expected = Fingerprint::file(program).unwrap();
    let bytes = fs::read(program).unwrap();
    assert_eq!(
        Fingerprint::bytes(&bytes),
        expected,
        "Worker changed while taking its corpus snapshot"
    );
    let copy = directory.join("pdfium_worker.exe");
    let mut file = fs::File::create_new(&copy).unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert_eq!(
        Fingerprint::file(program).unwrap(),
        expected,
        "Worker changed while taking its corpus snapshot"
    );
    Worker::new(copy, expected.sha256, library, root.join("inspections")).unwrap()
}
