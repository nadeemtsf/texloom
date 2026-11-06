import {test} from 'node:test';
import assert from 'node:assert/strict';
import {CloseController} from '../web/core/close-controller.js';
function setup({dirty=true,yes=true,fail=null}={}){
  const events=[],state={project:{id:'project'},revision:4,recovery:{protected_revision:4,error:null}};
  state.client={snapshot:{revision:4},flush:async()=>events.push('flush')};
  const controller=new CloseController({getState:()=>state,isDirty:()=>dirty,confirm:async(text,options)=>{events.push(['confirm',text,options]);return yes},
    closeWindow:async()=>events.push('request-close'),invoke:async(name,args)=>{events.push([name,args]);if(name===fail)throw Error('controlled '+fail);return {}},
    lock:()=>{state.saving=true;events.push('lock')},unlock:()=>{state.saving=false;events.push('unlock')},notify:text=>events.push(['notice',text])});
  const event=()=>({prevented:false,preventDefault(){this.prevented=true}});
  return {state,events,controller,event};
}
test('dirty close explicitly offers retained recovery, then closes without discarding the draft',async()=>{
  const t=setup(),first=t.event();await t.controller.handle(first);assert.equal(first.prevented,true);
  assert.match(t.events.find(e=>e[0]==='confirm')[1],/stay in Recover work/);assert.ok(t.events.includes('request-close'));
  const second=t.event();await t.controller.handle(second);assert.equal(second.prevented,false);
  assert.deepEqual(t.events.find(e=>e[0]==='close_document')[1],{projectId:'project',revision:4,discardRecovery:false});
});
test('declining close sends no native close or draft deletion',async()=>{
  const t=setup({yes:false}),e=t.event();await t.controller.handle(e);assert.equal(e.prevented,true);
  assert.ok(!t.events.some(x=>Array.isArray(x)&&x[0]==='close_document'));assert.ok(!t.events.includes('request-close'));
});
test('unprotected latest edits prevent closing rather than promise a missing draft',async()=>{
  const t=setup();await t.controller.handle(t.event());t.state.recovery.protected_revision=3;
  const e=t.event();await t.controller.handle(e);assert.equal(e.prevented,true);assert.equal(t.state.saving,false);
  assert.ok(!t.events.some(x=>x[0]==='close_document'));assert.match(t.events.at(-1)[1],/complete recovery copy/);
});
test('clean saved document retires its unnecessary recovery draft',async()=>{
  const t=setup({dirty:false});await t.controller.handle(t.event());
  assert.equal(t.events.find(e=>e[0]==='close_document')[1].discardRecovery,true);
});
test('a client switch while flushing prevents closing the other project',async()=>{
  const t=setup({dirty:false});t.state.client.flush=async()=>{t.state.project={id:'different'};t.state.client={}};
  const e=t.event();await t.controller.handle(e);assert.equal(e.prevented,true);assert.equal(t.state.saving,false);
  assert.ok(!t.events.some(x=>x[0]==='close_document'));
});
test('clean-exit marker failure after native close does not reopen a closed replica',async()=>{
  const t=setup({dirty:false,fail:'mark_clean_exit'}),e=t.event();await t.controller.handle(e);
  assert.equal(e.prevented,false);assert.ok(!t.events.includes('unlock'));assert.match(t.events.at(-1)[1],/mark_clean_exit/);
});
test('composition or an in-flight mutation prevents a close attempt',async()=>{
  for(const key of ['composing','mutating','opening','saving']){
    const t=setup();t.state[key]=true;const e=t.event();await t.controller.handle(e);assert.equal(e.prevented,true);
    assert.ok(!t.events.some(x=>x[0]==='confirm'));
  }
});
