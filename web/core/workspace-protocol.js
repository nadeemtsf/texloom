// Generated from frontend/protocol/workspace-protocol.ts. Run npm run protocol:emit; do not edit.
import { assertSnapshot, preparedReply } from './document-protocol.js';
export class WorkspaceProtocolError extends Error {
    constructor(message) { super(message); this.name = 'WorkspaceProtocolError'; }
}
function require(ok, message = 'Invalid project acknowledgement') {
    if (!ok)
        throw new WorkspaceProtocolError(message);
}
function record(v, keys) {
    require(v !== null && typeof v === 'object' && !Array.isArray(v));
    if (keys)
        require(Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v, k)), 'Invalid project reply fields');
}
const text = (v) => typeof v === 'string';
const nonempty = (v) => text(v) && v.length > 0 && !v.includes('\0');
const integer = (v, min = 0) => typeof v === 'number' && Number.isSafeInteger(v) && v >= min;
const nullable = (v) => v === null || text(v);
const hash = (v) => text(v) && /^[0-9a-f]{64}$/u.test(v);
const engine = (v) => text(v) && ['auto', 'pdflatex', 'xelatex', 'lualatex'].includes(v);
function strings(v) { require(Array.isArray(v) && v.every(text)); }
function buffers(v) {
    record(v);
    require(Object.keys(v).length > 0 && Object.keys(v).length <= 600 && Object.entries(v).every(([k, s]) => nonempty(k) && text(s)));
}
function saves(v) {
    require(Array.isArray(v));
    const ids = new Set();
    for (const item of v) {
        record(item, ['id', 'state', 'files']);
        require(nonempty(item.id) && !ids.has(item.id) && text(item.state) && ['preparing', 'prepared', 'committing', 'complete', 'rolled_back'].includes(item.state));
        ids.add(item.id);
        strings(item.files);
        require(new Set(item.files).size === item.files.length);
    }
}
function project(v) {
    record(v, ['id', 'name', 'main', 'sources', 'recoveries', 'recovery_error', 'restored', 'draft_warning', 'source_only']);
    buffers(v.sources);
    saves(v.recoveries);
    require(nonempty(v.id) && nonempty(v.name) && text(v.main) && Object.hasOwn(v.sources, v.main) &&
        nullable(v.recovery_error) && nullable(v.draft_warning) && typeof v.restored === 'boolean' && typeof v.source_only === 'boolean');
}
function inventory(v) {
    record(v, ['drafts', 'errors']);
    strings(v.errors);
    require(Array.isArray(v.drafts) && v.drafts.length <= 32);
    const ids = new Set();
    for (const d of v.drafts) {
        record(d, ['id', 'project', 'main', 'revision', 'source_bytes', 'files', 'warning']);
        require(text(d.id) && /^[0-9a-f]{32}$/u.test(d.id) && !ids.has(d.id) && nonempty(d.project) && nonempty(d.main) &&
            integer(d.revision) && integer(d.source_bytes) && nullable(d.warning));
        ids.add(d.id);
        require(integer(d.files, 1) && d.files <= 600);
    }
}
function controls(v, source) {
    require(text(source) && Array.isArray(v) && v.length <= 2000);
    const lines = source.split('\n').length, ids = new Set();
    for (const c of v) {
        record(c, ['id', 'kind', 'label', 'line', 'parameters', 'reason']);
        require(integer(c.id) && !ids.has(c.id) && nonempty(c.kind) && nonempty(c.label) && integer(c.line, 1) && c.line <= lines &&
            nullable(c.reason) && Array.isArray(c.parameters) && c.parameters.length <= 65);
        ids.add(c.id);
        const keys = new Set();
        for (const p of c.parameters) {
            record(p, ['key', 'label', 'value', 'reason']);
            require(nonempty(p.key) && !keys.has(p.key) && nonempty(p.label) && text(p.value) && nullable(p.reason));
            keys.add(p.key);
        }
    }
}
function model(v) {
    record(v, ['runs', 'protected', 'spacing']);
    strings(v.protected);
    require(Array.isArray(v.runs) && Array.isArray(v.spacing));
    const ids = new Set();
    for (const r of v.runs) {
        record(r, ['id', 'text', 'styles']);
        require(integer(r.id) && !ids.has(r.id) && text(r.text));
        ids.add(r.id);
        strings(r.styles);
    }
    ids.clear();
    for (const s of v.spacing) {
        record(s, ['id', 'value', 'unit', 'source', 'reason']);
        require(integer(s.id) && !ids.has(s.id) && (s.value === null || typeof s.value === 'number' && Number.isFinite(s.value)) &&
            text(s.unit) && text(s.source) && nullable(s.reason));
        ids.add(s.id);
    }
}
const commands = new Set([
    'open_project', 'open_recent', 'recent_projects', 'create_project', 'import_project_archive', 'export_project_archive', 'import_image',
    'review_encoding', 'preview_encoding', 'convert_encoding', 'copy_document', 'project_recoveries', 'recover_document',
    'discard_document_recovery', 'typing_drafts', 'open_typing_draft', 'discard_typing_draft', 'close_document', 'mark_clean_exit',
    'document_snapshot', 'document_prepare', 'describe_adapters', 'plan_adapter', 'transform', 'plan_text_object', 'describe_text_edit', 'preview_text_edit',
]);
export function acceptWorkspaceReply(name, args, value) {
    const a = args;
    if (['open_project', 'open_recent', 'open_typing_draft', 'recover_document'].includes(name))
        project(value);
    else if (name === 'document_snapshot')
        assertSnapshot(value);
    else if (name === 'document_prepare') {
        require(integer(a.revision));
        preparedReply(value, a.revision);
    }
    else if (name === 'typing_drafts' || name === 'discard_typing_draft')
        inventory(value);
    else if (name === 'project_recoveries' || name === 'discard_document_recovery')
        saves(value);
    else if (name === 'mark_clean_exit' || name === 'export_project_archive')
        require(value === null, 'Invalid project acknowledgement: expected null');
    else if (name === 'close_document') {
        record(value, ['warning']);
        require(nullable(value.warning));
    }
    else if (name === 'preview_encoding' || name === 'copy_document' || name === 'transform' || name === 'preview_text_edit')
        require(text(value));
    else if (name === 'recent_projects') {
        require(Array.isArray(value) && value.length <= 20);
        const paths = new Set();
        for (const r of value) {
            record(r, ['path', 'name', 'main', 'engine', 'opened']);
            require(nonempty(r.path) && !paths.has(r.path) && nonempty(r.name) && nonempty(r.main) && engine(r.engine) && integer(r.opened));
            paths.add(r.path);
        }
    }
    else if (name === 'import_project_archive' || name === 'convert_encoding' || name === 'create_project') {
        record(value, ['path']);
        require(nonempty(value.path));
    }
    else if (name === 'import_image') {
        record(value, ['asset', 'sha256']);
        require(text(value.asset) && /^images\/image-[0-9a-f]{32}\.(png|jpe?g|pdf)$/u.test(value.asset) && hash(value.sha256));
    }
    else if (name === 'review_encoding') {
        record(value, ['root', 'fingerprint', 'files']);
        require(nonempty(value.root) && hash(value.fingerprint) && Array.isArray(value.files));
        const files = new Set();
        for (const f of value.files) {
            record(f, ['file', 'detected', 'bytes', 'sha256']);
            require(nonempty(f.file) && !files.has(f.file) && nullable(f.detected) && integer(f.bytes) && hash(f.sha256));
            files.add(f.file);
        }
    }
    else if (name === 'describe_adapters')
        controls(value, a.source);
    else if (name === 'describe_text_edit')
        model(value);
    else if (name === 'plan_adapter' || name === 'plan_text_object') {
        record(value, name === 'plan_adapter' ? ['sources', 'file'] : ['sources', 'object_id', 'file', 'operation']);
        record(a.request);
        buffers(a.request.sources);
        buffers(value.sources);
        require(value.file === a.request.file && Object.hasOwn(value.sources, String(value.file)), 'Source plan changed its target');
        const allowed = new Set([String(a.request.file), String(a.request.main), 'texloom-objects.sty', 'texloom-blocks.sty']);
        for (const [file, source] of Object.entries(a.request.sources))
            require(Object.hasOwn(value.sources, file) &&
                (allowed.has(file) || value.sources[file] === source), 'Source plan changed unrelated buffers');
        for (const file of Object.keys(value.sources))
            require(Object.hasOwn(a.request.sources, file) || allowed.has(file), 'Source plan added an unrelated file');
        if (name === 'plan_text_object')
            require(nonempty(value.object_id) && nonempty(value.operation));
    }
    else
        throw new WorkspaceProtocolError('Missing project protocol validator');
    return value;
}
export async function invokeWorkspace(invoke, name, args) {
    return acceptWorkspaceReply(name, args, await invoke(name, args));
}
/** Existing document/build queues keep their own generated validators. */
export function createWorkspaceInvoke(native) {
    return async (name, args = {}) => {
        const value = await native(name, args);
        return commands.has(name) ? acceptWorkspaceReply(name, args, value) : value;
    };
}
