//! A versioned metadata header followed by the original PDF bytes.
use crate::service::CompileResult;
use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct UnverifiedCompilePreview {
    pub schema: u8,
    #[cfg_attr(feature = "typegen", ts(type = "\"first_pass\""))]
    pub phase: &'static str,
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub request_id: u64,
    pub engine: String,
    pub seconds: f64,
    pub diagnostics: Vec<texloom_build::engine::Diagnostic>,
    #[cfg_attr(feature = "typegen", ts(type = "number[] | Uint8Array"))]
    pub pdf: Vec<u8>,
}

pub fn encode(mut result: CompileResult) -> Result<Vec<u8>, String> {
    let pdf = std::mem::take(&mut result.pdf);
    encode_parts(&result, &pdf)
}

pub fn encode_preview(mut preview: UnverifiedCompilePreview) -> Result<Vec<u8>, String> {
    let pdf = std::mem::take(&mut preview.pdf);
    encode_parts(&preview, &pdf)
}

fn encode_parts(metadata: &impl Serialize, pdf: &[u8]) -> Result<Vec<u8>, String> {
    if pdf.len() > 60 * 1024 * 1024 {
        return Err("Compiled PDF exceeds the transport limit".into());
    }
    let metadata = serde_json::to_vec(metadata).map_err(|error| error.to_string())?;
    if metadata.len() > 64 * 1024 * 1024 {
        return Err("Compile metadata exceeds the transport limit".into());
    }
    let length = u32::try_from(metadata.len()).map_err(|_| "Compile metadata is too large")?;
    let mut wire = Vec::with_capacity(8 + metadata.len() + pdf.len());
    wire.extend_from_slice(b"TLP1");
    wire.extend_from_slice(&length.to_le_bytes());
    wire.extend_from_slice(&metadata);
    wire.extend_from_slice(&pdf);
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_metadata_and_every_pdf_byte_without_json_expansion() {
        let pdf: Vec<u8> = (0..=255).collect();
        let result = CompileResult {
            ok: true, build_id: "build".into(), revision: 7, pdf: pdf.clone(),
            objects: vec![], log: "Unicode: العربية".into(),
            engine: "pdflatex".into(), seconds: 1.25, verification_eligible: true,
            passes: 2, diagnostics: vec![], job_id: 42,
        };
        let mut expected = serde_json::to_value(&result).unwrap();
        expected["pdf"] = serde_json::json!([]);
        let wire = encode(result).unwrap();
        let body = tauri::ipc::IpcResponse::body(tauri::ipc::Response::new(wire.clone())).unwrap();
        assert!(matches!(body, tauri::ipc::InvokeResponseBody::Raw(bytes) if bytes == wire));
        assert_eq!(&wire[..4], b"TLP1");
        let length = u32::from_le_bytes(wire[4..8].try_into().unwrap()) as usize;
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&wire[8..8 + length]).unwrap(), expected);
        assert_eq!(&wire[8 + length..], pdf);
    }
}
