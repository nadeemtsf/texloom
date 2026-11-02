import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DocumentClient,editorText} from '../web/core/document-client.js';
const snapshot=(revision=0)=>({revision,sources:{'main.tex':'A'},main:'main.tex',engine:'auto',can_undo:revision>0,can_redo:false});
const tick=()=>new Promise(setImmediate);

test('visual preparation sends a typed operation while private probes carry no commit authority',async()=>{
 const calls=[];const client=new DocumentClient(async(name,args)=>{calls.push({name,...args});return {token:'probe',revision:1}},'p',snapshot());
 const sources={'main.tex':'B'},operation={kind:'text',request:{action:'move',id:'sig'}};
 await client.prepare(sources,'baseline',operation);await client.prepare(sources,'baseline');
 assert.deepEqual(calls[0].operation,operation);assert.equal(calls[0].baselineBuildId,'baseline');
 assert.equal(calls[0].revision,0);assert.equal(calls[1].operation,null);assert.equal(client.snapshot.revision,0);
});

test('cancel fences the request ID and ignores a compiler that returns success afterward',async()=>{
  const calls=[];let finish;
  const client=new DocumentClient(async(name,args)=>{
    calls.push({name,...args});
    if(name==='compile_document')return new Promise(r=>finish=r);
  },'p',snapshot());
  const compiling=client.compile('candidate');await tick();
  await client.cancel();finish({ok:true,revision:0});
  await assert.rejects(compiling,{name:'CompilationCancelled'});
  assert.equal(calls[0].requestId,1);assert.equal(calls[1].requestId,1);assert.equal(calls[1].token,'candidate');
});

test('source events are ordered against acknowledged revisions and only publish when drained',async()=>{
  const calls=[],published=[];let finish;
  const client=new DocumentClient(async(name,args)=>{
    calls.push({name,...args});if(calls.length===1)await new Promise(r=>finish=r);
    return snapshot(args.revision+1);
  },'project',snapshot(),{changed:s=>published.push(s.revision)});
  const first=client.edit('main.tex',[{offset:0,length:1,text:'B'}],1);
  const second=client.edit('main.tex',[{offset:1,length:0,text:'C'}],1);
  await tick();assert.equal(calls.length,1);assert.equal(client.pending,2);assert.deepEqual(published,[]);
  finish();await Promise.all([first,second]);
  assert.deepEqual(calls.map(c=>c.revision),[0,1]);assert.deepEqual(published,[2]);assert.equal(client.pending,0);
});

test('flush includes typing queued while an earlier acknowledgement is awaited',async()=>{
  const resolvers=[];
  const client=new DocumentClient(async(_name,args)=>{await new Promise(r=>resolvers.push(r));return snapshot(args.revision+1);},'p',snapshot());
  const first=client.edit('main.tex',[],1);await tick();let flushed=false;
  const flush=client.flush().then(s=>{flushed=true;return s;});
  const second=client.edit('main.tex',[],1);resolvers[0]();await first;await tick();
  assert.equal(flushed,false);resolvers[1]();await second;assert.equal((await flush).revision,2);
});

test('a rejected source edit retains the last acknowledged source and stops dependent deltas',async()=>{
  let calls=0,failures=0,published=0;
  const client=new DocumentClient(async()=>{calls++;throw new Error('Native edit failed');},'p',snapshot(),
    {failed:()=>failures++,changed:()=>published++});
  const jobs=[client.edit('main.tex',[],1),client.edit('main.tex',[],1)];
  const results=await Promise.allSettled(jobs);
  assert.ok(results.every(r=>r.status==='rejected'));assert.equal(calls,1);assert.equal(failures,1);
  assert.equal(published,0);assert.equal(client.snapshot.sources['main.tex'],'A');await assert.rejects(client.flush());
});

test('a refused visual commit does not poison later source typing',async()=>{
  const client=new DocumentClient(async(name,args)=>{
    if(name==='document_commit')throw new Error('Superseded visual candidate');return snapshot(args.revision+1);
  },'p',snapshot());
  await assert.rejects(client.commit('old-token','old-build'));
  assert.equal(client.error,null);await client.edit('main.tex',[],1);assert.equal(client.snapshot.revision,1);
});

test('out-of-order native revisions cannot replace the acknowledged document',async()=>{
  const client=new DocumentClient(async()=>snapshot(0),'p',snapshot(1));
  await assert.rejects(client.edit('main.tex',[],1),/backward/);assert.equal(client.snapshot.revision,1);
});

test('editor projection is a view and leaves canonical BOM and mixed line endings intact',()=>{
  const source='\ufeffA\r\n🦀B\rC\n';assert.equal(editorText(source),'A\n🦀B\nC\n');
  assert.equal(source,'\ufeffA\r\n🦀B\rC\n');
});

test('selection acknowledgements are ordered but do not publish a new source snapshot',async()=>{
  const calls=[];let published=0;
  const client=new DocumentClient(async(name,args)=>{
    calls.push(name);return name==='document_selection'?{revision:args.revision,selection:{file:args.file,anchor:args.anchor,head:args.head}}:snapshot(args.revision+1);
  },'p',snapshot(),{changed:()=>published++});
  await client.selection('main.tex',1,1);assert.equal(published,0);assert.equal(client.snapshot.sources['main.tex'],'A');
  const source=client.edit('main.tex',[],1),selection=client.selection('main.tex',0,1);
  await Promise.all([source,selection]);assert.equal(published,1);assert.equal(client.snapshot.revision,1);
  assert.deepEqual(calls,['document_selection','document_edit','document_selection']);
});
