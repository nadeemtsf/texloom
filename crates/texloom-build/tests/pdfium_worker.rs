#![cfg(windows)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use texloom_build::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let name = format!(
            "texloom-worker-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let p = std::env::temp_dir().join(name);
        fs::create_dir(&p).unwrap();
        Self(p.canonicalize().unwrap())
    }
    fn request(&self, value: Value) -> PathBuf {
        let path = self
            .0
            .join("request-\u{645}\u{631}\u{62d}\u{628}\u{627}-\u{1f600}.json");
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        path
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn request(library: &Path) -> Value {
    let pdf = root().join("tests/fixtures/objects/baseline.pdf");
    json!({"schema":texloom_build::inspection_worker::PROTOCOL,"request_id":digest(b"synthetic native worker request"),"library":library,"pdf":pdf,"pdf_sha256":digest(&fs::read(&pdf).unwrap())})
}
fn run(path: &Path) -> process::Output {
    process::run(
        &CommandSpec {
            program: Path::new(env!("CARGO_BIN_EXE_pdfium_worker")),
            args: &[path.to_string_lossy().into_owned()],
            directory: path.parent().unwrap(),
            env: &BTreeMap::new(),
            timeout: Duration::from_secs(10),
            memory_bytes: 512 * 1024 * 1024,
        },
        &Cancellation::default(),
    )
    .unwrap()
}
#[test]
fn bad_schema_unknown_fields_receipts_and_large_requests_fail_before_loading() {
    for case in 0..4 {
        let dir = Directory::new();
        let mut value = request(Path::new("not-a-library.dll"));
        match case {
            0 => value["schema"] = json!(3),
            1 => value["unrecognized"] = json!(true),
            2 => value["pdf_sha256"] = json!("changed"),
            _ => value["padding"] = json!("x".repeat(64_001)),
        }
        let path = dir.request(value);
        let output = run(&path);
        assert!(
            !output.success && !output.timed_out && !output.cancelled,
            "{}",
            output.text
        );
        assert!(!path.with_extension("inspection.json").exists());
        assert!(
            !output
                .text
                .contains("The specified module could not be found"),
            "Validation must precede DLL loading"
        );
    }
}
#[test]
fn an_unpinned_library_is_rejected_before_windows_loads_it() {
    let dir = Directory::new();
    let library = dir.0.join("pdfium.dll");
    fs::write(&library, b"not the pinned DLL").unwrap();
    let output = run(&dir.request(request(&library)));
    assert!(!output.success);
    assert!(
        output.text.contains("DLL checksum mismatch"),
        "{}",
        output.text
    );
}
#[test]
#[ignore = "requires explicitly prepared pinned Windows PDFium and reference evidence"]
fn native_inspector_matches_reference_and_refuses_overwrite_and_malformed_pdf() {
    let library = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set the pinned DLL path"),
    );
    let reference: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("TEXLOOM_PDFIUM_REFERENCE").expect("Set reference evidence"))
            .unwrap(),
    )
    .unwrap();
    let dir = Directory::new();
    let path = dir.request(request(&library));
    let output = run(&path);
    assert!(output.success && !output.truncated, "{}", output.text);
    let reply: Value = serde_json::from_str(output.text.trim()).unwrap();
    assert_eq!(reply["status"], "inspected");
    let result = fs::read(path.with_extension("inspection.json")).unwrap();
    assert_eq!(reply["sha256"], digest(&result));
    let view: Value = serde_json::from_slice(&result).unwrap();
    for field in ["pages", "records", "graphics", "pdfSha256"] {
        assert_eq!(reference[field], view[field], "Different {field}");
    }
    for (a, b) in reference["rasters"]
        .as_array()
        .unwrap()
        .iter()
        .zip(view["rasters"].as_array().unwrap())
    {
        for key in ["width", "height", "stride", "channels", "scale", "sha256"] {
            assert_eq!(a[key], b[key], "Different raster {key}");
        }
        assert_eq!(
            fs::read(a["path"].as_str().unwrap()).unwrap(),
            fs::read(b["path"].as_str().unwrap()).unwrap()
        );
    }
    let second = run(&path);
    assert!(!second.success);
    assert!(second.text.contains("output already exists"));
    assert_eq!(
        result,
        fs::read(path.with_extension("inspection.json")).unwrap()
    );
    let bad = Directory::new();
    let pdf = bad.0.join("malformed.pdf");
    let bytes = b"%PDF-1.7\ntruncated";
    fs::write(&pdf, bytes).unwrap();
    let mut value = request(&library);
    value["pdf"] = json!(pdf);
    value["pdf_sha256"] = json!(digest(bytes));
    let path = bad.request(value);
    let output = run(&path);
    assert!(!output.success && !output.timed_out, "{}", output.text);
    assert!(!path.with_extension("inspection.json").exists());
}

fn simple_pdf(stream: &str) -> Vec<u8> {
    let objects=vec!["<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".into(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
        format!("<< /Length {} >>\nstream\n{stream}\nendstream",stream.len())];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0];
    for (i, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}
#[test]
#[ignore = "requires explicitly prepared pinned Windows PDFium"]
fn marker_identity_spans_text_objects_and_refuses_duplicates_nested_paint_and_overlap() {
    let library = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set the pinned DLL path"),
    );
    let first = "BT /F1 12 Tf 20 150 Td (First) Tj ET";
    let second = "BT /F1 12 Tf 20 130 Td (Second) Tj ET";
    let good =
        format!("/Texloom_sig BMC {first} {second} EMC BT /F1 12 Tf 180 30 Td (Neighbor) Tj ET");
    let cases = [
        (good, None),
        (
            format!("/Texloom_sig BMC {first} EMC /Texloom_sig BMC {second} EMC"),
            Some("multiple rendered instances"),
        ),
        (
            format!("/Texloom_sig BMC /Texloom_other BMC {first} EMC EMC"),
            Some("Nested Texloom identities"),
        ),
        (
            format!("/Texloom_sig BMC {first} 20 20 5 5 re f EMC"),
            Some("non-text paint"),
        ),
        (
            format!("/Texloom_sig BMC {first} EMC {first}"),
            Some("crosses target and neighbor"),
        ),
        (first.into(), Some("marker is missing")),
    ];
    for (content, reason) in cases {
        let dir = Directory::new();
        let pdf = dir.0.join("marked.pdf");
        let bytes = simple_pdf(&content);
        fs::write(&pdf, &bytes).unwrap();
        let mut value = request(&library);
        value["pdf"] = json!(pdf);
        value["pdf_sha256"] = json!(digest(&bytes));
        value["target_marker"] = json!("Texloom_sig");
        let path = dir.request(value);
        let output = run(&path);
        if let Some(reason) = reason {
            assert!(!output.success && !output.timed_out, "{}", output.text);
            assert!(output.text.contains(reason), "{}", output.text);
        } else {
            assert!(output.success && !output.truncated, "{}", output.text);
            let view: Value =
                serde_json::from_slice(&fs::read(path.with_extension("inspection.json")).unwrap())
                    .unwrap();
            assert_eq!(view["target"]["instances"], 1);
            assert_eq!(view["target"]["glyphs"], 11);
            let selected: String = view["records"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["target"] == true)
                .map(|r| r["text"].as_str().unwrap())
                .collect();
            assert_eq!(selected, "FirstSecond");
        }
    }
}

fn evidence(view: &Value) -> texloom_build::verification::Inspection {
    use texloom_build::verification::*;
    let number = |v: &Value, k: &str| v[k].as_f64().unwrap();
    let rect = |v: &Value| Rect {
        x: number(v, "x"),
        y: number(v, "y"),
        width: number(v, "width"),
        height: number(v, "height"),
    };
    Inspection {
        pages: view["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| Page {
                width: number(p, "width"),
                height: number(p, "height"),
                crop: std::array::from_fn(|i| p["crop"][i].as_f64().unwrap()),
                rotation: p["rotation"].as_i64().unwrap() as i32,
                annotations: p["annotations"].as_u64().unwrap() as usize,
            })
            .collect(),
        glyphs: view["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| Glyph {
                text: g["text"].as_str().unwrap().into(),
                page: g["page"].as_u64().unwrap() as usize - 1,
                bounds: rect(g),
                target: g["target"].as_bool().unwrap(),
            })
            .collect(),
        graphics: view["graphics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| Graphic {
                kind: g["type"].as_i64().unwrap() as i32,
                page: g["page"].as_u64().unwrap() as usize - 1,
                bounds: rect(g),
            })
            .collect(),
        rasters: view["rasters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| Raster {
                width: r["width"].as_u64().unwrap() as usize,
                height: r["height"].as_u64().unwrap() as usize,
                stride: r["stride"].as_u64().unwrap() as usize,
                channels: r["channels"].as_u64().unwrap() as usize,
                scale: number(r, "scale"),
                pixels: fs::read(r["path"].as_str().unwrap()).unwrap(),
            })
            .collect(),
    }
}
#[test]
#[ignore = "requires explicitly prepared pinned Windows PDFium"]
fn independent_marker_inspection_passes_exact_movement_and_rejects_neighbor_drift() {
    use texloom_build::verification::{compare, Operation};
    let library = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set the pinned DLL path"),
    );
    let mut views = Vec::new();
    for (dx, dy, neighbor) in [(0, 0, 180), (3, 8, 180), (3, 8, 181)] {
        let dir = Directory::new();
        let pdf = dir.0.join("movement.pdf");
        let content=format!("q 1 0 0 1 {dx} {} cm /Texloom_sig BMC BT /F1 12 Tf 20 150 Td (First) Tj ET BT /F1 12 Tf 20 130 Td (Second) Tj ET EMC Q BT /F1 12 Tf {neighbor} 30 Td (Neighbor) Tj ET",-dy);
        let bytes = simple_pdf(&content);
        fs::write(&pdf, &bytes).unwrap();
        let mut value = request(&library);
        value["pdf"] = json!(pdf);
        value["pdf_sha256"] = json!(digest(&bytes));
        value["target_marker"] = json!("Texloom_sig");
        let path = dir.request(value);
        let output = run(&path);
        assert!(output.success, "{}", output.text);
        let view: Value =
            serde_json::from_slice(&fs::read(path.with_extension("inspection.json")).unwrap())
                .unwrap();
        views.push(evidence(&view));
    }
    assert!(compare(&views[0], &views[0], Operation::Identity).is_ok());
    assert!(compare(
        &views[0],
        &views[1],
        Operation::Displace { dx: 3.0, dy: 8.0 }
    )
    .is_ok());
    assert!(compare(
        &views[0],
        &views[2],
        Operation::Displace { dx: 3.0, dy: 8.0 }
    )
    .is_err());
}

#[test]
#[ignore = "requires explicitly prepared pinned Windows PDFium"]
fn parent_supervisor_checks_receipts_and_cleans_only_derived_evidence() {
    use texloom_build::inspection_worker::Worker;
    let library = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set the pinned DLL path"),
    );
    let program = PathBuf::from(env!("CARGO_BIN_EXE_pdfium_worker"));
    let program_sha = digest(&fs::read(&program).unwrap());
    let dir = Directory::new();
    let cache = dir.0.join("receipts-\u{1f600}");
    let worker = Worker::new(program, program_sha, library, cache.clone()).unwrap();
    let pdf = root().join("tests/fixtures/objects/baseline.pdf");
    let original = fs::read(&pdf).unwrap();
    let sha = digest(&original);
    let result = worker
        .inspect(&pdf, &sha, None, &Cancellation::default())
        .unwrap();
    assert_eq!(result.pdf_sha256, sha);
    assert_eq!(result.view.glyphs.len(), 204);
    assert_eq!(result.view.pages.len(), 1);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 1);
    drop(result);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    let refused = worker
        .inspect(
            &pdf,
            &sha,
            Some("Texloom_missing"),
            &Cancellation::default(),
        )
        .err()
        .unwrap();
    assert!(
        refused.starts_with("Independent inspection refused the PDF:"),
        "{refused}"
    );
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    assert!(worker
        .inspect(&pdf, &"0".repeat(64), None, &Cancellation::default())
        .is_err());
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(worker.inspect(&pdf, &sha, None, &cancelled).is_err());
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
    assert_eq!(fs::read(&pdf).unwrap(), original);
}
