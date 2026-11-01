mod service;
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Manager, State};
use service::{Backend, CompileResult, Project, SourceLocation, SourceRectangle, TransformRequest, ObjectEditRequest, ObjectPlan, SelectionPoint, ResolvedSelection, DocumentSnapshot, EditorChange, EditorRange, PreparedReply, SelectionReply};

#[tauri::command]
fn open_project(state: State<'_, Arc<Backend>>, path: Option<String>) -> Result<Project, String> {
    state.open_project(path)
}

#[tauri::command]
fn save_document(state: State<'_, Arc<Backend>>, project_id: String, revision: u64) -> Result<(), String> {
    state.save_document(&project_id, revision)
}

#[tauri::command]
async fn compile_document(state: State<'_, Arc<Backend>>, project_id: String, revision:u64, token: Option<String>) -> Result<CompileResult, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.compile_document(&project_id, revision, token.as_deref())).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn document_snapshot(state: State<'_, Arc<Backend>>, project_id:String)->Result<DocumentSnapshot,String> {state.document_snapshot(&project_id)}
#[tauri::command]
fn document_selection(state: State<'_, Arc<Backend>>,project_id:String,revision:u64,file:String,anchor:usize,head:usize,secondary:Vec<EditorRange>)->Result<SelectionReply,String> {
    state.document_selection(&project_id,revision,file,anchor,head,secondary)
}
#[tauri::command]
fn document_edit(state: State<'_, Arc<Backend>>,project_id:String,revision:u64,file:String,changes:Vec<EditorChange>,group:Option<u64>)->Result<DocumentSnapshot,String> {
    state.document_edit(&project_id,revision,&file,changes,group)
}
#[tauri::command]
fn document_profile(state: State<'_, Arc<Backend>>,project_id:String,revision:u64,main:String,engine:String)->Result<DocumentSnapshot,String> {
    state.document_profile(&project_id,revision,main,engine)
}
#[tauri::command]
fn document_history(state: State<'_, Arc<Backend>>,project_id:String,revision:u64,redo:bool)->Result<DocumentSnapshot,String> {
    state.document_history(&project_id,revision,redo)
}
#[tauri::command]
fn document_prepare(state: State<'_, Arc<Backend>>,project_id:String,revision:u64,sources:BTreeMap<String,String>)->Result<PreparedReply,String> {
    state.document_prepare(&project_id,revision,sources)
}
#[tauri::command]
fn document_commit(state: State<'_, Arc<Backend>>,project_id:String,token:String,build_id:String)->Result<DocumentSnapshot,String> {
    state.document_commit(&project_id,&token,&build_id)
}
#[tauri::command]
fn document_discard(state: State<'_, Arc<Backend>>,project_id:String,token:String)->Result<(),String> {state.document_discard(&project_id,&token)}

#[tauri::command]
fn transform(request: TransformRequest) -> Result<String, String> { service::transform(request) }

#[tauri::command]
fn plan_text_object(request: ObjectEditRequest) -> Result<ObjectPlan, String> { service::visual_plan(request) }

#[tauri::command]
async fn resolve_text_selection(state: State<'_, Arc<Backend>>, build_id: String, page: usize, points: Vec<SelectionPoint>) -> Result<ResolvedSelection, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.resolve_text_selection(&build_id, page, points)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn source_from_pdf(state: State<'_, Arc<Backend>>, build_id: String, page: usize, x: f64, y: f64) -> Result<SourceLocation, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.inverse(&build_id, page, x, y)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn export_pdf(state: State<'_, Arc<Backend>>, build_id: String, path: String) -> Result<(), String> {
    state.export_pdf(&build_id, &path)
}

#[tauri::command]
async fn pdf_from_source(state: State<'_, Arc<Backend>>, build_id: String, file: String, line: usize) -> Result<SourceRectangle, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.forward(&build_id, &file, line)).await.map_err(|e| e.to_string())?
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
        .invoke_handler(tauri::generate_handler![open_project, save_document, compile_document, document_snapshot, document_selection, document_edit, document_profile, document_history, document_prepare, document_commit, document_discard, transform, source_from_pdf, pdf_from_source, export_pdf, plan_text_object, resolve_text_selection])
        .run(tauri::generate_context!())
        .expect("Texloom could not start its desktop window");
}
