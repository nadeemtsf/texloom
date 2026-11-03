import type {
  DocumentSnapshot, EditorSelection, PreparedReply, RecoveryStatus,
  DocumentRequests,
} from '../../web/generated/document-ipc.js';

export type Sources = Record<string, string>;
export type Snapshot = DocumentSnapshot & { sources: Sources };
export type Command = keyof DocumentRequests;
export type NativeInvoke = (name: string, args: Record<string, unknown>) => Promise<unknown>;

export class DocumentProtocolError extends Error {
  constructor(message: string) { super(message); this.name = 'DocumentProtocolError'; }
}
function require(ok: unknown, message: string): asserts ok {
  if (!ok) throw new DocumentProtocolError(message);
}
function record(value: unknown, keys?: readonly string[]): asserts value is Record<string, unknown> {
  require(value !== null && typeof value === 'object' && !Array.isArray(value), 'Invalid document acknowledgement object');
  if (keys) require(Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key)), 'Invalid document acknowledgement fields');
}
export function assertRevision(value: unknown): asserts value is number {
  require(typeof value === 'number' && Number.isSafeInteger(value) && value >= 0, 'Invalid document revision');
}
function sources(value: unknown): asserts value is Sources {
  record(value);
  require(Object.entries(value).every(([file, text]) => file.length > 0 && typeof text === 'string'), 'Invalid document sources');
}
function protection(value: unknown, revision: number): asserts value is RecoveryStatus {
  record(value, ['draft_id', 'protected_revision', 'error']);
  require((value.draft_id === null || typeof value.draft_id === 'string' && /^[0-9a-f]{32}$/.test(value.draft_id)) &&
    (value.protected_revision === null || typeof value.protected_revision === 'number' && Number.isSafeInteger(value.protected_revision) && value.protected_revision >= 0 && value.protected_revision <= revision) &&
    (value.error === null || typeof value.error === 'string') && (value.draft_id !== null || value.protected_revision === null), 'Invalid recovery protection acknowledgement');
}
export function editorText(source: string): string {
  return source.replace(/^\uFEFF/u, '').replace(/\r\n?|\n/gu, '\n');
}
export function sameSources(left: Sources, right: Sources): boolean {
  if (left === right) return true;
  const files = Object.keys(left);
  return files.length === Object.keys(right).length && files.every(file => Object.hasOwn(right, file) && left[file] === right[file]);
}
function boundary(text: string, offset: unknown): offset is number {
  if (typeof offset !== 'number' || !Number.isSafeInteger(offset) || offset < 0 || offset > text.length) return false;
  const before = text.charCodeAt(offset - 1), after = text.charCodeAt(offset);
  return !(before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff);
}
function selection(value: unknown, buffers: Sources): asserts value is EditorSelection | null {
  if (value === null) return;
  record(value, ['file', 'anchor', 'head', 'secondary']);
  require(typeof value.file === 'string' && Object.hasOwn(buffers, value.file), 'Invalid document selection file');
  const source = buffers[value.file];
  require(typeof source === 'string', 'Invalid document selection source');
  const text = editorText(source);
  require(boundary(text, value.anchor) && boundary(text, value.head) && Array.isArray(value.secondary) && value.secondary.length <= 9999, 'Invalid document selection');
  for (const cursor of value.secondary) {
    record(cursor, ['anchor', 'head']);
    require(boundary(text, cursor.anchor) && boundary(text, cursor.head), 'Invalid secondary selection');
  }
}
const snapshotFields = ['revision', 'sources', 'main', 'engine', 'can_undo', 'can_redo', 'selection', 'recovery'] as const;
function metadata(value: Record<string, unknown>): asserts value is Record<string, unknown> & Pick<DocumentSnapshot, 'main' | 'engine' | 'can_undo' | 'can_redo'> {
  require(typeof value.main === 'string' && typeof value.engine === 'string' &&
    ['auto', 'pdflatex', 'xelatex', 'lualatex'].includes(value.engine) &&
    typeof value.can_undo === 'boolean' && typeof value.can_redo === 'boolean', 'Invalid document profile/history metadata');
}
export function assertSnapshot(value: unknown): asserts value is Snapshot {
  record(value, snapshotFields);
  assertRevision(value.revision); sources(value.sources); metadata(value);
  require(typeof value.main === 'string' && Object.hasOwn(value.sources, value.main), 'Invalid document main source');
  selection(value.selection, value.sources); protection(value.recovery, value.revision);
}
function nextRevision(previous: number, next: number): void {
  require(next >= previous, 'Document acknowledgement moved backward');
  require(next === previous || next === previous + 1, 'Document acknowledgement skipped a revision');
}
export function acceptReply(name: Command, args: Record<string, unknown>, value: unknown, previous: Snapshot): { snapshot: Snapshot; sourceChanged: boolean } {
  record(value); assertRevision(value.revision); nextRevision(previous.revision, value.revision);
  if (name === 'document_edit') {
    record(value, [...snapshotFields, 'kind', 'base_revision']);
    require(value.kind === 'source_delta' && value.base_revision === previous.revision, 'Invalid source edit acknowledgement; source synchronization stopped');
    metadata(value); sources(value.sources); protection(value.recovery, value.revision);
    require(value.main === previous.main && value.engine === previous.engine, 'Invalid source edit profile');
    const file = args.file, changed = value.revision !== previous.revision, files = Object.keys(value.sources);
    require(typeof file === 'string' && Object.hasOwn(previous.sources, file), 'Invalid edited source file');
    require(changed ? files.length === 1 && files[0] === file && value.sources[file] !== previous.sources[file] : files.length === 0, 'Invalid source edit acknowledgement contents');
    if (!changed) require(value.can_undo === previous.can_undo && value.can_redo === previous.can_redo, 'No-op source edit changed history');
    const buffers = changed ? { ...previous.sources, ...value.sources } : previous.sources;
    selection(value.selection, buffers);
    // The runtime checks above retain the actual Rust DTO contract. Construct
    // only canonical fields; wire tags never become document state.
    return { snapshot: { revision: value.revision, sources: buffers, main: value.main, engine: value.engine,
      can_undo: value.can_undo, can_redo: value.can_redo, selection: value.selection, recovery: value.recovery }, sourceChanged: true };
  }
  if (name === 'document_selection' || name === 'save_document' || name === 'retry_typing_recovery') {
    record(value, name === 'document_selection' ? ['revision', 'selection', 'recovery'] : ['revision', 'recovery']);
    require(value.revision === previous.revision, 'Metadata acknowledgement changed the source revision');
    protection(value.recovery, value.revision);
    const protectionChanged = value.recovery.draft_id !== previous.recovery.draft_id ||
      value.recovery.protected_revision !== previous.recovery.protected_revision || value.recovery.error !== previous.recovery.error;
    if (name === 'document_selection') {
      selection(value.selection, previous.sources);
      require(value.selection !== null && value.selection.file === args.file && value.selection.anchor === args.anchor && value.selection.head === args.head &&
        Array.isArray(args.secondary) && value.selection.secondary.length === args.secondary.length &&
        value.selection.secondary.every((cursor, index) => {
          const requested: unknown = (args.secondary as unknown[])[index];
          record(requested, ['anchor', 'head']);
          return cursor.anchor === requested.anchor && cursor.head === requested.head;
        }), 'Selection acknowledgement did not match the requested cursors');
      return { snapshot: { ...previous, selection: value.selection, recovery: value.recovery }, sourceChanged: protectionChanged };
    }
    return { snapshot: { ...previous, recovery: value.recovery }, sourceChanged: protectionChanged };
  }
  assertSnapshot(value);
  if (value.revision === previous.revision) require(sameSources(previous.sources, value.sources) && value.main === previous.main && value.engine === previous.engine &&
    value.can_undo === previous.can_undo && value.can_redo === previous.can_redo, 'No-op acknowledgement changed canonical state');
  if (name === 'document_profile') require(sameSources(previous.sources, value.sources) && value.main === args.main && value.engine === args.engine, 'Profile acknowledgement changed unexpected state');
  return { snapshot: value, sourceChanged: true };
}
export function preparedReply(value: unknown, base: number): PreparedReply {
  record(value, ['token', 'revision']); assertRevision(value.revision); nextRevision(base, value.revision);
  require(typeof value.token === 'string' && value.token.length > 0, 'Invalid prepared candidate token');
  return { token: value.token, revision: value.revision };
}
