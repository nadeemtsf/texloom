import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DocumentClient,editorText,sameSources} from '../web/core/document-client.js';
const snapshot=(revision=0)=>({revision,sources:{'main.tex':'A'},main:'main.tex',engine:'auto',can_undo:revision>0,can_redo:false,selection:null,recovery:{draft_id:null,protected_revision:null,error:null}});
const tick=()=>new Promise(setImmediate);
const delta=(base,file,source)=>({...snapshot(base+1),kind:'source_delta',base_revision:base,sources:{[file]:source},selection:null});

test('Save and protection retries follow acknowledged typing without replacing sources or freezing on save conflicts',async()=>{
 const calls=[],published=[];let conflict=true;
 const protection={draft_id:'a'.repeat(32),protected_revision:1,error:null};
 const client=new DocumentClient(async(name,args)=>{
  calls.push([name,args.revision]);
  if(name==='document_edit')return delta(args.revision,args.file,'B');
  if(name==='save_document'&&conflict){conflict=false;throw Error('External file conflict')}
  return {revision:args.revision,recovery:protection};
 },'p',snapshot(),{changed:s=>published.push(s)});
 const edit=client.edit('main.tex',[],1),save=client.save();await edit;await assert.rejects(save,/conflict/);
 assert.equal(client.error,null);await client.retryRecovery();await client.save();
 assert.deepEqual(calls,[['document_edit',0],['save_document',1],['retry_typing_recovery',1],['save_document',1]]);
 assert.equal(client.snapshot.sources['main.tex'],'B');assert.deepEqual(client.snapshot.recovery,protection);assert.ok(published.length>0);
});

test('invalid future recovery protection cannot replace the acknowledged source state',async()=>{
 const initial=snapshot();const client=new DocumentClient(async()=>({...delta(0,'main.tex','B'),recovery:{draft_id:'a'.repeat(32),protected_revision:2,error:null}}),'p',initial);
 await assert.rejects(client.edit('main.tex',[],1),/recovery protection/);assert.equal(client.snapshot,initial);
});

test('saved-buffer comparison ignores file order but preserves BOM, line endings and the exact file set',()=>{
 const saved={'main.tex':'\ufeffName\r\n','empty.sty':''};
 assert.equal(sameSources(saved,{'empty.sty':'','main.tex':'\ufeffName\r\n'}),true);
 for(const source of ['Name\r\n','\ufeffName\n','\ufeffName\r\n '])assert.equal(sameSources(saved,{...saved,'main.tex':source}),false);
 assert.equal(sameSources(saved,{'main.tex':saved['main.tex'],'different.sty':''}),false);
 assert.equal(sameSources(saved,{...saved,'added.sty':''}),false);
});

test('changed-file acknowledgements preserve other files, old snapshots and queued revision ordering',async()=>{
 const initial={...snapshot(),sources:{'main.tex':'A','chapter.tex':'Chapter','objects.sty':'Style'}};
 const published=[],calls=[];
 const client=new DocumentClient(async(_name,args)=>{calls.push(args);return delta(args.revision,args.file,args.changes[0].text);},'p',initial,{changed:s=>published.push(s)});
 const first=client.edit('main.tex',[{text:'B'}],1),second=client.edit('chapter.tex',[{text:'New chapter'}],1);
 const [one,two]=await Promise.all([first,second]);
 assert.deepEqual(calls.map(c=>c.revision),[0,1]);
 assert.deepEqual(one.sources,{'main.tex':'B','chapter.tex':'Chapter','objects.sty':'Style'});
 assert.deepEqual(two.sources,{'main.tex':'B','chapter.tex':'New chapter','objects.sty':'Style'});
 assert.deepEqual(initial.sources,{'main.tex':'A','chapter.tex':'Chapter','objects.sty':'Style'});
 assert.equal(published.length,1);assert.equal(published[0],two);assert.equal('kind' in two,false);assert.equal('base_revision' in two,false);
});

test('a no-op delta keeps source identity and publishes the authoritative revision',async()=>{
 const initial=snapshot();let published=0;
 const client=new DocumentClient(async()=>({...delta(0,'main.tex','A'),revision:0,sources:{},can_undo:false}),'p',initial,{changed:()=>published++});
 await client.edit('main.tex',[],1);assert.equal(client.snapshot.sources,initial.sources);assert.equal(published,1);assert.equal(client.snapshot.revision,0);
});

test('full history and visual acknowledgements can replace the file set after source deltas',async()=>{
 let revision=0;
 const client=new DocumentClient(async(name,args)=>{revision++;return name==='document_edit'?delta(args.revision,args.file,'B'):
  {...snapshot(revision),sources:{'main.tex':'Restored','new.sty':'New style'}}},'p',snapshot());
 await client.edit('main.tex',[],1);await client.history(false);
 assert.deepEqual(client.snapshot.sources,{'main.tex':'Restored','new.sty':'New style'});
 await client.commit('candidate','build');assert.equal(client.snapshot.revision,3);
});

for(const [reason,alter] of [
 ['wrong base',r=>({...r,base_revision:1})],
 ['skipped revision',r=>({...r,revision:2})],
 ['unknown kind',r=>({...r,kind:'other_delta'})],
 ['different file',r=>({...r,sources:{'chapter.tex':'B'}})],
 ['extra file',r=>({...r,sources:{...r.sources,'extra.sty':'Unexpected'}})],
 ['non-string source',r=>({...r,sources:{'main.tex':42}})],
 ['missing changed source',r=>({...r,sources:{}})],
 ['changed source without revision',r=>({...r,revision:0})],
 ['unchanged source with new revision',r=>({...r,sources:{'main.tex':'A'}})],
 ['profile change',r=>({...r,engine:'xelatex'})],
 ['invalid history flag',r=>({...r,can_undo:1})],
 ['invalid selection',r=>({...r,selection:{file:'main.tex',anchor:-1,head:0,secondary:[]}})],
])test(`invalid delta (${reason}) leaves the last acknowledged sources and stops dependent edits`,async()=>{
 const initial=snapshot();let calls=0,failures=0;
 const client=new DocumentClient(async()=>{calls++;return alter(delta(0,'main.tex','B'));},'p',initial,{failed:()=>failures++});
 const results=await Promise.allSettled([client.edit('main.tex',[],1),client.edit('main.tex',[],1)]);
 assert.ok(results.every(r=>r.status==='rejected'));assert.equal(calls,1);assert.equal(failures,1);assert.equal(client.snapshot,initial);
 await assert.rejects(client.flush());
});

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
    if(name==='document_prepare')return {token:'candidate',revision:1};
    if(name==='compile_document')return new Promise(r=>finish=r);
    return null;
  },'p',snapshot());
  await client.prepare({'main.tex':'B'},'baseline');calls.length=0;
  const compiling=client.compile('candidate');await tick();
  await client.cancel();finish({ok:true,revision:0});
  await assert.rejects(compiling,{name:'CompilationCancelled'});
  assert.equal(calls[0].requestId,1);assert.equal(calls[1].requestId,1);assert.equal(calls[1].token,'candidate');
});

test('source events are ordered against acknowledged revisions and only publish when drained',async()=>{
  const calls=[],published=[];let finish;
  const client=new DocumentClient(async(name,args)=>{
    calls.push({name,...args});if(calls.length===1)await new Promise(r=>finish=r);
    return delta(args.revision,args.file,'Edit '+(args.revision+1));
  },'project',snapshot(),{changed:s=>published.push(s.revision)});
  const first=client.edit('main.tex',[{offset:0,length:1,text:'B'}],1);
  const second=client.edit('main.tex',[{offset:1,length:0,text:'C'}],1);
  await tick();assert.equal(calls.length,1);assert.equal(client.pending,2);assert.deepEqual(published,[]);
  finish();await Promise.all([first,second]);
  assert.deepEqual(calls.map(c=>c.revision),[0,1]);assert.deepEqual(published,[2]);assert.equal(client.pending,0);
});

test('flush includes typing queued while an earlier acknowledgement is awaited',async()=>{
  const resolvers=[];
  const client=new DocumentClient(async(_name,args)=>{await new Promise(r=>resolvers.push(r));return delta(args.revision,args.file,'Edit '+(args.revision+1));},'p',snapshot());
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
    if(name==='document_commit')throw new Error('Superseded visual candidate');return delta(args.revision,args.file,'Edit '+(args.revision+1));
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
    calls.push(name);return name==='document_selection'?{revision:args.revision,selection:{file:args.file,anchor:args.anchor,head:args.head,secondary:args.secondary},recovery:snapshot().recovery}:delta(args.revision,args.file,'B');
  },'p',snapshot(),{changed:()=>published++});
  await client.selection('main.tex',1,1);assert.equal(published,0);assert.equal(client.snapshot.sources['main.tex'],'A');
  const source=client.edit('main.tex',[],1),selection=client.selection('main.tex',0,1);
  await Promise.all([source,selection]);assert.equal(published,1);assert.equal(client.snapshot.revision,1);
  assert.deepEqual(calls,['document_selection','document_edit','document_selection']);
});
