//! Read-only synthetic-fixture bridge for the new source intent planner.
use std::{collections::BTreeMap, env, fs, process::ExitCode};
use texloom_core::document::{intent::SourceIntent, structure::RegionKind, *};

fn run() -> Result<String, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 || args.len() > 4 {
        return Err("usage: source_intent_candidate FILE PARAGRAPH_INDEX replace TEXT | FILE PARAGRAPH_INDEX delete".into());
    }
    let source = fs::read_to_string(&args[0]).map_err(|e| e.to_string())?;
    let index = args[1].parse::<usize>().map_err(|e| e.to_string())?;
    let session = Session::new(
        BTreeMap::from([("main.tex".into(), source)]),
        Profile {
            main: "main.tex".into(),
            engine: "auto".into(),
        },
    )
    .map_err(|e| e.to_string())?;
    let projection = session
        .source_projection("main.tex")
        .map_err(|e| e.to_string())?;
    let region = projection
        .regions()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind() == RegionKind::PlainParagraph)
        .nth(index)
        .ok_or("No supported paragraph at that index")?
        .0;
    let target = projection
        .entity_reference(region)
        .ok_or("Unavailable target")?;
    let intent = match (args[2].as_str(), args.get(3)) {
        ("replace", Some(text)) => SourceIntent::ReplaceLiteralParagraph {
            target,
            text: text.clone(),
        },
        ("delete", None) => SourceIntent::DeleteParagraphContent { target },
        _ => return Err("Invalid action or payload".into()),
    };
    let plan = session
        .plan_source_intent(intent)
        .map_err(|e| format!("{e:?}"))?;
    let mut sources = plan.candidate_sources(&session).map_err(|e| e.to_string())?;
    sources.remove("main.tex").ok_or_else(|| "Missing candidate main source".into())
}
fn main() -> ExitCode {
    match run() {
        Ok(source) => {
            print!("{source}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
