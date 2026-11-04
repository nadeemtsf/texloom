import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import * as transactions from '../web/core/transactions.js';
import {DocumentClient,editorText,sameSources} from '../web/core/document-client.js';
import {TypingRecoveryController} from '../web/core/typing-recovery.js';
import {createWorkspaceInvoke} from '../web/core/workspace-protocol.js';
import {ProjectControls} from '../web/core/project-controls.js';
import {PDFSearch} from '../web/core/pdf-search.js';
import {SetupHelp} from '../web/core/setup-help.js';
import {installPerformanceDiagnostics,performanceOutcome} from '../web/core/performance-diagnostics.js';

const [app,html]=await Promise.all(['app.js','index.html'].map(name=>readFile(new URL('../web/'+name,import.meta.url),'utf8')));

// Run the complete production shell. IDs come only from the shipped HTML;
// looking up a missing control returns null, as it does in the actual WebView.
function boot({markup=html,openFailure=null,badSnapshot=false,adapterFailure=null}={}){
  const calls=[],elements=new Map(),options={},handlers={};
  function element(id=''){
    const classes=new Set();
    return {id,value:'',textContent:'',disabled:false,checked:false,open:false,children:[],
      classList:{add:n=>classes.add(n),remove:n=>classes.delete(n),contains:n=>classes.has(n),
        toggle(n,force){const enabled=force??!classes.has(n);if(enabled)classes.add(n);else classes.delete(n);return enabled}},
      setAttribute(){},addEventListener(){},replaceChildren(...children){this.children=children},
      append(...children){this.children.push(...children)},focus(){},showModal(){this.open=true},close(){this.open=false}};
  }
  for(const match of markup.matchAll(/\bid="([^"]+)"/g)){
    assert.ok(!elements.has(match[1]),'Duplicate HTML ID: '+match[1]);
    elements.set(match[1],element(match[1]));
  }
  const source='\\documentclass{article}\n\\begin{document}\nNative shell fixture\n\\end{document}\n';
  const project={id:'fixture',name:'Fixture',main:'main.tex',sources:{'main.tex':source},recoveries:[],recovery_error:null,restored:false,draft_warning:null,source_only:false};
  const snapshot={revision:0,sources:project.sources,main:project.main,engine:'auto',can_undo:false,can_redo:false,selection:null,recovery:{draft_id:null,protected_revision:null,error:null}};
  const model={value:'',setValue(v){this.value=v},setEOL(){},getValue(){return this.value}};
  const editor={createContextKey:()=>({set(){}}),updateOptions:o=>Object.assign(options,o),
    onDidChangeModelContent(){},onDidChangeCursorPosition(){},onDidChangeCursorSelection(){},
    addAction(){},addCommand(){},getDomNode:()=>element(),getSelections:()=>[],hasTextFocus:()=>false};
  const context={...transactions,DocumentClient,editorText,sameSources,TypingRecoveryController,createWorkspaceInvoke,ProjectControls,PDFSearch,SetupHelp,installPerformanceDiagnostics,performanceOutcome,structuredClone,console,
    self:{},pdfjs:{GlobalWorkerOptions:{}},pdfWorkerURL:'test-worker',EditorWorker:class{},
    monaco:{languages:{register(){},setMonarchTokensProvider(){},setLanguageConfiguration(){}},
      editor:{defineTheme(){},createModel:()=>model,create:()=>editor,EndOfLineSequence:{LF:0}},
      KeyMod:{CtrlCmd:1,Shift:2},KeyCode:{KeyS:3,Enter:4,KeyZ:5,KeyY:6}},
    CanvasController:class{clear(){}},
    document:{getElementById:id=>elements.get(id)||null,createElement:()=>element(),
      querySelectorAll:selector=>selector==='dialog[open]'?[...elements.values()].filter(e=>e.open):[],addEventListener:(name,fn)=>{handlers[name]=fn}},
    window:{addEventListener(){}},setTimeout:()=>1,clearTimeout(){},
    getCurrentWindow:()=>({onCloseRequested:fn=>{handlers.close=fn}}),
    open:async()=>null,save:async()=>null,confirm:async()=>true,
    nativeInvoke:async(name,args)=>{
      calls.push([name,args]);
      if(name==='open_project'){if(openFailure)throw Error(openFailure);return project}
      if(name==='document_snapshot')return badSnapshot?{...snapshot,sources:null}:snapshot;
      if(name==='typing_drafts')return {drafts:[],errors:[]};
      if(name==='plan_adapter'&&adapterFailure)throw Error(adapterFailure);
      throw Error('Test compiler unavailable');
    }};
  const script=app.replace(/^import .*;\r?$/gm,'').replace(/openProject\(\);\s*$/,'globalThis.startup = openProject();');
  vm.runInNewContext(script+'\nglobalThis.shell={state,updateControls,applyLayoutControl,showFailure};',context,{filename:'web/app.js'});
  return {context,calls,elements,options,model,handlers};
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
