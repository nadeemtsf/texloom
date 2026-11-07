mod service;
mod document_protocol;
mod build_protocol;
mod workspace_protocol;
#[cfg(feature = "typegen")]
pub use document_protocol::bindings as protocol_bindings;

/// Development-only Rust harness access; adds no IPC commands or app controls.
#[cfg(feature = "benchmark")]
pub mod benchmark {
    pub use crate::service::{Backend, DocumentSnapshot, EditorChange, SourceEditReply};
    pub use crate::service::{visual_plan, ObjectEditRequest, SelectionPoint};
}
use service::{
    Backend, CloseReply, CompileResult, DocumentSnapshot, EditorChange, EditorRange, ObjectEditRequest,
    ObjectPlan, PreparedReply, Project, RecoveryReply, ResolvedSelection, SelectionPoint, SelectionReply,
    SourceEditReply, SourceLocation, SourceRectangle, TransformRequest, VisualOperation,
};
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Manager, State};
async fn background<T:Send+'static>(operation:impl FnOnce()->Result<T,String>+Send+'static)->Result<T,String>{tauri::async_runtime::spawn_blocking(operation).await.map_err(|e|e.to_string())?}

#[tauri::command]
async fn describe_adapters(source:String)->Result<Vec<service::AdapterControl>,String>{let request=workspace_protocol::SourceControlsRequest{source};background(move||service::describe_adapters(&request.source)).await}
#[tauri::command]
async fn plan_adapter(request:service::AdapterRequest)->Result<service::AdapterPlan,String>{let request=workspace_protocol::AdapterCommandRequest{request};background(move||service::plan_adapter(request.request)).await}

#[tauri::command]
async fn describe_text_edit(content: String, document: String) -> Result<service::TextEditModel, String> {
    let request=workspace_protocol::TextModelRequest{content,document};background(move||service::describe_text_edit(&request.content, &request.document)).await
}
#[tauri::command]
async fn preview_text_edit(
    content: String,
    document: String,
    edits: Vec<service::LiteralEditRequest>,
    spacing_edits: Option<Vec<service::SpacingEditRequest>>,
) -> Result<String, String> {
    let request=workspace_protocol::TextPreviewRequest{content,document,edits,spacing_edits};
    background(move||service::preview_edit_parameters(&request.content,&request.content,&request.document,&request.edits,&request.spacing_edits.unwrap_or_default())).await
}

#[tauri::command]
async fn open_project(state: State<'_, Arc<Backend>>, path: Option<String>) -> Result<Project, String> {
    let request=workspace_protocol::OpenProjectRequest{path};
    let backend = state.inner().clone();
    background(move || backend.open_project(request.path)).await
}

#[tauri::command]
async fn recent_projects(state:State<'_,Arc<Backend>>)->Result<Vec<service::RecentProject>,String>{let _request=workspace_protocol::EmptyRequest{};let backend=state.inner().clone();background(move||backend.recent_projects()).await}
#[tauri::command]
async fn create_project(state:State<'_,Arc<Backend>>,template:String,name:String,parent:Option<String>)->Result<service::ImportedProject,String>{let request=workspace_protocol::CreateProjectRequest{template,name,parent};let backend=state.inner().clone();background(move||backend.create_project(&request.template,&request.name,request.parent.as_deref())).await}
#[tauri::command]
async fn open_recent(state:State<'_,Arc<Backend>>,path:String)->Result<Project,String>{let request=workspace_protocol::PathRequest{path};let backend=state.inner().clone();background(move||backend.open_recent(&request.path)).await}
#[tauri::command]
async fn import_project_archive(state:State<'_,Arc<Backend>>,path:String)->Result<service::ImportedProject,String>{let request=workspace_protocol::PathRequest{path};let backend=state.inner().clone();background(move||backend.import_project_archive(&request.path)).await}
#[tauri::command]
async fn export_project_archive(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,path:String)->Result<(),String>{let request=workspace_protocol::ProjectFileRequest{project_id,revision,path};let backend=state.inner().clone();background(move||backend.export_project_archive(&request.project_id,request.revision,&request.path)).await}
#[tauri::command]
async fn review_encoding(state:State<'_,Arc<Backend>>,path:String)->Result<service::EncodingReview,String>{let request=workspace_protocol::PathRequest{path};let backend=state.inner().clone();background(move||backend.review_encoding(&request.path)).await}
#[tauri::command]
async fn preview_encoding(state:State<'_,Arc<Backend>>,path:String,file:String,encoding:String)->Result<String,String>{let request=workspace_protocol::EncodingPreviewRequest{path,file,encoding};let backend=state.inner().clone();background(move||backend.preview_encoding(&request.path,&request.file,&request.encoding)).await}
#[tauri::command]
async fn convert_encoding(state:State<'_,Arc<Backend>>,path:String,parent:String,fingerprint:String,choices:Vec<service::EncodingChoice>)->Result<service::ImportedProject,String>{let request=workspace_protocol::ConvertEncodingRequest{path,parent,fingerprint,choices};let backend=state.inner().clone();background(move||backend.convert_encoding(&request.path,&request.parent,&request.fingerprint,request.choices)).await}
#[tauri::command]
async fn import_image(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,path:String)->Result<service::ImportedImage,String>{let request=workspace_protocol::ProjectFileRequest{project_id,revision,path};let backend=state.inner().clone();background(move||backend.import_image(&request.project_id,request.revision,&request.path)).await}

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
    let request=workspace_protocol::CopyProjectRequest{project_id,revision,parent};
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.copy_document(&request.project_id, request.revision, &request.parent))
        .await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn project_recoveries(state:State<'_,Arc<Backend>>,project_id:String)->Result<Vec<texloom_build::persistence::PendingSave>,String>{
    let request=workspace_protocol::ProjectRequest{project_id};
    let backend=state.inner().clone();
    background(move||backend.project_recoveries(&request.project_id)).await
}
#[tauri::command]
async fn recover_document(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,transaction_id:String,finish:bool)->Result<Project,String>{
    let request=workspace_protocol::SaveRecoveryRequest{project_id,revision,transaction_id,finish};
    let backend=state.inner().clone();
    background(move||backend.recover_document(&request.project_id,request.revision,&request.transaction_id,request.finish)).await
}
#[tauri::command]
async fn discard_document_recovery(state:State<'_,Arc<Backend>>,project_id:String,revision:u64,transaction_id:String)->Result<Vec<texloom_build::persistence::PendingSave>,String>{
    let request=workspace_protocol::DiscardSaveRequest{project_id,revision,transaction_id};
    let backend=state.inner().clone();
    background(move||backend.discard_document_recovery(&request.project_id,request.revision,&request.transaction_id)).await
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
    let request=workspace_protocol::CloseProjectRequest{project_id,revision,discard_recovery};
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(revision) = request.revision { backend.close_document_checked(&request.project_id, revision, request.discard_recovery.unwrap_or(false)) }
        else if request.discard_recovery.unwrap_or(false) { Err("A revision is required before discarding a live draft".into()) }
        else { backend.close_document(&request.project_id).map(|()|CloseReply::default()) }
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn mark_clean_exit(state: State<'_, Arc<Backend>>) -> Result<(), String> {
    let _request=workspace_protocol::EmptyRequest{};
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.mark_clean_exit()).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn typing_drafts(state: State<'_, Arc<Backend>>) -> Result<texloom_build::recovery::Inventory, String> {
    let _request=workspace_protocol::EmptyRequest{};
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.typing_drafts()).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn open_typing_draft(state: State<'_, Arc<Backend>>, draft_id: String) -> Result<Project, String> {
    let request=workspace_protocol::DraftRequest{draft_id};
    let backend = state.inner().clone();
    background(move || backend.open_typing_draft(&request.draft_id)).await
}
#[tauri::command]
async fn discard_typing_draft(state: State<'_, Arc<Backend>>, draft_id: String) -> Result<texloom_build::recovery::Inventory, String> {
    let request=workspace_protocol::DraftRequest{draft_id};
    let backend = state.inner().clone();
    background(move || backend.discard_typing_draft(&request.draft_id)).await
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
    let request=workspace_protocol::ProjectRequest{project_id};state.document_snapshot(&request.project_id)
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
    let request=workspace_protocol::PrepareVisualRequest{project_id,revision,sources,baseline_build_id,operation};
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.document_prepare_visual(&request.project_id, request.revision, request.sources, &request.baseline_build_id, request.operation)
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
async fn transform(request: TransformRequest) -> Result<String, String> {
    let request=workspace_protocol::TransformCommandRequest{request};background(move||service::transform(request.request)).await
}

#[tauri::command]
async fn plan_text_object(request: ObjectEditRequest) -> Result<ObjectPlan, String> {
    let request=workspace_protocol::ObjectCommandRequest{request};background(move||service::visual_plan(request.request)).await
}

#[tauri::command]
async fn resolve_text_selection(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    build_id: String,
    page: usize,
    points: Vec<SelectionPoint>,
    candidate: usize,
) -> Result<ResolvedSelection, String> {
    let request = build_protocol::ResolveSelectionRequest { project_id, revision, build_id, page, points, candidate };
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.resolve_text_selection(&request.project_id, request.revision, &request.build_id, request.page, request.points, request.candidate)
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
            recent_projects,create_project,open_recent,import_project_archive,export_project_archive,review_encoding,preview_encoding,convert_encoding,import_image,
            describe_adapters,
            plan_adapter,
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
