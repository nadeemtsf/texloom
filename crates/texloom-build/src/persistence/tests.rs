use super::*;
use serde_json::Value;
struct Fixture { root:PathBuf, before:Sources }
impl Fixture {
    fn new()->Self {
        let root=std::env::temp_dir().join(format!("texloom-save-test-{}",identity()));
        fs::create_dir(&root).unwrap();
        let before=BTreeMap::from([("main.tex".into(),"\u{feff}Main \u{1f600}\r\nKeep\r\n".into()),("part.tex".into(),"Part\n".into())]);
        for (name,text) in &before {fs::write(root.join(name),text).unwrap();}
        Self{root:root.canonicalize().unwrap(),before}
    }
    fn disk(&self)->DiskSnapshot {DiskSnapshot::capture(&self.root,&self.before).unwrap()}
    fn after(&self)->Sources {
        let mut after=self.before.clone();after.insert("main.tex".into(),"\u{feff}New \u{1f600}\r\nKeep\r\n".into());
        after.insert("texloom-objects.sty".into(),"Portable style\r\n".into());after
    }
    fn assert_sources(&self,sources:&Sources) {
        for (name,text) in sources {assert_eq!(fs::read(self.root.join(name)).unwrap(),text.as_bytes(),"{name}");}
    }
}
impl Drop for Fixture {fn drop(&mut self){fs::remove_dir_all(&self.root).unwrap();}}
fn id(root:&Path)->String {pending(root).unwrap().into_iter().next().unwrap().id}
fn crash(f:&Fixture)->String {
    let mut disk=f.disk();let after=f.after();
    let interrupted=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||disk.save_with(&after,|s|{
        if matches!(s,Stage::AfterReplace(0)){panic!("simulated process interruption after first replacement")}Ok(())
    })));
    assert!(interrupted.is_err());let id=id(&f.root);
    assert_eq!(pending(&f.root).unwrap()[0].state,SaveState::Committing);id
}

#[test]
fn external_changed_removed_and_new_files_are_conflicts_without_writes() {
    for kind in ["changed","removed","appeared"] {
        let f=Fixture::new();let mut disk=f.disk();let after=f.after();
        let name=if kind=="appeared"{"texloom-objects.sty"}else{"main.tex"};
        if kind=="removed"{fs::remove_file(f.root.join(name)).unwrap();}else{fs::write(f.root.join(name),"EXTERNAL").unwrap();}
        let outcome=disk.save(&after).unwrap();let SaveOutcome::Conflicts(conflicts)=outcome else{panic!("overwrote external source")};
        assert_eq!(conflicts.len(),1);assert_eq!(conflicts[0].kind,kind);assert_eq!(conflicts[0].file,name);
        assert!(!f.root.join(DIRECTORY).exists());
        if kind!="removed"{assert_eq!(fs::read(f.root.join(name)).unwrap(),b"EXTERNAL");}
        assert_eq!(fs::read(f.root.join("part.tex")).unwrap(),b"Part\n");
    }
}
#[test]
fn successful_save_preserves_bytes_and_handles_creation_deletion_and_nested_sources() {
    let f=Fixture::new();let mut disk=f.disk();let mut after=f.after();after.remove("part.tex");
    after.insert("chapters/\u{645}\u{631}\u{62d}\u{628}\u{627}.tex".into(),"\u{645}\u{631}\u{62d}\u{628}\u{627}\n".into());
    fs::write(f.root.join("main.texloom-saving"),b"UNRELATED").unwrap();
    assert!(matches!(disk.save(&after).unwrap(),SaveOutcome::Saved));f.assert_sources(&after);
    assert!(!f.root.join("part.tex").exists());assert_eq!(fs::read(f.root.join("main.texloom-saving")).unwrap(),b"UNRELATED");
    assert!(disk.conflicts(&after).unwrap().is_empty());assert!(pending(&f.root).unwrap().is_empty());
    assert!(matches!(disk.save(&after).unwrap(),SaveOutcome::Saved));
}
#[test]
fn a_failed_multi_file_save_rolls_back_and_retains_the_intended_bytes() {
    let f=Fixture::new();let mut disk=f.disk();let after=f.after();
    assert!(disk.save_with(&after,|s|if matches!(s,Stage::AfterReplace(0)){Err("injected disk/write error".into())}else{Ok(())}).is_err());
    f.assert_sources(&f.before);assert!(!f.root.join("texloom-objects.sty").exists());
    assert!(disk.conflicts(&f.before).unwrap().is_empty());
    let saves=pending(&f.root).unwrap();assert_eq!(saves.len(),1);assert_eq!(saves[0].state,SaveState::RolledBack);
    recover(&f.root,&saves[0].id,true).unwrap();f.assert_sources(&after);assert!(pending(&f.root).unwrap().is_empty());
}
#[test]
fn an_interrupted_save_can_finish_after_all_in_memory_state_is_lost() {
    let f=Fixture::new();let transaction=crash(&f);recover(&f.root,&transaction,true).unwrap();
    f.assert_sources(&f.after());assert!(pending(&f.root).unwrap().is_empty());
}
#[test]
fn an_interrupted_save_can_restore_originals_and_still_keep_the_pending_edit() {
    let f=Fixture::new();let transaction=crash(&f);recover(&f.root,&transaction,false).unwrap();
    f.assert_sources(&f.before);assert!(!f.root.join("texloom-objects.sty").exists());
    assert_eq!(pending(&f.root).unwrap()[0].state,SaveState::RolledBack);
    recover(&f.root,&transaction,true).unwrap();f.assert_sources(&f.after());
}
#[test]
fn external_edits_during_staging_or_recovery_are_never_rolled_over() {
    let f=Fixture::new();let mut disk=f.disk();
    assert!(disk.save_with(&f.after(),|s|{if matches!(s,Stage::Prepared){fs::write(f.root.join("main.tex"),b"EXTERNAL DURING SAVE").unwrap();}Ok(())}).is_err());
    let transaction=id(&f.root);
    assert!(recover(&f.root,&transaction,true).is_err());assert!(recover(&f.root,&transaction,false).is_err());
    assert_eq!(fs::read(f.root.join("main.tex")).unwrap(),b"EXTERNAL DURING SAVE");
    assert_eq!(fs::read(f.root.join("part.tex")).unwrap(),b"Part\n");
    assert!(!f.root.join("texloom-objects.sty").exists());
}
#[test]
fn corrupted_manifest_and_staged_bytes_cannot_authorize_recovery() {
    let f=Fixture::new();let transaction=crash(&f);let directory=f.root.join(DIRECTORY).join(&transaction);
    let manifest=fs::read(directory.join("manifest.json")).unwrap();
    let mut envelope:Value=serde_json::from_slice(&manifest).unwrap();envelope["sha256"]=serde_json::json!("0".repeat(64));
    fs::write(directory.join("manifest.json"),serde_json::to_vec(&envelope).unwrap()).unwrap();
    let original=fs::read(f.root.join("main.tex")).unwrap();assert!(recover(&f.root,&transaction,false).is_err());
    assert_eq!(fs::read(f.root.join("main.tex")).unwrap(),original);
    fs::write(directory.join("manifest.json"),manifest).unwrap();
    fs::write(directory.join("0000.old"),b"CORRUPTED BACKUP").unwrap();
    assert!(recover(&f.root,&transaction,false).is_err());assert_eq!(fs::read(f.root.join("main.tex")).unwrap(),original);
    fs::write(directory.join("0001.new"),b"CORRUPTED STAGED STYLE").unwrap();
    assert!(recover(&f.root,&transaction,true).is_err());assert!(!f.root.join("texloom-objects.sty").exists());
}
#[test]
fn unrecognized_save_folders_and_source_aliases_are_preserved_and_refused() {
    let f=Fixture::new();let mut disk=f.disk();
    for name in ["../outside.tex","nested//alias.tex","C:/outside.tex",".texloom-save/owner.tex"] {
        let mut after=f.after();after.insert(name.into(),"BAD".into());assert!(disk.save(&after).is_err());
    }
    fs::create_dir(f.root.join(DIRECTORY)).unwrap();fs::write(f.root.join(DIRECTORY).join("user-data"),b"KEEP").unwrap();
    assert!(disk.save(&f.after()).is_err());f.assert_sources(&f.before);
    assert_eq!(fs::read(f.root.join(DIRECTORY).join("user-data")).unwrap(),b"KEEP");
}
#[test]
fn two_texloom_instances_cannot_save_the_same_project_concurrently() {
    let f=Fixture::new();let mut one=f.disk();let mut two=f.disk();let after=f.after();
    assert!(matches!(one.save_with(&after,|stage|{
        if matches!(stage,Stage::Prepared){assert!(two.save(&after).is_err());}Ok(())
    }).unwrap(),SaveOutcome::Saved));f.assert_sources(&after);
}
#[test]
fn cleanup_is_flat_and_keeps_unknown_nested_content() {
    let f=Fixture::new();let mut disk=f.disk();let mut kept=None;
    assert!(matches!(disk.save_with(&f.after(),|stage|{
        if matches!(stage,Stage::Prepared){let folder=f.root.join(DIRECTORY).join(id(&f.root)).join("unknown");
            fs::create_dir(&folder).unwrap();fs::write(folder.join("keep"),b"KEEP").unwrap();kept=Some(folder);}
        Ok(())
    }).unwrap(),SaveOutcome::Saved));assert_eq!(fs::read(kept.unwrap().join("keep")).unwrap(),b"KEEP");
}
#[test]
fn a_moved_or_foreign_project_cannot_use_another_projects_journal() {
    let a=Fixture::new();let b=Fixture::new();let transaction=crash(&a);
    let base=journal_base(&b.root).unwrap();let target=base.join(&transaction);fs::create_dir(&target).unwrap();
    fs::copy(a.root.join(DIRECTORY).join(&transaction).join("manifest.json"),target.join("manifest.json")).unwrap();
    assert!(recover(&b.root,&transaction,true).is_err());b.assert_sources(&b.before);
}
#[cfg(windows)]
#[test]
fn a_locked_source_rolls_back_prior_replacements_and_can_be_recovered_after_unlock() {
    use std::os::windows::fs::OpenOptionsExt;
    let f=Fixture::new();let mut disk=f.disk();let mut after=f.after();after.insert("part.tex".into(),"Changed part".into());
    let lock=fs::OpenOptions::new().read(true).share_mode(1).open(f.root.join("part.tex")).unwrap();
    assert!(disk.save(&after).is_err());f.assert_sources(&f.before);let transaction=id(&f.root);drop(lock);
    recover(&f.root,&transaction,true).unwrap();f.assert_sources(&after);
}
#[cfg(unix)]
#[test]
fn symlinked_destinations_and_parents_cannot_escape_the_project() {
    let f=Fixture::new();let outside=Fixture::new();let mut disk=f.disk();
    std::os::unix::fs::symlink(&outside.root,f.root.join("link")).unwrap();
    let mut after=f.after();after.insert("link/main.tex".into(),"OVERWRITE".into());
    assert!(disk.save(&after).is_err());outside.assert_sources(&outside.before);
}
