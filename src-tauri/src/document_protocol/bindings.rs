//! ts-rs declarations and Serde fixtures for the source/history IPC slice.
use super::*;
use crate::build_protocol::*;
use crate::workspace_protocol::*;
use crate::service::{Project,ProjectProfile,RecentProject,ImportedProject,ImportedImage,EncodingIssue,EncodingReview,EncodingChoice,AdapterRequest,AdapterControl,AdapterParameter,AdapterPlan,TransformRequest,ObjectEditRequest,ObjectPlan,LiteralEditRequest,LiteralRunView,SpacingEditRequest,SpacingView,TextEditModel,VisualOperation};
use crate::service::{
    CloseReply, DocumentSnapshot, EditorSelection, PreparedReply, RecoveryReply, RecoveryStatus,
    SelectionReply, SourceEditReply, CompileResult, PositionedBlock, MappedTextObject,
    SourceLocation, SourceRectangle, SelectionPoint, ResolvedSelection,
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
        CommitRequest,
        texloom_build::engine::Severity,
        texloom_build::engine::Diagnostic,
        PositionedBlock,
        MappedTextObject,
        CompileResult,
        SourceLocation,
        SourceRectangle,
        BuildPurpose,
        CompileDocumentRequest,
        CancelCompilationRequest,
        BuildIdRequest,
        ProjectBuildRequest,
        DiscardCandidateRequest,
        ExportBuildRequest,
        ForwardMappingRequest,
        InverseMappingRequest,
        SelectionPoint,
        ResolvedSelection,
        ResolveSelectionRequest,
        texloom_build::persistence::SaveState,texloom_build::persistence::PendingSave,texloom_build::recovery::Draft,texloom_build::recovery::Inventory,
        Project,ProjectProfile,RecentProject,ImportedProject,ImportedImage,EncodingIssue,EncodingReview,EncodingChoice,
        AdapterRequest,AdapterControl,AdapterParameter,AdapterPlan,TransformRequest,ObjectEditRequest,ObjectPlan,LiteralEditRequest,LiteralRunView,SpacingEditRequest,SpacingView,TextEditModel,VisualOperation,
        EmptyRequest,OpenProjectRequest,PathRequest,CreateProjectRequest,ProjectRequest,DraftRequest,SourceControlsRequest,AdapterCommandRequest,ObjectCommandRequest,TransformCommandRequest,TextModelRequest,TextPreviewRequest,EncodingPreviewRequest,ConvertEncodingRequest,CopyProjectRequest,SaveRecoveryRequest,DiscardSaveRequest,ProjectFileRequest,CloseProjectRequest,PrepareVisualRequest
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
    text.push_str("}\n\nexport interface BuildRequests {\n");
    macro_rules! build_commands {
        ($($name:ident: $request:ty => $reply:ty),+ $(,)?) => {
            $(let _ = crate::$name;
              text.push_str(&format!("  {}: {};\n", stringify!($name), <$request>::ident()));)+
            text.push_str("}\n\nexport interface BuildReplies {\n");
            $(text.push_str(&format!("  {}: {};\n", stringify!($name), <$reply>::inline()));)+
        };
    }
    build_commands!(
        compile_document: CompileDocumentRequest => CompileResult,
        cancel_compilation: CancelCompilationRequest => (),
        verify_build: BuildIdRequest => (),
        pin_displayed_build: ProjectBuildRequest => (),
        document_discard: DiscardCandidateRequest => (),
        export_document_build: ExportBuildRequest => (),
        source_from_pdf: InverseMappingRequest => SourceLocation,
        pdf_from_source: ForwardMappingRequest => SourceRectangle,
        resolve_text_selection: ResolveSelectionRequest => ResolvedSelection,
    );
    text.push_str("}\n");
    text.push_str("\nexport interface WorkspaceRequests {\n");
    macro_rules! workspace_commands {
        ($($name:ident: $request:ty => $reply:ty),+ $(,)?)=>{
            $(let _=crate::$name;text.push_str(&format!("  {}: {};\n",stringify!($name),<$request>::ident()));)+
            text.push_str("}\n\nexport interface WorkspaceReplies {\n");
            $(text.push_str(&format!("  {}: {};\n",stringify!($name),<$reply>::inline()));)+
        };
    }
    workspace_commands!(
        open_project:OpenProjectRequest=>Project,open_recent:PathRequest=>Project,recent_projects:EmptyRequest=>Vec<RecentProject>,
        create_project:CreateProjectRequest=>ImportedProject,
        import_project_archive:PathRequest=>ImportedProject,export_project_archive:ProjectFileRequest=>(),import_image:ProjectFileRequest=>ImportedImage,
        review_encoding:PathRequest=>EncodingReview,preview_encoding:EncodingPreviewRequest=>String,convert_encoding:ConvertEncodingRequest=>ImportedProject,
        copy_document:CopyProjectRequest=>String,project_recoveries:ProjectRequest=>Vec<texloom_build::persistence::PendingSave>,recover_document:SaveRecoveryRequest=>Project,discard_document_recovery:DiscardSaveRequest=>Vec<texloom_build::persistence::PendingSave>,
        typing_drafts:EmptyRequest=>texloom_build::recovery::Inventory,open_typing_draft:DraftRequest=>Project,discard_typing_draft:DraftRequest=>texloom_build::recovery::Inventory,close_document:CloseProjectRequest=>CloseReply,mark_clean_exit:EmptyRequest=>(),
        document_snapshot:ProjectRequest=>DocumentSnapshot,document_prepare:PrepareVisualRequest=>PreparedReply,
        describe_adapters:SourceControlsRequest=>Vec<AdapterControl>,plan_adapter:AdapterCommandRequest=>AdapterPlan,transform:TransformCommandRequest=>String,plan_text_object:ObjectCommandRequest=>ObjectPlan,describe_text_edit:TextModelRequest=>TextEditModel,preview_text_edit:TextPreviewRequest=>String,
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
        "project": Project {id:"project".into(),name:"Fixture".into(),main:"main.tex".into(),sources:BTreeMap::from([("main.tex".into(),"\u{feff}A\r\n\u{1f600}\r\n".into())]),recoveries:vec![],recovery_error:None,restored:false,draft_warning:None,source_only:false},
        "pending_saves": vec![texloom_build::persistence::PendingSave{id:"b".repeat(32),state:texloom_build::persistence::SaveState::Committing,files:vec!["main.tex".into()]}],
        "inventory":texloom_build::recovery::Inventory{drafts:vec![texloom_build::recovery::Draft{id:"a".repeat(32),project:"C:/Fixture".into(),main:"main.tex".into(),revision:7,source_bytes:32,files:2,warning:None}],errors:vec![]},
        "adapter_controls":crate::service::describe_adapters("\\includegraphics[width=2cm]{image.png}\n\\vspace{2mm}").unwrap(),
        "text_model":crate::service::describe_text_edit("Hello \\textbf{world}","Hello \\textbf{world}").unwrap(),
        "recent_projects":vec![RecentProject{path:"C:/Fixture".into(),name:"Fixture".into(),main:"main.tex".into(),engine:"pdflatex".into(),opened:1_793_720_000}],
        "encoding_review":EncodingReview{root:"C:/Fixture".into(),fingerprint:"a".repeat(64),files:vec![EncodingIssue{file:"main.tex".into(),detected:Some("utf-16le".into()),bytes:42,sha256:"b".repeat(64)}]},
        "imported_project":ImportedProject{path:"C:/Fixture".into()},
        "imported_image":ImportedImage{asset:format!("images/image-{}.png","a".repeat(32)),sha256:"c".repeat(64)},
        "compile_request": CompileDocumentRequest { project_id: "project".into(), revision: 7,
            token: None, purpose: Some(BuildPurpose::Export), request_id: 1 },
        "compile_success": CompileResult { ok: true, build_id: "c".repeat(32), revision: 7,
            pdf: b"%PDF-1.7\n".to_vec(),
            blocks: vec![PositionedBlock { id: "legacy".into(), file: "main.tex".into(), page: 1,
                x: 12.0, y: 24.0, width: 100.0, size: 12.0, height: 15.0,
                content: "A".into(), start: 0, end: 1, line: 1 }],
            objects: vec![MappedTextObject { id: "signature".into(), file: "main.tex".into(),
                dx: 0.0, dy: 12.0, plain_text: Some("A".into()), content: "A".into(), line: 1, start: 0, end: 1 }],
            log: "Fixture only; not a renderable PDF".into(), engine: "pdflatex".into(), seconds: 0.25,
            verification_eligible: true, passes: 2, diagnostics: vec![], job_id: 42 },
        "compile_failure": CompileResult { ok: false, build_id: "d".repeat(32), revision: 7,
            pdf: vec![], blocks: vec![], objects: vec![], log: "Compiler initialization failed".into(),
            engine: "pdflatex".into(), seconds: 0.1, verification_eligible: false, passes: 0,
            diagnostics: vec![texloom_build::engine::Diagnostic { code: "compiler_preflight_failed".into(),
                severity: texloom_build::engine::Severity::Error, message: "Compiler unavailable".into(), file: None, line: None }],
            job_id: 43 },
        "source_location": SourceLocation { file: "main.tex".into(), line: 2 },
        "source_rectangle": SourceRectangle { page: 1, x: 12.0, y: -3.0, width: 1.25, height: 8.0 },
        "resolved_selection": ResolvedSelection { schema: 1, unit_id: "fixture:main.tex:3:4".into(),
            build_id: "c".repeat(32), revision: 7, file: "main.tex".into(), expected_source: "\u{feff}A\r\n\u{1f600}\r\n".into(),
            start: 0, end: 1, first_line: 1, last_line: 1, kind: "line".into(), content: "A".into(), plain_text: Some("A".into()), rectangles: vec![] },
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
