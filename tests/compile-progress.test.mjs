import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createCompileInvoke} from '../web/core/compile-progress.js';
import {PreliminaryPreview} from '../web/core/preliminary-preview.js';
import {acceptUnverifiedPreview,acceptCompilation} from '../web/core/build-protocol.js';
const fixture=JSON.parse(await readFile(new URL('./fixtures/document-ipc.json',import.meta.url),'utf8'));
const request={projectId:'project',revision:7,requestId:1,purpose:'preview',token:null,engine:'auto'};

test('Rust-generated preliminary bytes are bound to their request and cannot become a final compilation',()=>{
 const progress=acceptUnverifiedPreview(Uint8Array.from(fixture.compile_preview_binary).buffer,request);
 assert.deepEqual({...progress,pdf:Array.from(progress.pdf)},fixture.compile_preview_logical);
 assert.throws(()=>acceptCompilation(progress,{revision:7,sources:{},engine:'auto'}),{name:'BuildProtocolError'});
 assert.throws(()=>acceptUnverifiedPreview(fixture.compile_success,request),{name:'BuildProtocolError'});
 for(const mutate of [r=>r.project_id='other',r=>r.revision++,r=>r.request_id++,r=>r.phase='final',r=>r.schema=2,
  r=>r.build_id='forged',r=>r.verification_eligible=true,r=>r.engine='unknown',r=>r.pdf=[],r=>r.seconds=NaN,
  r=>r.diagnostics=[{code:'x',severity:'fatal',message:'x',file:null,line:null}]]){
  const value=structuredClone(fixture.compile_preview_logical);mutate(value);
  assert.throws(()=>acceptUnverifiedPreview(value,request),{name:'BuildProtocolError'});
 }
 assert.throws(()=>acceptUnverifiedPreview(fixture.compile_preview_logical,{...request,engine:'xelatex'}));
});

test('only a canonical preview gets a channel, which closes before any late message can publish',async()=>{
 const events=[];let channel,finish;
 const invoke=createCompileInvoke(async(name,args)=>{
  if(args.onPreview){channel=args.onPreview;return new Promise(resolve=>finish=resolve)}
  return 'ordinary';
 },fn=>({onmessage:fn}),{start:r=>({request:r}),message:(v,t)=>events.push([v,t.request.requestId]),end:()=>events.push('end')});
 for(const args of [{...request,purpose:'export'},{...request,token:'candidate'},{...request,purpose:'visual'}])
  assert.equal(await invoke('compile_document',args),'ordinary');
 assert.equal(await invoke('other',request),'ordinary');assert.deepEqual(events,[]);
 const result=invoke('compile_document',request),late=channel.onmessage;
 channel.onmessage('first');finish('settled');assert.equal(await result,'settled');
 late('late');channel.onmessage('later');assert.deepEqual(events,[['first',1],'end']);
});

test('optional channel, preview and end failures preserve the compiler result or original error',async()=>{
 for(const fail of ['channel','message','end']){
  const invoke=createCompileInvoke(async(_name,args)=>{args.onPreview?.onmessage('data');return 'settled'},
   fn=>{if(fail==='channel')throw Error('optional');return {onmessage:fn}},
   {start:()=>({}),message:()=>{if(fail==='message')throw Error('optional')},end:()=>{if(fail==='end')throw Error('optional')}});
  assert.equal(await invoke('compile_document',request),'settled');
 }
 const error=Error('compiler');const invoke=createCompileInvoke(async()=>{throw error},fn=>({onmessage:fn}),
  {start:()=>({}),message(){},end(){throw Error('optional')}});
 await assert.rejects(invoke('compile_document',request),value=>value===error);
});

function harness(){
 const calls=[];let resolve;
 const controller=new PreliminaryPreview({current:()=>true,render:()=>new Promise(r=>resolve=r),
  show:value=>calls.push(['show',value]),hide:value=>calls.push(['hide',value])});
 const value={dispose:()=>calls.push(['dispose'])};
 return {controller,calls,value,finish:()=>resolve(value)};
}
test('cancelled, superseded and settled in-flight preliminary renders are disposed without publication',async()=>{
 for(const action of ['clear','end','replace']){
  const h=harness(),ticket=h.controller.start(request),offer=h.controller.offer(ticket,{});
  if(action==='replace')h.controller.start(request);else h.controller[action](ticket);
  h.finish();assert.equal(await offer,false);assert.deepEqual(h.calls,[['dispose']]);
 }
});
test('a mounted preview stays independent until cleared and refuses duplicates or old tickets with identical IDs',async()=>{
 const h=harness(),old=h.controller.start(request),ticket=h.controller.start(request);
 h.controller.end(old);h.controller.clear(old);assert.equal(await h.controller.offer(old,{}),false);
 const offer=h.controller.offer(ticket,{});h.finish();assert.equal(await offer,true);
 assert.equal(await h.controller.offer(ticket,{}),false);h.controller.end(ticket);
 assert.equal(h.calls.length,1);h.controller.clear();h.controller.clear();
 assert.deepEqual(h.calls.map(x=>x[0]),['show','hide','dispose']);
});
test('optional presentation and cleanup failures are isolated and dispose the temporary PDF once',async()=>{
 let disposed=0;const controller=new PreliminaryPreview({current:()=>true,
  render:async()=>({dispose(){disposed++;throw Error('cleanup')}}),
  show(){throw Error('presentation')},hide(){throw Error('hide')}});
 assert.equal(await controller.offer(controller.start(request),{}),false);
 assert.equal(disposed,1);assert.equal(controller.active,null);controller.clear();
});
