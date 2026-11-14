import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DocumentClient,SourceIntentResolutionRequired} from '../web/core/document-client.js';
import {sourceTargetsReply,sourceDraftReply} from '../web/core/document-protocol.js';
import {readFileSync} from 'node:fs';

const op='a'.repeat(32), handle='b'.repeat(32);
const initial=(revision=0,text='Repeated.\n\nRepeated.')=>({revision,sources:{'main.tex':text},main:'main.tex',engine:'auto',can_undo:revision>0,can_redo:false,selection:null,recovery:{draft_id:null,protected_revision:null,error:null}});
const intent={kind:'delete_paragraph_content',target:handle};
const applied=(snapshot=initial(1,'Repeated.\n\n'),from=0,changed=true)=>({operation_id:op,outcome:{status:'source_draft_applied',from_revision:from,revision:from+Number(changed),changed,render_verification:'unresolved'},snapshot});
const tick=()=>new Promise(setImmediate);

test('actual Rust Serde target and source-draft fixtures pass the strict runtime without creating rendered authority',()=>{
  const wire=JSON.parse(readFileSync(new URL('./fixtures/document-ipc.json',import.meta.url),'utf8'));
  assert.deepEqual(sourceTargetsReply(wire.source_targets_reply,wire.snapshot,'main.tex'),wire.source_targets_reply);
  const reply=sourceDraftReply(wire.source_draft_reply,'d'.repeat(32),wire.snapshot,7);
  assert.equal(reply.outcome.render_verification,'unresolved');assert.equal(reply.snapshot.revision,8);
});

test('source target handles retain particular repeated occurrence and reject guessed or malformed coordinates',()=>{
  const snapshot=initial(0,'\ufeffRepeated.\r\n\r\nRepeated.');
  const reply={revision:0,file:'main.tex',targets:[{handle,text:'Repeated.',start:0,end:9},{handle:'c'.repeat(32),text:'Repeated.',start:11,end:20}]};
  assert.deepEqual(sourceTargetsReply(reply,snapshot,'main.tex'),reply);
  for(const mutate of [r=>r.targets[1].start=0,r=>r.targets[1].handle=handle,r=>r.targets[1].text='Other.',r=>r.revision=1,r=>r.targets[0].permission='visual']){
    const bad=structuredClone(reply);mutate(bad);assert.throws(()=>sourceTargetsReply(bad,snapshot,'main.tex'));
  }
});

test('lost acknowledgement freezes queued writes until explicit status lookup, without retry or duplicate history',async()=>{
  const calls=[];let release;
  const client=new DocumentClient(async(name,args)=>{
    calls.push([name,structuredClone(args)]);
    if(name==='document_source_intent'){await new Promise(r=>release=r);throw Error('Lost native acknowledgement')}
    if(name==='document_source_intent_status')return applied();
    if(name==='document_edit')return {...initial(2,'New draft'),kind:'source_delta',base_revision:1};
    throw Error('Unexpected command');
  },'p',initial());
  const operation=client.sourceIntent(0,op,intent);await tick();
  const queued=client.edit('main.tex',[]);const blocked=assert.rejects(queued,SourceIntentResolutionRequired);
  release();await assert.rejects(operation,/Lost native/);await blocked;
  assert.deepEqual(calls.map(c=>c[0]),['document_source_intent']);
  assert.equal(client.snapshot.revision,0);assert.equal(client.error,null);
  await assert.rejects(client.flush(),SourceIntentResolutionRequired);
  const resolved=await client.resolveSourceIntent(op);assert.equal(resolved.outcome.status,'source_draft_applied');
  assert.equal(client.snapshot.revision,1);
  await client.edit('main.tex',[]);assert.equal(client.snapshot.revision,2);
  assert.deepEqual(calls.map(c=>c[0]),['document_source_intent','document_source_intent_status','document_edit']);
  assert.deepEqual(calls[0][1],{request:{projectId:'p',revision:0,operationId:op,intent}});
});

test('cached applied receipt after Undo adopts current snapshot rather than historical source',async()=>{
  const current={...initial(2),can_redo:true};let calls=0;
  const client=new DocumentClient(async()=>{calls++;return applied(current)},'p',current);
  const reply=await client.sourceIntent(0,op,intent);
  assert.equal(reply.outcome.revision,1);assert.equal(client.snapshot.revision,2);
  assert.equal(client.snapshot.sources['main.tex'],current.sources['main.tex']);
  assert.equal(client.snapshot.can_redo,true);assert.equal(calls,1);
});

test('typed no-op and unchanged receipt lookup preserve the browser snapshot and pending visual candidate without publishing',async()=>{
  const before=initial();let publications=0;
  const client=new DocumentClient(async name=>name==='document_prepare'?{token:'candidate',revision:1}:applied(before,0,false),'p',before,{changed:()=>publications++});
  await client.prepare({'main.tex':'Candidate'},'build');
  await client.sourceIntent(0,op,{kind:'replace_literal_paragraph',target:handle,text:'Repeated.'});
  await client.resolveSourceIntent(op);
  assert.equal(client.snapshot,before);assert.equal(publications,0);
  assert.equal(client.candidates.has('candidate'),true);
});

test('uncertain protection or unknown lookup never clears the write freeze or silently retries',async()=>{
  let status='protection_uncertain';const calls=[];
  const uncertain=()=>({operation_id:op,outcome:status==='unknown'?{status}:{status,from_revision:0,error:'Synthetic durable store failure'},snapshot:initial()});
  const client=new DocumentClient(async(name)=>{calls.push(name);return uncertain()},'p',initial());
  assert.equal((await client.sourceIntent(0,op,intent)).outcome.status,'protection_uncertain');
  for(const action of [()=>client.edit('main.tex',[]),()=>client.history(false),()=>client.save(),()=>client.retryRecovery()]) await assert.rejects(action(),SourceIntentResolutionRequired);
  await client.resolveSourceIntent(op);status='unknown';await client.resolveSourceIntent(op);
  await assert.rejects(client.sourceIntent(0,'c'.repeat(32),intent),SourceIntentResolutionRequired);
  await assert.rejects(client.flush(),SourceIntentResolutionRequired);
  assert.deepEqual(calls,['document_source_intent','document_source_intent_status','document_source_intent_status']);
  assert.equal(client.snapshot.revision,0);assert.equal(client.error,null);
});

test('malformed mutation acknowledgements retain old state and require status resolution',async()=>{
  for(const mutate of [r=>r.operation_id='c'.repeat(32),r=>r.outcome.render_verification='passed',r=>r.outcome.revision=2,r=>r.outcome.extra=true,r=>r.snapshot.revision=-1]){
    const bad=applied();mutate(bad);
    const client=new DocumentClient(async name=>name==='document_source_intent'?bad:applied(),'p',initial());
    await assert.rejects(client.sourceIntent(0,op,intent));
    assert.equal(client.snapshot.revision,0);
    await assert.rejects(client.edit('main.tex',[]),SourceIntentResolutionRequired);
    await client.resolveSourceIntent(op);assert.equal((await client.flush()).revision,1);
  }
  assert.throws(()=>sourceDraftReply({...applied(),outcome:{status:'unknown'}},op,initial(),0));
});

test('source target replies lose authority when a source edit completes during their request',async()=>{
  let release;const client=new DocumentClient(async(name)=>{
    if(name==='document_source_targets'){await new Promise(r=>release=r);return {revision:0,file:'main.tex',targets:[{handle,text:'Repeated.',start:0,end:9}]}}
    return {...initial(1,'Updated.'),kind:'source_delta',base_revision:0};
  },'p',initial());
  const targets=client.sourceTargets('main.tex');await tick();
  await client.edit('main.tex',[]);release();
  await assert.rejects(targets,/source changed/i);assert.equal(client.snapshot.revision,1);
});

test('cursor coalescing cannot move a later cursor event across a typed source command',async()=>{
  const calls=[];let release;
  const client=new DocumentClient(async(name,args)=>{
    calls.push([name,structuredClone(args)]);
    if(name==='document_selection')return {revision:args.revision,selection:{file:args.file,anchor:args.anchor,head:args.head,secondary:args.secondary},recovery:initial().recovery};
    await new Promise(r=>release=r);return applied();
  },'p',initial());
  const before=client.selection('main.tex',0,0);
  const command=client.sourceIntent(0,op,intent);
  const after=client.selection('main.tex',1,1);
  await before;await tick();release();await command;await after;
  assert.deepEqual(calls.map(c=>c[0]),['document_selection','document_source_intent','document_selection']);
  assert.equal(calls[0][1].head,0);assert.equal(calls[2][1].revision,1);assert.equal(calls[2][1].head,1);
});
