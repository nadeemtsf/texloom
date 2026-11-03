import {DocumentClient} from '../../frontend/protocol/document-client.js';
import type {DocumentRequests, DocumentReplies} from '../../web/generated/document-ipc.js';
import type {NativeInvoke} from '../../frontend/protocol/document-protocol.js';
import type {BuildRequests, CompileResult} from '../../web/generated/document-ipc.js';
import {requestBuild} from '../../frontend/protocol/build-protocol.js';
declare const invoke: NativeInvoke;
declare const snapshot: unknown;
const client = new DocumentClient(invoke, 'project', snapshot);
const edit: DocumentRequests['document_edit'] = {projectId: 'project', revision: 0, file: 'main.tex',
  changes: [{offset: 0, length: 1, text: 'B'}], group: null};
void client.edit(edit.file, edit.changes, edit.group);
void client.selection('main.tex', 0, 1, [{anchor: 1, head: 1}]);
// @ts-expect-error Editor changes require a UTF-16 length.
void client.edit('main.tex', [{offset: 0, text: 'B'}]);
// @ts-expect-error A typing group is numeric, not a source ID.
void client.edit('main.tex', [], 'group');
// @ts-expect-error Cursor endpoints are numbers.
void client.selection('main.tex', '0', 1);
// @ts-expect-error A visual commit needs both opaque identities.
void client.commit('token');
// @ts-expect-error Callers cannot bypass queue-owned metadata construction.
void client.enqueue('document_history', {revision: 9, redo: false});
// @ts-expect-error Rust serializes revisions as numbers, not bigint.
const revision: DocumentReplies['document_edit']['revision'] = 1n;
void revision;
void client.compile(null, 'export').then((result: CompileResult) => result.pdf);
void client.forward('build', 'main.tex', 1);
// @ts-expect-error Build purposes come from the native enum.
void client.compile(null, 'publsh');
// @ts-expect-error Navigation uses numeric source lines.
void client.forward('build', 'main.tex', '1');
// @ts-expect-error The compiler request must carry its cancellation sequence.
const compileRequest: BuildRequests['compile_document'] = {projectId:'p', revision:0, token:null, purpose:'preview'};
void compileRequest;
// @ts-expect-error Build identities are opaque strings, not revisions.
void client.verify(1);
// @ts-expect-error A forward mapping request requires a source file.
void requestBuild(invoke, 'pdf_from_source', {buildId:'build', line:1});
// @ts-expect-error Native diagnostics require the serialized nullable source fields.
const diagnostic: CompileResult['diagnostics'][number] = {code:'error', severity:'error', message:'Failed'};
void diagnostic;
// @ts-expect-error Selection samples contain numeric PDF coordinates.
void client.resolve('build', 1, [{x:'12', y:24}]);
// @ts-expect-error candidate indexes are numbers
void client.resolve('build', 1, [], '1');

import {invokeWorkspace} from '../../frontend/protocol/workspace-protocol.js';
void invokeWorkspace(invoke, 'recent_projects', {});
void invokeWorkspace(invoke, 'create_project', {template:'article',name:'Example',parent:null});
// @ts-expect-error New projects require an explicit template identifier.
void invokeWorkspace(invoke, 'create_project', {name:'Example',parent:null});
void invokeWorkspace(invoke, 'import_image', {projectId:'project', revision:0, path:'image.png'});
// @ts-expect-error Native recovery acknowledgements use numeric revisions.
void invokeWorkspace(invoke, 'recover_document', {projectId:'project', revision:'0', transactionId:'save', finish:true});
// @ts-expect-error ZIP export is revision-bound.
void invokeWorkspace(invoke, 'export_project_archive', {projectId:'project', path:'out.zip'});
// @ts-expect-error Explicit encoding choices bind the original byte hash.
void invokeWorkspace(invoke, 'convert_encoding', {path:'project', parent:'copies', fingerprint:'hash', choices:[{file:'main.tex',encoding:'windows-1252'}]});
