//! Synthetic fixture bridge for the production literal editor.
use std::{env, fs, process::ExitCode};
fn run() -> Result<String, String> {
    let a: Vec<_> = env::args().skip(1).collect();
    if a.len() != 4 {
        return Err("usage: literal_candidate FILE MAIN RUN_ID TEXT".into());
    }
    let source = fs::read_to_string(&a[0]).map_err(|e| e.to_string())?;
    let main = fs::read_to_string(&a[1]).map_err(|e| e.to_string())?;
    let marker = "% proof:begin\n";
    let start = source.find(marker).ok_or("Missing fixture start")? + marker.len();
    let end = source.find("% proof:end").ok_or("Missing fixture end")?;
    let fragment = source.get(start..end).ok_or("Invalid fixture range")?;
    let id = a[2].parse().map_err(|_| "Invalid run ID")?;
    let updated = texloom_core::literal_text::apply(
        fragment,
        fragment,
        &main,
        &[texloom_core::literal_text::LiteralEdit {
            id,
            text: a[3].clone(),
        }],
    )?;
    let mut source = source.clone();
    source.replace_range(start..end, &updated);
    Ok(source)
}
fn main() -> ExitCode {
    match run() {
        Ok(s) => {
            print!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
