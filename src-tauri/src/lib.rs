mod service;
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Manager, State};
use service::{Backend, CompileRequest, CompileResult, Project, SourceLocation, SourceRectangle, TransformRequest, ObjectEditRequest, ObjectPlan, SelectionPoint, ResolvedSelection};

#[tauri::command]
fn open_project(state: State<'_, Arc<Backend>>, path: Option<String>) -> Result<Project, String> {
    state.open_project(path)
}

#[tauri::command]
fn save_project(state: State<'_, Arc<Backend>>, project_id: String, sources: BTreeMap<String, String>) -> Result<(), String> {
    state.save_project(&project_id, &sources)
}

#[tauri::command]
async fn compile_project(state: State<'_, Arc<Backend>>, request: CompileRequest) -> Result<CompileResult, String> {
    let backend = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || backend.compile(request)).await.map_err(|e| e.to_string())?
}

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
        .invoke_handler(tauri::generate_handler![open_project, save_project, compile_project, transform, source_from_pdf, pdf_from_source, export_pdf, plan_text_object, resolve_text_selection])
        .run(tauri::generate_context!())
        .expect("Texloom could not start its desktop window");
}
