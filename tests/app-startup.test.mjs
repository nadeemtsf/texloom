import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import * as transactions from '../web/core/transactions.js';
import {buildInlineIndex} from '../web/core/inline-text.js';
import {DocumentClient,editorText,sameSources} from '../web/core/document-client.js';
import {TypingRecoveryController} from '../web/core/typing-recovery.js';
import {CloseController} from '../web/core/close-controller.js';
import {createWorkspaceInvoke} from '../web/core/workspace-protocol.js';
import {ProjectControls} from '../web/core/project-controls.js';
import {PDFSearch} from '../web/core/pdf-search.js';
import {bindPDFScrolling} from '../web/core/pdf-scroll.js';
import {SetupHelp} from '../web/core/setup-help.js';
import {installPerformanceDiagnostics,performanceOutcome} from '../web/core/performance-diagnostics.js';
import {SourceHighlight,selectionSourceRange} from '../web/core/source-highlight.js';
import {applyTextInputAppearance,renderProtectedCommands} from '../web/core/text-input-appearance.js';
import {positionDialogNear,clearDialogPosition} from '../web/core/dialog-anchor.js';
import {createCompileInvoke} from '../web/core/compile-progress.js';
import {startPDFWorker} from '../web/core/pdf-document.js';
import {PreliminaryPreview} from '../web/core/preliminary-preview.js';
import {PreliminaryPDFSurface,renderPreliminaryPDF} from '../web/core/preliminary-pdf.js';
import {acceptUnverifiedPreview} from '../web/core/build-protocol.js';

const [app,html]=await Promise.all(['app.js','index.html'].map(name=>readFile(new URL('../web/'+name,import.meta.url),'utf8')));
const tick=()=>new Promise(setImmediate);
const finalReply={ok:true,build_id:'settled',revision:0,pdf:[37,80,68,70,45],objects:[],log:'',engine:'pdflatex',
 seconds:1,verification_eligible:true,passes:2,diagnostics:[],job_id:1};
function sendPreliminary(t){
 const args=t.context.progressRequest;
 args.onPreview.onmessage({schema:1,phase:'first_pass',project_id:args.projectId,revision:args.revision,
  request_id:args.requestId,engine:'pdflatex',seconds:.1,diagnostics:[],pdf:[37,80,68,70,45]});
}
async function changeSource(t,transform){
 const original=t.model.value;
 t.model.value=transform(original);
 t.handlers.change({changes:[{rangeOffset:0,rangeLength:original.length,text:t.model.value}]});
 await t.context.shell.state.client.flush();
}

test('the shipped shell requests and publishes only a settled PDF',async()=>{
 const t=boot({deferCompile:true});await tick();
 const state=t.context.shell.state;
 assert.equal(t.context.shell.preliminary,null);
 assert.equal(t.context.progressRequest.purpose,'preview');
 assert.equal(Object.hasOwn(t.context.progressRequest,'onPreview'),false);
 assert.equal(state.pdf,null);
 assert.equal(t.elements.get('pages').inert,false);
 assert.equal(t.context.workerStarts,1);
 assert.equal(t.context.finalPDFLoads,0);
 t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
 assert.equal(state.result.build_id,'settled');
 assert.equal(state.compiled,0);
 assert.equal(t.context.temporaryRenders,0);
 assert.equal(t.context.finalPDFLoads,1);
 assert.equal(t.context.workerDisposals,0);
 assert.equal(t.calls.some(([name])=>name==='verify_build'),true);
});

test('a source edit between verification and publication retains the earlier PDF',async()=>{
 const t=boot({deferCompile:true});await tick();
 t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
 const state=t.context.shell.state,previous=state.pdf;
 const stage=t.context.stageCandidate;
 t.context.stageCandidate=async options=>{
   const candidate=await stage(options);
   await changeSource(t,source=>source.replace('shell fixture','newer source'));
   return candidate;
 };
 const compiling=t.elements.get('compile').onclick();await tick();
 t.context.finishCompile({...structuredClone(finalReply),build_id:'stale-preview'});
 await compiling;
 assert.equal(state.revision,1);assert.match(state.sources['main.tex'],/newer source/);
 assert.equal(state.pdf,previous);assert.equal(state.result.build_id,'settled');
 assert.equal(state.compiled,0);assert.equal(state.busy,false);
 assert.equal(t.calls.some(([name,args])=>name==='pin_displayed_build'&&args.buildId==='stale-preview'),false);
 assert.equal(t.context.workerDisposals,1);
});

test('after a source error the shipped shell keeps the last PDF and publishes the corrected source',async()=>{
 const t=boot({deferCompile:true});await tick();
 t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
 const state=t.context.shell.state,previous=state.pdf;
 const original=state.sources['main.tex'];
 await changeSource(t,source=>source.replace('Native shell fixture','\\MissingFixtureCommand'));
 const failed=t.elements.get('compile').onclick();await tick();
 assert.equal(t.context.progressRequest.revision,1);
 t.context.finishCompile({...structuredClone(finalReply),ok:false,pdf:[],verification_eligible:false,
   revision:1,build_id:'failed',log:'! Undefined control sequence.'});
 await failed;
 assert.equal(state.pdf,previous);assert.equal(state.compiled,0);
 assert.match(t.elements.get('banner').textContent,/Undefined control sequence/);
 assert.equal(t.elements.get('banner').classList.contains('hidden'),false);

 await changeSource(t,()=>original.replace('Native shell fixture','Corrected source'));
 const corrected=t.elements.get('compile').onclick();await tick();
 assert.equal(t.context.progressRequest.revision,2);
 t.context.finishCompile({...structuredClone(finalReply),revision:2,build_id:'corrected'});await corrected;
 assert.notEqual(state.pdf,previous);assert.equal(state.result.build_id,'corrected');
 assert.equal(state.compiled,2);assert.equal(state.busy,false);
 assert.equal(t.elements.get('banner').classList.contains('hidden'),true);
 assert.equal(t.calls.some(([name,args])=>name==='pin_displayed_build'&&args.buildId==='failed'),false);
 assert.equal(t.calls.some(([name,args])=>name==='pin_displayed_build'&&args.buildId==='corrected'),true);
});

test('the experimental shell keeps preliminary bytes out of settled state, pinning and verification until final publication',async()=>{
 const t=boot({deferCompile:true,enablePreliminary:true});await tick();sendPreliminary(t);await tick();
 const state=t.context.shell.state;
 assert.equal(t.context.shell.preliminary.visible,true);assert.equal(state.result,null);assert.equal(state.pdf,null);assert.equal(state.compiled,-1);
 assert.equal(t.elements.get('pages').inert,true);assert.equal(t.elements.get('export').disabled,true);
 assert.equal(t.calls.some(([name])=>['verify_build','pin_displayed_build'].includes(name)),false);
 const late=t.context.progressRequest.onPreview.onmessage;
 t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
 assert.equal(state.result.build_id,'settled');assert.equal(state.compiled,0);assert.equal(t.context.shell.preliminary.visible,false);
 assert.equal(t.context.temporaryDisposals,1);assert.equal(t.elements.get('pages').inert,false);
 assert.equal(t.calls.some(([name])=>name==='verify_build'),true);
 late({});await tick();assert.equal(t.context.temporaryRenders,1);
});

test('a later compiler failure clears the preliminary view and retains the earlier PDF and source',async()=>{
 const t=boot({deferCompile:true,enablePreliminary:true});await tick();const state=t.context.shell.state;
 const previous={destroy(){throw Error('The earlier PDF must remain alive')}};
 state.pdf=previous;state.result={...finalReply,build_id:'earlier'};state.compiled=0;
 const sources={...state.sources};sendPreliminary(t);await tick();
 assert.equal(t.context.shell.preliminary.visible,true);
 t.context.failCompile(Error('Synthetic later pass failed'));await t.context.startup;
 assert.equal(state.pdf,previous);assert.equal(state.result.build_id,'earlier');assert.deepEqual({...state.sources},sources);
 assert.equal(t.context.temporaryDisposals,1);assert.equal(t.elements.get('pages').inert,false);
 assert.equal(t.context.finalPDFLoads,0);
 assert.equal(t.context.workerDisposals,1);
 assert.match(t.elements.get('banner').textContent,/Synthetic later pass failed/);
});

// Run the complete production shell. IDs come only from the shipped HTML;
// looking up a missing control returns null, as it does in the actual WebView.
function boot({markup=html,openFailure=null,pathFailure=null,badSnapshot=false,adapterFailure=null,deferCompile=false,enablePreliminary=false,pendingSave=null,badRecoverySnapshot=false,withChapter=false}={}){
  const calls=[],compileStates=[],elements=new Map(),options={},handlers={};
  function element(id=''){
    const classes=new Set(),attributes=new Map();
    return {id,value:'',textContent:'',disabled:false,checked:false,open:false,children:[],style:{},inert:false,clientWidth:640,clientHeight:900,scrollTop:0,
      classList:{add:n=>classes.add(n),remove:n=>classes.delete(n),contains:n=>classes.has(n),
        toggle(n,force){const enabled=force??!classes.has(n);if(enabled)classes.add(n);else classes.delete(n);return enabled}},
      setAttribute(k,v){attributes.set(k,v)},getAttribute:k=>attributes.get(k)??null,removeAttribute:k=>attributes.delete(k),
      addEventListener(){},replaceChildren(...children){this.children=children},
      append(...children){for(const child of children)child.parentElement=this;this.children.push(...children)},
      remove(){if(this.parentElement)this.parentElement.children=this.parentElement.children.filter(x=>x!==this)},
      focus(){},showModal(){this.open=true},close(){this.open=false}};
  }
  for(const match of markup.matchAll(/\bid="([^"]+)"/g)){
    assert.ok(!elements.has(match[1]),'Duplicate HTML ID: '+match[1]);
    elements.set(match[1],element(match[1]));
  }
  const source='\\documentclass{article}\n\\begin{document}\nNative shell fixture\n\\end{document}\n';
  const project={id:'fixture',name:'Fixture',main:'main.tex',sources:{'main.tex':source},recoveries:pendingSave?[pendingSave]:[],recovery_error:null,restored:false,draft_warning:null,source_only:false};
  if(withChapter)project.sources['chapter.tex']='Fixture chapter.\n';
  const snapshot={revision:0,sources:project.sources,main:project.main,engine:'auto',can_undo:false,can_redo:false,selection:null,recovery:{draft_id:null,protected_revision:null,error:null}};
  const model={value:'',setValue(v){this.value=v},setEOL(){},getValue(){return this.value},getLineCount(){return this.value.split('\n').length}};
  const editor={createContextKey:()=>({set(){}}),updateOptions:o=>Object.assign(options,o),
    onDidChangeModelContent(fn){handlers.change=fn},onDidChangeCursorPosition(){},onDidChangeCursorSelection(){},
    addAction(){},addCommand(){},getDomNode:()=>element(),getSelections:()=>[],hasTextFocus:()=>false,
    setPosition(position){handlers.lastPosition=position},focus(){}};
  let nativeSource=source;
  const context={buildInlineIndex,draftRecoveryEnabled:false,...transactions,bindPDFScrolling,SourceHighlight,selectionSourceRange,applyTextInputAppearance,renderProtectedCommands,positionDialogNear,clearDialogPosition,DocumentClient,editorText,sameSources,TypingRecoveryController,CloseController,createWorkspaceInvoke,ProjectControls,PDFSearch,SetupHelp,installPerformanceDiagnostics,performanceOutcome,structuredClone,console,
    createCompileInvoke,startPDFWorker,PreliminaryPreview,PreliminaryPDFSurface,renderPreliminaryPDF,acceptUnverifiedPreview,
    capturePDFScrollAnchor:()=>null,
    Channel:class{constructor(callback){this.onmessage=callback}},
    PDFPageView:class{},EventBus:class{},SimpleLinkService:class{},self:{},pdfjs:{GlobalWorkerOptions:{},PDFWorker:{create(){
      context.workerStarts++;return {promise:Promise.resolve(),destroy(){context.workerDisposals++}};
    }}},pdfWorkerURL:'test-worker',EditorWorker:class{},
    monaco:{languages:{register(){},setMonarchTokensProvider(){},setLanguageConfiguration(){}},
      editor:{defineTheme(){},createModel:()=>model,create:()=>editor,EndOfLineSequence:{LF:0}},
      KeyMod:{CtrlCmd:1,Shift:2},KeyCode:{KeyS:3,Enter:4,KeyZ:5,KeyY:6}},
    CanvasController:class{clear(){}bind(){}},
    document:{getElementById:id=>elements.get(id)||null,createElement:()=>element(),
      querySelectorAll:selector=>selector==='dialog[open]'?[...elements.values()].filter(e=>e.open):[],addEventListener:(name,fn)=>{handlers[name]=fn}},
    window:{addEventListener(){}},setTimeout:()=>1,clearTimeout(){},
    getCurrentWindow:()=>({onCloseRequested:fn=>{handlers.close=fn}}),
    open:async()=>null,save:async()=>null,confirm:async()=>true,
    nativeInvoke:async(name,args)=>{
      calls.push([name,args]);
      if(name==='open_project'){
        if(openFailure)throw Error(openFailure);
        if(args.path&&pathFailure)throw Error(pathFailure);
        return args.path?{...project,id:'opened-path',name:'Opened path'}:project;
      }
      if(name==='document_snapshot')return badSnapshot||badRecoverySnapshot&&args.projectId==='recovered-save'?{...snapshot,sources:null}:snapshot;
      if(name==='project_recoveries')return project.recoveries;
      if(name==='recover_document')return {...project,id:'recovered-save',recoveries:[]};
      if(name==='discard_document_recovery')return [];
      if(name==='document_edit'){
        const [{offset,length,text}]=args.changes;
        nativeSource=nativeSource.slice(0,offset)+text+nativeSource.slice(offset+length);
        return {...snapshot,kind:'source_delta',base_revision:args.revision,revision:args.revision+1,
          can_undo:true,sources:{'main.tex':nativeSource}};
      }
      if(name==='typing_drafts')return {drafts:[],errors:[]};
      if(name==='close_document')return {warning:null};
      if(name==='compile_document')compileStates.push({saving:context.shell.state.saving,opening:context.shell.state.opening,projectId:context.shell.state.project.id});
      if(name==='compile_document'&&deferCompile){context.progressRequest=args;return new Promise((resolve,reject)=>{context.finishCompile=resolve;context.failCompile=reject})}
      if(deferCompile&&['verify_build','pin_displayed_build','cancel_compilation','export_document_build'].includes(name))return null;
      if(name==='plan_adapter'&&adapterFailure)throw Error(adapterFailure);
      throw Error('Test compiler unavailable');
    }};
  context.workerStarts=0;context.workerDisposals=0;context.finalPDFLoads=0;
  if(deferCompile){
    context.temporaryDisposals=0;context.temporaryRenders=0;
    context.location={href:'http://localhost/'};context.URL=URL;
    context.renderPreliminaryPDF=async()=>{context.temporaryRenders++;return {canvas:element(),page:1,pageCount:1,dispose(){context.temporaryDisposals++}}};
    context.preparePDFView=async(_library,_result,{workerStartup})=>{
      const worker=await workerStartup?.take();context.finalPDFLoads++;
      return {pdf:{destroy:async()=>worker?.destroy()},fragment:element(),pageCount:1,scale:1,pages:[],dispose(){}};
    };
    context.publishPDFView=()=>{};
  }
  let script=app.replace(/^import .*;\r?$/gm,'').replace(/openProject\(\);\s*$/,'globalThis.startup = openProject();');
  if(enablePreliminary)script=script.replace('const ENABLE_PRELIMINARY_PREVIEW = false;','const ENABLE_PRELIMINARY_PREVIEW = true;');
  vm.runInNewContext(script+'\nglobalThis.shell={state,preliminary,updateControls,applyLayoutControl,showFailure,projectControls};',context,{filename:'web/app.js'});
  return {context,calls,compileStates,elements,options,model,handlers};
}

test('the complete shipped shell opens a project and enables source editing',async()=>{
  const t=boot();await t.context.startup;
  assert.equal(t.calls[0][0],'open_project');assert.equal(t.calls[0][1].path,null);
  assert.equal(t.context.shell.state.project.name,'Fixture');
  assert.match(t.model.value,/Native shell fixture/);assert.equal(t.options.readOnly,false);
  assert.equal(t.elements.get('project-name').textContent,'Fixture');
  assert.notEqual(t.elements.get('status').textContent,'Starting…');
  assert.equal(typeof t.elements.get('open').onclick,'function');
  assert.equal(typeof t.handlers.close,'function');
  assert.equal(t.elements.get('cancel-compile').disabled,true);
});

test('Projects open path requests exactly one compile after adopting and unlocking, even with Auto off',async()=>{
  const t=boot();await t.context.startup;
  const before=t.compileStates.length;t.elements.get('auto').checked=false;
  t.elements.get('project-path').value='C:/synthetic/main.tex';
  await t.context.shell.projectControls.openPath();
  assert.equal(t.compileStates.length,before+1);
  assert.deepEqual(t.compileStates.at(-1),{saving:false,opening:false,projectId:'opened-path'});
  const request=t.calls.filter(([name])=>name==='compile_document').at(-1)[1];
  assert.equal(request.projectId,'opened-path');assert.equal(request.purpose,'preview');
  assert.equal(t.context.shell.state.compileApproved,true);
  assert.equal(t.context.shell.state.busy,false);assert.equal(t.context.shell.state.saving,false);
  assert.equal(t.context.shell.state.project.id,'opened-path');
  // The deliberately unavailable compiler must show its error, not the old
  // review-before-Compile prompt. This does not claim a real renderer pass.
  assert.equal(t.elements.get('banner').classList.contains('hidden'),false);
  assert.match(t.elements.get('banner').textContent,/Test compiler unavailable/);
  assert.doesNotMatch(t.elements.get('banner').textContent,/Review it before choosing Compile/);
  assert.equal(t.context.shell.state.canUndo,false);assert.equal(t.context.shell.state.canRedo,false);
});

test('a missing Projects path retains the displayed source and never requests another compile',async()=>{
  const t=boot({pathFailure:'Project path does not exist'});await t.context.startup;
  const before=t.compileStates.length,source=t.model.value,client=t.context.shell.state.client;
  t.elements.get('project-path').value='C:/missing/main.tex';
  await t.elements.get('project-open-path').onclick();
  assert.equal(t.compileStates.length,before);assert.equal(t.context.shell.state.client,client);
  assert.equal(t.model.value,source);assert.equal(t.context.shell.state.saving,false);
  assert.match(t.elements.get('toast').textContent,/does not exist/);
});

test('toolbar Open reserves the project switch while confirming unsaved source',async()=>{
  const t=boot();await t.context.startup;
  const state=t.context.shell.state,client=state.client;
  state.saved={'main.tex':'Earlier saved text'};
  t.context.open=async()=> 'C:/first/main.tex';
  let resolveConfirm,showConfirm;
  const confirmation=new Promise(resolve=>{resolveConfirm=resolve});
  const confirming=new Promise(resolve=>{showConfirm=resolve});
  t.context.confirm=()=>{showConfirm();return confirmation};
  const first=t.elements.get('open').onclick();
  await confirming;
  assert.equal(t.options.readOnly,true);
  t.elements.get('project-path').value='C:/second/main.tex';
  const second=t.elements.get('project-open-path').onclick();
  resolveConfirm(false);
  await Promise.all([first,second]);
  assert.equal(t.calls.filter(([name])=>name==='open_project').length,1);
  assert.equal(state.client,client);assert.equal(state.opening,false);assert.ok(!state.saving);
  assert.equal(t.options.readOnly,false);
  assert.match(t.elements.get('toast').textContent,/Wait for the current operation/);
});

test('a file click waiting on the previous project cannot navigate its replacement',async()=>{
  const t=boot({withChapter:true});await t.context.startup;
  const state=t.context.shell.state,old=state.client,flush=old.flush.bind(old);
  const chapter=t.elements.get('file-list').children.find(child=>child.textContent==='chapter.tex');
  let release;
  const pending=new Promise(resolve=>{release=resolve});
  old.flush=()=>{old.flush=flush;return pending};
  const navigating=chapter.onclick();
  t.context.open=async()=> 'C:/other/main.tex';
  await t.elements.get('open').onclick();
  release(old.snapshot);await navigating;
  assert.equal(state.project.id,'opened-path');assert.equal(state.file,'main.tex');
});

test('a diagnostic retains its project identity and still opens its file within that project',async()=>{
  const t=boot({withChapter:true});await t.context.startup;
  const state=t.context.shell.state,diagnostic={file:'chapter.tex',line:1,message:'Fixture problem',severity:'error'};
  t.context.shell.showFailure(Object.assign(Error('Fixture error'),{diagnostics:[diagnostic]}));
  const action=t.elements.get('diagnostics').children[0].onclick;
  await action();
  assert.equal(state.file,'chapter.tex');assert.equal(t.handlers.lastPosition.lineNumber,1);

  t.context.open=async()=> 'C:/other/main.tex';
  await t.elements.get('open').onclick();
  await action();
  assert.equal(state.project.id,'opened-path');assert.equal(state.file,'main.tex');
});

test('a previous project navigation failure cannot replace the current editor banner',async()=>{
  const t=boot({withChapter:true});await t.context.startup;
  const state=t.context.shell.state,old=state.client,flush=old.flush.bind(old);
  const chapter=t.elements.get('file-list').children.find(child=>child.textContent==='chapter.tex');
  let fail;
  const pending=new Promise((_,reject)=>{fail=reject});
  old.flush=()=>{old.flush=flush;return pending};
  const navigating=chapter.onclick();
  t.context.open=async()=> 'C:/other/main.tex';
  await t.elements.get('open').onclick();
  const banner=t.elements.get('banner'),current=banner.textContent;
  fail(Error('Previous source disconnected'));await navigating;
  assert.equal(state.project.id,'opened-path');assert.equal(state.file,'main.tex');
  assert.equal(banner.textContent,current);assert.equal(t.options.readOnly,false);
});

test('an export whose destination arrives after a project switch cannot export the replacement',async()=>{
  const t=boot({deferCompile:true});await tick();
  t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
  let selectPath;
  t.context.save=()=>new Promise(resolve=>{selectPath=resolve});
  const exporting=t.elements.get('export').onclick();
  t.context.open=async()=> 'C:/other/main.tex';
  await t.elements.get('open').onclick();
  assert.equal(t.context.shell.state.project.id,'opened-path');

  selectPath('C:/fixture/first-project.pdf');await tick();
  const builds=t.calls.filter(([name,args])=>name==='compile_document'&&args.purpose==='export');
  if(builds.length)t.context.finishCompile({...structuredClone(finalReply),build_id:'wrong-project'});
  await exporting;

  assert.deepEqual(builds,[]);
  assert.equal(t.calls.some(([name])=>name==='export_document_build'),false);
  assert.match(t.elements.get('toast').textContent,/Project changed.*Export again/i);
});

test('an export cannot start when its source flush overlaps a project switch',async()=>{
  const t=boot({deferCompile:true});await tick();
  t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
  const client=t.context.shell.state.client;
  let acknowledgeSource;
  client.tail=new Promise(resolve=>{acknowledgeSource=resolve});
  t.context.save=async()=> 'C:/fixture/first-project.pdf';
  const exporting=t.elements.get('export').onclick();await tick();
  t.context.open=async()=> 'C:/other/main.tex';
  const opening=t.elements.get('open').onclick();await tick();
  assert.equal(t.context.shell.state.opening,true);

  acknowledgeSource();await tick();
  const builds=t.calls.filter(([name,args])=>name==='compile_document'&&args.purpose==='export');
  if(builds.length)t.context.finishCompile({...structuredClone(finalReply),build_id:'overlapping-export'});
  await Promise.all([opening,exporting]);

  assert.deepEqual(builds,[]);
  assert.equal(t.calls.some(([name])=>name==='export_document_build'),false);
  assert.equal(t.context.shell.state.project.id,'opened-path');
  assert.match(t.elements.get('toast').textContent,/Wait for the current operation before exporting/i);
});

test('an export for the active project builds and writes its requested destination',async()=>{
  const t=boot({deferCompile:true});await tick();
  t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
  t.context.save=async()=> 'C:/fixture/export.pdf';
  const exporting=t.elements.get('export').onclick();await tick();
  assert.equal(t.context.progressRequest.purpose,'export');
  t.context.finishCompile({...structuredClone(finalReply),build_id:'exported'});
  await exporting;
  const command=t.calls.find(([name])=>name==='export_document_build');
  assert.equal(command[1].projectId,'fixture');assert.equal(command[1].buildId,'exported');
  assert.equal(command[1].path,'C:/fixture/export.pdf');
  assert.equal(t.context.shell.state.result.build_id,'settled');
  assert.equal(t.context.shell.state.busy,false);
});

test('cancelling at the verified export boundary prevents the destination write',async()=>{
  const t=boot({deferCompile:true});await tick();
  t.context.finishCompile(structuredClone(finalReply));await t.context.startup;
  const stage=t.context.stageCandidate;
  t.context.stageCandidate=async options=>{
    const prepared=await stage(options);
    await t.elements.get('cancel-compile').onclick();
    return prepared;
  };
  t.context.save=async()=> 'C:/fixture/export.pdf';
  const exporting=t.elements.get('export').onclick();await tick();
  t.context.finishCompile({...structuredClone(finalReply),build_id:'cancelled-export'});
  await exporting;

  assert.equal(t.calls.some(([name,args])=>name==='verify_build'&&args.buildId==='cancelled-export'),true);
  assert.equal(t.calls.some(([name])=>name==='export_document_build'),false);
  assert.equal(t.context.shell.state.result.build_id,'settled');
  assert.equal(t.context.shell.state.busy,false);
});

test('removing the cancel control reproduces the native startup regression',()=>{
  const markup=html.replace(/<button id="cancel-compile"[^>]*>.*?<\/button>/,'');
  assert.notEqual(markup,html);
  assert.throws(()=>boot({markup}),/Cannot set properties of null/);
});

test('the visible cancel control fences the active job through the production handler',async()=>{
  const t=boot();await t.context.startup;
  const {state,updateControls}=t.context.shell;let cancelled=0;
  state.client={cancel:async()=>{cancelled++}};state.busy=true;state.pending='auto';
  const epoch=state.buildEpoch;updateControls();
  assert.equal(t.elements.get('cancel-compile').disabled,false);
  await t.elements.get('cancel-compile').onclick();
  assert.equal(cancelled,1);assert.equal(state.cancelRequested,true);
  assert.equal(state.buildEpoch,epoch+1);assert.equal(state.pending,false);
  state.mutating=true;updateControls();assert.equal(t.elements.get('cancel-compile').disabled,true);
});

test('an initial native open failure leaves a visible explanation and working retry control',async()=>{
  const t=boot({openFailure:'Project folder is unavailable'});await t.context.startup;
  assert.equal(t.context.shell.state.project,null);assert.equal(t.options.readOnly,false);
  assert.equal(t.elements.get('status').textContent,'Could not open project');
  assert.match(t.elements.get('banner').textContent,/Project folder is unavailable.*Open project/);
  assert.equal(t.elements.get('banner').classList.contains('hidden'),false);
  assert.equal(t.elements.get('open').disabled,false);
  assert.equal(typeof t.elements.get('open').onclick,'function');
});

test('the shipped setup help explains a known failure without invoking native operations',async()=>{
  const t=boot();await t.context.startup;
  const calls=t.calls.length,before=structuredClone(t.context.shell.state.sources);
  t.context.shell.showFailure(Error('xelatex was not found. Install MiKTeX or TeX Live, then reopen Texloom.'));
  assert.equal(t.elements.get('setup-error-help').classList.contains('hidden'),false);
  t.elements.get('setup-error-help').onclick();
  assert.equal(t.elements.get('setup-dialog').open,true);
  assert.equal(t.elements.get('setup-context').textContent,'LaTeX compiler not found');
  assert.match(t.elements.get('setup-steps').children.map(c=>c.textContent).join(' '),/reopen Texloom/i);
  assert.equal(t.calls.length,calls);assert.deepEqual({...t.context.shell.state.sources},before);
  t.handlers.keydown({ctrlKey:true,key:'s',target:{closest:()=>null},preventDefault(){assert.fail('A setup dialog must retain its own keyboard input')}});
  t.elements.get('setup-close').onclick();assert.equal(t.elements.get('setup-dialog').open,false);
  t.context.shell.showFailure(Error('Unsupported transformation'));
  assert.equal(t.elements.get('setup-error-help').classList.contains('hidden'),true);
  t.elements.get('setup-help').onclick();assert.equal(t.elements.get('setup-context').textContent,'Set up LaTeX for Texloom');
});

test('an invalid native snapshot cannot replace the existing editor or destroy its displayed PDF',async()=>{
  const t=boot({badSnapshot:true});let destroyed=0,aborted=0;
  const previous={id:'accepted',name:'Accepted'};
  Object.assign(t.context.shell.state,{project:previous,sources:{'main.tex':'Unsaved accepted text'},
    pdf:{destroy(){destroyed++}},renderAbort:{abort(){aborted++}}});
  t.model.value='Unsaved accepted text';
  await t.context.startup;
  assert.equal(t.context.shell.state.project,previous);
  assert.equal(t.context.shell.state.sources['main.tex'],'Unsaved accepted text');
  assert.equal(t.model.value,'Unsaved accepted text');assert.equal(destroyed,0);assert.equal(aborted,0);
  assert.deepEqual(t.calls.filter(([name])=>name==='close_document').map(([,args])=>args.projectId),['fixture']);
});

const pendingSave={id:'pending-save',state:'prepared',files:['main.tex']};

test('the interrupted Save confirmation excludes a competing recovery action',async()=>{
  const t=boot({pendingSave});await t.context.startup;
  const state=t.context.shell.state;
  state.saved={'main.tex':'Earlier saved text'};
  t.elements.get('recovery-choice').value=pendingSave.id;
  let resolveConfirm,confirmations=0;
  const decision=new Promise(resolve=>{resolveConfirm=resolve});
  t.context.confirm=()=>{confirmations++;return decision};
  const applying=t.elements.get('recovery-finish').onclick();
  const discarding=t.elements.get('recovery-discard').onclick();
  resolveConfirm(true);await Promise.all([applying,discarding]);
  assert.equal(confirmations,1);
  assert.equal(t.calls.filter(([name])=>name==='recover_document').length,1);
  assert.equal(t.calls.some(([name])=>name==='discard_document_recovery'),false);
  assert.equal(state.project.id,'recovered-save');assert.equal(state.saving,false);
});

test('a recovered Save with an invalid snapshot releases the rejected session',async()=>{
  const t=boot({pendingSave,badRecoverySnapshot:true});await t.context.startup;
  const state=t.context.shell.state,client=state.client;
  t.elements.get('recovery-choice').value=pendingSave.id;
  await t.elements.get('recovery-finish').onclick();
  assert.equal(state.client,client);assert.equal(state.project.id,'fixture');
  assert.deepEqual(t.calls.filter(([name])=>name==='close_document').map(([,args])=>args.projectId),['recovered-save']);
  assert.match(t.elements.get('toast').textContent,/could not be displayed/i);
  assert.equal(state.saving,false);
});

test('a refused layout value stays readable inside the open modal and cannot change source or history',async()=>{
  const explanation='Unsupported parameter value; use a simple bounded numeric value';
  const t=boot({adapterFailure:explanation});await t.context.startup;
  const {state,applyLayoutControl}=t.context.shell;
  state.result={verification_eligible:true};state.compiled=state.revision;
  const before=structuredClone(state.sources),revision=state.revision,callCount=t.calls.length;
  t.elements.get('controls-dialog').showModal();
  const context={identity:transactions.sessionIdentity(state),file:state.file,source:state.sources[state.file],sources:before,main:state.project.main};
  await applyLayoutControl(context,{control:0,key:'factor',value:'4',selection:null});
  assert.equal(t.elements.get('controls-dialog').open,true);
  assert.equal(t.elements.get('controls-message').textContent,explanation);
  assert.notEqual(t.elements.get('toast').textContent,explanation);
  assert.deepEqual({...state.sources},before);assert.equal(state.revision,revision);
  assert.equal(state.canUndo,false);assert.equal(state.canRedo,false);
  assert.deepEqual(t.calls.slice(callCount).map(([name])=>name),['plan_adapter']);
});

test('stale layout controls explain the refusal in the modal before asking native code to plan',async()=>{
  const t=boot();await t.context.startup;
  t.elements.get('controls-dialog').showModal();
  await t.context.shell.applyLayoutControl({identity:'obsolete'},{key:'factor',value:'1.15'});
  assert.match(t.elements.get('controls-message').textContent,/source changed.*Reopen Layout controls/);
  assert.equal(t.elements.get('controls-dialog').open,true);
  assert.equal(t.calls.some(([name])=>name==='plan_adapter'),false);
});
