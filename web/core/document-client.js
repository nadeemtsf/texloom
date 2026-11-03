// Generated from frontend/protocol/document-client.ts. Run npm run protocol:emit; do not edit.
import { acceptReply, assertRevision, assertSnapshot, DocumentProtocolError, preparedReply, } from './document-protocol.js';
export { editorText, sameSources } from './document-protocol.js';
function errorOf(value) { return value instanceof Error ? value : new Error(String(value)); }
export class DocumentClient {
    invoke;
    projectId;
    snapshot;
    pending = 0;
    outstanding = 0;
    error = null;
    unpublished = false;
    tail = Promise.resolve();
    compileSerial = 0;
    activeToken = null;
    changed;
    failed;
    constructor(invoke, projectId, snapshot, options = {}) {
        this.invoke = invoke;
        this.projectId = projectId;
        assertSnapshot(snapshot);
        this.snapshot = snapshot;
        this.changed = options.changed ?? (() => { });
        this.failed = options.failed ?? (() => { });
    }
    enqueue(name, args, fatal = true, mutating = true) {
        this.outstanding++;
        if (mutating)
            this.pending++;
        const job = this.tail.then(async () => {
            if (this.error)
                throw this.error;
            // Caller arguments cannot replace queue-owned project/revision identity.
            const request = { ...args, projectId: this.projectId };
            if (name !== 'document_commit')
                request.revision = this.snapshot.revision;
            const next = await this.invoke(name, request);
            const accepted = acceptReply(name, request, next, this.snapshot);
            this.snapshot = accepted.snapshot;
            if (accepted.sourceChanged)
                this.unpublished = true;
            return this.snapshot;
        }).catch((value) => {
            const error = errorOf(value);
            if ((fatal || error instanceof DocumentProtocolError) && !this.error) {
                this.error = error;
                this.failed(error);
            }
            throw error;
        }).finally(() => {
            this.outstanding--;
            if (mutating)
                this.pending--;
            if (!this.outstanding && !this.error && this.unpublished) {
                this.unpublished = false;
                this.changed(this.snapshot);
            }
        });
        this.tail = job.catch(() => { });
        return job;
    }
    async flush() {
        for (;;) {
            const tail = this.tail;
            await tail;
            if (this.error)
                throw this.error;
            if (tail === this.tail && !this.outstanding)
                return this.snapshot;
        }
    }
    edit(file, changes, group = null) {
        return this.enqueue('document_edit', { file, changes, group });
    }
    profile(main, engine) { return this.enqueue('document_profile', { main, engine }); }
    history(redo) { return this.enqueue('document_history', { redo }); }
    save() { return this.enqueue('save_document', {}, false); }
    retryRecovery() { return this.enqueue('retry_typing_recovery', {}, false); }
    selection(file, anchor, head, secondary = []) {
        return this.enqueue('document_selection', { file, anchor, head, secondary }, true, false);
    }
    async prepare(sources, baselineBuildId, operation = null) {
        await this.flush();
        const base = this.snapshot.revision;
        return preparedReply(await this.invoke('document_prepare', { projectId: this.projectId, revision: base, sources, baselineBuildId, operation }), base);
    }
    async compile(token = null, purpose = 'preview') {
        this.activeToken = token;
        const requestId = ++this.compileSerial;
        assertRevision(requestId);
        try {
            return await this.invoke('compile_document', { projectId: this.projectId, revision: this.snapshot.revision, token, purpose, requestId });
        }
        finally {
            if (requestId !== this.compileSerial) {
                const error = new Error('Compilation cancelled or superseded.');
                error.name = 'CompilationCancelled';
                throw error;
            }
        }
    }
    async cancel() {
        const requestId = this.compileSerial;
        assertRevision(this.compileSerial + 1);
        this.compileSerial++;
        return this.invoke('cancel_compilation', { projectId: this.projectId, requestId, token: this.activeToken });
    }
    async verify(buildId) { return this.invoke('verify_build', { buildId }); }
    async pin(buildId) { return this.invoke('pin_displayed_build', { projectId: this.projectId, buildId }); }
    commit(token, buildId) { return this.enqueue('document_commit', { token, buildId }, false); }
    async discard(token) { return this.invoke('document_discard', { projectId: this.projectId, token }); }
}
