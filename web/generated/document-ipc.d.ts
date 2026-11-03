// Generated from Rust with ts-rs 11.1.0. Do not edit.
// IPC integers are numbers; runtime guards require safe integers.

export type EditorChange = { offset: number, length: number, text: string, };

export type EditorRange = { anchor: number, head: number, };

export type EditorSelection = { file: string, anchor: number, head: number, secondary: Array<EditorRange>, };

export type RecoveryStatus = { draft_id: string | null, protected_revision: number | null, error: string | null, };

export type DocumentSnapshot = { revision: number, sources: { [key in string]?: string }, main: string, engine: string, can_undo: boolean, can_redo: boolean, selection: EditorSelection | null, recovery: RecoveryStatus, };

export type SourceEditReply = { kind: 'source_delta', base_revision: number, revision: number, sources: { [key in string]?: string }, main: string, engine: string, can_undo: boolean, can_redo: boolean, selection: EditorSelection | null, recovery: RecoveryStatus, };

export type SelectionReply = { revision: number, selection: EditorSelection | null, recovery: RecoveryStatus, };

export type RecoveryReply = { revision: number, recovery: RecoveryStatus, };

export type PreparedReply = { token: string, revision: number, };

export type CloseReply = { warning: string | null, };

export type RevisionRequest = { projectId: string, revision: number, };

export type EditRequest = { projectId: string, revision: number, file: string, changes: Array<EditorChange>, group: number | null, };

export type SelectionRequest = { projectId: string, revision: number, file: string, anchor: number, head: number, secondary: Array<EditorRange>, };

export type ProfileRequest = { projectId: string, revision: number, main: string, engine: string, };

export type HistoryRequest = { projectId: string, revision: number, redo: boolean, };

export type CommitRequest = { projectId: string, token: string, buildId: string, };

export type Severity = "error" | "warning" | "info";

export type Diagnostic = { code: string, severity: Severity, message: string, file: string | null, line: number | null, };

export type PositionedBlock = { id: string, file: string, page: number, x: number, y: number, width: number, size: number, content: string, height: number, start: number, end: number, line: number, };

export type MappedTextObject = { id: string, file: string, dx: number, dy: number, plain_text: string | null, content: string, line: number, start: number, end: number, };

export type CompileResult = { ok: boolean, build_id: string, revision: number, pdf: Array<number>, blocks: Array<PositionedBlock>, objects: Array<MappedTextObject>, log: string, engine: string, seconds: number, verification_eligible: boolean, passes: number, diagnostics: Array<Diagnostic>, job_id: number, };

export type SourceLocation = { file: string, line: number, };

export type SourceRectangle = { page: number, x: number, y: number, width: number, height: number, };

export type BuildPurpose = "preview" | "visual" | "export";

export type CompileDocumentRequest = { projectId: string, revision: number, token: string | null, purpose: BuildPurpose | null, requestId: number, };

export type CancelCompilationRequest = { projectId: string, requestId: number, token: string | null, };

export type BuildIdRequest = { buildId: string, };

export type ProjectBuildRequest = { projectId: string, buildId: string, };

export type DiscardCandidateRequest = { projectId: string, token: string, };

export type ExportBuildRequest = { projectId: string, revision: number, buildId: string, path: string, };

export type ForwardMappingRequest = { buildId: string, file: string, line: number, };

export type InverseMappingRequest = { buildId: string, page: number, x: number, y: number, };

export type SelectionPoint = { x: number, y: number, };

export type ResolvedSelection = { schema: number, unit_id: string, build_id: string, revision: number, file: string, expected_source: string, start: number, end: number, first_line: number, last_line: number, kind: string, content: string, plain_text: string | null, rectangles: Array<SourceRectangle>, };

export type ResolveSelectionRequest = { projectId: string, revision: number, buildId: string, page: number, points: Array<SelectionPoint>, };

export interface DocumentRequests {
  document_edit: EditRequest;
  document_selection: SelectionRequest;
  document_profile: ProfileRequest;
  document_history: HistoryRequest;
  document_commit: CommitRequest;
  save_document: RevisionRequest;
  retry_typing_recovery: RevisionRequest;
}

export interface DocumentReplies {
  document_edit: SourceEditReply;
  document_selection: SelectionReply;
  document_profile: DocumentSnapshot;
  document_history: DocumentSnapshot;
  document_commit: DocumentSnapshot;
  save_document: RecoveryReply;
  retry_typing_recovery: RecoveryReply;
}

export interface BuildRequests {
  compile_document: CompileDocumentRequest;
  cancel_compilation: CancelCompilationRequest;
  verify_build: BuildIdRequest;
  pin_displayed_build: ProjectBuildRequest;
  document_discard: DiscardCandidateRequest;
  export_document_build: ExportBuildRequest;
  source_from_pdf: InverseMappingRequest;
  pdf_from_source: ForwardMappingRequest;
  resolve_text_selection: ResolveSelectionRequest;
}

export interface BuildReplies {
  compile_document: { ok: boolean, build_id: string, revision: number, pdf: Array<number>, blocks: Array<PositionedBlock>, objects: Array<MappedTextObject>, log: string, engine: string, seconds: number, verification_eligible: boolean, passes: number, diagnostics: Array<Diagnostic>, job_id: number, };
  cancel_compilation: null;
  verify_build: null;
  pin_displayed_build: null;
  document_discard: null;
  export_document_build: null;
  source_from_pdf: { file: string, line: number, };
  pdf_from_source: { page: number, x: number, y: number, width: number, height: number, };
  resolve_text_selection: { schema: number, unit_id: string, build_id: string, revision: number, file: string, expected_source: string, start: number, end: number, first_line: number, last_line: number, kind: string, content: string, plain_text: string | null, rectangles: Array<SourceRectangle>, };
}
