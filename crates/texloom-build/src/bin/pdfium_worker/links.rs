use super::{
    api::{Handle, PageRect, Pdfium, Result},
    check, digest, rectangle,
};
use serde_json::{json, Value};

pub(super) fn signatures(
    api: &Pdfium,
    document: Handle,
    page: Handle,
    count: i32,
) -> Result<Vec<String>> {
    (0..count)
        .map(|index| {
            let annot = api.own(
                unsafe { (api.FPDFPage_GetAnnot)(page, index) },
                api.FPDFPage_CloseAnnot,
            )?;
            let link = unsafe { (api.FPDFAnnot_GetLink)(annot.raw) };
            check(
                !link.is_null() && unsafe { (api.FPDFAnnot_CountAttachmentPoints)(annot.raw) } == 0,
                "This PDF annotation needs a separate inline editing adapter",
            )?;
            let mut rect = PageRect::default();
            check(
                unsafe { (api.FPDFAnnot_GetRect)(annot.raw, &mut rect) } != 0,
                "Missing PDF link rectangle",
            )?;
            let bounds = [rect.left, rect.bottom, rect.right, rect.top].map(f64::from);
            rectangle(bounds)?;
            let action = unsafe { (api.FPDFLink_GetAction)(link) };
            let kind = if action.is_null() {
                0
            } else {
                unsafe { (api.FPDFAction_GetType)(action) }
            };
            let destination = match kind {
                0 | 1 => {
                    let dest = unsafe { (api.FPDFLink_GetDest)(document, link) };
                    check(!dest.is_null(), "Missing local PDF link destination")?;
                    local_destination(api, document, dest)?
                }
                3 => {
                    let length = unsafe {
                        (api.FPDFAction_GetURIPath)(document, action, std::ptr::null_mut(), 0)
                    };
                    check(
                        (2..=16_384).contains(&length),
                        "Invalid PDF link URI length",
                    )?;
                    let mut uri = vec![0u8; length as usize];
                    let copied = unsafe {
                        (api.FPDFAction_GetURIPath)(
                            document,
                            action,
                            uri.as_mut_ptr().cast(),
                            length,
                        )
                    };
                    check(
                        copied == length && uri.last() == Some(&0),
                        "Invalid PDF link URI",
                    )?;
                    json!(uri)
                }
                _ => {
                    return Err(
                        "This PDF link action needs a separate inline editing adapter".into(),
                    )
                }
            };
            let flags = unsafe { (api.FPDFAnnot_GetFlags)(annot.raw) };
            Ok(digest(
                &serde_json::to_vec(&(bounds, flags, kind, destination))
                    .map_err(|error| error.to_string())?,
            ))
        })
        .collect()
}

fn local_destination(api: &Pdfium, document: Handle, dest: Handle) -> Result<Value> {
    let index = unsafe { (api.FPDFDest_GetDestPageIndex)(document, dest) };
    let mut count = 0u32;
    let mut parameters = [0f32; 4];
    let mode = unsafe { (api.FPDFDest_GetView)(dest, &mut count, parameters.as_mut_ptr()) };
    check(
        index >= 0 && mode != 0 && count <= 4 && parameters.iter().all(|n| n.is_finite()),
        "Invalid local PDF link destination",
    )?;
    // For XYZ, retain the distinction between a fixed coordinate and null.
    let (mut hx, mut hy, mut hz, mut x, mut y, mut z) = (0, 0, 0, 0f32, 0f32, 0f32);
    let xyz = unsafe {
        (api.FPDFDest_GetLocationInPage)(dest, &mut hx, &mut hy, &mut hz, &mut x, &mut y, &mut z)
    };
    check(
        [x, y, z].iter().all(|n| n.is_finite()),
        "Invalid PDF link coordinates",
    )?;
    Ok(json!([
        index,
        mode,
        &parameters[..count as usize],
        xyz,
        hx,
        hy,
        hz,
        x,
        y,
        z
    ]))
}
