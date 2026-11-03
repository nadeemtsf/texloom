//! Opt-in native-service measurement. Never included in the application package.
//! Supply a NEW workspace-owned output directory. No TeX engine or UI input.
use serde_json::{json, Value};
use std::{fs, hint::black_box, path::Path, time::Instant};
use texloom_core::document::projection::EditorProjection;
use texloom_desktop_lib::benchmark::{Backend, EditorChange};

const WARMUP: usize = 10;
const SAMPLES: usize = 100;

fn distribution(mut values: Vec<f64>) -> Value {
    values.sort_by(f64::total_cmp);
    let count = values.len();
    json!({
        "samples": count,
        "median_ms": (values[(count - 1) / 2] + values[count / 2]) / 2.0,
        "p95_ms": values[(count * 95).div_ceil(100) - 1],
        "max_ms": values[count - 1]
    })
}

fn source(bytes: usize) -> String {
    let mut text = "\u{feff}% Native source editing fixture\r\n".to_string();
    let line =
        "% Unicode \u{03b1} \u{1f600} and Windows line endings; source remains authoritative.\r\n";
    while text.len() + line.len() < bytes {
        text.push_str(line);
    }
    text.push_str(&" ".repeat(bytes - text.len() - 1));
    text.push('X');
    assert_eq!(text.len(), bytes);
    text
}

fn measure(base: &Path, files: usize, bytes: usize, delta: bool) -> Value {
    let mode = if delta {
        "changed_file"
    } else {
        "full_snapshot"
    };
    let root = base.join(format!("{mode}-files-{files}-bytes-{bytes}"));
    fs::create_dir(&root).unwrap();
    let project_root = root.join("project");
    fs::create_dir(&project_root).unwrap();
    let original = source(bytes);
    for index in 0..files {
        let name = if index == 0 {
            "main.tex".into()
        } else {
            format!("chapter-{index:03}.tex")
        };
        fs::write(project_root.join(name), original.as_bytes()).unwrap();
    }
    let disk_before =
        texloom_build::snapshot::Fingerprint::file(&project_root.join("main.tex")).unwrap();
    let backend = Backend::new(root.join("cache"));
    let project = backend
        .open_project(Some(project_root.join("main.tex").to_string_lossy().into()))
        .unwrap();
    assert_eq!(project.sources.len(), files);
    let offset = EditorProjection::new(&original)
        .editor_offset(original.len() - 1)
        .unwrap();
    backend
        .document_selection(&project.id, 0, "main.tex".into(), offset, offset, vec![])
        .unwrap();
    let mut revision = 0;
    let (mut editing, mut encoding, mut combined) = (Vec::new(), Vec::new(), Vec::new());
    let mut response_bytes = 0;
    for index in 0..WARMUP + SAMPLES {
        let change = EditorChange {
            offset,
            length: 1,
            text: if index % 2 == 0 { "x" } else { "y" }.into(),
        };
        let started = Instant::now();
        let (next_revision, payload, edited, encoded) = if delta {
            let reply = backend
                .document_edit_delta(&project.id, revision, "main.tex", vec![change], Some(1))
                .unwrap();
            let edited = started.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(reply.sources.len(), 1);
            assert_eq!(reply.base_revision, revision);
            let serialization = Instant::now();
            let payload = serde_json::to_vec(&reply).unwrap();
            (
                reply.revision,
                payload,
                edited,
                serialization.elapsed().as_secs_f64() * 1000.0,
            )
        } else {
            let snapshot = backend
                .document_edit(&project.id, revision, "main.tex", vec![change], Some(1))
                .unwrap();
            let edited = started.elapsed().as_secs_f64() * 1000.0;
            let serialization = Instant::now();
            let payload = serde_json::to_vec(&snapshot).unwrap();
            (
                snapshot.revision,
                payload,
                edited,
                serialization.elapsed().as_secs_f64() * 1000.0,
            )
        };
        assert_eq!(next_revision, revision + 1);
        revision = next_revision;
        response_bytes = black_box(payload.len());
        let total = started.elapsed().as_secs_f64() * 1000.0;
        if index >= WARMUP {
            editing.push(edited);
            encoding.push(encoded);
            combined.push(total);
        }
        black_box(payload);
    }
    let final_sources = backend.document_snapshot(&project.id).unwrap().sources;
    let undone = backend
        .document_history(&project.id, revision, false)
        .unwrap();
    assert_eq!(
        undone.sources, project.sources,
        "Coalesced undo must restore every original byte"
    );
    let redone = backend
        .document_history(&project.id, undone.revision, true)
        .unwrap();
    assert_eq!(
        redone.sources, final_sources,
        "Redo must restore the final accepted source"
    );
    let disk_after =
        texloom_build::snapshot::Fingerprint::file(&project_root.join("main.tex")).unwrap();
    assert_eq!(disk_before, disk_after, "Typing must not save to disk");
    backend.close_document(&project.id).unwrap();
    json!({
        "response_mode": mode, "production_ipc": delta,
        "files": files, "bytes_per_file": bytes, "project_source_bytes": files * bytes,
        "warmup_edits": WARMUP, "measured_edits": SAMPLES, "response_bytes": response_bytes,
        "native_edit_and_reply": distribution(editing),
        "reply_json_encoding": distribution(encoding),
        "native_edit_through_response_encoding": distribution(combined),
        "coalesced_undo_redo_exact": true, "original_disk_bytes_unchanged": true
    })
}

fn main() {
    let argument = std::env::args_os()
        .nth(1)
        .expect("Pass a new workspace-owned output directory");
    let base = std::path::PathBuf::from(argument);
    fs::create_dir(&base)
        .expect("Benchmark output must be a new directory with an existing parent");
    let base = base.canonicalize().unwrap();
    let cases = [
        (1, 16 * 1024),
        (1, 256 * 1024),
        (16, 64 * 1024),
        (64, 256 * 1024),
        (128, 256 * 1024),
    ]
    .into_iter()
    .flat_map(|(files, bytes)| [false, true].map(|delta| measure(&base, files, bytes, delta)))
    .collect::<Vec<_>>();
    let report = json!({
        "schema": 2,
        "os": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "logical_processors_seen": std::thread::available_parallelism().map(|n| n.get()).ok(),
        "profile": "release",
        "implementation_base": "c83dda1ece80758bbfdaddf89a78021857bae7cf",
        "github_sha_at_run": std::env::var("GITHUB_SHA").ok(),
        "github_ref_at_run": std::env::var("GITHUB_REF").ok(),
        "measured_api": "Shared native edit authority: full DocumentSnapshot reference vs production changed-file SourceEditReply",
        "cases": cases,
        "limitations": [
            "Synthetic source projects; one run with uncontrolled background load",
            "CI measurements describe the runner, not the user's laptop",
            "No WebView IPC transport, JavaScript parsing, editor paint, native typing or IME measurement",
            "No compiler job or initialized build cache; no recovery journal or disk save in measured edits",
            "Response/snapshot destruction and transport are outside the measured interval",
            "Not a native UI acceptance gate or a comparison to the 50 ms editor-paint target"
        ]
    });
    fs::write(
        base.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "TEXLOOM_SOURCE_BENCHMARK_BEGIN\n{}\nTEXLOOM_SOURCE_BENCHMARK_END",
        serde_json::to_string_pretty(&report).unwrap()
    );
}
