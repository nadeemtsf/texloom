//! Opt-in headless service cost diagnostic. No TeX, window, transport or Save.
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs::{self, OpenOptions}, hint::black_box, io::Write,
    path::{Path, PathBuf}, time::Instant};
use texloom_desktop_lib::benchmark::{Backend, DraftSourceIntent, EditorChange, SourceDraftOutcome, SourceDraftCostState};
type Result<T> = std::result::Result<T, String>;
const WARMUP: usize = 5;
const SAMPLES: usize = 30;
const SEED: u64 = 20_261_114;
const REPEATED: &str = "Repeated paragraph.";
const LITERAL: &str = "Updated & 50%.";
const ESCAPED: &str = "Updated \\& 50\\%.";

fn require(ok: bool, message: &str) -> Result<()> { if ok { Ok(()) } else { Err(message.into()) } }
fn write_new(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    OpenOptions::new().write(true).create_new(true).open(path)
        .and_then(|mut f| f.write_all(bytes.as_ref())).map_err(|e| e.to_string())
}
struct Fixture { sources: BTreeMap<String,String>, changed: String, offset: usize }
fn fixture(total: usize) -> Result<Fixture> {
    let start="\u{feff}\\documentclass{article}\r\n%";
    let body=format!("\r\n\\begin{{document}}\r\n{REPEATED}\r\n\r\n");
    let tail="\r\n\r\nTail remains unchanged.\r\n\\end{document}\r\n";
    let padding=16*1024-start.len()-body.len()-REPEATED.len()-tail.len();
    let prefix=format!("{start}{}{body}","x".repeat(padding));
    let original=format!("{prefix}{REPEATED}{tail}");
    require(original.len()==16*1024,"Wrong declared main size")?;
    let changed=format!("{prefix}{ESCAPED}{tail}");
    let offset=prefix.strip_prefix('\u{feff}').unwrap().replace("\r\n","\n").encode_utf16().count();
    let mut sources=BTreeMap::from([("main.tex".into(),original)]);
    let mut remaining=total-16*1024;
    let mut index=0;
    while remaining>0 {
        let bytes=remaining.min(1024*1024);
        require(bytes>=4,"Auxiliary fixture too small")?;
        sources.insert(format!("aux-{index:03}.tex"),format!("%{}\r\n","a".repeat(bytes-3)));
        remaining-=bytes; index+=1;
    }
    require(sources.values().map(String::len).sum::<usize>()==total,"Wrong total source size")?;
    Ok(Fixture {sources,changed,offset})
}
fn exact(actual: &BTreeMap<String,String>, gold: &Fixture, changed: bool) -> Result<()> {
    require(actual.len()==gold.sources.len(),"Unexpected source file set")?;
    for (name,original) in &gold.sources {
        let expected=if changed && name=="main.tex" {&gold.changed} else {original};
        require(actual.get(name)==Some(expected),"Exact full-source condition failed")?;
    }
    Ok(())
}
#[derive(Clone,Copy)]
enum Mode { Typed, TypedDelta, Source }
impl Mode {
    fn name(self)-> &'static str {match self {Self::Typed=>"typed_preprotected_full_snapshot",Self::TypedDelta=>"typed_preprotected_changed_file",Self::Source=>"source_edit_delta"}}
}
struct Arm { backend: Backend, project: String, revision: u64, path: PathBuf, mode: Mode }
fn arm(root: &Path, mode: Mode, gold: &Fixture) -> Result<Arm> {
    let path=root.join(mode.name());
    let project_path=path.join("project"); fs::create_dir_all(&project_path).map_err(|e|e.to_string())?;
    for (name,source) in &gold.sources {write_new(&project_path.join(name),source)?;}
    let backend=Backend::new(path.join("cache"));
    let opened=backend.open_project(Some(project_path.join("main.tex").to_string_lossy().into_owned()))?;
    exact(&opened.sources,gold,false)?;
    Ok(Arm {backend,project:opened.id,revision:0,path:project_path,mode})
}
fn timed<T: Serialize>(work: impl FnOnce()->Result<T>) -> Result<(T,Vec<u8>,f64,f64,f64)> {
    let started=Instant::now(); let reply=work()?;
    let service=started.elapsed().as_secs_f64()*1000.;
    let encoding=Instant::now(); let payload=serde_json::to_vec(&reply).map_err(|e|e.to_string())?;
    let encoded=encoding.elapsed().as_secs_f64()*1000.;
    let total=started.elapsed().as_secs_f64()*1000.;
    Ok((reply,payload,service,encoded,total))
}
fn trial(arm: &mut Arm, gold: &Fixture, total_bytes: usize, index: usize, order: usize) -> Result<Value> {
    let before=arm.revision;
    let (mut discovery, mut handle)=(None,String::new());
    if matches!(arm.mode,Mode::Typed | Mode::TypedDelta) {
        let start=Instant::now();
        let targets=arm.backend.document_source_targets(&arm.project,before,"main.tex")?;
        discovery=Some(start.elapsed().as_secs_f64()*1000.);
        let matches=targets.targets.into_iter().filter(|t|t.text==REPEATED).collect::<Vec<_>>();
        require(matches.len()==2,"Expected two source occurrences")?;
        handle=matches[1].handle.clone();
    }
    let (next,payload,service,encoded,combined)=match arm.mode {
        Mode::Typed=> {
            let operation=format!("{:032x}",index+1);
            let (reply,payload,s,e,t)=timed(||arm.backend.document_source_intent(&arm.project,before,&operation,
                DraftSourceIntent::ReplaceLiteralParagraph {target:handle,text:LITERAL.into()}))?;
            require(matches!(&reply.outcome,SourceDraftOutcome::SourceDraftApplied {from_revision,revision,changed:true,render_verification:"unresolved"}
                if *from_revision==before && *revision==before+1),"Typed trial refused or remained uncertain")?;
            require(reply.snapshot.recovery.protected_revision==Some(reply.snapshot.revision) && reply.snapshot.recovery.error.is_none(),"Typed acknowledgement unprotected")?;
            exact(&reply.snapshot.sources,gold,true)?;
            (reply.snapshot.revision,payload,s,e,t)
        },
        Mode::TypedDelta=> {
            let operation=format!("{:032x}",index+1);
            let (reply,payload,s,e,t)=timed(||arm.backend.benchmark_source_intent_delta(&arm.project,before,&operation,
                DraftSourceIntent::ReplaceLiteralParagraph {target:handle,text:LITERAL.into()}))?;
            require(matches!(&reply.outcome,SourceDraftOutcome::SourceDraftApplied {from_revision,revision,changed:true,render_verification:"unresolved"}
                if *from_revision==before && *revision==before+1),"Ablation trial refused or remained uncertain")?;
            let delta=match &reply.state {SourceDraftCostState::Delta {delta}=>delta,_=>return Err("Fresh ablation unexpectedly needed a full snapshot".into())};
            require(delta.base_revision==before && delta.sources.len()==1 && delta.sources.get("main.tex")==Some(&gold.changed),"Unexpected ablation delta")?;
            require(delta.recovery.protected_revision==Some(delta.revision) && delta.recovery.error.is_none(),"Ablation acknowledgement unprotected")?;
            exact(&arm.backend.document_snapshot(&arm.project)?.sources,gold,true)?;
            (delta.revision,payload,s,e,t)
        },
        Mode::Source=> {
            let (reply,payload,s,e,t)=timed(||arm.backend.document_edit_delta(&arm.project,before,"main.tex",
                vec![EditorChange {offset:gold.offset,length:REPEATED.encode_utf16().count(),text:ESCAPED.into()}],None))?;
            require(reply.base_revision==before && reply.sources.len()==1 && reply.sources.get("main.tex")==Some(&gold.changed),"Unexpected source delta")?;
            require(reply.recovery.protected_revision==Some(reply.revision) && reply.recovery.error.is_none(),"Source acknowledgement unprotected")?;
            exact(&arm.backend.document_snapshot(&arm.project)?.sources,gold,true)?;
            (reply.revision,payload,s,e,t)
        },
    };
    require(next==before+1,"One edit must advance one revision")?;
    let response_bytes=black_box(payload.len()); drop(payload);
    let undo=arm.backend.document_history(&arm.project,next,false)?;
    require(undo.revision==next+1 && !undo.can_undo && undo.can_redo,"Undo must restore one complete source step")?;
    require(undo.recovery.protected_revision==Some(undo.revision) && undo.recovery.error.is_none(),"Undo unprotected")?;
    exact(&undo.sources,gold,false)?;
    arm.revision=undo.revision;
    Ok(json!({"mode":arm.mode.name(),"project_source_bytes":total_bytes,"trial_index":index,
        "arm_order":order,"warmup":index<WARMUP,"target_discovery_ms":discovery,
        "native_service_and_reply_ms":service,"reply_json_encoding_ms":encoded,
        "native_service_through_encoding_ms":combined,"response_bytes":response_bytes,
        "revision_before":before,"revision_after_edit":next,"revision_after_undo":arm.revision,
        "exact_full_source_and_one_step_undo":true,"protected_acknowledgement":true}))
}
fn distribution(mut values:Vec<f64>)->Value {
    values.sort_by(f64::total_cmp); let n=values.len();
    json!({"samples":n,"median":(values[(n-1)/2]+values[n/2])/2.,"p95":values[(n*95).div_ceil(100)-1],"max":values[n-1]})
}
fn run(base:&Path)->Result<Value> {
    let mut raw=OpenOptions::new().write(true).create_new(true).open(base.join("samples.jsonl")).map_err(|e|e.to_string())?;
    let mut rows=Vec::new(); let mut seed=SEED;
    for bytes in [1024*1024,8*1024*1024,32*1024*1024] {
        let gold=fixture(bytes)?;
        let case=base.join(format!("source-bytes-{bytes}")); fs::create_dir(&case).map_err(|e|e.to_string())?;
        let mut arms=[arm(&case,Mode::Typed,&gold)?,arm(&case,Mode::TypedDelta,&gold)?,arm(&case,Mode::Source,&gold)?];
        for index in 0..WARMUP+SAMPLES {
            let mut order=[0,1,2];
            for i in (1..3).rev() {
                seed=seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                order.swap(i,((seed>>32) as usize)%(i+1));
            }
            for (position,arm_index) in order.into_iter().enumerate() {
                let row=trial(&mut arms[arm_index],&gold,bytes,index,position)?;
                writeln!(raw,"{}",row).map_err(|e|e.to_string())?;raw.flush().map_err(|e|e.to_string())?;
                rows.push(row);
            }
        }
        for arm in &arms {for (name,expected) in &gold.sources {
            require(fs::read(arm.path.join(name)).map_err(|e|e.to_string())?==expected.as_bytes(),"Original source file changed")?;
        }}
        println!("Checked three arms at {bytes} source bytes; exact source/Undo/protection/originals passed.");
    }
    let mut summaries=Vec::new();
    for bytes in [1024*1024,8*1024*1024,32*1024*1024] {for mode in [Mode::Typed,Mode::TypedDelta,Mode::Source] {
        let selected=rows.iter().filter(|r|r["project_source_bytes"]==bytes && r["mode"]==mode.name() && r["warmup"]==false).collect::<Vec<_>>();
        require(selected.len()==SAMPLES,"Missing planned timing samples")?;
        let mut summary=json!({"project_source_bytes":bytes,"mode":mode.name(),"warmups":WARMUP,"samples":SAMPLES,
            "source_undo_protection_originals_passed":true});
        for field in ["native_service_and_reply_ms","reply_json_encoding_ms","native_service_through_encoding_ms","response_bytes"] {
            summary[field]=distribution(selected.iter().map(|r|r[field].as_f64().unwrap()).collect());
        }
        if matches!(mode,Mode::Typed | Mode::TypedDelta) {summary["target_discovery_ms"]=distribution(selected.iter().map(|r|r["target_discovery_ms"].as_f64().unwrap()).collect());}
        summaries.push(summary);
    }}
    Ok(json!({"schema":2,"kind":"headless_source_draft_cost_development_diagnostic","status":"passed","profile":"release",
        "os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"seed":SEED,"case_size_order":[1048576,8388608,33554432],
        "warmups_per_arm":WARMUP,"samples_per_arm":SAMPLES,"summaries":summaries,"raw_rows":rows.len(),
        "scope":"Source-equivalent requests with different planning/protection/reply contracts; no UI, transport, TeX or final performance study",
        "timing_exclusions":["target enumeration from main endpoint","assertions","reply/payload destruction","Undo","file checks","WebView/OS input/paint"],
        "background_load_controlled":false,"final_evaluation":false}))
}
fn main() {
    let result=(||->Result<()> {
        let base=PathBuf::from(std::env::args().nth(1).ok_or("Pass a fresh absolute output directory")?);
        require(base.is_absolute(),"Output must be absolute")?;
        let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().canonicalize().map_err(|e|e.to_string())?;
        let allowed=repo.join("artifacts").canonicalize().map_err(|e|e.to_string())?;
        require(base.parent().ok_or("Missing output parent")?.canonicalize().map_err(|e|e.to_string())?.starts_with(allowed),"Output must remain under repository artifacts")?;
        fs::create_dir(&base).map_err(|e|e.to_string())?;
        match run(&base) {
            Ok(report)=>write_new(&base.join("results.json"),serde_json::to_vec_pretty(&report).map_err(|e|e.to_string())?),
            Err(error)=>{write_new(&base.join("failure.json"),serde_json::to_vec_pretty(&json!({"status":"failed","error":error})).unwrap())?;Err(error)}
        }
    })();
    if let Err(error)=result {eprintln!("{error}");std::process::exit(1);}
}
