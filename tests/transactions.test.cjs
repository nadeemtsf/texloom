const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');

async function setup(){
 const transactions=await import('../web/core/transactions.js');
 const {DocumentClient,editorText}=await import('../web/core/document-client.js');
 const elements=new Map();
 function element(id){if(!elements.has(id))elements.set(id,{value:'',checked:false,disabled:false,open:false,textContent:'',classList:{add(){},remove(){},toggle(){}},setAttribute(){},replaceChildren(){},append(){},addEventListener(){}});return elements.get(id)}
 const model={setValue(){},setEOL(){},getValue:()=>'',getOffsetAt:p=>p.column-1};
 const composition={};
 const editor={onDidChangeModelContent(fn){control.onEdit=fn},onDidChangeCursorPosition(){},onDidChangeCursorSelection(fn){control.onSelection=fn},addCommand(){},hasTextFocus:()=>false,updateOptions(){},getDomNode:()=>({addEventListener(name,fn){composition[name]=fn}})};
 const control={compile:async()=>({ok:true,log:'',seconds:0,engine:'test',build_id:'new-pdf',blocks:[],objects:[]}),prepare:async result=>({result,pdf:{destroy:async()=>{}},pageCount:1,objects:new Map(),unclosed:new Set(),pageSizes:new Map()})};
 const {NativeDocumentDouble}=require('./native-document-double.cjs');const native=new NativeDocumentDouble(control);
 const context=vm.createContext({
  ...transactions,DocumentClient,editorText,initialSnapshot:native.snapshot(),inside:()=>true,self:{},EditorWorker:class{},pdfWorkerURL:'test-worker',
  CanvasController:class{clear(){}bind(){}selectObject(){}},
  monaco:{languages:{register(){},setMonarchTokensProvider(){},setLanguageConfiguration(){}},editor:{defineTheme(){},createModel:()=>model,create:()=>editor,EndOfLineSequence:{LF:0}},KeyMod:{CtrlCmd:1,Shift:2},KeyCode:{KeyS:1,Enter:2,KeyZ:3,KeyY:4}},
  pdfjs:{GlobalWorkerOptions:{}},structuredClone,crypto:require('node:crypto').webcrypto,
  document:{getElementById:element,querySelector:()=>null,querySelectorAll:()=>[],addEventListener(){}},window:{addEventListener(){}},
  getCurrentWindow:()=>({onCloseRequested(){}}),open:async()=>null,save:async()=>null,confirm:async()=>false,
  invoke:(name,args)=>native.invoke(name,args),
  setTimeout:(fn,ms)=>{const t=setTimeout(fn,ms);t.unref();return t},clearTimeout,console,
  preparePDFView:async(_pdfjs,result)=>control.prepare(result)
 });
 let script=fs.readFileSync(require('node:path').join(__dirname,'../web/app.js'),'utf8').replace(/^import .*;\r?$/gm,'');
 script=script.replace(/\nopenProject\(\);\s*$/,'\n');vm.runInContext(script,context);
 const run=s=>vm.runInContext(s,context);
 run("state.project={id:'test',main:'main.tex'};state.sources={'main.tex':'Original source','studio.sty':'Style source'};state.saved=structuredClone(state.sources);state.file='main.tex';state.revision=0;state.compiled=0;state.result={build_id:'last-good',blocks:[]};state.view={pageCount:1};syncEditor=()=>{};renderOverlays=()=>{};renderInspector=()=>{};publishPrepared=(prepared,result)=>{state.pdf=prepared.pdf;state.result=result;state.compiled=state.revision};");
 run("state.client=new DocumentClient(invoke,'test',initialSnapshot,{changed:acceptDocument,failed:syncFailed});");
 return {run,control,native,model,composition};
}

test('compiler failure leaves source, history, revision and last good PDF intact',async()=>{
 const t=await setup();t.control.compile=async()=>({ok:false,log:'main.tex:8: Undefined control sequence.'});
 assert.equal(await t.run("visualTransaction('main.tex','Broken candidate')"),false);
 assert.equal(t.run("state.sources['main.tex']"),'Original source');assert.equal(t.run('state.revision'),0);
 assert.equal(t.native.history.length,0);assert.equal(t.run('state.result.build_id'),'last-good');
});
test('source commits after PDF preparation and retains other files and undo snapshot',async()=>{
 const t=await setup();let finish;t.control.prepare=()=>new Promise(r=>finish=r);
 const pending=t.run("visualTransaction('main.tex','Moved source')");await new Promise(setImmediate);
 assert.equal(t.run("state.sources['main.tex']"),'Original source');assert.equal(t.native.history.length,0);
 finish({pageCount:1,pdf:{}});assert.equal(await pending,true);
 assert.equal(t.run("state.sources['main.tex']"),'Moved source');assert.equal(t.run("state.sources['studio.sty']"),'Style source');
 assert.equal(t.native.history[0].sources['main.tex'],'Original source');assert.equal(t.run('state.revision'),1);
});
test('typing during compile prevents the older candidate from committing',async()=>{
 const t=await setup();let finish;t.control.compile=()=>new Promise(r=>finish=r);
 const pending=t.run("visualTransaction('main.tex','Older move')");
 await new Promise(setImmediate);
 t.run("state.sources['main.tex']='Newer typed source';state.revision++");finish({ok:true,build_id:'old-pdf'});await pending;
 assert.equal(t.run("state.sources['main.tex']"),'Newer typed source');assert.equal(t.run('state.result.build_id'),'last-good');
});
test('late PDF preparation is destroyed when source changes',async()=>{
 const t=await setup();let finish,destroyed=false;t.control.prepare=()=>new Promise(r=>finish=r);
 const pending=t.run("visualTransaction('main.tex','Older move')");await new Promise(setImmediate);
 t.run('state.revision++');finish({pdf:{destroy:async()=>{destroyed=true}}});await pending;
 assert.equal(destroyed,true);assert.equal(t.run('state.result.build_id'),'last-good');assert.equal(t.native.history.length,0);
});
test('PDF parse/render failure preserves the entire accepted document',async()=>{
 const t=await setup();t.control.prepare=async()=>{throw new Error('PDF render failed')};
 await t.run("visualTransaction('main.tex','Candidate')");assert.equal(t.run("state.sources['main.tex']"),'Original source');
 assert.equal(t.run('state.compiled'),0);assert.equal(t.run('state.result.build_id'),'last-good');
});
test('layout validation rejection destroys prepared PDF and does not add undo history',async()=>{
 const t=await setup();let destroyed=false;
 t.control.prepare=async()=>({pdf:{destroy:async()=>{destroyed=true}},pageCount:2,objects:new Map()});
 await t.run("commitSources({'main.tex':'Candidate'},{policy:{id:'name',kind:'move'}})");
 assert.equal(destroyed,true);assert.equal(t.native.history.length,0);assert.equal(t.run('state.revision'),0);
});
test('shared undo and redo restore source, main file and engine',async()=>{
 const t=await setup();await t.run("visualTransaction('main.tex','Moved source')");
 await t.run('undo()');assert.equal(t.run("state.sources['main.tex']"),'Original source');
 await t.run('undo(true)');assert.equal(t.run("state.sources['main.tex']"),'Moved source');
});

test('Monaco source events and model undo/redo reach the same native history',async()=>{
 const t=await setup();
 t.control.onEdit({changes:[{rangeOffset:0,rangeLength:15,text:'Typed source'}]});
 await t.run('state.client.flush()');
 assert.equal(t.native.sources['main.tex'],'Typed source');assert.equal(t.run("state.sources['main.tex']"),'Typed source');
 await t.model.undo();assert.equal(t.native.sources['main.tex'],'Original source');
 await t.model.redo();assert.equal(t.native.sources['main.tex'],'Typed source');
 assert.equal(t.run('state.revision'),3);
});

test('a successful compiler result for the wrong revision cannot publish or commit',async()=>{
 const t=await setup();t.control.compile=async()=>({ok:true,build_id:'wrong-revision',revision:999});
 assert.equal(await t.run("visualTransaction('main.tex','Candidate')"),false);
 assert.equal(t.native.history.length,0);assert.equal(t.run('state.result.build_id'),'last-good');
});

test('primary and secondary Monaco cursors reach the ordered native metadata protocol',async()=>{
 const t=await setup();
 t.control.onSelection({reason:3,selection:{selectionStartLineNumber:1,selectionStartColumn:2,positionLineNumber:1,positionColumn:4},secondarySelections:[{selectionStartLineNumber:1,selectionStartColumn:8,positionLineNumber:1,positionColumn:9}]});
 await t.run('state.client.flush()');
 const s=t.run('state.client.snapshot.selection');assert.equal(s.anchor,1);assert.equal(s.head,3);
 assert.equal(s.secondary[0].anchor,7);assert.equal(s.secondary[0].head,8);assert.equal(t.run('state.revision'),0);
});

test('composition pauses canvas actions, compilation and history until it ends',async()=>{
 const t=await setup();let builds=0;t.control.compile=async()=>{builds++;throw new Error('Not expected');};
 t.composition.compositionstart();assert.equal(t.run('fresh()'),false);
 await t.run('compile()');await t.model.undo();assert.equal(builds,0);
 t.composition.compositionend();assert.equal(t.run('state.composing'),false);
});
