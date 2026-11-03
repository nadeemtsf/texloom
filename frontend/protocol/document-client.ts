/** Ordered browser replica. Rust owns text/history and visual approvals. */
import type { DocumentRequests, EditorChange, EditorRange, BuildPurpose, CompileResult, SourceLocation, SourceRectangle, SelectionPoint, ResolvedSelection } from '../../web/generated/document-ipc.js';
import { acceptCompilation, acceptSourceLocation, acceptSourceRectangle, acceptResolvedSelection, assertSourceLine, BuildProtocolError, invokeUnit, requestBuild } from './build-protocol.js';
import type { CompileContext } from './build-protocol.js';
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
  private readonly candidates = new Map<string, { base: number; context: CompileContext }>();
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
      if (accepted.snapshot.revision !== this.snapshot.revision) this.candidates.clear();
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
    const context = { sources: { ...sources }, engine: this.snapshot.engine };
    const reply = preparedReply(await this.invoke('document_prepare', { projectId: this.projectId, revision: base, sources, baselineBuildId, operation }), base);
    this.candidates.set(reply.token, { base, context: { ...context, revision: reply.revision } });
    return reply;
  }
  async compile(token: string | null = null, purpose: BuildPurpose = 'preview'): Promise<CompileResult> {
    const snapshot = this.snapshot, candidate = token === null ? null : this.candidates.get(token);
    if (token !== null && (!candidate || candidate.base !== snapshot.revision)) throw new BuildProtocolError('The visual candidate is no longer current');
    const context = candidate?.context ?? { revision: snapshot.revision, sources: snapshot.sources, engine: snapshot.engine };
    this.activeToken = token;
    const requestId = ++this.compileSerial; assertRevision(requestId);
    try {
      const result = await requestBuild(this.invoke, 'compile_document', { projectId: this.projectId, revision: snapshot.revision, token, purpose, requestId });
      return acceptCompilation(result, context);
    }
    finally {
      if (requestId !== this.compileSerial) { const error = new Error('Compilation cancelled or superseded.'); error.name = 'CompilationCancelled'; throw error; }
    }
  }
  async cancel(): Promise<void> {
    const requestId = this.compileSerial; assertRevision(this.compileSerial + 1); this.compileSerial++;
    const token = this.activeToken;
    if (token !== null) this.candidates.delete(token);
    return invokeUnit(this.invoke, 'cancel_compilation', { projectId: this.projectId, requestId, token });
  }
  async verify(buildId: string): Promise<void> { return invokeUnit(this.invoke, 'verify_build', { buildId }); }
  async pin(buildId: string): Promise<void> { return invokeUnit(this.invoke, 'pin_displayed_build', { projectId: this.projectId, buildId }); }
  async exportBuild(buildId: string, path: string): Promise<void> {
    const snapshot = await this.flush();
    return invokeUnit(this.invoke, 'export_document_build', { projectId: this.projectId, revision: snapshot.revision, buildId, path });
  }
  private assertMappingCurrent(snapshot: Snapshot): void {
    if (this.error || this.pending || snapshot.revision !== this.snapshot.revision) throw new BuildProtocolError('The source changed during navigation. Compile the current source.');
  }
  async forward(buildId: string, file: string, line: number): Promise<SourceRectangle> {
    const snapshot = await this.flush(); assertSourceLine(snapshot.sources, file, line);
    const reply = await requestBuild(this.invoke, 'pdf_from_source', { buildId, file, line });
    this.assertMappingCurrent(snapshot); return acceptSourceRectangle(reply);
  }
  async inverse(buildId: string, page: number, x: number, y: number): Promise<SourceLocation> {
    const snapshot = await this.flush();
    const reply = await requestBuild(this.invoke, 'source_from_pdf', { buildId, page, x, y });
    this.assertMappingCurrent(snapshot); return acceptSourceLocation(reply, snapshot.sources);
  }
  async resolve(buildId: string, page: number, points: SelectionPoint[]): Promise<ResolvedSelection> {
    const snapshot = await this.flush();
    const reply = await requestBuild(this.invoke, 'resolve_text_selection', { projectId: this.projectId, revision: snapshot.revision, buildId, page, points });
    this.assertMappingCurrent(snapshot); return acceptResolvedSelection(reply, { snapshot, buildId, page });
  }
  commit(token: string, buildId: string): Promise<Snapshot> { return this.enqueue('document_commit', { token, buildId }, false).then(snapshot => { this.candidates.delete(token); return snapshot; }); }
  async discard(token: string): Promise<void> {
    try { await invokeUnit(this.invoke, 'document_discard', { projectId: this.projectId, token }); }
    finally { this.candidates.delete(token); }
  }
}
