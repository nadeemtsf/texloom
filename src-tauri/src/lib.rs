mod service;
use service::{
    Backend, CompileResult, DocumentSnapshot, EditorChange, EditorRange, ObjectEditRequest,
    ObjectPlan, PreparedReply, Project, ResolvedSelection, SelectionPoint, SelectionReply,
    SourceLocation, SourceRectangle, TransformRequest,
};
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Manager, State};

#[tauri::command]
fn open_project(state: State<'_, Arc<Backend>>, path: Option<String>) -> Result<Project, String> {
    state.open_project(path)
}

#[tauri::command]
fn save_document(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
) -> Result<(), String> {
    state.save_document(&project_id, revision)
}

#[tauri::command]
async fn compile_document(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    token: Option<String>,
    purpose: Option<String>,
    request_id: u64,
) -> Result<CompileResult, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.compile_document(
            &project_id,
            revision,
            token.as_deref(),
            purpose.as_deref(),
            request_id,
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
    state.cancel_compilation(&project_id, request_id, token.as_deref())
}
#[tauri::command]
fn pin_displayed_build(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    build_id: String,
) -> Result<(), String> {
    state.pin_displayed_build(&project_id, &build_id)
}
#[tauri::command]
fn close_document(state: State<'_, Arc<Backend>>, project_id: String) -> Result<(), String> {
    state.close_document(&project_id)
}
#[tauri::command]
async fn verify_build(state: State<'_, Arc<Backend>>, build_id: String) -> Result<(), String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.verify_build(&build_id))
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
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.export_document_build(&project_id, revision, &build_id, &path)
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
fn document_selection(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    file: String,
    anchor: usize,
    head: usize,
    secondary: Vec<EditorRange>,
) -> Result<SelectionReply, String> {
    state.document_selection(&project_id, revision, file, anchor, head, secondary)
}
#[tauri::command]
fn document_edit(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    file: String,
    changes: Vec<EditorChange>,
    group: Option<u64>,
) -> Result<DocumentSnapshot, String> {
    state.document_edit(&project_id, revision, &file, changes, group)
}
#[tauri::command]
fn document_profile(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    main: String,
    engine: String,
) -> Result<DocumentSnapshot, String> {
    state.document_profile(&project_id, revision, main, engine)
}
#[tauri::command]
fn document_history(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    redo: bool,
) -> Result<DocumentSnapshot, String> {
    state.document_history(&project_id, revision, redo)
}
#[tauri::command]
async fn document_prepare(
    state: State<'_, Arc<Backend>>,
    project_id: String,
    revision: u64,
    sources: BTreeMap<String, String>,
    baseline_build_id: String,
) -> Result<PreparedReply, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.verify_project_build(&project_id, revision, &baseline_build_id, true)?;
        backend.document_prepare(&project_id, revision, sources)
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
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.document_commit(&project_id, &token, &build_id)
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
    state.document_discard(&project_id, &token)
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
    build_id: String,
    page: usize,
    points: Vec<SelectionPoint>,
) -> Result<ResolvedSelection, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backend.resolve_text_selection(&build_id, page, points)
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
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.inverse(&build_id, page, x, y))
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
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.forward(&build_id, &file, line))
        .await
        .map_err(|e| e.to_string())?
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            app.manage(Arc::new(Backend::new(root)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_project,
            save_document,
            compile_document,
            cancel_compilation,
            pin_displayed_build,
            close_document,
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
            resolve_text_selection
        ])
        .run(tauri::generate_context!())
        .expect("Texloom could not start its desktop window");
}
