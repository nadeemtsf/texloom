//! ts-rs declarations and Serde fixtures for the source/history IPC slice.
use super::*;
use crate::service::{
    CloseReply, DocumentSnapshot, EditorSelection, PreparedReply, RecoveryReply, RecoveryStatus,
    SelectionReply, SourceEditReply,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use ts_rs::TS;

pub fn declarations() -> String {
    let mut text = String::from("// Generated from Rust with ts-rs 11.1.0. Do not edit.\n// IPC integers are numbers; runtime guards require safe integers.\n\n");
    macro_rules! emit {
        ($($ty:ty),+ $(,)?) => { $(text.push_str("export "); text.push_str(&<$ty>::decl()); text.push_str("\n\n");)+ };
    }
    emit!(
        EditorChange,
        EditorRange,
        EditorSelection,
        RecoveryStatus,
        DocumentSnapshot,
        SourceEditReply,
        SelectionReply,
        RecoveryReply,
        PreparedReply,
        CloseReply,
        RevisionRequest,
        EditRequest,
        SelectionRequest,
        ProfileRequest,
        HistoryRequest,
        CommitRequest
    );
    text.push_str("export interface DocumentRequests {\n");
    macro_rules! commands {
        ($($name:ident: $request:ty => $reply:ty),+ $(,)?) => {
            $(let _ = crate::$name;
              text.push_str(&format!("  {}: {};\n", stringify!($name), <$request>::ident()));)+
            text.push_str("}\n\nexport interface DocumentReplies {\n");
            $(text.push_str(&format!("  {}: {};\n", stringify!($name), <$reply>::ident()));)+
        };
    }
    commands!(
        document_edit: EditRequest => SourceEditReply,
        document_selection: SelectionRequest => SelectionReply,
        document_profile: ProfileRequest => DocumentSnapshot,
        document_history: HistoryRequest => DocumentSnapshot,
        document_commit: CommitRequest => DocumentSnapshot,
        save_document: RevisionRequest => RecoveryReply,
        retry_typing_recovery: RevisionRequest => RecoveryReply,
    );
    text.push_str("}\n");
    text
}

fn protection(revision: u64) -> RecoveryStatus {
    RecoveryStatus {
        draft_id: Some("a".repeat(32)),
        protected_revision: Some(revision),
        error: None,
    }
}
pub fn fixtures() -> Value {
    let selection = || EditorSelection {
        file: "main.tex".into(),
        anchor: 0,
        head: 1,
        secondary: vec![EditorRange { anchor: 4, head: 4 }],
    };
    json!({
        "snapshot": DocumentSnapshot { revision: 7,
            sources: BTreeMap::from([("main.tex".into(), "\u{feff}A\r\n\u{1f600}\r\n".into()), ("style.sty".into(), "% keep\r\n".into())]),
            main: "main.tex".into(), engine: "auto".into(), can_undo: true, can_redo: false,
            selection: Some(selection()), recovery: protection(7) },
        "edit_request": EditRequest { project_id: "project".into(), revision: 7, file: "main.tex".into(),
            changes: vec![EditorChange { offset: 0, length: 1, text: "B".into() }], group: Some(1) },
        "source_delta": SourceEditReply { kind: "source_delta", base_revision: 7, revision: 8,
            sources: BTreeMap::from([("main.tex".into(), "\u{feff}B\r\n\u{1f600}\r\n".into())]),
            main: "main.tex".into(), engine: "auto".into(), can_undo: true, can_redo: false,
            selection: Some(selection()), recovery: protection(8) },
        "selection_reply": SelectionReply { revision: 8, selection: Some(selection()), recovery: protection(8) },
        "recovery_reply": RecoveryReply { revision: 8, recovery: protection(8) },
        "prepared_reply": PreparedReply { token: "b".repeat(32), revision: 9 },
        "close_reply": CloseReply { warning: None },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_document_types_and_serde_fixtures_are_current() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let actual = std::fs::read_to_string(root.join("web/generated/document-ipc.d.ts")).unwrap();
        assert_eq!(
            actual.replace("\r\n", "\n"),
            declarations(),
            "Run npm run protocol:rust:generate"
        );
        let actual: Value = serde_json::from_slice(
            &std::fs::read(root.join("tests/fixtures/document-ipc.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, fixtures(), "Run npm run protocol:rust:generate");
    }
}
