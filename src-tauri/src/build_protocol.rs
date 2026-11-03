//! Flat command records shared by the real compiler/mapping handlers and type export.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum BuildPurpose { Preview, Visual, Export }
impl BuildPurpose {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Preview => "preview", Self::Visual => "visual", Self::Export => "export" }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CompileDocumentRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub token: Option<String>,
    pub purpose: Option<BuildPurpose>,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub request_id: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CancelCompilationRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub request_id: u64,
    pub token: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct BuildIdRequest { pub build_id: String }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ProjectBuildRequest { pub project_id: String, pub build_id: String }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct DiscardCandidateRequest { pub project_id: String, pub token: String }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ExportBuildRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub build_id: String,
    pub path: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ForwardMappingRequest { pub build_id: String, pub file: String, pub line: usize }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct InverseMappingRequest { pub build_id: String, pub page: usize, pub x: f64, pub y: f64 }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ResolveSelectionRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub build_id: String,
    pub page: usize,
    pub points: Vec<crate::service::SelectionPoint>,
}
