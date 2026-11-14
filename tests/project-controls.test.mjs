import {test} from 'node:test';
import assert from 'node:assert/strict';
import {ProjectControls} from '../web/core/project-controls.js';
function setup(){const elements=new Map(),calls=[],notices=[];let locked=false;
 const element=id=>({id,value:'',children:[],disabled:false,open:false,addEventListener(){},setAttribute(){},append(...items){this.children.push(...items)},replaceChildren(){this.children=[]},close(){this.open=false},showModal(){this.open=true}});
 const document={getElementById:id=>{if(!elements.has(id))elements.set(id,element(id));return elements.get(id)},createElement:()=>element('generated')};
 const state={project:{id:'p',name:'Project'},sources:{'main.tex':'Original'},revision:0,client:{},result:{build_id:'before'}};
 const options={document,getState:()=>state,flush:async()=>{calls.push('flush')},isDirty:()=>false,lock:()=>{locked=true},unlock:()=>{locked=false},
  invoke:async(name,args)=>{calls.push({name,args});if(name==='export_project_archive')return null;if(name==='import_image')return {asset:'images/image-test.png'};if(name==='open_project')return {id:'new',main:'main.tex',recoveries:[],recovery_error:null};throw Error(name)},
  confirm:async()=>false,open:async()=>null,save:async()=>null,adopt:async project=>{state.project=project},compile:async()=>calls.push('compile'),notify:text=>notices.push(text),cursor:()=>({file:'main.tex',source:'Original',selection:[0,0]}),insertImage:async()=>calls.push('insert')};
 const controls=new ProjectControls(options);return {controls,state,elements,calls,notices,options,locked:()=>locked};
}
test('opening a path retains dirty work when switching is declined',async()=>{const t=setup();t.controls.isDirty=()=>true;t.elements.get('project-path').value='C:/fixture/main.tex';await t.controls.openPath();assert.equal(t.state.project.id,'p');assert.deepEqual(t.calls,['flush']);assert.equal(t.locked(),false)});

test('valid existing path opens and compiles once only after adoption and unlock',async()=>{
 const t=setup();let adoptedOptions;
 t.controls.adopt=async(project,options)=>{assert.equal(t.locked(),true);adoptedOptions=options;t.state.project=project;t.state.client={id:'new-client'}};
 t.controls.compile=async reason=>{assert.equal(t.locked(),false);assert.equal(t.state.project.id,'new');assert.equal(reason,'manual');t.calls.push('compile')};
 t.elements.get('project-path').value=' "C:/fixture/main.tex" ';await t.controls.openPath();
 assert.deepEqual(adoptedOptions,{compileOnOpen:true});assert.equal(t.calls.filter(c=>c==='compile').length,1);
 assert.deepEqual(t.calls.find(c=>c?.name==='open_project').args,{path:'C:/fixture/main.tex'});assert.equal(t.notices.length,0);
});

test('invalid path never compiles the previously open project',async()=>{
 const t=setup();t.controls.invoke=async()=>{throw Error('Path does not exist')};t.elements.get('project-path').value='C:/missing/main.tex';
 await assert.rejects(t.controls.openPath(),/does not exist/);assert.equal(t.calls.includes('compile'),false);assert.equal(t.state.project.id,'p');assert.equal(t.locked(),false);
});

test('automatic open does not compile source-only, recovery-blocked or non-tex main files',async()=>{
 for(const overrides of [{source_only:true},{restored:true},{recovery_error:'pending recovery'},{recoveries:[{id:'pending'}]},{main:'notes.text'}]){
  const t=setup();t.controls.invoke=async()=>({id:'new',main:'main.tex',recoveries:[],recovery_error:null,...overrides});
  t.elements.get('project-path').value='C:/fixture';await t.controls.openPath();assert.equal(t.calls.includes('compile'),false,JSON.stringify(overrides));
 }
});

test('a superseded project or client cannot receive the automatic compile',async()=>{
 for(const change of [t=>{t.state.project={id:'other'}},t=>{t.state.client={id:'other'}}]){
  const t=setup();t.controls.unlock=()=>{change(t)};t.elements.get('project-path').value='C:/fixture/main.tex';await t.controls.openPath();assert.equal(t.calls.includes('compile'),false);
 }
});

test('opening a recent project uses the same automatic compile route',async()=>{
 const t=setup();t.controls.invoke=async name=>name==='recent_projects'?[{name:'Fixture',main:'main.tex',path:'C:/fixture'}]:{id:'recent',main:'main.tex',recoveries:[],recovery_error:null};
 await t.controls.show();await t.elements.get('project-recents').children[0].onclick();assert.equal(t.state.project.id,'recent');assert.equal(t.calls.filter(c=>c==='compile').length,1);
});

test('compilation failure retains the adopted project and leaves switching unlocked',async()=>{
 const t=setup();t.controls.compile=async()=>{throw Error('Compiler failed')};t.elements.get('project-path').value='C:/fixture';
 await assert.rejects(t.controls.openPath(),/Compiler failed/);assert.equal(t.state.project.id,'new');assert.equal(t.locked(),false);
});
test('ZIP export flushes source and binds the acknowledged revision without reopening the project',async()=>{const t=setup();t.controls.flush=async()=>{t.state.revision=7;t.calls.push('flush')};t.controls.save=async()=> 'C:/copy.zip';await t.controls.exportZip();assert.deepEqual(t.calls[1],{name:'export_project_archive',args:{projectId:'p',revision:7,path:'C:/copy.zip'}});assert.equal(t.state.project.id,'p');assert.equal(t.locked(),false);assert.match(t.notices[0],/exported/)});
test('importing an image cannot insert source against a failed new baseline',async()=>{const t=setup();t.controls.open=async()=> 'C:/image.png';await t.controls.image();assert.equal(t.state.sources['main.tex'],'Original');assert.equal(t.calls.includes('insert'),false);assert.match(t.notices[0],/Compile the project before inserting/);assert.equal(t.locked(),false)});
test('failed native project loading always releases editing and retains the current project',async()=>{const t=setup();await assert.rejects(t.controls.switchProject(async()=>{throw Error('Not UTF-8')}),/UTF-8/);assert.equal(t.state.project.id,'p');assert.equal(t.locked(),false)});
test('creating a starter project opens its source without automatically running LaTeX',async()=>{const t=setup();t.controls.$('project-new-name').value=' New letter ';t.controls.$('project-template').value='letter';t.controls.invoke=async(name,args)=>{t.calls.push({name,args});return name==='create_project'?{path:'C:/new-letter'}:{id:'new',recoveries:[],recovery_error:null}};await t.controls.create();assert.equal(t.state.project.id,'new');assert.deepEqual(t.calls.filter(c=>typeof c==='object').map(c=>c.name),['create_project','open_project']);assert.equal(t.calls.includes('compile'),false);assert.equal(t.locked(),false)});
