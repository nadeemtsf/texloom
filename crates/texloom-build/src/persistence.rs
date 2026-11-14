//! Explicit project saves, external-edit preconditions and recoverable journals.
//! Multiple file replacements are recoverable, not filesystem-wide atomic.
use crate::snapshot::{is_source, relative, Fingerprint};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap,BTreeSet}, fs::{self,File}, io::{Read,Write},
    path::{Path,PathBuf}, sync::atomic::{AtomicU64,Ordering}, time::{SystemTime,UNIX_EPOCH}};
type Result<T> = std::result::Result<T,String>;
pub type Sources = BTreeMap<String,String>;
const OWNER: &[u8] = b"Texloom project save journals v1\n";
const DIRECTORY: &str = ".texloom-save";
const MAX_FILE: u64 = 4_000_000;
static NEXT: AtomicU64 = AtomicU64::new(0);
fn fail(e: impl std::fmt::Display)->String{e.to_string()}
fn require(ok:bool,message:&str)->Result<()>{if ok{Ok(())}else{Err(message.into())}}
fn no_reparse(path:&Path)->Result<()> {
    let m=fs::symlink_metadata(path).map_err(fail)?;
    #[cfg(windows)] { use std::os::windows::fs::MetadataExt;
        require(m.file_attributes()&0x400==0,"Reparse point in save path")?; }
    require(!m.file_type().is_symlink(),"Symlink in save path")
}
fn file_name(name:&str)->Result<PathBuf> {
    let path=relative(name)?;
    require((is_source(&path)||name==crate::project_io::PROFILE)&&path.components().next().is_some_and(|p|p.as_os_str()!=DIRECTORY),"Invalid project save filename")?;
    Ok(path)
}
fn root(path:&Path)->Result<PathBuf> {
    no_reparse(path)?;let path=path.canonicalize().map_err(fail)?;
    require(path.is_dir(),"Project root is not a directory")?;Ok(path)
}
fn destination(root:&Path,name:&str,create:bool)->Result<PathBuf> {
    checked_destination(root,file_name(name)?,create)
}
fn checked_destination(root:&Path,relative:PathBuf,create:bool)->Result<PathBuf> {
    let mut parent=root.to_path_buf();
    let parts=relative.components().collect::<Vec<_>>();
    for part in &parts[..parts.len()-1] {
        parent.push(part);
        if create&&!parent.exists(){fs::create_dir(&parent).map_err(fail)?;}
        if parent.exists(){no_reparse(&parent)?;require(parent.is_dir(),"Save parent is not a directory")?;}
        else {return Ok(root.join(relative));}
    }
    let path=root.join(relative);
    if path.exists(){no_reparse(&path)?;require(path.is_file(),"Save destination is not a file")?;}
    Ok(path)
}
fn read(path:&Path,max:u64)->Result<Vec<u8>> {
    no_reparse(path)?;let mut f=File::open(path).map_err(fail)?;
    require(f.metadata().map_err(fail)?.is_file(),"Save evidence is not a regular file")?;
    let mut bytes=Vec::new();Read::by_ref(&mut f).take(max+1).read_to_end(&mut bytes).map_err(fail)?;
    require(bytes.len()as u64<=max,"Save evidence exceeds byte budget")?;Ok(bytes)
}
fn state(root:&Path,name:&str)->Result<Option<Fingerprint>> {
    let path=destination(root,name,false)?;
    match fs::symlink_metadata(&path) {
        Ok(_)=>Ok(Some(Fingerprint::bytes(&read(&path,MAX_FILE)?))),
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(e)=>Err(fail(e)),
    }
}
fn write_new(path:&Path,bytes:&[u8])->Result<()> {
    let mut f=File::create_new(path).map_err(fail)?;f.write_all(bytes).map_err(fail)?;f.sync_all().map_err(fail)
}
fn identity()->String {
    Fingerprint::bytes(format!("{}:{:?}:{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH),NEXT.fetch_add(1,Ordering::Relaxed)).as_bytes()).sha256[..32].into()
}
fn valid_id(id:&str)->bool{id.len()==32&&id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())}

#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskSnapshot { pub root:PathBuf, pub files:BTreeMap<String,Option<Fingerprint>> }
#[derive(Debug,Serialize)]
pub struct Conflict { pub file:String, pub kind:&'static str }
pub enum SaveOutcome { Saved, Conflicts(Vec<Conflict>) }
impl DiskSnapshot {
    /// Track app-owned build-profile metadata alongside sources without adding
    /// it to the editable LaTeX buffers or trusting a later external change.
    pub fn track_project_profile(&mut self)->Result<()> {
        let name=crate::project_io::PROFILE;
        let value=state(&self.root,name)?;
        self.files.entry(name.into()).or_insert(value);Ok(())
    }
    pub fn capture(path:&Path,sources:&Sources)->Result<Self> {
        validate_sources(sources)?;let root=root(path)?;let mut files=BTreeMap::new();
        for (name,text) in sources {
            let actual=state(&root,name)?;
            require(actual==Some(Fingerprint::bytes(text.as_bytes())),"A project file changed while opening; reopen it")?;
            files.insert(name.clone(),actual);
        }
        Ok(Self{root,files})
    }
    pub fn conflicts(&self,sources:&Sources)->Result<Vec<Conflict>> {
        validate_sources(sources)?;require(root(&self.root)?==self.root,"Project root changed")?;
        let names=self.files.keys().chain(sources.keys()).collect::<BTreeSet<_>>();let mut conflicts=Vec::new();
        for name in names {
            let expected=self.files.get(name).cloned().flatten();let actual=state(&self.root,name)?;
            if actual!=expected {conflicts.push(Conflict{file:name.clone(),kind:match(expected,actual){(None,Some(_))=>"appeared",(Some(_),None)=>"removed",_=>"changed"}});}
        }
        Ok(conflicts)
    }
    pub fn save(&mut self,sources:&Sources)->Result<SaveOutcome>{self.save_with(sources,|_|Ok(()))}
    fn save_with(&mut self,sources:&Sources,mut hook:impl FnMut(Stage)->Result<()>)->Result<SaveOutcome> {
        let conflicts=self.conflicts(sources)?;if !conflicts.is_empty(){return Ok(SaveOutcome::Conflicts(conflicts));}
        let base=journal_base(&self.root)?;let _lock=project_lock(&base)?;
        require(pending(&self.root)?.iter().all(|p|p.state==SaveState::RolledBack),"Recover the interrupted project save before saving again")?;
        let names=self.files.keys().chain(sources.keys()).cloned().collect::<BTreeSet<_>>();
        let mut entries=Vec::new();
        for name in names {
            let before=self.files.get(&name).cloned().flatten();
            let after=sources.get(&name).map(|s|Fingerprint::bytes(s.as_bytes()));
            if before!=after{entries.push(Entry{file:name,before,after});}
        }
        if entries.is_empty(){return Ok(SaveOutcome::Saved);}
        require(self.files.keys().chain(sources.keys()).collect::<BTreeSet<_>>().len()<=1200,"Reload the project before tracking more source files")?;
        require(pending(&self.root)?.len()<32,"Save recovery retention is full; review earlier recovered saves")?;
        let id=identity();let directory=base.join(&id);fs::create_dir(&directory).map_err(fail)?;
        let mut journal=Journal{directory,manifest:Manifest{schema:1,id,project:self.root.clone(),state:SaveState::Preparing,entries}};
        journal.persist()?;
        // Backups and all intended bytes exist and are flushed before originals
        // are replaced. A failed preparation never touches the originals.
        let result:Result<()>=(||{
        for (index,e) in journal.manifest.entries.iter().enumerate() {
            if let Some(expected)=&e.before {
                let bytes=read(&destination(&self.root,&e.file,false)?,MAX_FILE)?;
                require(Fingerprint::bytes(&bytes)==*expected,"Source changed while preparing Save")?;
                write_new(&journal.file(index,"old"),&bytes)?;
            }
            if let Some(text)=sources.get(&e.file){write_new(&journal.file(index,"new"),text.as_bytes())?;}
        }
        journal.manifest.state=SaveState::Prepared;journal.persist()?;
            hook(Stage::Prepared)?;
            let conflicts=self.conflicts(sources)?;
            require(conflicts.is_empty(),"Source changed while staging Save; original files were preserved")?;
            journal.manifest.state=SaveState::Committing;journal.persist()?;
            journal.apply(true,&mut hook)?;
            hook(Stage::Complete)?;
            journal.manifest.state=SaveState::Complete;journal.persist()?;Ok(())
        })();
        if let Err(error)=result {
            // Rollback never overwrites an unknown external edit. If it cannot
            // finish, the journal remains for explicit recovery.
            return match journal.rollback(){Ok(())=>Err(format!("Save was rolled back: {error}. Your unsaved edits remain available.")),Err(recovery)=>Err(format!("Save needs recovery: {error}; {recovery}"))};
        }
        for e in &journal.manifest.entries{self.files.insert(e.file.clone(),e.after.clone());}
        journal.cleanup();Ok(SaveOutcome::Saved)
    }
}
fn validate_sources(sources:&Sources)->Result<()> {
    require(sources.keys().filter(|name|name.as_str()!=crate::project_io::PROFILE).count()<=600&&sources.values().map(String::len).sum::<usize>()<=60*1024*1024,"Project exceeds save limits")?;
    let mut names=BTreeSet::new();
    for (name,text) in sources {file_name(name)?;require(text.len()as u64<=if name==crate::project_io::PROFILE{65536}else{MAX_FILE}&&names.insert(name.to_lowercase()),"Invalid or case-colliding project sources")?;}
    Ok(())
}
fn journal_base(project:&Path)->Result<PathBuf> {
    let base=project.join(DIRECTORY);
    match fs::create_dir(&base){Ok(())=>write_new(&base.join("owner"),OWNER)?,Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>{},Err(e)=>return Err(fail(e))}
    no_reparse(&base)?;require(base.is_dir()&&base.canonicalize().map_err(fail)?.parent()==Some(project),"Invalid project save journal root")?;
    require(read(&base.join("owner"),100)?==OWNER,"Unrecognized project save folder; no files changed")?;Ok(base)
}
fn project_lock(base:&Path)->Result<File> {
    let path=base.join("save.lock");if path.exists(){no_reparse(&path)?;}
    let f=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path).map_err(fail)?;
    f.try_lock().map_err(|_|"Another Texloom instance is saving this project".to_string())?;Ok(f)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
#[cfg_attr(feature="typegen",derive(ts_rs::TS))]
pub enum SaveState { Preparing, Prepared, Committing, Complete, RolledBack }
#[derive(Clone,Serialize,Deserialize)]
#[cfg_attr(feature="typegen",derive(ts_rs::TS))]
pub struct PendingSave { pub id:String, pub state:SaveState, pub files:Vec<String> }
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry { file:String,before:Option<Fingerprint>,after:Option<Fingerprint> }
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest { schema:u32,id:String,project:PathBuf,state:SaveState,entries:Vec<Entry> }
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope { sha256:String,data:Manifest }
struct Journal { directory:PathBuf,manifest:Manifest }
#[derive(Clone,Copy)]
enum Stage { Prepared, BeforeReplace(usize), AfterReplace(usize), Complete }
impl Journal {
    fn file(&self,index:usize,kind:&str)->PathBuf{self.directory.join(format!("{index:04}.{kind}"))}
    fn persist(&self)->Result<()> {
        no_reparse(&self.directory)?;
        let data=serde_json::to_vec(&self.manifest).map_err(fail)?;
        let bytes=serde_json::to_vec(&Envelope{sha256:Fingerprint::bytes(&data).sha256,data:serde_json::from_slice(&data).map_err(fail)?}).map_err(fail)?;
        require(bytes.len()<=1_000_000,"Save manifest exceeds byte budget")?;
        let next=self.directory.join(format!("manifest-{}.next",identity()));
        write_new(&next,&bytes)?;fs::rename(next,self.directory.join("manifest.json")).map_err(fail)
    }
    fn open(project:&Path,id:&str)->Result<Self> {
        require(valid_id(id),"Invalid save transaction ID")?;
        let directory=journal_base(project)?.join(id);no_reparse(&directory)?;
        let envelope:Envelope=serde_json::from_slice(&read(&directory.join("manifest.json"),1_000_000)?).map_err(fail)?;
        require(Fingerprint::bytes(&serde_json::to_vec(&envelope.data).map_err(fail)?).sha256==envelope.sha256,"Save journal integrity check failed")?;
        let m=envelope.data;
        require(m.schema==1&&m.id==id&&m.project==project&&m.entries.len()<=1200,"Save journal belongs to a different project")?;
        let mut names=BTreeSet::new();
        for e in &m.entries {file_name(&e.file)?;require(names.insert(e.file.to_lowercase()),"Duplicate save journal file")?;
            for hash in [&e.before,&e.after].into_iter().flatten(){require(hash.bytes<=MAX_FILE&&hash.sha256.len()==64&&hash.sha256.bytes().all(|b|b.is_ascii_hexdigit()),"Invalid save fingerprint")?;}}
        require(m.entries.iter().filter_map(|e|e.before.as_ref()).map(|f|f.bytes).sum::<u64>()<=60*1024*1024
            &&m.entries.iter().filter_map(|e|e.after.as_ref()).map(|f|f.bytes).sum::<u64>()<=60*1024*1024,"Save journal exceeds total source byte budget")?;
        Ok(Self{directory,manifest:m})
    }
    fn check_current(&self)->Result<()> {
        for e in &self.manifest.entries {let actual=state(&self.manifest.project,&e.file)?;
            require(actual==e.before||actual==e.after,"An external edit conflicts with save recovery; no recovery files were replaced")?;}
        Ok(())
    }
    fn apply(&self,forward:bool,hook:&mut impl FnMut(Stage)->Result<()>)->Result<()> {
        self.check_current()?;
        for (index,e) in self.manifest.entries.iter().enumerate() {
            let expected=if forward{&e.after}else{&e.before};
            if &state(&self.manifest.project,&e.file)?==expected {continue;}
            if let Some(expected)=expected {
                let input=self.file(index,if forward{"new"}else{"old"});
                require(Fingerprint::bytes(&read(&input,MAX_FILE)?)==*expected,"Save staged/backup checksum mismatch")?;
            }
        }
        let indices:Vec<_>=if forward{(0..self.manifest.entries.len()).collect()}else{(0..self.manifest.entries.len()).rev().collect()};
        for index in indices {
            let e=&self.manifest.entries[index];let expected=if forward{&e.after}else{&e.before};
            if &state(&self.manifest.project,&e.file)?==expected{continue;}
            hook(Stage::BeforeReplace(index))?;
            let other=if forward{&e.before}else{&e.after};
            require(&state(&self.manifest.project,&e.file)?==other,"Source changed during save replacement")?;
            let target=destination(&self.manifest.project,&e.file,true)?;
            // Retain the intended bytes when rolling back a partially completed
            // Save, so a later recovery can finish it without relying on RAM.
            if !forward&&e.after.is_some()&&!self.file(index,"new").exists(){
                let bytes=read(&target,MAX_FILE)?;require(Some(Fingerprint::bytes(&bytes))==e.after,"Intended save bytes changed")?;
                write_new(&self.file(index,"new"),&bytes)?;
            }
            if let Some(expected)=expected {
                let input=self.file(index,if forward{"new"}else{"old"});
                let bytes=read(&input,MAX_FILE)?;require(Fingerprint::bytes(&bytes)==*expected,"Save staged/backup checksum mismatch")?;
                let replace=self.file(index,"replace");
                if replace.exists(){require(Fingerprint::bytes(&read(&replace,MAX_FILE)?)==*expected,"Unexpected save replacement bytes")?;}
                else {write_new(&replace,&bytes)?;}
                fs::rename(&replace,&target).map_err(fail)?;
            } else {fs::remove_file(&target).map_err(fail)?;}
            require(&state(&self.manifest.project,&e.file)?==expected,"Save output changed during replacement")?;
            hook(Stage::AfterReplace(index))?;
        }Ok(())
    }
    fn rollback(&mut self)->Result<()> {
        self.apply(false,&mut |_|Ok(()))?;self.manifest.state=SaveState::RolledBack;self.persist()
    }
    fn cleanup(&self) {
        // No recursive deletion. Unknown directories/files leave the journal in
        // place. This folder contains only explicitly recognized derived data.
        if no_reparse(&self.directory).is_err(){return;}
        let mut allowed=BTreeSet::from(["manifest.json".to_string()]);
        for i in 0..self.manifest.entries.len(){for kind in ["old","new","replace"]{allowed.insert(format!("{i:04}.{kind}"));}}
        let Ok(entries)=fs::read_dir(&self.directory)else{return;};let entries=entries.flatten().collect::<Vec<_>>();
        if entries.iter().any(|e|!allowed.contains(&e.file_name().to_string_lossy().to_string())||!e.file_type().is_ok_and(|k|k.is_file())||no_reparse(&e.path()).is_err()){return;}
        for e in &entries {
            let name=e.file_name().to_string_lossy().to_string();
            if name=="manifest.json" {
                let Ok(data)=serde_json::to_vec(&self.manifest)else{return;};
                let Ok(manifest)=serde_json::from_slice(&data)else{return;};
                let Ok(expected)=serde_json::to_vec(&Envelope{sha256:Fingerprint::bytes(&data).sha256,data:manifest})else{return;};
                if read(&e.path(),1_000_000).ok().as_ref()!=Some(&expected){return;}
            } else {
                let Some((index,kind))=name.split_once('.')else{return;};
                let Ok(index)=index.parse::<usize>()else{return;};
                let Some(record)=self.manifest.entries.get(index)else{return;};
                let Ok(bytes)=read(&e.path(),MAX_FILE)else{return;};let actual=Some(Fingerprint::bytes(&bytes));
                let known=match kind{"old"=>actual==record.before,"new"=>actual==record.after,"replace"=>actual==record.before||actual==record.after,_=>false};
                if !known{return;}
            }
        }
        for e in entries{let _=fs::remove_file(e.path());}let _=fs::remove_dir(&self.directory);
    }
}
pub fn pending(path:&Path)->Result<Vec<PendingSave>> {
    let project=root(path)?;let base=project.join(DIRECTORY);if !base.exists(){return Ok(vec![]);}
    journal_base(&project)?;let mut out=Vec::new();
    for (visited,entry) in fs::read_dir(&base).map_err(fail)?.enumerate() {
        require(visited<128,"Project save journal directory exceeds entry budget")?;
        let entry=entry.map_err(fail)?;let id=entry.file_name().to_string_lossy().to_string();
        if !valid_id(&id){continue;}require(out.len()<32,"Too many pending project save journals")?;
        let journal=Journal::open(&project,&id)?;
        if journal.manifest.state!=SaveState::Complete{out.push(PendingSave{id,state:journal.manifest.state,files:journal.manifest.entries.iter().map(|e|e.file.clone()).collect()});}
    }Ok(out)
}
/// User-selected completion/rollback. A changed external file is never restored
/// over. Caller must reload its disk baseline after this succeeds.
pub fn recover(path:&Path,id:&str,finish:bool)->Result<()> {
    let project=root(path)?;let base=journal_base(&project)?;let _lock=project_lock(&base)?;
    let mut journal=Journal::open(&project,id)?;
    if finish {journal.apply(true,&mut |_|Ok(()))?;journal.manifest.state=SaveState::Complete;journal.persist()?;journal.cleanup();Ok(())}
    else {journal.rollback()}
}


/// Discard only checked derived recovery files. It never edits project files.
pub fn discard_recovery(path:&Path,id:&str)->Result<()> {
    let project=root(path)?;let base=journal_base(&project)?;let _lock=project_lock(&base)?;
    let journal=Journal::open(&project,id)?;
    journal.cleanup();
    require(!journal.directory.exists(),"Recovery data contains unexpected files; it was left in place")
}
/// Save the canonical editor version to a new project directory. The original
/// text files are deliberately not re-read: a copy is also the conflict escape.
pub fn copy_project(project:&Path,sources:&Sources,parent:&Path)->Result<PathBuf> {
    validate_sources(sources)?;
    let project=root(project)?;let parent=root(parent)?;
    require(!parent.starts_with(&project),"Choose a copy folder outside the original project")?;
    let mut assets=Vec::new();let mut total=sources.values().map(String::len).sum::<usize>();
    fn visit(base:&Path,dir:&Path,depth:usize,assets:&mut Vec<(String,Vec<u8>,Fingerprint)>,total:&mut usize,count:usize)->Result<()> {
        require(depth<=20,"Project copy nesting exceeds its limit")?;
        for e in fs::read_dir(dir).map_err(fail)? {
            let e=e.map_err(fail)?;let name=e.file_name().to_string_lossy().to_string();
            if name.starts_with('.')||matches!(name.as_str(),"node_modules"|"target"|"builds"){continue;}
            no_reparse(&e.path())?;let kind=e.file_type().map_err(fail)?;
            if kind.is_dir(){visit(base,&e.path(),depth+1,assets,total,count)?;}
            else if kind.is_file()&&crate::snapshot::is_asset(&e.path()) {
                let bytes=read(&e.path(),60*1024*1024)?;
                *total=total.checked_add(bytes.len()).ok_or("Project copy size overflow")?;
                require(*total<=60*1024*1024&&assets.len()+count<600,"Project copy exceeds file/byte limits")?;
                let fingerprint=Fingerprint::bytes(&bytes);
                require(Fingerprint::file(&e.path())?==fingerprint,"An asset changed during project copy")?;
                let name=e.path().strip_prefix(base).map_err(fail)?.to_string_lossy().replace('\\',"/");
                assets.push((name,bytes,fingerprint));
            }
        }Ok(())
    }
    visit(&project,&project,0,&mut assets,&mut total,sources.len())?;
    let directory=parent.join(format!("Texloom-copy-{}",&identity()[..12]));
    fs::create_dir(&directory).map_err(fail)?;let directory=directory.canonicalize().map_err(fail)?;
    require(directory.parent()==Some(parent.as_path()),"Copy destination escaped its selected folder")?;
    let marker=directory.join(".texloom-copy-pending");write_new(&marker,b"Texloom incomplete project copy v1\n")?;
    let result:Result<()>=(||{
        for (name,text) in sources {write_new(&destination(&directory,name,true)?,text.as_bytes())?;}
        for (name,bytes,expected) in &assets {
            // Assets use the same strict relative-path grammar, with a separate
            // extension check; source destination() intentionally only accepts text.
            let relative=relative(name)?;require(crate::snapshot::is_asset(&relative),"Invalid copied asset")?;
            let target=checked_destination(&directory,relative.clone(),true)?;
            write_new(&target,bytes)?;
            let original=project.join(&relative);no_reparse(&original)?;
            require(Fingerprint::file(&original)?==*expected,"An asset changed during project copy")?;
        }
        fs::remove_file(&marker).map_err(fail)?;Ok(())
    })();
    result.map_err(|e|format!("Project copy was not completed at {}: {e}",directory.display()))?;
    Ok(directory)
}

/// Salvage canonical source when the original project folder is unavailable.
/// This deliberately copies no assets and never recreates the original path.
pub fn copy_sources(sources:&Sources,parent:&Path)->Result<PathBuf> {
    validate_sources(sources)?;let parent=root(parent)?;
    let directory=parent.join(format!("Texloom-recovered-sources-{}",&identity()[..12]));
    fs::create_dir(&directory).map_err(fail)?;let directory=directory.canonicalize().map_err(fail)?;
    require(directory.parent()==Some(parent.as_path()),"Source copy escaped its selected folder")?;
    let marker=directory.join(".texloom-copy-pending");write_new(&marker,b"Texloom incomplete source recovery copy v1\n")?;
    let result:Result<()>=(||{
        for (name,text) in sources {write_new(&destination(&directory,name,true)?,text.as_bytes())?;}
        fs::remove_file(&marker).map_err(fail)?;Ok(())
    })();
    result.map_err(|e|format!("Source copy was not completed at {}: {e}",directory.display()))?;
    Ok(directory)
}

#[cfg(test)]
mod tests;
