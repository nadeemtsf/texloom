//! Cross-engine PDFs are compiled on Linux and inspected by the exact Windows
//! worker. Target membership comes from mark handles, never fixture string IDs.
#![cfg(windows)]
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use texloom_build::{
    inspection_worker::Worker,
    scheduler::Cancellation,
    snapshot::Fingerprint,
    verification::{compare, Operation},
};

#[test]
#[ignore = "requires compiled three-engine fixtures and pinned Windows PDFium"]
fn marked_content_corpus_from_real_tex_engines() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let corpus = PathBuf::from(
        std::env::var_os("TEXLOOM_NATIVE_CORPUS").expect("Set compiled fixture folder"),
    );
    let library =
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL"));
    let program = PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker"));
    let private =
        std::env::temp_dir().join(format!("texloom-engine-inspections-{}", std::process::id()));
    fs::create_dir(&private).unwrap();
    let worker = Worker::new(
        program.clone(),
        Fingerprint::file(&program).unwrap().sha256,
        library,
        private.clone(),
    )
    .unwrap();
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("tests/fixtures/proof/manifest.json")).unwrap())
            .unwrap();
    let cancel = Cancellation::default();
    let mut rows = Vec::new();
    for engine in ["pdflatex", "lualatex", "xelatex"] {
        for fixture in manifest["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["expect"] != "refuse" && f["expect"] != "cross-page")
        {
            let id = fixture["id"].as_str().unwrap();
            let base = corpus.join(engine).join(id);
            let inspect = |variant: &str, marker: Option<&str>| {
                let pdf = base.join(variant).join("main.pdf");
                let sha = Fingerprint::file(&pdf).unwrap().sha256;
                worker.inspect(&pdf, &sha, marker, &cancel)
            };
            let original = inspect("baseline", None).unwrap();
            let zero = inspect("zero", None).unwrap();
            let identity = compare(&original.view, &zero.view, Operation::Identity).unwrap();
            drop(original);
            drop(zero);
            let marker = format!("Texloom_proof_{id}");
            let zero = inspect("zero", Some(&marker)).unwrap();
            let moved = inspect("moved", Some(&marker)).unwrap();
            let checked = compare(
                &zero.view,
                &moved.view,
                Operation::Displace {
                    dx: manifest["dx"].as_f64().unwrap(),
                    dy: manifest["dy"].as_f64().unwrap(),
                },
            );
            if fixture["expect"] == "overlap-refuse" {
                assert!(
                    checked
                        .as_ref()
                        .is_err_and(|reason| reason.contains("overlaps")),
                    "{engine}/{id}: {checked:?}"
                );
                rows.push(json!({"engine":engine,"fixture":id,"status":"expected-refusal","reason":"target raster overlaps other paint"}));
            } else {
                let checked = checked.unwrap_or_else(|reason| panic!("{engine}/{id}: {reason}"));
                rows.push(json!({"engine":engine,"fixture":id,"status":"pass","identity_max_error_bp":identity.max_geometry_error_bp,
                    "movement_max_error_bp":checked.max_geometry_error_bp,"target_glyphs":checked.target_glyphs}));
            }
        }
    }
    assert_eq!(rows.iter().filter(|r| r["status"] == "pass").count(), 24);
    assert_eq!(
        rows.iter()
            .filter(|r| r["status"] == "expected-refusal")
            .count(),
        3
    );
    assert_eq!(fs::read_dir(&private).unwrap().count(), 0);
    fs::remove_dir(private).unwrap();
    println!("TEXLOOM_NATIVE_CORPUS_BEGIN\n{}\nTEXLOOM_NATIVE_CORPUS_END",serde_json::to_string_pretty(&json!({"schema":1,"status":"pass","target_identity":"PDFium mark handles","cases":rows,
        "limitations":["cross-page fixture stops at its measured refusal before zero/moved compilation","native mouse/IME acceptance is separate"]})).unwrap());
}
