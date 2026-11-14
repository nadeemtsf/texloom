use std::collections::BTreeMap;
use texloom_core::document::{structure::RegionKind, *};

fn session() -> Session {
    Session::new(BTreeMap::from([("main.tex".into(),
        "\u{feff}\\documentclass{article}\r\n\\begin{document}\r\nAlpha.\r\n\r\nBeta.\r\n\\end{document}\r\n".into())]),
        Profile {main:"main.tex".into(),engine:"auto".into()}).unwrap()
}
fn patch(s:&Session,old:&str,new:&str)->Patch {
    let at=s.sources()["main.tex"].find(old).unwrap();
    Patch {base:s.revision(),profile:None,files:vec![FileChange::Edit {file:"main.tex".into(),
        edits:vec![Replacement {range:at..at+old.len(),expected:old.into(),text:new.into()}]}]}
}
fn edit(s:&mut Session,old:&str,new:&str) {
    let p=s.prepare(patch(s,old,new)).unwrap();s.commit(p,Origin::Source,Some(TypingGroup(7))).unwrap();
}
fn equal(a:&Session,b:&Session) {
    assert_eq!(a.sources(),b.sources());assert_eq!(a.profile(),b.profile());assert_eq!(a.revision(),b.revision());
    assert_eq!(a.selection(),b.selection());assert_eq!(a.history_len(),b.history_len());
    assert_eq!(a.can_undo(),b.can_undo());assert_eq!(a.can_redo(),b.can_redo());
}
#[test]
fn fork_preserves_mixed_history_and_redo_but_mutates_independently() {
    let mut original=session();
    let at=original.sources()["main.tex"].find("Alpha").unwrap();
    original.set_selection(Selection {file:"main.tex".into(),anchor:at,head:at+5,secondary:vec![]}).unwrap();
    edit(&mut original,"Alpha","Changed");edit(&mut original,"Changed","Final");
    let p=original.prepare(Patch {base:original.revision(),profile:None,
        files:vec![FileChange::Create {file:"style.sty".into(),text:"% untouched\r\n".into()}]}).unwrap();
    original.commit(p,Origin::Canvas,None).unwrap();
    let p=original.prepare(Patch {base:original.revision(),files:vec![],profile:Some(ProfileChange {
        expected:original.profile().clone(),replacement:Profile {main:"main.tex".into(),engine:"xelatex".into()}})}).unwrap();
    original.commit(p,Origin::Settings,None).unwrap();original.undo().unwrap();
    let mut fork=original.fork_for_preflight();equal(&original,&fork);
    for _ in 0..2 {assert!(original.undo().unwrap());assert!(fork.undo().unwrap());equal(&original,&fork);}
    for _ in 0..3 {assert!(original.redo().unwrap());assert!(fork.redo().unwrap());equal(&original,&fork);}
    let before=original.sources().clone();edit(&mut fork,"Final","Fork only");
    assert_eq!(original.sources(),&before);assert_ne!(fork.sources(),original.sources());
    fork.undo().unwrap();assert_eq!(fork.sources(),original.sources());
}
#[test]
fn fork_cannot_share_commit_tokens_or_source_entity_authority() {
    let mut original=session();let mut fork=original.fork_for_preflight();
    let p=original.prepare(patch(&original,"Alpha","Original")).unwrap();
    assert_eq!(fork.commit(p,Origin::Source,None),Err(EditError::ForeignSession));
    let p=fork.prepare(patch(&fork,"Alpha","Fork")).unwrap();
    assert_eq!(original.commit(p,Origin::Source,None),Err(EditError::ForeignSession));
    let projection=original.source_projection("main.tex").unwrap();
    let index=projection.regions().iter().position(|r|r.kind()==RegionKind::PlainParagraph).unwrap();
    let reference=projection.entity_reference(index).unwrap();
    assert!(matches!(fork.resolve_entity_reference(&reference),Err(EditError::ForeignSession)));
    let projection=fork.source_projection("main.tex").unwrap();
    let reference=projection.entity_reference(index).unwrap();
    assert!(matches!(original.resolve_entity_reference(&reference),Err(EditError::ForeignSession)));
    equal(&original,&fork);
}
#[test]
fn fork_retains_noop_redo_and_typing_groups_with_normal_guards() {
    let mut original=session();edit(&mut original,"Alpha","First");
    let mut fork=original.fork_for_preflight();
    edit(&mut original,"First","Second");edit(&mut fork,"First","Second");
    assert_eq!(original.history_len(),1);equal(&original,&fork);
    original.undo().unwrap();fork.undo().unwrap();equal(&original,&fork);
    let p=fork.prepare(patch(&fork,"Alpha","Alpha")).unwrap();
    fork.commit(p,Origin::Source,None).unwrap();equal(&original,&fork);assert!(fork.can_redo());
    let mut invalid=patch(&fork,"Alpha","Other");
    if let FileChange::Edit {edits,..}=&mut invalid.files[0] {edits[0].expected="Wrong".into();}
    assert!(matches!(fork.prepare(invalid),Err(EditError::ExpectedTextMismatch)));
    equal(&original,&fork);
}
