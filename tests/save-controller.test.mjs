import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';

const app=await readFile(new URL('../web/app.js',import.meta.url),'utf8');
const begin=app.indexOf('async function saveProject(){'),end=app.indexOf('function compileCandidate(){',begin);
assert.ok(begin>=0&&end>begin);
function setup({failure=null,parent='COPY-PARENT',busy=false}={}){
  const events=[],classes=[];
  const state={project:{id:'project'},revision:2,sources:{'main.tex':'LOCAL'},saved:{'main.tex':'DISK'},busy,
    client:{flush:async()=>events.push('flush')}};
  const saved=state.saved;
  const context={state,clone:structuredClone,editorReadOnly:()=>events.push(['readonly',!!state.saving]),updateControls:()=>{},
    status:()=>events.push('status'),toast:text=>events.push(['toast',text]),message:error=>error.message||String(error),
    $:()=>({classList:{remove:name=>classes.push(name)}}),
    open:async options=>{events.push(['dialog',options.directory]);return parent;},
    invoke:async(name,args)=>{events.push([name,args]);if(failure)throw Error(failure);return 'COPY-PATH';}};
  vm.runInNewContext(app.slice(begin,end)+';this.saveProject=saveProject;this.saveProjectCopy=saveProjectCopy;',context);
  return {context,state,events,saved,classes};
}
test('Save marks only acknowledged source as saved and releases editor locking',async()=>{
  const t=setup();await t.context.saveProject();
  assert.deepEqual(t.state.saved,t.state.sources);assert.notEqual(t.state.saved,t.state.sources);
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
