import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {acceptCompilation,decodeCompilation,acceptSourceLocation,acceptSourceRectangle,acceptResolvedSelection} from '../web/core/build-protocol.js';
import {DocumentClient} from '../web/core/document-client.js';
const fixture=JSON.parse(await readFile(new URL('./fixtures/document-ipc.json',import.meta.url),'utf8'));
const context={revision:fixture.snapshot.revision,sources:fixture.snapshot.sources,engine:fixture.snapshot.engine};
const clone=value=>structuredClone(value),tick=()=>new Promise(setImmediate);
function envelope(reply,pdf=reply.pdf) {
 const metadata=new TextEncoder().encode(JSON.stringify({...reply,pdf:[]}));
 const bytes=new Uint8Array(8+metadata.length+pdf.length);
 bytes.set([84,76,80,49]);new DataView(bytes.buffer).setUint32(4,metadata.length,true);
 bytes.set(metadata,8);bytes.set(pdf,8+metadata.length);return bytes.buffer;
}

test('binary compile replies preserve metadata and all bytes through the actual client boundary',async()=>{
 const native=acceptCompilation(decodeCompilation(Uint8Array.from(fixture.compile_binary_reply).buffer),context);
 assert.deepEqual({...native,pdf:Array.from(native.pdf)},fixture.compile_binary_logical);
 const fallback=acceptCompilation(decodeCompilation(fixture.compile_binary_reply),context);
 assert.deepEqual({...fallback,pdf:Array.from(fallback.pdf)},fixture.compile_binary_logical);
 const original=clone(fixture.compile_success);original.pdf.push(...Array.from({length:256},(_,i)=>i));
 original.log='Unicode: العربية';
 const wire=envelope(original),decoded=acceptCompilation(decodeCompilation(wire),context);
 assert.ok(decoded.pdf instanceof Uint8Array);
 assert.equal(decoded.pdf.buffer,wire,'decoding views the original payload instead of copying it');
 assert.deepEqual({...decoded,pdf:Array.from(decoded.pdf)},original);
 const client=new DocumentClient(async name=>{
  assert.equal(name,'compile_document');return wire;
 },'project',clone(fixture.snapshot));
 const result=await client.compile();
 assert.deepEqual(Array.from(result.pdf),original.pdf);
 assert.equal(result.job_id,42);
 const failed=acceptCompilation(decodeCompilation(envelope(fixture.compile_failure)),context);
 assert.equal(failed.ok,false);assert.equal(failed.pdf.length,0);
});

test('binary envelope refuses corruption and still enforces compilation identity and eligibility',()=>{
 const valid=envelope(fixture.compile_success);
 for(const mutate of [
  b=>b.slice(0,7),
  b=>{new Uint8Array(b)[3]=50;return b;},
  b=>{new DataView(b).setUint32(4,0,true);return b;},
  b=>{new DataView(b).setUint32(4,b.byteLength,true);return b;},
  b=>{new Uint8Array(b)[8]=255;return b;},
  b=>{new Uint8Array(b)[8]=91;return b;},
 ])assert.throws(()=>decodeCompilation(mutate(valid.slice(0))),{name:'BuildProtocolError'});
 for(const mutate of [r=>r.revision++,r=>r.authorized=true,r=>r.verification_eligible=true]){
  const reply=clone(fixture.compile_failure);mutate(reply);
  assert.throws(()=>acceptCompilation(decodeCompilation(envelope(reply)),context),{name:'BuildProtocolError'});
 }
 const metadata=new TextEncoder().encode(JSON.stringify(fixture.compile_success));
 const embedded=new Uint8Array(8+metadata.length);embedded.set([84,76,80,49]);
 new DataView(embedded.buffer).setUint32(4,metadata.length,true);embedded.set(metadata,8);
 assert.throws(()=>decodeCompilation(embedded.buffer),/metadata contains PDF/);
 const oversized=new Uint8Array(8+2+60*1024*1024+1);oversized.set([84,76,80,49]);
 new DataView(oversized.buffer).setUint32(4,2,true);oversized.set([123,125],8);
 assert.throws(()=>decodeCompilation(oversized.buffer),/PDF bytes/);
 for(const byte of [-1,256,1.5,NaN,undefined])assert.throws(()=>decodeCompilation([84,76,80,49,byte]),/envelope byte/);
});

test('a binary reply arriving after cancellation cannot become the current compilation',async()=>{
 let finish;
 const client=new DocumentClient(async name=>name==='compile_document'?new Promise(resolve=>finish=resolve):null,
  'project',clone(fixture.snapshot));
 const compiling=client.compile();
 await client.cancel();finish(envelope(fixture.compile_success));
 await assert.rejects(compiling,{name:'CompilationCancelled'});
 assert.equal(client.snapshot.revision,fixture.snapshot.revision);assert.equal(client.error,null);
});

test('actual Rust Serde compiler and mapping fixtures cross the strict runtime boundary',()=>{
 assert.equal(fixture.compile_request.projectId,'project');assert.equal(fixture.compile_request.purpose,'export');
 assert.equal(acceptCompilation(fixture.compile_success,context),fixture.compile_success);
 assert.equal(acceptCompilation(fixture.compile_failure,context),fixture.compile_failure);
 assert.deepEqual(acceptSourceLocation(fixture.source_location,context.sources),{file:'main.tex',line:2});
 assert.equal(acceptSourceRectangle(fixture.source_rectangle).y,-3);
 const nullable=clone(fixture.compile_success);nullable.objects[0].plain_text=null;
 assert.equal(acceptCompilation(nullable,context),nullable,'unavailable plain previews remain readable');
 assert.equal(fixture.compile_success.job_id,42,'native scheduler ID is not the browser request counter');
 assert.equal(acceptResolvedSelection(fixture.resolved_selection,{snapshot:context,buildId:fixture.compile_success.build_id,page:1}),fixture.resolved_selection);
 const bullet={...fixture.resolved_selection,kind:'bullet_item'};
 assert.equal(acceptResolvedSelection(bullet,{snapshot:context,buildId:fixture.compile_success.build_id,page:1}),bullet);
});

test('selection mappings must match the exact captured source bytes and UTF-16 fragment before any probe',()=>{
 const selectionContext={snapshot:context,buildId:fixture.compile_success.build_id,page:1};
 for(const mutate of [s=>s.build_id='other',s=>s.revision++,s=>s.expected_source='A\n😀\n',
  s=>s.content='Wrong target',s=>s.end=3,s=>s.file='outside.tex',s=>s.first_line=0,
  s=>s.kind='arbitrary',s=>s.rectangles=[{...fixture.source_rectangle,page:2}]]){
  const reply=clone(fixture.resolved_selection);mutate(reply);
  assert.throws(()=>acceptResolvedSelection(reply,selectionContext),{name:'BuildProtocolError'});
 }
});

for(const [name,mutate] of [
 ['missing success flag',r=>delete r.ok],
 ['unknown fields',r=>r.authorized=true],
 ['wrong revision',r=>r.revision++],
 ['unsafe job identity',r=>r.job_id=Number.MAX_SAFE_INTEGER+1],
 ['empty build identity',r=>r.build_id=''],
 ['unknown engine',r=>r.engine='test'],
 ['nonfinite timing',r=>r.seconds=Infinity],
 ['unknown recipe pass',r=>r.passes=7],
 ['fractional PDF byte',r=>r.pdf.push(1.25)],
 ['overflow PDF byte',r=>r.pdf.push(256)],
 ['sparse PDF bytes',r=>r.pdf.length++],
 ['missing PDF header',r=>r.pdf=[]],
 ['failed compiler with output',r=>{r.ok=false;r.verification_eligible=false}],
 ['eligible compiler with errors',r=>r.diagnostics=clone(fixture.compile_failure.diagnostics)],
 ['unknown diagnostic severity',r=>r.diagnostics=[{code:'x',severity:'fatal',message:'Error',file:null,line:null}]],
 ['missing diagnostic location',r=>r.diagnostics=[{code:'x',severity:'error',message:'Error'}]],
 ['unknown object source',r=>r.objects[0].file='missing.tex'],
 ['offset splitting surrogate pair',r=>r.objects[0].end=3],
 ['offset beyond source',r=>r.objects[0].end=999],
 ['duplicate text identity',r=>r.objects.push(clone(r.objects[0]))],
 ['nonfinite object displacement',r=>r.objects[0].dy=NaN],
])test('compiler protocol refuses '+name,()=>{
 const reply=clone(fixture.compile_success);mutate(reply);
 assert.throws(()=>acceptCompilation(reply,context),{name:'BuildProtocolError'});
});

test('explicit engine selection is bound to the reply and readable diagnostics do not grant verification eligibility',()=>{
 assert.throws(()=>acceptCompilation(fixture.compile_success,{...context,engine:'xelatex'}),/compiler metadata/);
 const readable={...fixture.compile_success,verification_eligible:false,diagnostics:fixture.compile_failure.diagnostics};
 assert.equal(acceptCompilation(readable,context),readable);
 const failed={...fixture.compile_failure,verification_eligible:true};
 assert.throws(()=>acceptCompilation(failed,context),/failed compilation/);
});

test('mapping refuses guessed rectangles and sources outside the captured editor snapshot',()=>{
 for(const rectangle of [{page:1,x:0,y:0,width:0,height:1},{...fixture.source_rectangle,x:NaN},
  {...fixture.source_rectangle,page:0},{...fixture.source_rectangle,extra:true}])assert.throws(()=>acceptSourceRectangle(rectangle));
 for(const location of [{file:'../outside.tex',line:1},{file:'main.tex',line:999},{file:'main.tex',line:0}])assert.throws(()=>acceptSourceLocation(location,context.sources));
});

for(const binary of [false,true])test('prepared compiler replies are checked against candidate buffers and cannot be reused after discard (binary='+binary+')',async()=>{
 const calls=[];const client=new DocumentClient(async(name,args)=>{
  calls.push([name,args]);
  if(name==='document_prepare')return {token:'prepared',revision:8};
  if(name==='document_discard')return null;
  const result={...fixture.compile_success,revision:8,objects:[{...fixture.compile_success.objects[0],end:10}]};
  return binary?envelope(result):result;
 },'project',clone(fixture.snapshot));
 const reply=await client.prepare({'main.tex':'Long candidate source'},'baseline');
 const result=await client.compile(reply.token);assert.equal(result.revision,8);assert.equal(client.snapshot.revision,7);
 assert.equal(calls[1][1].revision,7,'the command uses the canonical base while the result uses the candidate revision');
 await client.discard(reply.token);await assert.rejects(client.compile(reply.token),/no longer current/);
 assert.equal(calls.length,3);
});

test('ordinary compiler failure and malformed build replies leave the source queue usable',async()=>{
 let malformed=true;const client=new DocumentClient(async(name,args)=>{
  if(name==='compile_document')return malformed?{ok:true}:fixture.compile_failure;
  return {revision:args.revision,recovery:fixture.snapshot.recovery};
 },'project',clone(fixture.snapshot));
 await assert.rejects(client.compile(),{name:'BuildProtocolError'});assert.equal(client.error,null);
 malformed=false;assert.equal((await client.compile()).ok,false);await client.save();
 assert.equal(client.snapshot.sources['main.tex'],fixture.snapshot.sources['main.tex']);
});

test('unit acknowledgements cannot forge successful native verification, pin, cancellation, export or discard',async()=>{
 const commands=[],client=new DocumentClient(async(name)=>{commands.push(name);return {ok:true}},'project',clone(fixture.snapshot));
 for(const run of [()=>client.verify('build'),()=>client.pin('build'),()=>client.cancel(),()=>client.exportBuild('build','copy.pdf'),()=>client.discard('token')])
  await assert.rejects(run(),{name:'BuildProtocolError'});
 assert.deepEqual(commands,['verify_build','pin_displayed_build','cancel_compilation','export_document_build','document_discard']);
 assert.equal(client.error,null);
});

test('navigation waits for source acknowledgements and rejects a reply after newer typing',async()=>{
 const initial=clone(fixture.snapshot);let finish,calls=0;
 const client=new DocumentClient(async(name,args)=>{
  if(name==='pdf_from_source'){calls++;return new Promise(resolve=>finish=resolve)}
  return {...initial,kind:'source_delta',base_revision:7,revision:8,sources:{'main.tex':'New source'},selection:null,
   recovery:{draft_id:null,protected_revision:null,error:null}};
 },'project',initial);
 const navigating=client.forward('build','main.tex',1);await tick();assert.equal(calls,1);
 await client.edit('main.tex',[{offset:0,length:1,text:'B'}]);finish(fixture.source_rectangle);
 await assert.rejects(navigating,/source changed/);
 const invalid=client.forward('build','unknown.tex',1);await assert.rejects(invalid);assert.equal(calls,1);
});
