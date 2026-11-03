import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import * as transactions from '../web/core/transactions.js';
import {DocumentClient,editorText,sameSources} from '../web/core/document-client.js';

const [app,html]=await Promise.all(['app.js','index.html'].map(name=>readFile(new URL('../web/'+name,import.meta.url),'utf8')));

// Run the complete production shell. IDs come only from the shipped HTML;
// looking up a missing control returns null, as it does in the actual WebView.
function boot({markup=html,openFailure=null}={}){
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
  const project={id:'fixture',name:'Fixture',main:'main.tex',sources:{'main.tex':source},recoveries:[]};
  const snapshot={revision:0,sources:project.sources,main:project.main,engine:'auto',can_undo:false,can_redo:false,selection:null};
  const model={value:'',setValue(v){this.value=v},setEOL(){},getValue(){return this.value}};
  const editor={createContextKey:()=>({set(){}}),updateOptions:o=>Object.assign(options,o),
    onDidChangeModelContent(){},onDidChangeCursorPosition(){},onDidChangeCursorSelection(){},
    addAction(){},addCommand(){},getDomNode:()=>element(),getSelections:()=>[],hasTextFocus:()=>false};
  const context={...transactions,DocumentClient,editorText,sameSources,structuredClone,console,
    self:{},pdfjs:{GlobalWorkerOptions:{}},pdfWorkerURL:'test-worker',EditorWorker:class{},
    monaco:{languages:{register(){},setMonarchTokensProvider(){},setLanguageConfiguration(){}},
      editor:{defineTheme(){},createModel:()=>model,create:()=>editor,EndOfLineSequence:{LF:0}},
      KeyMod:{CtrlCmd:1,Shift:2},KeyCode:{KeyS:3,Enter:4,KeyZ:5,KeyY:6}},
    CanvasController:class{clear(){}},
    document:{getElementById:id=>elements.get(id)||null,createElement:()=>element(),
      querySelectorAll:()=>[],addEventListener:(name,fn)=>{handlers[name]=fn}},
    window:{addEventListener(){}},setTimeout:()=>1,clearTimeout(){},
    getCurrentWindow:()=>({onCloseRequested:fn=>{handlers.close=fn}}),
    open:async()=>null,save:async()=>null,confirm:async()=>true,
    invoke:async(name,args)=>{
      calls.push([name,args]);
      if(name==='open_project'){if(openFailure)throw Error(openFailure);return project}
      if(name==='document_snapshot')return snapshot;
      throw Error('Test compiler unavailable');
    }};
  const script=app.replace(/^import .*;\r?$/gm,'').replace(/openProject\(\);\s*$/,'globalThis.startup = openProject();');
  vm.runInNewContext(script+'\nglobalThis.shell={state,updateControls};',context,{filename:'web/app.js'});
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
