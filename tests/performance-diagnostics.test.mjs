import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createPerformanceDiagnostics,installPerformanceDiagnostics} from '../web/core/performance-diagnostics.js';
import {stageCandidate} from '../web/core/transactions.js';

function harness(options={}){
  let time=100,id=0;const frames=new Map(),timers=new Map(),document={visibilityState:'visible'};
  const diagnostics=createPerformanceDiagnostics({enabled:true,now:()=>time,document,
    requestFrame:fn=>{frames.set(++id,fn);return id},cancelFrame:id=>frames.delete(id),
    setTimer:fn=>{timers.set(++id,fn);return id},clearTimer:id=>timers.delete(id),...options});
  return {diagnostics,document,frames,timers,at:n=>{time=n},frame(n){time=n;const [id,fn]=frames.entries().next().value;frames.delete(id);fn()}};
}

test('ordinary builds do not sample clocks or expose a diagnostic API',()=>{
  const host={},diagnostics=installPerformanceDiagnostics({host,enabled:false,now:()=>assert.fail('Disabled diagnostics sampled a clock'),newCollectorId:()=>assert.fail('Disabled diagnostics requested randomness')});
  assert.equal(diagnostics.begin('text-move'),null);assert.equal(host.texloomPerformance,undefined);
  assert.equal(diagnostics.snapshot().records.length,0);
});

test('collector identity and clear epochs preserve lost-row accounting and never reuse IDs',()=>{
  const id='12345678-1234-4123-8123-123456789abc',h=harness({newCollectorId:()=>id});
  h.diagnostics.begin('text-move').finish('published');h.diagnostics.begin('text-edit');
  h.diagnostics.clear();const after=h.diagnostics.snapshot();
  assert.equal(after.schema,2);assert.equal(after.collector_id,id);assert.equal(after.epoch,1);
  assert.deepEqual(after.clear_history,{completed:1,pending:1});assert.equal(after.first_id,3);assert.equal(after.last_id,2);
  h.diagnostics.begin('text-edit').finish('refused');assert.equal(h.diagnostics.snapshot().records[0].id,3);
});

test('failed random identity stays explicitly unknown and invalid/discarded starts are counted',()=>{
  const h=harness({newCollectorId:()=>{throw Error('No randomness')}});
  assert.equal(h.diagnostics.snapshot().collector_id,null);
  h.at(NaN);assert.equal(h.diagnostics.begin('text-edit'),null);h.at(100);
  h.diagnostics.begin('text-move').discard();const snapshot=h.diagnostics.snapshot();
  assert.equal(snapshot.invalid_starts,1);assert.equal(snapshot.discarded,1);assert.equal(snapshot.records.length,0);
});

test('publication and the two-frame opportunity are distinct, bounded timestamps',()=>{
  const h=harness(),trace=h.diagnostics.begin('text-move');
  h.at(130);trace.mark('build-returned');h.at(150);trace.mark('published');
  trace.mark('PRIVATE DOCUMENT TEXT');trace.finish('published',{paint:true});
  assert.equal(h.diagnostics.snapshot().records.length,0);assert.equal(h.diagnostics.snapshot().pending[0].operation_ms,50);
  h.frame(160);assert.equal(h.diagnostics.snapshot().records.length,0);h.frame(176);
  const snapshot=h.diagnostics.snapshot(),record=snapshot.records[0];
  assert.equal(record.operation_ms,50);assert.equal(record.frame_proxy_ms,76);assert.equal(record.frame_status,'observed');
  assert.deepEqual(record.phases.map(p=>p.name),['build-returned','published']);
  assert.equal(JSON.stringify(snapshot).includes('PRIVATE'),false);assert.equal(h.frames.size+h.timers.size,0);
  record.phases.length=0;assert.equal(h.diagnostics.snapshot().records[0].phases.length,2);
});

test('hidden, superseded and timed-out presentation never receive a visible-latency value',()=>{
  for(const reason of ['hidden','superseded','timeout']){
    const h=harness();let current=true;
    h.diagnostics.begin('source-selection').finish('selected',{paint:true,isCurrent:()=>current});
    if(reason==='hidden'){h.document.visibilityState='hidden';h.frame(110)}
    if(reason==='superseded'){current=false;h.frame(110)}
    if(reason==='timeout')[...h.timers.values()][0]();
    const record=h.diagnostics.snapshot().records[0];assert.equal(record.frame_status,reason);assert.equal(record.frame_proxy_ms,null);
    assert.equal(h.frames.size+h.timers.size,0);
  }
});

test('overflow and unfinished traces stay visible and storage remains bounded',()=>{
  const h=harness({capacity:2,maxActive:1}),first=h.diagnostics.begin('compile-manual');
  assert.equal(h.diagnostics.begin('text-move'),null);assert.equal(h.diagnostics.snapshot().dropped,1);
  for(let n=0;n<70;n++)first.mark('build-returned');first.finish('compile-failed');
  h.diagnostics.begin('text-edit').finish('refused');
  h.diagnostics.begin('compile-auto');h.diagnostics.stop();
  const snapshot=h.diagnostics.snapshot();assert.equal(snapshot.records.length,2);assert.equal(snapshot.evicted,1);
  assert.equal(snapshot.records[1].frame_status,'stopped');assert.equal(snapshot.records[1].outcome,null);
  assert.equal(snapshot.records[1].frame_proxy_ms,null);assert.equal(snapshot.active,0);assert.equal(snapshot.dropped,1);
  assert.equal(h.diagnostics.begin('text-move'),null);h.diagnostics.clear();assert.equal(h.diagnostics.snapshot().records.length,0);
});

test('a clock discontinuity cannot produce a plausible presentation latency',()=>{
  const h=harness(),trace=h.diagnostics.begin('object-selection');h.at(99);trace.finish('selected',{paint:true});h.frame(120);h.frame(136);
  const record=h.diagnostics.snapshot().records[0];assert.equal(record.frame_status,'invalid-clock');
  assert.equal(record.operation_ms,null);assert.equal(record.frame_proxy_ms,null);
});

test('candidate phases surround the real pipeline without granting failed candidates later phases',async()=>{
  const h=harness(),trace=h.diagnostics.begin('text-move');
  const run=compile=>stageCandidate({sources:{'main.tex':'Synthetic'},revision:1,identity:'now',getIdentity:()=>'now',
    compile,prepare:async()=>({pdf:{destroy(){}},pageCount:1}),verify:async()=>{h.at(140)},trace});
  const candidate=await run(async()=>{h.at(120);return {ok:true,revision:1}});assert.equal(candidate.result.ok,true);
  trace.finish('published');assert.deepEqual(h.diagnostics.snapshot().records[0].phases.map(p=>p.name),['build-returned','pdf-prepared','frontend-validated','build-rechecked']);
  const failed=h.diagnostics.begin('text-move');
  await assert.rejects(stageCandidate({sources:{'main.tex':'Synthetic'},revision:1,identity:'now',getIdentity:()=>'now',
    compile:async()=>({ok:false,log:'! Synthetic error'}),prepare:async()=>assert.fail('Failed compilation reached preparation'),trace:failed}),{name:'CompilationFailure'});
  failed.finish('compile-failed');assert.deepEqual(h.diagnostics.snapshot().records[1].phases.map(p=>p.name),['build-returned']);
});
