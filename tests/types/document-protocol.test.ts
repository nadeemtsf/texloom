import {DocumentClient} from '../../frontend/protocol/document-client.js';
import type {DocumentRequests, DocumentReplies} from '../../web/generated/document-ipc.js';
import type {NativeInvoke} from '../../frontend/protocol/document-protocol.js';
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
