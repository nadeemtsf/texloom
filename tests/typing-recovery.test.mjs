import {test} from 'node:test';
import assert from 'node:assert/strict';
import {TypingRecoveryController} from '../web/core/typing-recovery.js';

const draft={id:'a'.repeat(32),project:'P:/Project',main:'main.tex',revision:3,files:2,source_bytes:32,warning:null};
function setup({dirty=false,confirmed=true,invoke=null}={}){
 const elements=new Map(),events=[],state={project:{id:'active'},sources:{'main.tex':'CURRENT'},revision:0,
  recovery:{draft_id:'b'.repeat(32),protected_revision:0,error:null}};
 const element=id=>{
  if(!elements.has(id)){
   const classes=new Set();elements.set(id,{value:'',textContent:'',children:[],disabled:false,open:false,
    classList:{toggle(name,yes){if(yes)classes.add(name);else classes.delete(name)},contains:name=>classes.has(name)},
    addEventListener(){},replaceChildren(){this.children=[];this.value=''},append(option){this.children.push(option);if(!this.value)this.value=option.value},
    showModal(){this.open=true},close(){this.open=false}});
  }return elements.get(id);
 };
 const call=async(name,args)=>{
  events.push([name,args]);if(invoke)return invoke(name,args,state);
  if(name==='typing_drafts')return {drafts:[draft],errors:[]};
  if(name==='open_typing_draft')return {id:'recovered',restored:true};
  if(name==='discard_typing_draft')return {drafts:[],errors:[]};
 };
 state.client={flush:async()=>events.push('flush'),retryRecovery:async()=>events.push('retry')};
 const controller=new TypingRecoveryController({invoke:call,document:{getElementById:element,createElement:()=>({})},getState:()=>state,
  isDirty:()=>dirty,adopt:async project=>{events.push(['adopt',project.id]);state.project=project;state.client={}},
  lock:()=>{state.saving=true;events.push('lock')},unlock:()=>{state.saving=false;events.push('unlock')},
  confirm:async()=>{events.push('confirm');return confirmed},notify:text=>events.push(['notice',text]),saveCopy:()=>events.push('copy')});
 return {state,controller,events,element};
}

test('recovery protection warnings persist independently of compilation and never replace editor sources',()=>{
 const t=setup(),sources=t.state.sources;
 t.controller.renderProtection();assert.equal(t.element('draft-status').textContent,'Recovery copy protected');
 t.state.revision=1;t.controller.renderProtection();assert.equal(t.element('draft-status').textContent,'Protecting changes…');
 t.state.recovery={draft_id:null,protected_revision:null,error:'Disk full'};t.controller.renderProtection();
 assert.match(t.element('draft-warning-text').textContent,/Save your work.*Disk full/);
 assert.equal(t.element('draft-warning').classList.contains('hidden'),false);assert.equal(t.state.sources,sources);
 t.state.recovery={draft_id:'b'.repeat(32),protected_revision:1,error:null};t.controller.renderProtection();
 assert.equal(t.element('draft-warning').classList.contains('hidden'),true);
 t.state.project.draft_warning='Interrupted record was excluded';t.controller.renderProtection();
 assert.match(t.element('draft-warning-text').textContent,/Interrupted record/);
});

test('reviewing a crash inventory lists project identity but never restores automatically',async()=>{
 const t=setup();await t.controller.show();assert.equal(t.element('draft-dialog').open,true);
 assert.match(t.element('draft-details').textContent,/P:\/Project.*\n2 source files/);
 assert.deepEqual(t.events.filter(e=>Array.isArray(e)&&e[0]==='open_typing_draft'),[]);
});

test('declining replacement or discard leaves editor sources and native drafts untouched',async()=>{
 const t=setup({dirty:true,confirmed:false});await t.controller.show();
 await t.controller.restore();await t.controller.discard();
 assert.equal(t.state.sources['main.tex'],'CURRENT');assert.ok(!t.events.includes('lock'));
 assert.ok(!t.events.some(e=>Array.isArray(e)&&['open_typing_draft','discard_typing_draft'].includes(e[0])));
});

test('recovery adoption flushes the active editor, sends only the draft ID and releases locking',async()=>{
 const t=setup({dirty:true});await t.controller.show();await t.controller.restore();
 assert.equal(t.state.project.id,'recovered');assert.equal(t.state.saving,false);
 const open=t.events.find(e=>Array.isArray(e)&&e[0]==='open_typing_draft');assert.deepEqual(open[1],{draftId:draft.id});
 assert.ok(t.events.indexOf('flush')<t.events.indexOf(open));assert.ok(t.events.indexOf('lock')<t.events.indexOf('flush'));
 assert.equal(t.element('draft-dialog').open,false);
});

test('a delayed recovery result cannot replace a different current project and its new native session is retained safely',async()=>{
 let finish;const t=setup({invoke:async(name,_args,state)=>{
  if(name==='typing_drafts')return {drafts:[draft],errors:[]};
  if(name==='open_typing_draft')return new Promise(resolve=>finish=resolve);
 }});
 await t.controller.show();const restoring=t.controller.restore();await new Promise(setImmediate);
 t.state.client={};t.state.project={id:'new-current'};finish({id:'late-recovered'});await restoring;
 assert.equal(t.state.project.id,'new-current');assert.equal(t.state.sources['main.tex'],'CURRENT');
 const closed=t.events.find(e=>Array.isArray(e)&&e[0]==='close_document');
 assert.deepEqual(closed[1],{projectId:'late-recovered'});assert.equal(t.state.saving,false);
 assert.ok(!t.events.some(e=>Array.isArray(e)&&e[0]==='adopt'));
});

test('unreadable recovery evidence is visible and cannot enable restoration or deletion',async()=>{
 const t=setup({invoke:async()=>{throw Error('Checksum mismatch')}});await t.controller.show();
 assert.match(t.element('draft-errors').textContent,/Checksum mismatch/);
 assert.equal(t.element('draft-restore').disabled,true);assert.equal(t.element('draft-discard').disabled,true);
 await t.controller.restore();await t.controller.discard();assert.equal(t.events.filter(Array.isArray).filter(e=>e[0]==='typing_drafts').length,1);
});

test('explicit draft discard changes neither current buffers nor project files through the frontend',async()=>{
 const t=setup();await t.controller.show();const sources=t.state.sources;await t.controller.discard();
 assert.equal(t.state.sources,sources);assert.equal(t.state.project.id,'active');assert.equal(t.state.saving,false);
 assert.deepEqual(t.events.find(e=>Array.isArray(e)&&e[0]==='discard_typing_draft')[1],{draftId:draft.id});
 assert.equal(t.element('draft-dialog').open,false);
});

test('busy/composing/saving operations fence recovery and retry uses the ordered source client',async()=>{
 const t=setup();t.state.composing=true;await t.controller.show();assert.ok(!t.events.some(e=>Array.isArray(e)&&e[0]==='typing_drafts'));
 t.state.composing=false;await t.controller.retry();assert.ok(t.events.includes('retry'));assert.equal(t.state.saving,false);
});
