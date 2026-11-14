//! Development fixture bridge, not an application worker or an IPC command.
//! Record IDs are supplied by the explicit fixture oracle. Production requires
//! marker-derived identity in a packaged worker, never IDs from the frontend.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use texloom_build::verification::{self as v, *};
type Error = Box<dyn std::error::Error>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    before: PathBuf,
    after: PathBuf,
    before_pdf: PathBuf,
    after_pdf: PathBuf,
    before_target: Vec<usize>,
    after_target: Vec<usize>,
    operation: Policy,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Policy {
    Identity,
    Displace { dx: f64, dy: f64 },
}
#[derive(Deserialize)]
struct View {
    renderer: String,
    pdfium: String,
    binding: String,
    #[serde(rename = "pdfSha256")]
    pdf_sha256: String,
    pages: Vec<PageView>,
    records: Vec<RecordView>,
    graphics: Vec<GraphicView>,
    rasters: Vec<RasterView>,
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
struct RecordView {
    text: String,
    page: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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
fn read(path: &Path, max: u64) -> std::result::Result<Vec<u8>, Error> {
    let mut data = Vec::new();
    File::open(path)?.take(max + 1).read_to_end(&mut data)?;
    if data.len() as u64 > max {
        return Err("Evidence file exceeds byte budget".into());
    }
    Ok(data)
}
fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn inspection(json: &Path, pdf: &Path, target: &[usize]) -> std::result::Result<Inspection, Error> {
    let raw: View = serde_json::from_slice(&read(json, 12_000_000)?)?;
    if raw.renderer != "pdfium" || raw.pdfium != "156.0.8076.0" || raw.binding != "5.14.0" {
        return Err("Unpinned independent inspector".into());
    }
    if digest(&read(pdf, 40_000_000)?) != raw.pdf_sha256 {
        return Err("PDF receipt does not match inspected bytes".into());
    }
    if raw.pages.len() > 20
        || raw.records.len() > 50_000
        || raw.graphics.len() > 10_000
        || raw.rasters.len() != raw.pages.len()
    {
        return Err("Evidence exceeds object budget".into());
    }
    let mut selected = std::collections::BTreeSet::new();
    for &index in target {
        if index >= raw.records.len() || !selected.insert(index) {
            return Err("Invalid or duplicate target record ID".into());
        }
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
        .enumerate()
        .map(|(i, r)| {
            Ok(Glyph {
                text: r.text,
                page: r.page.checked_sub(1).ok_or("Invalid physical page")?,
                bounds: Rect {
                    x: r.x,
                    y: r.y,
                    width: r.width,
                    height: r.height,
                },
                target: selected.contains(&i),
            })
        })
        .collect::<std::result::Result<Vec<_>, Error>>()?;
    let graphics = raw
        .graphics
        .into_iter()
        .map(|r| {
            Ok(Graphic {
                kind: r.kind,
                page: r.page.checked_sub(1).ok_or("Invalid graphic page")?,
                bounds: Rect {
                    x: r.x,
                    y: r.y,
                    width: r.width,
                    height: r.height,
                },
            })
        })
        .collect::<std::result::Result<Vec<_>, Error>>()?;
    let mut bytes = 0usize;
    let mut rasters = Vec::new();
    let parent = json
        .canonicalize()?
        .parent()
        .ok_or("Evidence has no directory")?
        .to_path_buf();
    for r in raw.rasters {
        let path = r.path.canonicalize()?;
        if path.parent() != Some(parent.as_path()) {
            return Err("Raster escaped its evidence directory".into());
        }
        let pixels = read(&path, 64_000_000)?;
        bytes = bytes
            .checked_add(pixels.len())
            .ok_or("Raster budget overflow")?;
        if bytes > 256_000_000 || digest(&pixels) != r.sha256 {
            return Err("Invalid raster receipt or byte budget".into());
        }
        rasters.push(Raster {
            width: r.width,
            height: r.height,
            stride: r.stride,
            channels: r.channels,
            scale: r.scale,
            pixels,
        });
    }
    Ok(Inspection {
        pages,
        glyphs,
        graphics,
        rasters,
    })
}
fn main() -> std::result::Result<(), Error> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: displacement_check REQUEST.json".into());
    }
    let request: Request = serde_json::from_slice(&read(Path::new(&args[1]), 1_000_000)?)?;
    let a = inspection(&request.before, &request.before_pdf, &request.before_target)?;
    let b = inspection(&request.after, &request.after_pdf, &request.after_target)?;
    let policy = match request.operation {
        Policy::Identity => Operation::Identity,
        Policy::Displace { dx, dy } => Operation::Displace { dx, dy },
    };
    let result = v::compare(&a, &b, policy)?;
    println!(
        "{}",
        serde_json::json!({"status":"pass","glyphs":result.glyphs,"target_glyphs":result.target_glyphs,"pixels":result.pixels,"max_geometry_error_bp":result.max_geometry_error_bp})
    );
    Ok(())
}
