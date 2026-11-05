import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';

const app=await readFile(new URL('../web/app.js',import.meta.url),'utf8');
const begin=app.indexOf('async function saveProject(){'),end=app.indexOf('const selectionProofs=new WeakMap();',begin);
assert.ok(begin>=0&&end>begin);
function setup({failure=null,parent='COPY-PARENT',busy=false,drafts=[],confirmed=true}={}){
  const events=[],classes=[],elements=new Map();
  const element=id=>{
    if(!elements.has(id))elements.set(id,{value:'',open:false,textContent:'',children:[],
      classList:{remove:name=>{if(id==='save-copy')classes.push(name)},toggle(){}},
      replaceChildren(){this.children=[];this.value=''},append(item){this.children.push(item);if(!this.value)this.value=item.value},
      showModal(){this.open=true},close(){this.open=false}});
    return elements.get(id);
  };
  const state={project:{id:'project',recoveries:drafts},revision:2,sources:{'main.tex':'LOCAL'},saved:{'main.tex':'DISK'},busy,
    client:{flush:async()=>events.push('flush')}};
  const saved=state.saved;
  const context={state,clone:structuredClone,editorReadOnly:()=>events.push(['readonly',!!state.saving]),updateControls:()=>{},
    status:()=>events.push('status'),toast:text=>events.push(['toast',text]),message:error=>error.message||String(error),
    $:element,document:{createElement:()=>({})},dirty:()=>JSON.stringify(state.saved)!==JSON.stringify(state.sources),
    confirm:async()=>confirmed,compile:async()=>events.push('compile'),
    openProject:async(_path,project)=>{events.push(['adopt',project.id]);state.project=project},
    open:async options=>{events.push(['dialog',options.directory]);return parent;},
    invoke:async(name,args)=>{
      events.push([name,args]);if(failure&&name==='save_document')throw Error(failure);
      if(name==='project_recoveries')return drafts;
      if(name==='recover_document')return {id:'recovered',recoveries:[]};
      if(name==='discard_document_recovery')return [];
      return 'COPY-PATH';
    }};
  state.client.save=()=>context.invoke('save_document',{projectId:state.project.id,revision:state.revision});
  vm.runInNewContext(app.slice(begin,end)+
    ';this.saveProject=saveProject;this.saveProjectCopy=saveProjectCopy;this.showSaveRecovery=showSaveRecovery;this.applySaveRecovery=applySaveRecovery;this.discardSaveRecovery=discardSaveRecovery;',context);
  return {context,state,events,saved,classes,element};
}
test('Save marks only acknowledged source as saved and releases editor locking',async()=>{
  const t=setup();await t.context.saveProject();
  assert.deepEqual({...t.state.saved},t.state.sources);assert.notEqual(t.state.saved,t.state.sources);
  assert.equal(t.state.saving,false);assert.equal(t.events.filter(e=>Array.isArray(e)&&e[0]==='save_document').length,1);
  assert.deepEqual(t.events.filter(e=>Array.isArray(e)&&e[0]==='readonly'),[['readonly',true],['readonly',false]]);
});
test('a refused Save preserves editor work and exposes a project-copy escape',async()=>{
  const t=setup({failure:'Save conflict: main.tex changed'});await t.context.saveProject();
  assert.equal(t.state.saved,t.saved);assert.equal(t.state.sources['main.tex'],'LOCAL');assert.equal(t.state.saving,false);
  assert.deepEqual(t.classes,['hidden']);assert.ok(!t.events.includes('status'));
});
test('saving a project copy keeps the original project dirty and sends only native identity',async()=>{
  const t=setup();await t.context.saveProjectCopy();assert.equal(t.state.saved,t.saved);assert.equal(t.state.project.id,'project');
  const event=t.events.find(e=>Array.isArray(e)&&e[0]==='copy_document');
  assert.equal(event[1].projectId,'project');assert.equal(event[1].revision,2);assert.equal(event[1].parent,'COPY-PARENT');
  assert.equal(event[1].sources,undefined);assert.equal(t.state.saving,false);
});
test('cancelled copy and busy Save perform no disk operation',async()=>{
  const cancelled=setup({parent:null});await cancelled.context.saveProjectCopy();
  assert.ok(!cancelled.events.some(e=>Array.isArray(e)&&e[0]==='copy_document'));assert.equal(cancelled.state.saving,false);
  const busy=setup({busy:true});await busy.context.saveProject();
  assert.ok(!busy.events.includes('flush'));assert.equal(busy.state.saved,busy.saved);
});
test('reviewing recovery lists files and never applies a draft automatically',async()=>{
  const t=setup({drafts:[{id:'draft',state:'committing',files:['main.tex','style.sty']}]});
  await t.context.showSaveRecovery();assert.equal(t.element('recovery-dialog').open,true);
  assert.equal(t.element('recovery-files').textContent,'main.tex\nstyle.sty');
  assert.ok(!t.events.some(e=>Array.isArray(e)&&e[0]==='recover_document'));assert.equal(t.state.sources['main.tex'],'LOCAL');
});
test('recovery requires the selected draft and current revision and reopens only its returned project',async()=>{
  const t=setup({drafts:[{id:'draft',state:'committing',files:['main.tex']}]});
  await t.context.showSaveRecovery();await t.context.applySaveRecovery(true);
  const event=t.events.find(e=>Array.isArray(e)&&e[0]==='recover_document');
  assert.equal(event[1].transactionId,'draft');assert.equal(event[1].revision,2);assert.equal(event[1].finish,true);
  assert.ok(t.events.some(e=>Array.isArray(e)&&e[0]==='adopt'&&e[1]==='recovered'));assert.equal(t.state.saving,false);
});
test('declining recovery or draft deletion changes neither disk nor editor state',async()=>{
  const t=setup({drafts:[{id:'draft',state:'rolled_back',files:['main.tex']}],confirmed:false});
  await t.context.showSaveRecovery();await t.context.applySaveRecovery(false);await t.context.discardSaveRecovery();
  assert.ok(!t.events.some(e=>Array.isArray(e)&&['recover_document','discard_document_recovery'].includes(e[0])));
  assert.equal(t.state.sources['main.tex'],'LOCAL');assert.equal(t.state.saved,t.saved);
});
test('explicit draft deletion retains the editor version and closes an empty recovery list',async()=>{
  const t=setup({drafts:[{id:'draft',state:'rolled_back',files:['main.tex']}]});
  await t.context.showSaveRecovery();await t.context.discardSaveRecovery();
  assert.equal(t.state.sources['main.tex'],'LOCAL');assert.equal(t.state.saved,t.saved);
  assert.equal(t.element('recovery-dialog').open,false);assert.equal(t.state.project.recoveries.length,0);
});
