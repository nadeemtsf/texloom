//! Headless development bridge to native draft endpoints; no window, TeX or Save.
use serde_json::{json, Value};
use std::{fs, io::{self, BufRead, Write}, path::{Path, PathBuf}};
use texloom_desktop_lib::benchmark::Backend;
type Result<T> = std::result::Result<T, String>;

fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key).and_then(Value::as_str).ok_or_else(|| format!("Missing string: {key}"))
}
fn revision(v: &Value) -> Result<u64> {
    v.get("revision").and_then(Value::as_u64).ok_or_else(|| "Missing revision".into())
}
fn emit(value: Value) -> Result<()> {
    println!("{value}"); io::stdout().flush().map_err(|e| e.to_string())
}
fn write_new(path: &Path, text: &str) -> Result<()> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
        .and_then(|mut f| f.write_all(text.as_bytes())).map_err(|e| e.to_string())
}
fn run() -> Result<()> {
    let root = PathBuf::from(std::env::args().nth(1).ok_or("Pass a fresh absolute run directory")?);
    if !root.is_absolute() { return Err("Run directory must be absolute".into()); }
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    let cache = root.join("cache");
    let mut backend = Some(Backend::new(cache.clone()));
    let mut current = String::new();
    for (index, line) in io::stdin().lock().lines().enumerate() {
        if index >= 256 { return Err("Development command budget exhausted".into()); }
        let line = line.map_err(|e| e.to_string())?;
        if line.len() > 1_000_000 { return Err("Command budget exceeded".into()); }
        let v: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        let result = (|| -> Result<Value> {
            let b = backend.as_ref().ok_or("Backend unavailable")?;
            let id = v.get("projectId").and_then(Value::as_str).unwrap_or(&current).to_string();
            Ok(match string(&v, "command")? {
                "open" => {
                    let project = root.join(format!("project-{index}"));
                    fs::create_dir(&project).map_err(|e| e.to_string())?;
                    let sources = v.get("sources").and_then(Value::as_object).ok_or("Missing sources")?;
                    for (name, source) in sources {
                        if !["main.tex", "notes.tex"].contains(&name.as_str()) { return Err("Unexpected fixture file".into()); }
                        write_new(&project.join(name), source.as_str().ok_or("Invalid fixture source")?)?;
                    }
                    let p = b.open_project(Some(project.join("main.tex").to_string_lossy().into_owned()))?;
                    current = p.id.clone();
                    json!({"projectId":p.id,"snapshot":b.document_snapshot(&current)?})
                },
                "targets" => serde_json::to_value(b.document_source_targets(&id, revision(&v)?, string(&v,"file")?)?).map_err(|e| e.to_string())?,
                "intent" => serde_json::to_value(b.document_source_intent(&id, revision(&v)?, string(&v,"operationId")?,
                    serde_json::from_value(v.get("intent").cloned().ok_or("Missing intent")?).map_err(|e| e.to_string())?)?).map_err(|e| e.to_string())?,
                "status" => serde_json::to_value(b.document_source_intent_status(&id, string(&v,"operationId")?)?).map_err(|e| e.to_string())?,
                "snapshot" => serde_json::to_value(b.document_snapshot(&id)?).map_err(|e| e.to_string())?,
                "history" => serde_json::to_value(b.document_history(&id, revision(&v)?, v.get("redo").and_then(Value::as_bool).ok_or("Missing redo")?)?).map_err(|e| e.to_string())?,
                "restart_recovery" => {
                    let draft = b.document_snapshot(&id)?.recovery.draft_id.ok_or("No protected draft")?;
                    drop(backend.take());
                    backend = Some(Backend::new(cache.clone()));
                    let b = backend.as_ref().ok_or("Backend unavailable")?;
                    let p = b.open_typing_draft(&draft)?;
                    current = p.id.clone();
                    json!({"projectId":p.id,"snapshot":b.document_snapshot(&current)?})
                },
                _ => return Err("Unknown development command".into()),
            })
        })();
        match result {
            Ok(value) => emit(json!({"ok":true,"result":value}))?,
            Err(error) => emit(json!({"ok":false,"error":error}))?,
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); }
}
