//! Request records used by the actual Tauri handlers. The opt-in generator
//! exports these and the real response records; it adds no IPC commands.
use crate::service::{EditorChange, EditorRange};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct RevisionRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct EditRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub file: String,
    pub changes: Vec<EditorChange>,
    #[cfg_attr(feature = "typegen", ts(type = "number | null"))]
    pub group: Option<u64>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SelectionRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub file: String,
    pub anchor: usize,
    pub head: usize,
    pub secondary: Vec<EditorRange>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ProfileRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub main: String,
    pub engine: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct HistoryRequest {
    pub project_id: String,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub revision: u64,
    pub redo: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CommitRequest {
    pub project_id: String,
    pub token: String,
    pub build_id: String,
}

#[cfg(feature = "typegen")]
pub mod bindings;
