mod api;
mod target;
use api::{Pdfium, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: u32,
    request_id: String,
    library: PathBuf,
    pdf: PathBuf,
    pdf_sha256: String,
    #[serde(default)]
    target_marker: Option<String>,
    #[serde(default)]
    target_translation: Option<[f64;2]>,
}
fn read(path: &Path, max: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("Input exceeds worker byte budget".into());
    }
    Ok(bytes)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn create(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
fn check(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn bounded(count: i32, max: i32) -> Result<i32> {
    check(
        (0..=max).contains(&count),
        "Invalid or excessive PDF object count",
    )?;
    Ok(count)
}
fn rectangle(values: [f64; 4]) -> Result<()> {
    check(
        values.iter().all(|n| n.is_finite()) && values[2] >= values[0] && values[3] >= values[1],
        "Invalid PDF object bounds",
    )
}
fn inspect(request: &Request, prefix: &Path) -> Result<Value> {
    let bytes = read(&request.pdf, 40_000_000)?;
    check(
        digest(&bytes) == request.pdf_sha256,
        "PDF bytes changed before independent inspection",
    )?;
    let api = Pdfium::load(&request.library)?;
    // `bytes` remains alive and immovable until the document guard closes.
    let document = api.own(
        unsafe {
            (api.FPDF_LoadMemDocument64)(bytes.as_ptr().cast(), bytes.len(), std::ptr::null())
        },
        api.FPDF_CloseDocument,
    )?;
    let page_count = bounded(unsafe { (api.FPDF_GetPageCount)(document.raw) }, 20)?;
    check(page_count > 0, "The PDF has no physical pages")?;
    let (mut pages, mut records, mut graphics, mut rasters) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut total_pixels = 0usize;
    let mut instances = std::collections::BTreeSet::new();
    let mut target_glyphs = 0usize;
    for index in 0..page_count {
        let page = api.own(
            unsafe { (api.FPDF_LoadPage)(document.raw, index) },
            api.FPDF_ClosePage,
        )?;
        let width = unsafe { (api.FPDF_GetPageWidthF)(page.raw) } as f64;
        let height = unsafe { (api.FPDF_GetPageHeightF)(page.raw) } as f64;
        check(
            width.is_finite()
                && height.is_finite()
                && width > 0.0
                && height > 0.0
                && width <= 14_400.0
                && height <= 14_400.0,
            "Invalid PDF page dimensions",
        )?;
        // Use the page's effective box, including inherited MediaBox/CropBox
        // and their intersection, exactly as the pinned renderer does.
        let mut bounds = api::PageRect::default();
        check(
            unsafe { (api.FPDF_GetPageBoundingBox)(page.raw, &mut bounds) } != 0,
            "The PDF has no usable page box",
        )?;
        let crop = [bounds.left, bounds.bottom, bounds.right, bounds.top].map(f64::from);
        rectangle(crop)?;
        let rotation = unsafe { (api.FPDFPage_GetRotation)(page.raw) };
        check(
            rotation == 0 && crop[0].abs() < 1e-6 && crop[1].abs() < 1e-6,
            "Crop/rotation needs a coordinate adapter",
        )?;
        let annotations = bounded(unsafe { (api.FPDFPage_GetAnnotCount)(page.raw) }, 10_000)?;
        pages.push(json!({"width":width,"height":height,"crop":crop,"rotation":rotation*90,"annotations":annotations}));
        let text = api.own(
            unsafe { (api.FPDFText_LoadPage)(page.raw) },
            api.FPDFText_ClosePage,
        )?;
        let record_start = records.len();
        let mut text_objects = Vec::new();
        let mut selected_objects = Vec::new();
        let chars = bounded(unsafe { (api.FPDFText_CountChars)(text.raw) }, 50_000)?;
        for i in 0..chars {
            let code = unsafe { (api.FPDFText_GetUnicode)(text.raw, i) };
            let value = char::from_u32(code).ok_or("Invalid Unicode character in PDF")?;
            if value.is_whitespace() {
                continue;
            }
            check(code != 0, "PDF text has no Unicode mapping")?;
            let (mut left, mut right, mut bottom, mut top) = (0.0, 0.0, 0.0, 0.0);
            check(
                unsafe {
                    (api.FPDFText_GetCharBox)(
                        text.raw,
                        i,
                        &mut left,
                        &mut right,
                        &mut bottom,
                        &mut top,
                    )
                } != 0,
                "Missing PDF character geometry",
            )?;
            rectangle([left, bottom, right, top])?;
            check(records.len() < 50_000, "Document exceeds glyph budget")?;
            records.push(json!({"text":value.to_string(),"page":index+1,"x":left,"y":height-top,"width":right-left,"height":top-bottom}));
        }
        let objects = bounded(unsafe { (api.FPDFPage_CountObjects)(page.raw) }, 10_000)?;
        for i in 0..objects {
            let object = unsafe { (api.FPDFPage_GetObject)(page.raw, i) };
            check(!object.is_null(), "Missing page object")?;
            let kind = unsafe { (api.FPDFPageObj_GetType)(object) };
            check(
                (1..=4).contains(&kind)||(kind==5&&request.target_marker.is_none()),
                "Form XObjects or unknown paint require an inspection adapter",
            )?;
            let selected = if let Some(name) = &request.target_marker {
                target::marked(&api, object, name, index, &mut instances)?
            } else {
                false
            };
            check(!selected || kind == 1, "Target contains non-text paint")?;
            if selected {selected_objects.push(object);}
            if kind == 1 && request.target_marker.is_none() {
                continue;
            }
            let (mut left, mut bottom, mut right, mut top) = (0f32, 0f32, 0f32, 0f32);
            check(
                unsafe {
                    (api.FPDFPageObj_GetBounds)(
                        object,
                        &mut left,
                        &mut bottom,
                        &mut right,
                        &mut top,
                    )
                } != 0,
                "Missing non-text object geometry",
            )?;
            rectangle([left as f64, bottom as f64, right as f64, top as f64])?;
            if kind == 1 {
                text_objects.push(target::TextObject {
                    bounds: [left as f64, bottom as f64, right as f64, top as f64],
                    selected,
                });
                continue;
            }
            check(
                graphics.len() < 10_000,
                "Document exceeds non-text object budget",
            )?;
            graphics.push(json!({"type":kind,"page":index+1,"x":left as f64,"y":height-top as f64,"width":right as f64-left as f64,"height":top as f64-bottom as f64}));
        }
        if request.target_marker.is_some() {
            target_glyphs +=
                target::attribute(&mut records[record_start..], &text_objects, height)?;
        }
        // Predict the requested drawing with the inspector's own object API,
        // independently of TeX literals or candidate pixels. Geometry records
        // remain the original baseline; only this prediction's raster changes.
        if let Some([dx,dy])=request.target_translation {for object in selected_objects {unsafe{(api.FPDFPageObj_Transform)(object,1.0,0.0,0.0,1.0,dx,-dy);}}}
        let (w, h) = (
            (width * 2.0).ceil() as usize,
            (height * 2.0).ceil() as usize,
        );
        let count = w.checked_mul(h).ok_or("Raster budget overflow")?;
        total_pixels = total_pixels
            .checked_add(count)
            .ok_or("Raster budget overflow")?;
        check(
            count <= 16_000_000 && total_pixels <= 64_000_000,
            "PDF exceeds raster budget",
        )?;
        let bitmap = api.own(
            unsafe { (api.FPDFBitmap_CreateEx)(w as i32, h as i32, 2, std::ptr::null_mut(), 0) },
            api.FPDFBitmap_Destroy,
        )?;
        check(
            unsafe { (api.FPDFBitmap_FillRect)(bitmap.raw, 0, 0, w as i32, h as i32, 0xffffffff) } != 0,
            "PDFium could not initialize the raster background",
        )?;
        unsafe {
            (api.FPDF_RenderPageBitmap)(bitmap.raw, page.raw, 0, 0, w as i32, h as i32, 0, 1);
        }
        let stride = usize::try_from(unsafe { (api.FPDFBitmap_GetStride)(bitmap.raw) })
            .map_err(|_| "Invalid bitmap stride")?;
        let length = stride.checked_mul(h).ok_or("Bitmap length overflow")?;
        check(
            stride >= w * 3 && length <= 64_000_000,
            "Invalid or excessive bitmap buffer",
        )?;
        let buffer = unsafe { (api.FPDFBitmap_GetBuffer)(bitmap.raw) };
        check(!buffer.is_null(), "PDFium returned no raster buffer")?;
        // PDFium owns this exact stride*height allocation until bitmap drops.
        let pixels = unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), length) };
        let path = prefix.with_extension(format!("p{}.raster", index + 1));
        create(&path, pixels)?;
        rasters.push(json!({"path":path,"width":w,"height":h,"stride":stride,"channels":3,"scale":2,"sha256":digest(pixels)}));
    }
    if request.target_marker.is_some() {
        check(
            instances.len() == 1,
            "Target marker is missing or has multiple rendered instances",
        )?;
        check(target_glyphs > 0, "Target marker has no visible glyphs")?;
    }
    Ok(
        json!({"schema":texloom_build::inspection_worker::PROTOCOL,"renderer":"pdfium","pdfium":texloom_build::inspection_worker::PDFIUM_VERSION,"binding":"rust-worker-v1",
        "librarySha256":api::DLL_SHA256,"pdfSha256":request.pdf_sha256,
            "pages":pages,"records":records,"graphics":graphics,"rasters":rasters,
            "target":request.target_marker.as_ref().map(|name|json!({"marker":name,"instances":instances.len(),"glyphs":target_glyphs})),"translation":request.target_translation,
            "limitations":["Experimental Windows worker; broader marker attribution, UserUnit and native commit binding need acceptance.","Forms and nonzero page origins/rotation are refused."]}),
    )
}
pub fn run() -> Result<()> {
    check(
        cfg!(target_arch = "x86_64"),
        "This development PDFium pin requires Windows x64",
    )?;
    let args: Vec<_> = std::env::args_os().collect();
    check(args.len() == 2, "usage: pdfium_worker REQUEST.json")?;
    let path = fs::canonicalize(&args[1]).map_err(|e| e.to_string())?;
    let request_bytes = read(&path, 64_000)?;
    let request: Request = serde_json::from_slice(&request_bytes).map_err(|e| e.to_string())?;
    check(
        request.schema == texloom_build::inspection_worker::PROTOCOL,
        "Unsupported worker protocol",
    )?;
    check(
        request.request_id.len() == 64
            && request
                .request_id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "Invalid inspection request identity",
    )?;
    if let Some(marker) = &request.target_marker {
        check(
            marker.starts_with("Texloom_")
                && marker.len() > 8
                && marker.len() <= 100
                && marker
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
            "Invalid target marker",
        )?;
    }
    if let Some([dx,dy])=request.target_translation {check(request.target_marker.is_some()&&dx.is_finite()&&dy.is_finite()&&dx.abs()<=2000.0&&dy.abs()<=2000.0,"Invalid target raster prediction")?;}
    let prefix = path.with_extension("inspection.json");
    // All output names derive from the request file in its private directory.
    check(
        !prefix.exists(),
        "Independent inspection output already exists",
    )?;
    let mut result = inspect(&request, &prefix)?;
    result["requestId"] = json!(request.request_id);
    result["requestSha256"] = json!(digest(&request_bytes));
    let bytes = serde_json::to_vec(&result).map_err(|e| e.to_string())?;
    create(&prefix, &bytes)?;
    println!(
        "{}",
        json!({"schema":texloom_build::inspection_worker::PROTOCOL,"status":"inspected","request_id":request.request_id,"request_sha256":digest(&request_bytes),"inspection":prefix,"sha256":digest(&bytes)})
    );
    Ok(())
}
