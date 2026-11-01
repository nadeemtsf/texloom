//! CLI fixture adapter: emits a candidate using the same Rust functions as the app.
use std::{env, fs, process::ExitCode};
fn run() -> Result<String, String> {
    let a: Vec<String> = env::args().skip(1).collect();
    if a.len() != 8 { return Err("usage: proof_candidate FILE MAIN START END ID DX DY ACTION".into()); }
    let source = fs::read_to_string(&a[0]).map_err(|e| e.to_string())?;
    let document = fs::read_to_string(&a[1]).map_err(|e| e.to_string())?;
    let start = a[2].parse::<usize>().map_err(|e| e.to_string())?;
    let end = a[3].parse::<usize>().map_err(|e| e.to_string())?;
    let dx = a[5].parse::<f64>().map_err(|e| e.to_string())?;
    let dy = a[6].parse::<f64>().map_err(|e| e.to_string())?;
    let wrapped = texloom_core::wrap_text_object(&source, start..end, &a[4], dx, dy, &document)?;
    if a[7] == "detach" { texloom_core::unwrap_text_object(&wrapped, &a[4]) }
    else if a[7] == "wrap" { Ok(wrapped) }
    else { Err("Unknown action".into()) }
}
fn main() -> ExitCode {
    match run() {
        Ok(source) => { print!("{source}"); ExitCode::SUCCESS }
        Err(error) => { eprintln!("{error}"); ExitCode::FAILURE }
    }
}
