//! Actual project/recovery/adapter command records, exported by the developer generator.
use crate::service::*;
use serde::{Deserialize, Serialize};
macro_rules! record {($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{
    #[derive(Serialize,Deserialize)] #[serde(rename_all="camelCase",deny_unknown_fields)]
    #[cfg_attr(feature="typegen",derive(ts_rs::TS))]
    pub struct $name {$(pub $field:$ty),*}
};}
record!(EmptyRequest {});
record!(OpenProjectRequest{path:Option<String>});
record!(PathRequest { path: String });
record!(CreateProjectRequest{template:String,name:String,parent:Option<String>});
record!(ProjectRequest { project_id: String });
record!(DraftRequest { draft_id: String });
record!(SourceControlsRequest { source: String });
record!(AdapterCommandRequest {
    request: AdapterRequest
});
record!(ObjectCommandRequest {
    request: ObjectEditRequest
});
record!(TransformCommandRequest {
    request: TransformRequest
});
record!(TextModelRequest {
    content: String,
    document: String
});
record!(TextPreviewRequest{content:String,document:String,edits:Vec<LiteralEditRequest>,spacing_edits:Option<Vec<SpacingEditRequest>>});
record!(EncodingPreviewRequest {
    path: String,
    file: String,
    encoding: String
});
record!(ConvertEncodingRequest{path:String,parent:String,fingerprint:String,choices:Vec<EncodingChoice>});
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CopyProjectRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub parent: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SaveRecoveryRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub transaction_id: String,
    pub finish: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct DiscardSaveRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub transaction_id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ProjectFileRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub path: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CloseProjectRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number | null"))]
    pub revision: Option<u64>,
    pub discard_recovery: Option<bool>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PrepareVisualRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub sources: std::collections::BTreeMap<String, String>,
    pub baseline_build_id: String,
    pub operation: Option<VisualOperation>,
}
