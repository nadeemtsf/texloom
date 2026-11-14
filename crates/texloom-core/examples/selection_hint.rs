//! Test-only bridge to the same conservative source-unit resolver as Tauri.
use std::{env, fs, process::ExitCode};
fn run() -> Result<String, String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 3 && args.len() != 4 {
        return Err("usage: selection_hint FILE MAIN LINE,LINE [CANDIDATE]".into());
    }
    let source = fs::read_to_string(&args[0]).map_err(|e| e.to_string())?;
    let main = fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
    let lines: Vec<usize> = args[2]
        .split(',')
        .map(|s| s.parse().map_err(|_| "Invalid source line".to_string()))
        .collect::<Result<_, _>>()?;
    if lines.is_empty() || lines.len() > 64 {
        return Err("Select one to 64 source lines".into());
    }
    let candidate = args
        .get(3)
        .map(|v| v.parse::<usize>())
        .transpose()
        .map_err(|_| "Invalid candidate index")?
        .unwrap_or(0);
    let unit = texloom_core::resolve_selection_hint(&source, &lines, &main, candidate)?;
    Ok(format!(
        r#"{{"start":{},"end":{},"first_line":{},"last_line":{},"kind":"{}"}}"#,
        unit.range.start, unit.range.end, unit.first_line, unit.last_line, unit.kind
    ))
}
fn main() -> ExitCode {
    match run() {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
