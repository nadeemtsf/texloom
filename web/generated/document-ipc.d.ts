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
