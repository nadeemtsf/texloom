//! Parent-side supervised inspection and checked evidence transport.
//! Configuration comes from trusted native packaging, never from frontend IPC.
//! An inspection receipt alone is not a document-commit authorization.
use crate::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
    verification::*,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, String>;
pub const DLL_SHA256: &str = "b2c15cd4537d8efaf23b85b3353abe7a0b863e9c52b2f017dd87bb86a55f9c72";
pub const PROTOCOL: u32 = 2;
static NEXT: AtomicU64 = AtomicU64::new(0);
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hash_string(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn no_reparse(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(fail)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        require(
            metadata.file_attributes() & 0x400 == 0,
            "Inspection path is a reparse point",
        )?;
    }
    require(
        !metadata.file_type().is_symlink(),
        "Inspection path is a symlink",
    )
}
fn read(path: &Path, max: u64) -> Result<Vec<u8>> {
    no_reparse(path)?;
    let mut file = OpenOptions::new();
    file.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        file.share_mode(1);
    }
    let mut file = file.open(path).map_err(fail)?;
    require(
        file.metadata().map_err(fail)?.is_file(),
        "Inspection evidence is not a file",
    )?;
    require(
        file.metadata().map_err(fail)?.len() <= max,
        "Inspection file exceeds byte budget",
    )?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(fail)?;
    require(
        bytes.len() as u64 <= max,
        "Inspection file exceeds byte budget",
    )?;
    Ok(bytes)
}
fn pin(path: &Path, expected: &str, max: u64) -> Result<File> {
    require(
        hash_string(expected),
        "Invalid expected inspection checksum",
    )?;
    no_reparse(path)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1);
    }
    let mut file = options.open(path).map_err(fail)?;
    require(
        file.metadata().map_err(fail)?.is_file(),
        "Inspection input is not a file",
    )?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(fail)?;
    require(
        bytes.len() as u64 <= max && digest(&bytes) == expected,
        "Inspection input checksum mismatch",
    )?;
    Ok(file)
}
struct Workspace {
    root: PathBuf,
    parent: PathBuf,
}
impl Workspace {
    fn new(parent: &Path) -> Result<Self> {
        let name = digest(
            format!(
                "{}:{:?}:{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH).map_err(fail)?,
                NEXT.fetch_add(1, Ordering::Relaxed)
            )
            .as_bytes(),
        );
        let root = parent.join(format!("inspection-{name}"));
        fs::create_dir(&root).map_err(fail)?;
        Ok(Self {
            root,
            parent: parent.into(),
        })
    }
    fn file(&self, path: &Path) -> Result<PathBuf> {
        require(
            path.is_absolute(),
            "Inspection evidence path must be absolute",
        )?;
        no_reparse(path)?;
        let path = path.canonicalize().map_err(fail)?;
        require(
            path.parent() == Some(self.root.as_path()),
            "Inspection evidence escaped its workspace",
        )?;
        Ok(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        // Workers produce flat derived evidence only. Never recursively follow
        // a directory inserted into this workspace, even on a failure path.
        if no_reparse(&self.root).is_err()
            || self
                .root
                .canonicalize()
                .ok()
                .as_ref()
                .and_then(|p| p.parent())
                != Some(self.parent.as_path())
        {
            return;
        }
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| kind.is_file())
                    && no_reparse(&entry.path()).is_ok()
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        let _ = fs::remove_dir(&self.root);
    }
}
#[derive(Serialize)]
struct Request<'a> {
    schema: u32,
    request_id: &'a str,
    library: &'a Path,
    pdf: &'a Path,
    pdf_sha256: &'a str,
    target_marker: Option<&'a str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    schema: u32,
    status: String,
    request_id: String,
    request_sha256: String,
    inspection: PathBuf,
    sha256: String,
}
#[derive(Deserialize)]
struct View {
    schema: u32,
    renderer: String,
    pdfium: String,
    binding: String,
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "requestSha256")]
    request_sha256: String,
    #[serde(rename = "librarySha256")]
    library_sha256: String,
    #[serde(rename = "pdfSha256")]
    pdf_sha256: String,
    pages: Vec<PageView>,
    records: Vec<GlyphView>,
    graphics: Vec<GraphicView>,
    rasters: Vec<RasterView>,
    target: Option<TargetView>,
}
#[derive(Deserialize)]
struct PageView {
    width: f64,
    height: f64,
    crop: [f64; 4],
    rotation: i32,
    annotations: usize,
}
#[derive(Deserialize)]
struct GlyphView {
    text: String,
    page: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    target: Option<bool>,
}
#[derive(Deserialize)]
struct GraphicView {
    #[serde(rename = "type")]
    kind: i32,
    page: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
#[derive(Deserialize)]
struct RasterView {
    path: PathBuf,
    width: usize,
    height: usize,
    stride: usize,
    channels: usize,
    scale: f64,
    sha256: String,
}
#[derive(Deserialize)]
struct TargetView {
    marker: String,
    instances: usize,
    glyphs: usize,
}
pub struct Worker {
    program: PathBuf,
    program_sha256: String,
    library: PathBuf,
    private_root: PathBuf,
}
pub struct CheckedInspection {
    pub view: Inspection,
    pub pdf_sha256: String,
    pub request_sha256: String,
    _workspace: Workspace,
}
impl Worker {
    /// `program_sha256` must originate from a trusted packaged manifest.
    pub fn new(
        program: PathBuf,
        program_sha256: String,
        library: PathBuf,
        private_root: PathBuf,
    ) -> Result<Self> {
        let _program = pin(&program, &program_sha256, 50_000_000)?;
        let _library = pin(&library, DLL_SHA256, 30_000_000)?;
        fs::create_dir_all(&private_root).map_err(fail)?;
        no_reparse(&private_root)?;
        Ok(Self {
            program: program.canonicalize().map_err(fail)?,
            program_sha256,
            library: library.canonicalize().map_err(fail)?,
            private_root: private_root.canonicalize().map_err(fail)?,
        })
    }
    pub fn inspect(
        &self,
        pdf: &Path,
        pdf_sha256: &str,
        marker: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<CheckedInspection> {
        cancel.check()?;
        let _program = pin(&self.program, &self.program_sha256, 50_000_000)?;
        let _library = pin(&self.library, DLL_SHA256, 30_000_000)?;
        let _pdf = pin(pdf, pdf_sha256, 40_000_000)?;
        let pdf = pdf.canonicalize().map_err(fail)?;
        let workspace = Workspace::new(&self.private_root)?;
        let request_id = workspace
            .root
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("Invalid inspection workspace")?
            .trim_start_matches("inspection-");
        let request = Request {
            schema: PROTOCOL,
            request_id,
            library: &self.library,
            pdf: &pdf,
            pdf_sha256,
            target_marker: marker,
        };
        let bytes = serde_json::to_vec(&request).map_err(fail)?;
        let request_sha256 = digest(&bytes);
        let path = workspace.root.join("request.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(fail)?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(fail)?;
        drop(file);
        let output = process::run(
            &CommandSpec {
                program: &self.program,
                args: &[path.to_string_lossy().into_owned()],
                directory: &workspace.root,
                env: &BTreeMap::new(),
                timeout: Duration::from_secs(10),
                memory_bytes: 512 * 1024 * 1024,
            },
            cancel,
        )
        .map_err(fail)?;
        cancel.check()?;
        require(
            !output.timed_out,
            "Independent inspection exceeded its time limit",
        )?;
        require(
            !output.truncated,
            "Independent inspection returned excessive output",
        )?;
        if !output.success {
            let detail = output.text.trim().chars().take(1000).collect::<String>();
            return Err(if detail.is_empty() {
                "Independent inspection process failed".into()
            } else {
                format!("Independent inspection refused the PDF: {detail}")
            });
        }
        let reply: Reply = serde_json::from_str(output.text.trim()).map_err(fail)?;
        let view = read_receipt(
            &workspace,
            &reply,
            request_id,
            &request_sha256,
            pdf_sha256,
            marker,
            cancel,
        )?;
        // Recheck the input at completion; the native document gate must also
        // recheck its build/revision receipt immediately before committing.
        require(
            digest(&read(&pdf, 40_000_000)?) == pdf_sha256,
            "PDF changed during independent inspection",
        )?;
        cancel.check()?;
        Ok(CheckedInspection {
            view,
            pdf_sha256: pdf_sha256.into(),
            request_sha256,
            _workspace: workspace,
        })
    }
}
fn read_receipt(
    workspace: &Workspace,
    reply: &Reply,
    id: &str,
    request_sha: &str,
    pdf_sha: &str,
    marker: Option<&str>,
    cancel: &Cancellation,
) -> Result<Inspection> {
    cancel.check()?;
    require(
        reply.schema == PROTOCOL
            && reply.status == "inspected"
            && reply.request_id == id
            && reply.request_sha256 == request_sha,
        "Independent inspection response belongs to a different request",
    )?;
    let path = workspace.file(&reply.inspection)?;
    require(
        path == workspace.root.join("request.inspection.json"),
        "Unexpected inspection result filename",
    )?;
    let bytes = read(&path, 12_000_000)?;
    require(
        hash_string(&reply.sha256) && digest(&bytes) == reply.sha256,
        "Inspection result checksum mismatch",
    )?;
    let raw: View = serde_json::from_slice(&bytes).map_err(fail)?;
    require(
        raw.schema == PROTOCOL
            && raw.renderer == "pdfium"
            && raw.pdfium == "126.0.6462.0"
            && raw.binding == "rust-worker-v1"
            && raw.library_sha256 == DLL_SHA256,
        "Unpinned independent inspection evidence",
    )?;
    require(
        raw.request_id == id && raw.request_sha256 == request_sha && raw.pdf_sha256 == pdf_sha,
        "Inspection evidence belongs to different inputs",
    )?;
    require(
        !raw.pages.is_empty()
            && raw.pages.len() <= 20
            && raw.records.len() <= 50_000
            && raw.graphics.len() <= 10_000
            && raw.rasters.len() == raw.pages.len(),
        "Inspection object budget exceeded",
    )?;
    let selected = raw
        .records
        .iter()
        .filter(|g| g.target == Some(true))
        .count();
    match (marker, raw.target.as_ref()) {
        (Some(name), Some(target)) => require(
            target.marker == name
                && target.instances == 1
                && target.glyphs == selected
                && selected > 0
                && raw.records.iter().all(|g| g.target.is_some()),
            "Independent target identity is incomplete or mismatched",
        )?,
        (None, None) => require(selected == 0, "Unrequested target evidence")?,
        _ => return Err("Independent target identity is missing or unexpected".into()),
    }
    let pages = raw
        .pages
        .into_iter()
        .map(|p| Page {
            width: p.width,
            height: p.height,
            crop: p.crop,
            rotation: p.rotation,
            annotations: p.annotations,
        })
        .collect();
    let glyphs = raw
        .records
        .into_iter()
        .map(|g| {
            Ok(Glyph {
                text: g.text,
                page: g.page.checked_sub(1).ok_or("Invalid glyph page")?,
                bounds: Rect {
                    x: g.x,
                    y: g.y,
                    width: g.width,
                    height: g.height,
                },
                target: g.target.unwrap_or(false),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let graphics = raw
        .graphics
        .into_iter()
        .map(|g| {
            Ok(Graphic {
                kind: g.kind,
                page: g.page.checked_sub(1).ok_or("Invalid graphic page")?,
                bounds: Rect {
                    x: g.x,
                    y: g.y,
                    width: g.width,
                    height: g.height,
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut rasters = Vec::new();
    let mut total = 0usize;
    for (index, r) in raw.rasters.into_iter().enumerate() {
        cancel.check()?;
        let path = workspace.file(&r.path)?;
        require(
            path == workspace
                .root
                .join(format!("request.inspection.p{}.raster", index + 1)),
            "Unexpected inspection raster filename",
        )?;
        let length = r
            .stride
            .checked_mul(r.height)
            .ok_or("Invalid raster length")?;
        total = total.checked_add(length).ok_or("Raster budget overflow")?;
        require(
            length <= 64_000_000 && total <= 256_000_000,
            "Inspection raster byte budget exceeded",
        )?;
        let pixels = read(&path, 64_000_000)?;
        require(
            pixels.len() == length && hash_string(&r.sha256) && digest(&pixels) == r.sha256,
            "Inspection raster checksum or length mismatch",
        )?;
        rasters.push(Raster {
            width: r.width,
            height: r.height,
            stride: r.stride,
            channels: r.channels,
            scale: r.scale,
            pixels,
        });
    }
    let view = Inspection {
        pages,
        glyphs,
        graphics,
        rasters,
    };
    crate::verification::validate(&view).map_err(fail)?;
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    struct Parent(PathBuf);
    impl Drop for Parent {
        fn drop(&mut self) {
            let _ = fs::remove_dir(&self.0);
        }
    }
    struct Fixture {
        workspace: Workspace,
        _parent: Parent,
        reply: Reply,
        raw: Value,
        marker: Option<&'static str>,
    }
    const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const REQUEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const PDF: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    impl Fixture {
        fn new(marked: bool) -> Self {
            let parent = std::env::temp_dir().join(format!(
                "texloom-receipt-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&parent).unwrap();
            let parent = parent.canonicalize().unwrap();
            let workspace = Workspace::new(&parent).unwrap();
            let pixels = vec![255; 16 * 16 * 3];
            let raster = workspace.root.join("request.inspection.p1.raster");
            fs::write(&raster, &pixels).unwrap();
            let marker = marked.then_some("Texloom_sig");
            let raw = json!({"schema":PROTOCOL,"renderer":"pdfium","pdfium":"126.0.6462.0","binding":"rust-worker-v1",
                "requestId":ID,"requestSha256":REQUEST,"librarySha256":DLL_SHA256,"pdfSha256":PDF,
                "pages":[{"width":8.0,"height":8.0,"crop":[0.0,0.0,8.0,8.0],"rotation":0,"annotations":0}],
                "records":[{"text":"A","page":1,"x":2.0,"y":2.0,"width":2.0,"height":3.0,"target":marked}],"graphics":[],
                "rasters":[{"path":raster,"width":16,"height":16,"stride":48,"channels":3,"scale":2.0,"sha256":digest(&pixels)}],
                "target":marker.map(|name|json!({"marker":name,"instances":1,"glyphs":1}))});
            let reply = Reply {
                schema: PROTOCOL,
                status: "inspected".into(),
                request_id: ID.into(),
                request_sha256: REQUEST.into(),
                inspection: workspace.root.join("request.inspection.json"),
                sha256: String::new(),
            };
            let mut fixture = Self {
                workspace,
                _parent: Parent(parent),
                reply,
                raw,
                marker,
            };
            fixture.publish();
            fixture
        }
        fn publish(&mut self) {
            let bytes = serde_json::to_vec(&self.raw).unwrap();
            self.reply.sha256 = digest(&bytes);
            fs::write(self.workspace.root.join("request.inspection.json"), bytes).unwrap();
        }
        fn checked(&self) -> Result<Inspection> {
            read_receipt(
                &self.workspace,
                &self.reply,
                ID,
                REQUEST,
                PDF,
                self.marker,
                &Cancellation::default(),
            )
        }
    }
    #[test]
    fn exact_receipts_load_typed_owned_evidence() {
        for marked in [false, true] {
            let f = Fixture::new(marked);
            let result = f.checked().unwrap();
            assert_eq!(result.glyphs[0].target, marked);
            assert_eq!(result.rasters[0].pixels.len(), 768);
        }
    }
    #[test]
    fn replayed_response_or_other_input_identity_is_rejected() {
        for case in 0..6 {
            let mut f = Fixture::new(false);
            match case {
                0 => f.reply.schema = 1,
                1 => f.reply.request_id = REQUEST.into(),
                2 => f.reply.request_sha256 = ID.into(),
                3 => {
                    f.raw["requestId"] = json!(REQUEST);
                    f.publish();
                }
                4 => {
                    f.raw["requestSha256"] = json!(ID);
                    f.publish();
                }
                _ => {
                    f.raw["pdfSha256"] = json!(ID);
                    f.publish();
                }
            }
            assert!(f.checked().is_err());
        }
    }
    #[test]
    fn result_and_raster_corruption_cannot_be_hidden_by_json_success() {
        let f = Fixture::new(false);
        fs::write(&f.reply.inspection, b"changed").unwrap();
        assert!(f.checked().unwrap_err().contains("checksum"));
        let f = Fixture::new(false);
        fs::write(
            f.workspace.root.join("request.inspection.p1.raster"),
            vec![0; 768],
        )
        .unwrap();
        assert!(f.checked().unwrap_err().contains("checksum"));
    }
    #[test]
    fn inspector_version_library_and_target_claims_are_not_trusted() {
        for key in ["renderer", "pdfium", "binding", "librarySha256"] {
            let mut f = Fixture::new(false);
            f.raw[key] = json!("wrong");
            f.publish();
            assert!(f.checked().is_err());
        }
        for case in 0..5 {
            let mut f = Fixture::new(true);
            match case {
                0 => f.raw["target"] = Value::Null,
                1 => f.raw["target"]["marker"] = json!("Texloom_other"),
                2 => f.raw["target"]["instances"] = json!(2),
                3 => f.raw["target"]["glyphs"] = json!(2),
                _ => {
                    f.raw["records"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("target");
                }
            }
            f.publish();
            assert!(f.checked().is_err());
        }
        let mut f = Fixture::new(false);
        f.raw["records"][0]["target"] = json!(true);
        f.publish();
        assert!(f.checked().is_err());
    }
    #[test]
    fn evidence_paths_cannot_escape_or_alias_expected_outputs() {
        for raster in [false, true] {
            let mut f = Fixture::new(false);
            let outside = f.workspace.parent.join("outside");
            fs::write(&outside, b"keep").unwrap();
            if raster {
                f.raw["rasters"][0]["path"] = json!(outside);
                f.publish();
            } else {
                f.reply.inspection = outside.clone();
            }
            assert!(f.checked().unwrap_err().contains("escaped"));
            assert_eq!(fs::read(&outside).unwrap(), b"keep");
            fs::remove_file(outside).unwrap();
        }
        let mut f = Fixture::new(false);
        let other = f.workspace.root.join("different.raster");
        fs::write(&other, vec![255; 768]).unwrap();
        f.raw["rasters"][0]["path"] = json!(other);
        f.publish();
        assert!(f.checked().unwrap_err().contains("filename"));
    }
    #[test]
    fn invalid_geometry_and_raster_budgets_are_rejected() {
        for case in 0..5 {
            let mut f = Fixture::new(false);
            match case {
                0 => f.raw["records"][0]["page"] = json!(0),
                1 => f.raw["pages"][0]["width"] = json!(-1),
                2 => f.raw["rasters"][0]["channels"] = json!(1),
                3 => f.raw["rasters"][0]["stride"] = json!(usize::MAX),
                _ => f.raw["pages"][0]["annotations"] = json!(1),
            }
            f.publish();
            assert!(f.checked().is_err());
        }
        let f = Fixture::new(false);
        File::options()
            .write(true)
            .open(&f.reply.inspection)
            .unwrap()
            .set_len(12_000_001)
            .unwrap();
        assert!(f.checked().unwrap_err().contains("budget"));
    }
    #[test]
    fn cancellation_prevents_reading_or_accepting_evidence() {
        let f = Fixture::new(false);
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert!(read_receipt(&f.workspace, &f.reply, ID, REQUEST, PDF, None, &cancelled).is_err());
    }
    #[test]
    fn cleanup_does_not_descend_into_unexpected_directories() {
        let f = Fixture::new(false);
        let root = f.workspace.root.clone();
        let parent = f.workspace.parent.clone();
        let nested = root.join("unexpected");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("keep"), b"owned test data").unwrap();
        drop(f);
        assert_eq!(fs::read(nested.join("keep")).unwrap(), b"owned test data");
        fs::remove_file(nested.join("keep")).unwrap();
        fs::remove_dir(nested).unwrap();
        fs::remove_dir(root).unwrap();
        fs::remove_dir(parent).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn symlinked_evidence_is_rejected_even_inside_the_workspace() {
        let mut f = Fixture::new(false);
        let alias = f.workspace.root.join("alias.json");
        std::os::unix::fs::symlink(&f.reply.inspection, &alias).unwrap();
        f.reply.inspection = alias.clone();
        assert!(f.checked().is_err());
        fs::remove_file(alias).unwrap();
    }
}
