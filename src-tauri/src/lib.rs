mod service;
mod document_protocol;
mod build_protocol;
#[cfg(feature = "typegen")]
pub use document_protocol::bindings as protocol_bindings;

/// Development-only Rust harness access; adds no IPC commands or app controls.
#[cfg(feature = "benchmark")]
pub mod benchmark {
    pub use crate::service::{Backend, DocumentSnapshot, EditorChange, SourceEditReply};
}
use service::{
    Backend, CloseReply, CompileResult, DocumentSnapshot, EditorChange, EditorRange, ObjectEditRequest,
    ObjectPlan, PreparedReply, Project, RecoveryReply, ResolvedSelection, SelectionPoint, SelectionReply,
    SourceEditReply, SourceLocation, SourceRectangle, TransformRequest, VisualOperation,
};
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Manager, State};

#[tauri::command]
fn describe_text_edit(content: String, document: String) -> Result<service::TextEditModel, String> {
    service::describe_text_edit(&content, &document)
}
#[tauri::command]
fn preview_text_edit(
    content: String,
    document: String,
    edits: Vec<service::LiteralEditRequest>,
    spacing_edits: Option<Vec<service::SpacingEditRequest>>,
) -> Result<String, String> {
    service::preview_edit_parameters(
        &content,
        &content,
        &document,
        &edits,
        &spacing_edits.unwrap_or_default(),
    )
}

#[tauri::command]
async fn open_project(state: State<'_, Arc<Backend>>, path: Option<String>) -> Result<Project, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.open_project(path)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn save_document(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
) -> Result<RecoveryReply, String> {
    let request = document_protocol::RevisionRequest { project_id, revision };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.save_document(&request.project_id, request.revision))
        .await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn copy_document(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    parent: String,
) -> Result<String, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.copy_document(&project_id, revision, &parent))
        .await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn project_recoveries(state:State<'_,Arc<Backend>>,project_id:String)->Result<Vec<texloom_build::persistence::PendingSave>,String>{
    let backend=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move||backend.project_recoveries(&project_id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn recover_document(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,transaction_id:String,finish:bool)->Result<Project,String>{
    let backend=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move||backend.recover_document(&project_id,revision,&transaction_id,finish)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn discard_document_recovery(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,transaction_id:String)->Result<Vec<texloom_build::persistence::PendingSave>,String>{
    let backend=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move||backend.discard_document_recovery(&project_id,revision,&transaction_id)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn compile_document(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    token: Option<String>,
    purpose: Option<build_protocol::BuildPurpose>,
    request_id: u64,
) -> Result<CompileResult, String> {
    let request = build_protocol::CompileDocumentRequest { project_id, revision, token, purpose, request_id };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.compile_document(
            &request.project_id,
            request.revision,
            request.token.as_deref(),
            request.purpose.as_ref().map(build_protocol::BuildPurpose::as_str),
            request.request_id,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn cancel_compilation(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    request_id: u64,
    token: Option<String>,
) {
    let request = build_protocol::CancelCompilationRequest { project_id, request_id, token };
    state.cancel_compilation(&request.project_id, request.request_id, request.token.as_deref())
}
#[tauri::command]
fn pin_displayed_build(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    build_id: String,
) -> Result<(), String> {
    let request = build_protocol::ProjectBuildRequest { project_id, build_id };
    state.pin_displayed_build(&request.project_id, &request.build_id)
}
#[tauri::command]
async fn close_document(state: State<'_, Arc<Backend>>, project_id: String, revision: Option<u64>, discard_recovery: Option<bool>) -> Result<CloseReply, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(revision) = revision { backend.close_document_checked(&project_id, revision, discard_recovery.unwrap_or(false)) }
        else if discard_recovery.unwrap_or(false) { Err("A revision is required before discarding a live draft".into()) }
        else { backend.close_document(&project_id).map(|()|CloseReply::default()) }
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn mark_clean_exit(state: State<'_, Arc<Backend>>) -> Result<(), String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.mark_clean_exit()).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn typing_drafts(state: State<'_, Arc<Backend>>) -> Result<texloom_build::recovery::Inventory, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.typing_drafts()).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn open_typing_draft(state: State<'_, Arc<Backend>>, draft_id: String) -> Result<Project, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.open_typing_draft(&draft_id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn discard_typing_draft(state: State<'_, Arc<Backend>>, draft_id: String) -> Result<texloom_build::recovery::Inventory, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.discard_typing_draft(&draft_id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn retry_typing_recovery(state: State<'_, Arc<Backend>>, project_id: String, revision: u64) -> Result<RecoveryReply, String> {
    let request = document_protocol::RevisionRequest { project_id, revision };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.retry_typing_recovery(&request.project_id, request.revision)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn verify_build(state: State<'_, Arc<Backend>>, build_id: String) -> Result<(), String> {
    let request = build_protocol::BuildIdRequest { build_id };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.verify_build(&request.build_id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn export_document_build(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    build_id: String,
    path: String,
) -> Result<(), String> {
    let request = build_protocol::ExportBuildRequest { project_id, revision, build_id, path };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.export_document_build(&request.project_id, request.revision, &request.build_id, &request.path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn document_snapshot(
    state: State<'_, Arc<Backend>>,
    project_id: String,
) -> Result<DocumentSnapshot, String> {
    state.document_snapshot(&project_id)
}
#[tauri::command]
async fn document_selection(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    file: String,
    anchor: usize,
    head: usize,
    secondary: Vec<EditorRange>,
) -> Result<SelectionReply, String> {
    let request = document_protocol::SelectionRequest { project_id, revision, file, anchor, head, secondary };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.document_selection(&request.project_id, request.revision, request.file, request.anchor, request.head, request.secondary)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn document_edit(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    file: String,
    changes: Vec<EditorChange>,
    group: Option<u64>,
) -> Result<SourceEditReply, String> {
    let request = document_protocol::EditRequest { project_id, revision, file, changes, group };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.document_edit_delta(&request.project_id, request.revision, &request.file, request.changes, request.group)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn document_profile(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    main: String,
    engine: String,
) -> Result<DocumentSnapshot, String> {
    let request = document_protocol::ProfileRequest { project_id, revision, main, engine };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.document_profile(&request.project_id, request.revision, request.main, request.engine)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn document_history(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    redo: bool,
) -> Result<DocumentSnapshot, String> {
    let request = document_protocol::HistoryRequest { project_id, revision, redo };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.document_history(&request.project_id, request.revision, request.redo)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn document_prepare(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    sources: BTreeMap<String, String>,
    baseline_build_id: String,
    operation: Option<VisualOperation>,
) -> Result<PreparedReply, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.document_prepare_visual(&project_id, revision, sources, &baseline_build_id, operation)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn document_commit(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    token: String,
    build_id: String,
) -> Result<DocumentSnapshot, String> {
    let request = document_protocol::CommitRequest { project_id, token, build_id };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.document_commit(&request.project_id, &request.token, &request.build_id)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn document_discard(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    token: String,
) -> Result<(), String> {
    let request = build_protocol::DiscardCandidateRequest { project_id, token };
    state.document_discard(&request.project_id, &request.token)
}

#[tauri::command]
fn transform(request: TransformRequest) -> Result<String, String> {
    service::transform(request)
}

#[tauri::command]
fn plan_text_object(request: ObjectEditRequest) -> Result<ObjectPlan, String> {
    service::visual_plan(request)
}

#[tauri::command]
async fn resolve_text_selection(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    build_id: String,
    page: usize,
    points: Vec<SelectionPoint>,
) -> Result<ResolvedSelection, String> {
    let request = build_protocol::ResolveSelectionRequest { project_id, revision, build_id, page, points };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.resolve_text_selection(&request.project_id, request.revision, &request.build_id, request.page, request.points)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn source_from_pdf(
    state: State<'_, Arc<Backend>>,
    build_id: String,
    page: usize,
    x: f64,
    y: f64,
) -> Result<SourceLocation, String> {
    let request = build_protocol::InverseMappingRequest { build_id, page, x, y };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.inverse(&request.build_id, request.page, request.x, request.y))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pdf_from_source(
    state: State<'_, Arc<Backend>>,
    build_id: String,
    file: String,
    line: usize,
) -> Result<SourceRectangle, String> {
    let request = build_protocol::ForwardMappingRequest { build_id, file, line };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.forward(&request.build_id, &request.file, request.line))
        .await
        .map_err(|e| e.to_string())?
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            let resources = app.path().resource_dir()?;
            app.manage(Arc::new(Backend::new(root).with_verification_resources(resources)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_project,
            save_document,
            copy_document,
            project_recoveries,
            recover_document,
            discard_document_recovery,
            typing_drafts,
            open_typing_draft,
            discard_typing_draft,
            retry_typing_recovery,
            compile_document,
            cancel_compilation,
            pin_displayed_build,
            close_document,
            mark_clean_exit,
            verify_build,
            export_document_build,
            document_snapshot,
            document_selection,
            document_edit,
            document_profile,
            document_history,
            document_prepare,
            document_commit,
            document_discard,
            transform,
            source_from_pdf,
            pdf_from_source,
            plan_text_object,
            describe_text_edit,
            preview_text_edit,
            resolve_text_selection
        ])
        .run(tauri::generate_context!())
        .expect("Texloom could not start its desktop window");
}
