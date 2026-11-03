/** Ordered browser replica. Rust owns text/history and visual approvals. */
import type { DocumentRequests, EditorChange, EditorRange } from '../../web/generated/document-ipc.js';
import {
  acceptReply, assertRevision, assertSnapshot, DocumentProtocolError, preparedReply,
} from './document-protocol.js';
import type { Command, NativeInvoke, Snapshot, Sources } from './document-protocol.js';
export { editorText, sameSources } from './document-protocol.js';
type Args<C extends Command> = Omit<DocumentRequests[C], 'projectId' | 'revision'>;
type Options = { changed?: (snapshot: Snapshot) => void; failed?: (error: Error) => void };
function errorOf(value: unknown): Error { return value instanceof Error ? value : new Error(String(value)); }

export class DocumentClient {
  snapshot: Snapshot;
  pending = 0;
  outstanding = 0;
  error: Error | null = null;
  private unpublished = false;
  private tail: Promise<unknown> = Promise.resolve();
  private compileSerial = 0;
  private activeToken: string | null = null;
  private readonly changed: (snapshot: Snapshot) => void;
  private readonly failed: (error: Error) => void;
  constructor(private readonly invoke: NativeInvoke, readonly projectId: string, snapshot: unknown, options: Options = {}) {
    assertSnapshot(snapshot); this.snapshot = snapshot;
    this.changed = options.changed ?? (() => {}); this.failed = options.failed ?? (() => {});
  }
  private enqueue<C extends Command>(name: C, args: Args<C>, fatal = true, mutating = true): Promise<Snapshot> {
    this.outstanding++; if (mutating) this.pending++;
    const job = this.tail.then(async () => {
      if (this.error) throw this.error;
      // Caller arguments cannot replace queue-owned project/revision identity.
      const request: Record<string, unknown> = { ...args, projectId: this.projectId };
      if (name !== 'document_commit') request.revision = this.snapshot.revision;
      const next: unknown = await this.invoke(name, request);
      const accepted = acceptReply(name, request, next, this.snapshot);
      this.snapshot = accepted.snapshot;
      if (accepted.sourceChanged) this.unpublished = true;
      return this.snapshot;
    }).catch((value: unknown) => {
      const error = errorOf(value);
      if ((fatal || error instanceof DocumentProtocolError) && !this.error) { this.error = error; this.failed(error); }
      throw error;
    }).finally(() => {
      this.outstanding--; if (mutating) this.pending--;
      if (!this.outstanding && !this.error && this.unpublished) { this.unpublished = false; this.changed(this.snapshot); }
    });
    this.tail = job.catch(() => {});
    return job;
  }
  async flush(): Promise<Snapshot> {
    for (;;) {
      const tail = this.tail; await tail;
      if (this.error) throw this.error;
      if (tail === this.tail && !this.outstanding) return this.snapshot;
    }
  }
  edit(file: string, changes: EditorChange[], group: number | null = null): Promise<Snapshot> {
    return this.enqueue('document_edit', { file, changes, group });
  }
  profile(main: string, engine: string): Promise<Snapshot> { return this.enqueue('document_profile', { main, engine }); }
  history(redo: boolean): Promise<Snapshot> { return this.enqueue('document_history', { redo }); }
  save(): Promise<Snapshot> { return this.enqueue('save_document', {}, false); }
  retryRecovery(): Promise<Snapshot> { return this.enqueue('retry_typing_recovery', {}, false); }
  selection(file: string, anchor: number, head: number, secondary: EditorRange[] = []): Promise<Snapshot> {
    return this.enqueue('document_selection', { file, anchor, head, secondary }, true, false);
  }
  async prepare(sources: Sources, baselineBuildId: string, operation: unknown = null) {
    await this.flush();
    const base = this.snapshot.revision;
    return preparedReply(await this.invoke('document_prepare', { projectId: this.projectId, revision: base, sources, baselineBuildId, operation }), base);
  }
  async compile(token: string | null = null, purpose = 'preview'): Promise<unknown> {
    this.activeToken = token;
    const requestId = ++this.compileSerial; assertRevision(requestId);
    try { return await this.invoke('compile_document', { projectId: this.projectId, revision: this.snapshot.revision, token, purpose, requestId }); }
    finally {
      if (requestId !== this.compileSerial) { const error = new Error('Compilation cancelled or superseded.'); error.name = 'CompilationCancelled'; throw error; }
    }
  }
  async cancel(): Promise<unknown> {
    const requestId = this.compileSerial; assertRevision(this.compileSerial + 1); this.compileSerial++;
    return this.invoke('cancel_compilation', { projectId: this.projectId, requestId, token: this.activeToken });
  }
  async verify(buildId: string): Promise<unknown> { return this.invoke('verify_build', { buildId }); }
  async pin(buildId: string): Promise<unknown> { return this.invoke('pin_displayed_build', { projectId: this.projectId, buildId }); }
  commit(token: string, buildId: string): Promise<Snapshot> { return this.enqueue('document_commit', { token, buildId }, false); }
  async discard(token: string): Promise<unknown> { return this.invoke('document_discard', { projectId: this.projectId, token }); }
}
