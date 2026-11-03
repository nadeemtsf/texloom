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

export type ResolveSelectionRequest = { projectId: string, revision: number, buildId: string, page: number, points: Array<SelectionPoint>, candidate: number, };

export type SaveState = "preparing" | "prepared" | "committing" | "complete" | "rolled_back";

export type PendingSave = { id: string, state: SaveState, files: Array<string>, };

export type Draft = { id: string, project: string, main: string, revision: number, source_bytes: number, files: number, warning: string | null, };

export type Inventory = { drafts: Array<Draft>, errors: Array<string>, };

export type Project = { id: string, name: string, main: string, sources: { [key in string]?: string }, recoveries: Array<PendingSave>, recovery_error: string | null, restored: boolean, draft_warning: string | null, source_only: boolean, };

export type ProjectProfile = { schema: number, main: string, engine: string, };

export type RecentProject = { path: string, name: string, main: string, engine: string, opened: number, };

export type ImportedProject = { path: string, };

export type ImportedImage = { asset: string, sha256: string, };

export type EncodingIssue = { file: string, detected: string | null, bytes: number, sha256: string, };

export type EncodingReview = { root: string, fingerprint: string, files: Array<EncodingIssue>, };

export type EncodingChoice = { file: string, encoding: string, sha256: string, };

export type AdapterRequest = { sources: { [key in string]?: string }, main: string, file: string, expected_source: string, control: number | null, selection: [number, number] | null, key: string, value: string, };

export type AdapterControl = { id: number, kind: string, label: string, line: number, parameters: Array<AdapterParameter>, reason: string | null, };

export type AdapterParameter = { key: string, label: string, value: string, reason: string | null, };

export type AdapterPlan = { sources: { [key in string]?: string }, file: string, };

export type TransformRequest = { source: string, action: string, id: string, changes: Record<string, unknown>, selection: [number, number] | null, content: string, plain: boolean, page: number, x: number, y: number, width: number, size: number, };

export type ObjectEditRequest = { sources: { [key in string]?: string }, main: string, file: string, action: string, id: string, selection: [number, number] | null, expected: string | null, dx: number | null, dy: number | null, content: string | null, plain: boolean, literal_edits: Array<LiteralEditRequest> | null, spacing_edits: Array<SpacingEditRequest> | null, };

export type ObjectPlan = { sources: { [key in string]?: string }, object_id: string, file: string, operation: string, };

export type LiteralEditRequest = { id: number, text: string, };

export type LiteralRunView = { id: number, text: string, styles: Array<string>, };

export type SpacingEditRequest = { id: number, value: string, };

export type SpacingView = { id: number, value: number | null, unit: string, source: string, reason: string | null, };

export type TextEditModel = { runs: Array<LiteralRunView>, protected: Array<string>, spacing: Array<SpacingView>, };

export type VisualOperation = { "kind": "adapter", request: AdapterRequest, } | { "kind": "text", request: ObjectEditRequest, } | { "kind": "legacy", file: string, request: TransformRequest, };

export type EmptyRequest = Record<string, never>;

export type OpenProjectRequest = { path: string | null, };

export type PathRequest = { path: string, };

export type CreateProjectRequest = { template: string, name: string, parent: string | null, };

export type ProjectRequest = { projectId: string, };

export type DraftRequest = { draftId: string, };

export type SourceControlsRequest = { source: string, };

export type AdapterCommandRequest = { request: AdapterRequest, };

export type ObjectCommandRequest = { request: ObjectEditRequest, };

export type TransformCommandRequest = { request: TransformRequest, };

export type TextModelRequest = { content: string, document: string, };

export type TextPreviewRequest = { content: string, document: string, edits: Array<LiteralEditRequest>, spacingEdits: Array<SpacingEditRequest> | null, };

export type EncodingPreviewRequest = { path: string, file: string, encoding: string, };

export type ConvertEncodingRequest = { path: string, parent: string, fingerprint: string, choices: Array<EncodingChoice>, };

export type CopyProjectRequest = { projectId: string, revision: number, parent: string, };

export type SaveRecoveryRequest = { projectId: string, revision: number, transactionId: string, finish: boolean, };

export type DiscardSaveRequest = { projectId: string, revision: number, transactionId: string, };

export type ProjectFileRequest = { projectId: string, revision: number, path: string, };

export type CloseProjectRequest = { projectId: string, revision: number | null, discardRecovery: boolean | null, };

export type PrepareVisualRequest = { projectId: string, revision: number, sources: { [key in string]?: string }, baselineBuildId: string, operation: VisualOperation | null, };

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

export interface WorkspaceRequests {
  open_project: OpenProjectRequest;
  open_recent: PathRequest;
  recent_projects: EmptyRequest;
  create_project: CreateProjectRequest;
  import_project_archive: PathRequest;
  export_project_archive: ProjectFileRequest;
  import_image: ProjectFileRequest;
  review_encoding: PathRequest;
  preview_encoding: EncodingPreviewRequest;
  convert_encoding: ConvertEncodingRequest;
  copy_document: CopyProjectRequest;
  project_recoveries: ProjectRequest;
  recover_document: SaveRecoveryRequest;
  discard_document_recovery: DiscardSaveRequest;
  typing_drafts: EmptyRequest;
  open_typing_draft: DraftRequest;
  discard_typing_draft: DraftRequest;
  close_document: CloseProjectRequest;
  mark_clean_exit: EmptyRequest;
  document_snapshot: ProjectRequest;
  document_prepare: PrepareVisualRequest;
  describe_adapters: SourceControlsRequest;
  plan_adapter: AdapterCommandRequest;
  transform: TransformCommandRequest;
  plan_text_object: ObjectCommandRequest;
  describe_text_edit: TextModelRequest;
  preview_text_edit: TextPreviewRequest;
}

export interface WorkspaceReplies {
  open_project: { id: string, name: string, main: string, sources: { [key in string]?: string }, recoveries: Array<PendingSave>, recovery_error: string | null, restored: boolean, draft_warning: string | null, source_only: boolean, };
  open_recent: { id: string, name: string, main: string, sources: { [key in string]?: string }, recoveries: Array<PendingSave>, recovery_error: string | null, restored: boolean, draft_warning: string | null, source_only: boolean, };
  recent_projects: Array<{ path: string, name: string, main: string, engine: string, opened: number, }>;
  create_project: { path: string, };
  import_project_archive: { path: string, };
  export_project_archive: null;
  import_image: { asset: string, sha256: string, };
  review_encoding: { root: string, fingerprint: string, files: Array<EncodingIssue>, };
  preview_encoding: string;
  convert_encoding: { path: string, };
  copy_document: string;
  project_recoveries: Array<{ id: string, state: SaveState, files: Array<string>, }>;
  recover_document: { id: string, name: string, main: string, sources: { [key in string]?: string }, recoveries: Array<PendingSave>, recovery_error: string | null, restored: boolean, draft_warning: string | null, source_only: boolean, };
  discard_document_recovery: Array<{ id: string, state: SaveState, files: Array<string>, }>;
  typing_drafts: { drafts: Array<Draft>, errors: Array<string>, };
  open_typing_draft: { id: string, name: string, main: string, sources: { [key in string]?: string }, recoveries: Array<PendingSave>, recovery_error: string | null, restored: boolean, draft_warning: string | null, source_only: boolean, };
  discard_typing_draft: { drafts: Array<Draft>, errors: Array<string>, };
  close_document: { warning: string | null, };
  mark_clean_exit: null;
  document_snapshot: { revision: number, sources: { [key in string]?: string }, main: string, engine: string, can_undo: boolean, can_redo: boolean, selection: EditorSelection | null, recovery: RecoveryStatus, };
  document_prepare: { token: string, revision: number, };
  describe_adapters: Array<{ id: number, kind: string, label: string, line: number, parameters: Array<AdapterParameter>, reason: string | null, }>;
  plan_adapter: { sources: { [key in string]?: string }, file: string, };
  transform: string;
  plan_text_object: { sources: { [key in string]?: string }, object_id: string, file: string, operation: string, };
  describe_text_edit: { runs: Array<LiteralRunView>, protected: Array<string>, spacing: Array<SpacingView>, };
  preview_text_edit: string;
}
