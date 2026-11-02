//! Conservative experimental marker-to-glyph attribution. No client rectangle
//! or visible-string lookup establishes target identity here.
use super::{
    api::{Handle, Pdfium, Result},
    check,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
pub struct TextObject {
    pub bounds: [f64; 4],
    pub selected: bool,
}
pub fn marked(
    api: &Pdfium,
    object: Handle,
    name: &str,
    page: i32,
    instances: &mut BTreeSet<(i32, usize)>,
) -> Result<bool> {
    let count = unsafe { (api.FPDFPageObj_CountMarks)(object) };
    check(
        (0..=64).contains(&count),
        "Invalid or excessive marked-content stack",
    )?;
    let (mut hit, mut texloom) = (false, 0);
    for index in 0..count {
        let mark = unsafe { (api.FPDFPageObj_GetMark)(object, index as u32) };
        check(!mark.is_null(), "Missing marked-content handle")?;
        let mut bytes = 0u32;
        check(
            unsafe { (api.FPDFPageObjMark_GetName)(mark, std::ptr::null_mut(), 0, &mut bytes) }
                != 0,
            "Unreadable marked-content name",
        )?;
        check(
            (2..=1024).contains(&bytes) && bytes % 2 == 0,
            "Invalid marked-content name length",
        )?;
        let mut buffer = vec![0u16; (bytes / 2) as usize];
        let capacity = bytes;
        check(
            unsafe {
                (api.FPDFPageObjMark_GetName)(
                    mark,
                    buffer.as_mut_ptr().cast(),
                    capacity,
                    &mut bytes,
                )
            } != 0
                && bytes == capacity
                && buffer.last() == Some(&0),
            "Marked-content name changed",
        )?;
        let value = String::from_utf16(&buffer[..buffer.len() - 1])
            .map_err(|_| "Invalid marked-content Unicode")?;
        if value.starts_with("Texloom_") {
            texloom += 1;
        }
        if value == name {
            hit = true;
            instances.insert((page, mark as usize));
        }
    }
    check(
        !hit || texloom == 1,
        "Nested Texloom identities are unsupported",
    )?;
    Ok(hit)
}
pub fn attribute(records: &mut [Value], objects: &[TextObject], height: f64) -> Result<usize> {
    check(
        records
            .len()
            .checked_mul(objects.len())
            .is_some_and(|n| n <= 5_000_000),
        "Target attribution budget exceeded",
    )?;
    let mut selected = 0;
    for record in records {
        let x = record["x"].as_f64().ok_or("Missing glyph x")?;
        let top = height - record["y"].as_f64().ok_or("Missing glyph y")?;
        let right = x + record["width"].as_f64().ok_or("Missing glyph width")?;
        let bottom = top - record["height"].as_f64().ok_or("Missing glyph height")?;
        let mut membership = None;
        for object in objects {
            let [l, b, r, t] = object.bounds;
            if x >= l - 0.02 && right <= r + 0.02 && bottom >= b - 0.02 && top <= t + 0.02 {
                check(
                    membership.is_none_or(|value| value == object.selected),
                    "Glyph attribution crosses target and neighbor objects",
                )?;
                membership = Some(object.selected);
            }
        }
        let target = membership.ok_or("Glyph has no unambiguous text-object coverage")?;
        record["target"] = json!(target);
        selected += usize::from(target);
    }
    Ok(selected)
}
