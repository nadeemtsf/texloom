import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {acceptWorkspaceReply,createWorkspaceInvoke,WorkspaceProtocolError} from '../web/core/workspace-protocol.js';
const wire=JSON.parse(await readFile(new URL('./fixtures/document-ipc.json',import.meta.url),'utf8'));
for(const [command,key,args={}] of [
  ['open_project','project'],['open_recent','project'],['recover_document','project'],['open_typing_draft','project'],
  ['recent_projects','recent_projects'],['review_encoding','encoding_review'],['import_project_archive','imported_project'],
  ['convert_encoding','imported_project'],['import_image','imported_image'],['project_recoveries','pending_saves'],
  ['discard_document_recovery','pending_saves'],['typing_drafts','inventory'],['discard_typing_draft','inventory'],
  ['document_snapshot','snapshot'],['document_prepare','prepared_reply',{revision:8}],
  ['describe_adapters','adapter_controls',{source:'\\includegraphics[width=2cm]{image.png}\n\\vspace{2mm}'}],['describe_text_edit','text_model'],
])test('actual Rust Serde '+command+' reply passes the workspace boundary',()=>{
  assert.ok(Object.hasOwn(wire,key),key);assert.deepEqual(acceptWorkspaceReply(command,args,wire[key]),wire[key]);
});
for(const [command,key,change] of [
  ['open_project','project',v=>{v.main='absent.tex'}],['open_project','project',v=>{delete v.restored}],
  ['project_recoveries','pending_saves',v=>{v[0].state='done'}],['typing_drafts','inventory',v=>{v.drafts[0].revision='7'}],
  ['recent_projects','recent_projects',v=>{v[0].engine='shell'}],['import_image','imported_image',v=>{v.asset='../image.png'}],
  ['review_encoding','encoding_review',v=>{v.fingerprint='wrong'}],['describe_text_edit','text_model',v=>{delete v.spacing}],
])test('workspace refuses malformed '+command+' reply before adoption',()=>{
  const value=structuredClone(wire[key]);change(value);assert.throws(()=>acceptWorkspaceReply(command,{},value),WorkspaceProtocolError);
});
test('source plans retain unrelated files, refuse new files and bind their target',()=>{
  const args={request:{sources:{'main.tex':'Main','part.tex':'Old','other.sty':'Keep'},main:'main.tex',file:'part.tex'}};
  const plan={sources:{...args.request.sources,'part.tex':'New'},file:'part.tex'};
  assert.equal(acceptWorkspaceReply('plan_adapter',args,plan),plan);
  for(const bad of [{...plan,file:'main.tex'},{...plan,sources:{...plan.sources,'other.sty':'Changed'}},{...plan,sources:{...plan.sources,'extra.tex':'New'}}])assert.throws(()=>acceptWorkspaceReply('plan_adapter',args,bad),WorkspaceProtocolError);
});
test('unit acknowledgements are exact and native errors propagate without becoming successes',async()=>{
  for(const name of ['mark_clean_exit','export_project_archive'])for(const value of [undefined,{},true,'ok'])assert.throws(()=>acceptWorkspaceReply(name,{},value),WorkspaceProtocolError);
  const failure=new Error('Disk is locked');const invoke=createWorkspaceInvoke(async()=>{throw failure});await assert.rejects(invoke('open_project',{}),e=>e===failure);
  assert.equal(await createWorkspaceInvoke(async()=>null)('mark_clean_exit',{}),null);
});
