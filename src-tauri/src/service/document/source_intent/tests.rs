use super::*;

const SOURCE: &str = "\u{feff}\\documentclass{article}\r\n\\begin{document}\r\nRepeated.\r\n\r\nRepeated.\r\n\r\nTail.\r\n\\end{document}\r\n";
fn fixture(test: impl FnOnce(&Backend, &Project, &Path)) {
    let root = std::env::temp_dir().join(format!("texloom-source-intent-{}", Uuid::new_v4().simple()));
    fs::create_dir_all(root.join("project")).unwrap();
    fs::write(root.join("project/main.tex"), SOURCE).unwrap();
    let backend = Backend::new(root.join("app"));
    let project = backend.open_project(Some(root.join("project/main.tex").to_string_lossy().into_owned())).unwrap();
    test(&backend, &project, &root);
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}
fn targets(b: &Backend, p: &Project, base: u64) -> Vec<SourceTarget> {
    b.document_source_targets(&p.id, base, "main.tex").unwrap().targets
}
fn applied(reply: &SourceDraftReply, from: u64, revision: u64, changed: bool) {
    assert!(matches!(&reply.outcome, SourceDraftOutcome::SourceDraftApplied {
        from_revision: f, revision: r, changed: c, render_verification: "unresolved"
    } if (*f,*r,*c) == (from,revision,changed)));
}

#[test]
fn repeated_occurrence_is_exact_draft_history_and_retry_never_reexecutes_after_undo() {
    fixture(|b,p,root| {
        let list = targets(b,p,0);
        assert_eq!(list.len(),3);
        assert_eq!(list[0].text,list[1].text);
        let intent = DraftSourceIntent::DeleteParagraphContent { target: list[1].handle.clone() };
        let op = "a".repeat(32);
        let reply = b.document_source_intent(&p.id,0,&op,intent.clone()).unwrap();
        applied(&reply,0,1,true);
        let mut expected = SOURCE.to_string();
        let second = expected.rfind("Repeated.").unwrap();
        expected.replace_range(second..second+"Repeated.".len(),"");
        assert_eq!(reply.snapshot.sources["main.tex"],expected);
        assert_eq!(reply.snapshot.recovery.protected_revision,Some(1));
        assert_eq!(fs::read_to_string(root.join("project/main.tex")).unwrap(),SOURCE);
        let retry = b.document_source_intent(&p.id,0,&op,intent.clone()).unwrap();
        applied(&retry,0,1,true);
        assert_eq!(retry.snapshot.revision,1);
        let undo = b.document_history(&p.id,1,false).unwrap();
        assert_eq!(undo.sources["main.tex"],SOURCE);
        let after_undo = b.document_source_intent(&p.id,0,&op,intent).unwrap();
        applied(&after_undo,0,1,true);
        assert_eq!(after_undo.snapshot.revision,2);
        assert_eq!(after_undo.snapshot.sources["main.tex"],SOURCE);
        assert!(after_undo.snapshot.can_redo);
        let status = b.document_source_intent_status(&p.id,&op).unwrap();
        assert_eq!(serde_json::to_value(&status.outcome).unwrap(),serde_json::to_value(&reply.outcome).unwrap());
        assert_eq!(status.snapshot.revision,2);
        assert_eq!(b.document_history(&p.id,2,true).unwrap().sources["main.tex"],expected);
    });
}

#[test]
fn literal_draft_escapes_payload_and_changed_draft_cancels_candidate_and_build_only_once() {
    fixture(|b,p,_| {
        let list = targets(b,p,0);
        let prepared = b.document_prepare(&p.id,0,BTreeMap::from([("main.tex".into(),SOURCE.replace("Tail.","Other."))])).unwrap();
        let lease = b.scheduler.request_sequence(&p.id,1).unwrap().acquire().unwrap();
        let reply = b.document_source_intent(&p.id,0,&"b".repeat(32),DraftSourceIntent::ReplaceLiteralParagraph {
            target:list[1].handle.clone(),text:"New & \\input{private} ש".into()
        }).unwrap();
        applied(&reply,0,1,true);
        assert!(reply.snapshot.sources["main.tex"].contains("New \\& \\textbackslash{}input\\{private\\} ש"));
        assert_eq!(reply.snapshot.sources["main.tex"].matches("Repeated.").count(),1);
        assert!(lease.cancel.cancelled());
        assert!(!b.documents.lock().unwrap()[&p.id].pending.contains_key(&prepared.token));
    });
}

#[test]
fn no_op_keeps_redo_selection_pending_candidate_and_running_job() {
    fixture(|b,p,_| {
        let first = targets(b,p,0)[0].handle.clone();
        b.document_source_intent(&p.id,0,&"c".repeat(32),DraftSourceIntent::ReplaceLiteralParagraph { target:first,text:"Changed.".into() }).unwrap();
        b.document_history(&p.id,1,false).unwrap();
        let list = targets(b,p,2);
        b.document_selection(&p.id,2,"main.tex".into(),list[0].start,list[0].end,vec![]).unwrap();
        let before = b.document_snapshot(&p.id).unwrap();
        let prepared = b.document_prepare(&p.id,2,BTreeMap::from([("main.tex".into(),SOURCE.replace("Tail.","Other."))])).unwrap();
        let lease = b.scheduler.request_sequence(&p.id,1).unwrap().acquire().unwrap();
        let reply = b.document_source_intent(&p.id,2,&"d".repeat(32),DraftSourceIntent::ReplaceLiteralParagraph {
            target:list[0].handle.clone(),text:list[0].text.clone()
        }).unwrap();
        applied(&reply,2,2,false);
        assert!(reply.snapshot.can_redo);
        assert_eq!(serde_json::to_value(reply.snapshot.selection).unwrap(),serde_json::to_value(before.selection).unwrap());
        assert!(!lease.cancel.cancelled());
        assert!(b.documents.lock().unwrap()[&p.id].pending.contains_key(&prepared.token));
    });
}

#[test]
fn stale_foreign_and_reopened_handles_refuse_even_when_source_bytes_match() {
    fixture(|b,p,root| {
        let old = targets(b,p,0)[0].handle.clone();
        let other_path = root.join("other"); fs::create_dir_all(&other_path).unwrap();
        fs::write(other_path.join("main.tex"),SOURCE).unwrap();
        let other = b.open_project(Some(other_path.join("main.tex").to_string_lossy().into_owned())).unwrap();
        assert!(b.document_source_intent(&other.id,0,&"e".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:old.clone() }).is_err());
        b.document_source_intent(&p.id,0,&"f".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:old.clone() }).unwrap();
        b.document_history(&p.id,1,false).unwrap();
        assert!(b.document_source_intent(&p.id,2,&"1".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:old.clone() }).is_err());
        b.close_document_checked(&p.id,2,false).unwrap();
        let reopened = b.open_project(Some(root.join("project/main.tex").to_string_lossy().into_owned())).unwrap();
        assert!(b.document_source_intent(&reopened.id,0,&"2".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:old }).is_err());
        assert!(matches!(b.document_source_intent_status(&reopened.id,&"f".repeat(32)).unwrap().outcome,SourceDraftOutcome::Unknown));
    });
}

#[test]
fn receipt_payload_conflicts_capacity_and_unknown_lookup_cannot_mutate_source() {
    fixture(|b,p,_| {
        let target = targets(b,p,0)[0].handle.clone();
        let op = "3".repeat(32);
        let no_op = DraftSourceIntent::ReplaceLiteralParagraph { target:target.clone(),text:"Repeated.".into() };
        b.document_source_intent(&p.id,0,&op,no_op.clone()).unwrap();
        assert!(b.document_source_intent(&p.id,0,&op,DraftSourceIntent::DeleteParagraphContent { target:target.clone() }).is_err());
        for n in 1..MAX_OPERATIONS {
            b.document_source_intent(&p.id,0,&format!("{n:032x}"),no_op.clone()).unwrap();
        }
        assert!(b.document_source_intent(&p.id,0,&"4".repeat(32),no_op.clone()).is_err());
        applied(&b.document_source_intent(&p.id,0,&op,no_op).unwrap(),0,0,false);
        assert!(matches!(b.document_source_intent_status(&p.id,&"5".repeat(32)).unwrap().outcome,SourceDraftOutcome::Unknown));
        assert_eq!(b.document_snapshot(&p.id).unwrap().sources["main.tex"],SOURCE);
    });
}

#[test]
fn strict_tagged_request_rejects_extra_fields_invalid_combinations_and_limits() {
    use crate::document_protocol::SourceIntentRequest;
    let request = serde_json::json!({"projectId":"p","revision":0,"operationId":"a".repeat(32),
        "intent":{"kind":"delete_paragraph_content","target":"b".repeat(32)}});
    assert!(serde_json::from_value::<SourceIntentRequest>(request.clone()).is_ok());
    for extra in [serde_json::json!({"kind":"delete_paragraph_content","target":"b".repeat(32),"text":"wrong"}),
        serde_json::json!({"kind":"replace_literal_paragraph","target":"b".repeat(32)}),
        serde_json::json!({"kind":"delete_asset","target":"b".repeat(32)})] {
        let mut invalid=request.clone(); invalid["intent"]=extra;
        assert!(serde_json::from_value::<SourceIntentRequest>(invalid).is_err());
    }
    let mut extra=request; extra["offset"]=0.into();
    assert!(serde_json::from_value::<SourceIntentRequest>(extra).is_err());
    fixture(|b,p,_| {
        let handle=targets(b,p,0)[0].handle.clone();
        for text in ["\n".into()," ".into(),"\0".into(),"x".repeat(256_001)] {
            assert!(b.document_source_intent(&p.id,0,&"6".repeat(32),DraftSourceIntent::ReplaceLiteralParagraph { target:handle.clone(),text }).is_err());
        }
        assert_eq!(b.document_snapshot(&p.id).unwrap().revision,0);
    });
}

#[test]
fn recovery_failure_is_uncertain_cached_and_does_not_mutate_canonical_source() {
    fixture(|b,p,root| {
        let handle=targets(b,p,0)[0].handle.clone();
        // Remove the journal through the native test slot, then make its store
        // path unavailable. This is a synthetic I/O failure, not real data loss.
        b.documents.lock().unwrap().get_mut(&p.id).unwrap().protection = Default::default();
        let recovery=root.join("app/.source-recovery");
        let retained=root.join("retained-recovery");
        if recovery.exists() { fs::rename(&recovery,&retained).unwrap(); }
        fs::write(&recovery,"blocked synthetic store").unwrap();
        let op="7".repeat(32);
        let intent=DraftSourceIntent::DeleteParagraphContent { target:handle };
        let reply=b.document_source_intent(&p.id,0,&op,intent.clone()).unwrap();
        assert!(matches!(reply.outcome,SourceDraftOutcome::ProtectionUncertain { from_revision:0,.. }));
        assert_eq!(reply.snapshot.sources["main.tex"],SOURCE);
        assert_eq!(reply.snapshot.revision,0);
        fs::remove_file(&recovery).unwrap();
        if retained.exists() { fs::rename(retained,&recovery).unwrap(); }
        let retry=b.document_source_intent(&p.id,0,&op,intent).unwrap();
        assert!(matches!(retry.outcome,SourceDraftOutcome::ProtectionUncertain { .. }));
        assert_eq!(retry.snapshot.revision,0);
        assert!(b.document_source_intent(&p.id,0,&"0".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:targets(b,p,0)[0].handle.clone() }).is_err());
        assert!(b.document_edit_delta(&p.id,0,"main.tex",vec![],None).is_err());
        assert!(b.document_history(&p.id,0,false).is_err());
        assert!(b.document_profile(&p.id,0,"main.tex".into(),"auto".into()).is_err());
        assert!(b.save_document(&p.id,0).is_err());
        assert!(b.retry_typing_recovery(&p.id,0).is_err());
        assert!(b.close_document_checked(&p.id,0,true).is_err());
        assert!(matches!(b.document_source_intent_status(&p.id,&op).unwrap().outcome,SourceDraftOutcome::ProtectionUncertain { .. }));
    });
}

#[test]
fn accepted_draft_recovers_exact_shared_history_without_operation_or_visual_authority() {
    let root=std::env::temp_dir().join(format!("texloom-intent-restart-{}",Uuid::new_v4().simple()));
    fs::create_dir_all(root.join("project")).unwrap();
    fs::write(root.join("project/main.tex"),SOURCE).unwrap();
    let backend=Backend::new(root.join("app"));
    let project=backend.open_project(Some(root.join("project/main.tex").to_string_lossy().into_owned())).unwrap();
    let handle=targets(&backend,&project,0)[1].handle.clone();
    let op="8".repeat(32);
    let reply=backend.document_source_intent(&project.id,0,&op,DraftSourceIntent::ReplaceLiteralParagraph { target:handle.clone(),text:"Recovered draft.".into() }).unwrap();
    applied(&reply,0,1,true);
    drop(backend);
    let backend=Backend::new(root.join("app"));
    let inventory=backend.typing_drafts().unwrap();
    assert_eq!(inventory.drafts.len(),1);
    let recovered=backend.open_typing_draft(&inventory.drafts[0].id).unwrap();
    assert_eq!(recovered.sources,reply.snapshot.sources);
    assert!(matches!(backend.document_source_intent_status(&recovered.id,&op).unwrap().outcome,SourceDraftOutcome::Unknown));
    assert!(backend.document_source_intent(&recovered.id,1,&"9".repeat(32),DraftSourceIntent::DeleteParagraphContent { target:handle }).is_err());
    assert_eq!(backend.document_history(&recovered.id,1,false).unwrap().sources["main.tex"],SOURCE);
    assert_eq!(fs::read_to_string(root.join("project/main.tex")).unwrap(),SOURCE);
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}
