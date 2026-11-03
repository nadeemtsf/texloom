import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {DocumentClient} from '../web/core/document-client.js';
import {DocumentProtocolError, assertSnapshot} from '../web/core/document-protocol.js';
const wire = JSON.parse(await readFile(new URL('./fixtures/document-ipc.json', import.meta.url), 'utf8'));
const copy = value => structuredClone(value);

test('actual Rust Serde records pass the strict document client without rewriting source bytes or selection indices', async () => {
  const initial = copy(wire.snapshot), sent = [];
  const client = new DocumentClient(async (name, args) => {
    sent.push({name, args});
    return copy({document_edit: wire.source_delta, document_selection: wire.selection_reply,
      retry_typing_recovery: wire.recovery_reply, document_prepare: wire.prepared_reply}[name]);
  }, 'project', initial);
  const text = initial.sources['main.tex'], style = initial.sources['style.sty'];
  await client.edit(wire.edit_request.file, wire.edit_request.changes, wire.edit_request.group);
  assert.deepEqual(sent[0].args, wire.edit_request);
  assert.equal(initial.sources['main.tex'], text);
  assert.equal(client.snapshot.sources['style.sty'], style);
  const {file, anchor, head, secondary} = wire.selection_reply.selection;
  await client.selection(file, anchor, head, secondary); await client.retryRecovery();
  assert.deepEqual(client.snapshot.selection, wire.selection_reply.selection);
  assert.deepEqual(client.snapshot.sources['main.tex'], wire.source_delta.sources['main.tex']);
  assert.deepEqual(await client.prepare(client.snapshot.sources, 'baseline'), wire.prepared_reply);
});

for (const [reason, change] of [
  ['missing selection', s => {delete s.selection}],
  ['missing recovery', s => {delete s.recovery}],
  ['unknown field', s => {s.unexpected = true}],
  ['invalid source type', s => {s.sources['style.sty'] = 42}],
  ['array sources', s => {s.sources = []}],
  ['missing main file', s => {s.main = 'missing.tex'}],
  ['invalid engine', s => {s.engine = 'custom-engine'}],
  ['nonboolean history', s => {s.can_redo = 'yes'}],
  ['unsafe revision', s => {s.revision = Number.MAX_SAFE_INTEGER + 1}],
  ['split surrogate', s => {s.selection.head = 3}],
  ['outside normalized text', s => {s.selection.head = 6}],
  ['invalid secondary cursor', s => {s.selection.secondary[0].head = 3}],
  ['missing secondary fields', s => {delete s.selection.secondary[0].anchor}],
  ['future recovery protection', s => {s.recovery.protected_revision = 8}],
  ['protection without a draft', s => {s.recovery.draft_id = null}],
]) test(`initial and full snapshots refuse ${reason} before replacing an accepted replica`, async () => {
  const bad = copy(wire.snapshot); change(bad);
  assert.throws(() => assertSnapshot(bad), DocumentProtocolError);
  assert.throws(() => new DocumentClient(async () => null, 'p', bad), DocumentProtocolError);
  const initial = copy(wire.snapshot), client = new DocumentClient(async () => bad, 'p', initial);
  await assert.rejects(client.history(false), DocumentProtocolError);
  assert.equal(client.snapshot, initial);
});

test('malformed visual commit acknowledgements freeze dependent source edits while retaining the accepted source', async () => {
  let calls = 0, failures = 0;
  const initial = copy(wire.snapshot);
  const client = new DocumentClient(async () => {calls++; return {...copy(wire.snapshot), sources: {'main.tex': 'Accepted natively?'}}}, 'p', initial, {failed: () => failures++});
  const results = await Promise.allSettled([client.commit('token', 'build'), client.edit('main.tex', [], 1)]);
  assert.ok(results.every(r => r.status === 'rejected'));
  assert.equal(calls, 1); assert.equal(failures, 1); assert.equal(client.snapshot, initial);
  assert.ok(client.error instanceof DocumentProtocolError);
});

for (const [name, reply] of [
  ['save_document', {...wire.recovery_reply, sources: {'main.tex': 'Unexpected'}}],
  ['retry_typing_recovery', {...wire.recovery_reply, revision: 9}],
  ['document_selection', {...wire.selection_reply, selection: {...wire.selection_reply.selection, head: 0}}],
]) test(`${name} cannot smuggle source/revision/cursor changes into metadata`, async () => {
  const initial = {...copy(wire.snapshot), revision: 8};
  const client = new DocumentClient(async () => copy(reply), 'p', initial);
  const operation = name === 'save_document' ? client.save() : name === 'retry_typing_recovery' ? client.retryRecovery() : client.selection('main.tex', 0, 1, [{anchor: 4, head: 4}]);
  await assert.rejects(operation, DocumentProtocolError);
  assert.equal(client.snapshot, initial); assert.ok(client.error instanceof DocumentProtocolError);
});

test('a no-op full snapshot or profile acknowledgement cannot change unrelated source data', async () => {
  for (const method of ['history', 'profile']) {
    const initial = copy(wire.snapshot), bad = {...copy(wire.snapshot), sources: {...wire.snapshot.sources, 'style.sty': 'Changed'}};
    const client = new DocumentClient(async () => bad, 'p', initial);
    await assert.rejects(method === 'history' ? client.history(false) : client.profile('main.tex', 'auto'), DocumentProtocolError);
    assert.equal(client.snapshot, initial);
  }
});

test('ordinary cursor acknowledgements avoid source publication but a new protection error still publishes', async () => {
  let changed = 0, fail = false;
  const initial = {...copy(wire.snapshot), revision: 8};
  const client = new DocumentClient(async () => ({...copy(wire.selection_reply), recovery: fail ? {...wire.recovery_reply.recovery, error: 'Disk full'} : initial.recovery}), 'p', initial, {changed: () => changed++});
  await client.selection('main.tex', 0, 1, [{anchor: 4, head: 4}]);
  assert.equal(changed, 0); assert.equal(client.snapshot.sources, initial.sources);
  fail = true; await client.selection('main.tex', 0, 1, [{anchor: 4, head: 4}]);
  assert.equal(changed, 1); assert.equal(client.snapshot.recovery.error, 'Disk full');
});

test('malformed prepared candidates cannot acquire a usable token and do not poison later typing', async () => {
  const client = new DocumentClient(async name => name === 'document_prepare' ? {token: 42, revision: 8} : copy(wire.source_delta), 'p', copy(wire.snapshot));
  await assert.rejects(client.prepare(client.snapshot.sources, 'baseline'), DocumentProtocolError);
  assert.equal(client.error, null);
  await client.edit('main.tex', wire.edit_request.changes, 1);
  assert.equal(client.snapshot.revision, 8);
});

test('queue-owned identity cannot be replaced by caller properties', async () => {
  let sent;
  const initial = copy(wire.snapshot);
  const client = new DocumentClient(async (_name, args) => {sent = args; return initial}, 'owned', initial);
  // JavaScript can call this TypeScript-private method; request identity must
  // still be constructed safely at runtime. The type contract forbids it.
  await client.enqueue('document_history', {redo: false, projectId: 'foreign', revision: 900});
  assert.equal(sent.projectId, 'owned'); assert.equal(sent.revision, initial.revision);
});
