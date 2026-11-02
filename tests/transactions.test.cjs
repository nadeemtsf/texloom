const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');

async function setup(){
 const transactions=await import('../web/core/transactions.js');
 const selectionProof=await import('../web/core/selection-proof.js');
 const {DocumentClient,editorText}=await import('../web/core/document-client.js');
 const elements=new Map();
 function element(id){if(!elements.has(id))elements.set(id,{value:'',checked:false,disabled:false,open:false,textContent:'',classList:{add(){},remove(){},toggle(){}},setAttribute(){},replaceChildren(){},append(){},addEventListener(){},focus(){},showModal(){this.open=true},close(){this.open=false}});return elements.get(id)}
 const model={setValue(){},setEOL(){},getValue:()=>'',getOffsetAt:p=>p.column-1};
 const composition={};
 const editor={onDidChangeModelContent(fn){control.onEdit=fn},onDidChangeCursorPosition(){},onDidChangeCursorSelection(fn){control.onSelection=fn},addCommand(){},
  createContextKey(name,value){const key={value,set(next){this.value=next}};(control.contextKeys??=new Map()).set(name,key);return key},
  addAction(action){(control.actions??=[]).push(action)},hasTextFocus:()=>false,updateOptions(){},getDomNode:()=>({addEventListener(name,fn){composition[name]=fn}})};
 const control={compile:async()=>({ok:true,log:'',seconds:0,engine:'test',build_id:'new-pdf',blocks:[],objects:[]}),prepare:async result=>({result,pdf:{destroy:async()=>{}},pageCount:1,objects:new Map(),unclosed:new Set(),pageSizes:new Map()})};
 const {NativeDocumentDouble}=require('./native-document-double.cjs');const native=new NativeDocumentDouble(control);
 const context=vm.createContext({
  ...transactions,...selectionProof,validateSelectionProbe:(...args)=>control.prove?control.prove(...args):selectionProof.validateSelectionProbe(...args),DocumentClient,editorText,initialSnapshot:native.snapshot(),inside:()=>true,self:{},EditorWorker:class{},pdfWorkerURL:'test-worker',
  CanvasController:class{clear(){}bind(){}selectObject(){}},
  monaco:{languages:{register(){},setMonarchTokensProvider(){},setLanguageConfiguration(){}},editor:{defineTheme(){},createModel:()=>model,create:()=>editor,EndOfLineSequence:{LF:0}},KeyMod:{CtrlCmd:1,Shift:2},KeyCode:{KeyS:1,Enter:2,KeyZ:3,KeyY:4}},
  pdfjs:{GlobalWorkerOptions:{}},structuredClone,crypto:require('node:crypto').webcrypto,
  document:{getElementById:element,createElement:()=>element('generated-'+Math.random()),querySelector:()=>null,querySelectorAll:()=>[],addEventListener(){}},window:{addEventListener(){}},
  getCurrentWindow:()=>({onCloseRequested(fn){control.onClose=fn}}),open:async()=>null,save:async()=>control.save?.()??null,confirm:async()=>false,
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

test('clean close marks the cache while refusing a dirty close preserves its recovery copies',async()=>{
 const t=await setup();let prevented=false;
 await t.control.onClose({preventDefault(){prevented=true}});
 assert.equal(t.control.cleanExits,1);assert.equal(prevented,false);
 t.run("state.sources['main.tex']='Unsaved edit'");
 await t.control.onClose({preventDefault(){prevented=true}});
 assert.equal(prevented,true);assert.equal(t.control.cleanExits,1);
});

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

test('context menu history follows native canvas commits after Monaco resets',async()=>{
 const t=await setup();const undoKey=t.control.contextKeys.get('texloomCanUndo'),redoKey=t.control.contextKeys.get('texloomCanRedo');
 assert.equal(undoKey.value,false);await t.run("visualTransaction('main.tex','Moved source')");
 assert.equal(undoKey.value,true);assert.equal(redoKey.value,false);
 await t.control.actions.find(a=>a.id==='texloom.undo').run();
 assert.equal(t.native.sources['main.tex'],'Original source');assert.equal(undoKey.value,false);assert.equal(redoKey.value,true);
 await t.control.actions.find(a=>a.id==='texloom.redo').run();assert.equal(t.native.sources['main.tex'],'Moved source');
 t.composition.compositionstart();assert.equal(undoKey.value,false);assert.equal(redoKey.value,false);
});

test('dependencies changing during PDF preparation reject the candidate and destroy its view',async()=>{
 const t=await setup();let destroyed=false;
 t.control.prepare=async()=>({pageCount:1,pdf:{destroy:async()=>{destroyed=true}}});
 t.control.verify=async()=>{throw new Error('Build dependency changed')};
 assert.equal(await t.run("visualTransaction('main.tex','Candidate')"),false);
 assert.equal(destroyed,true);assert.equal(t.native.history.length,0);assert.equal(t.run('state.result.build_id'),'last-good');
});

test('unresolved compiler diagnostics cannot commit visual source',async()=>{
 const t=await setup();t.control.compile=async()=>({ok:true,build_id:'unresolved',verification_eligible:false,log:'Unresolved references',
  diagnostics:[{code:'unresolved_reference',severity:'error',message:'Reference missing'}]});
 assert.equal(await t.run("visualTransaction('main.tex','Candidate')"),false);assert.equal(t.native.history.length,0);
});

test('cancellation after the compiler finishes still prevents PDF/source publication',async()=>{
 const t=await setup();let finish,destroyed=false;
 t.control.prepare=()=>new Promise(r=>finish=r);
 const pending=t.run("visualTransaction('main.tex','Candidate')");await new Promise(setImmediate);
 await t.run('cancelCompile()');finish({pageCount:1,pdf:{destroy:async()=>{destroyed=true}}});
 assert.equal(await pending,false);assert.equal(destroyed,true);assert.equal(t.native.history.length,0);
 assert.equal(t.run('state.pending'),false);assert.equal(t.run('state.result.build_id'),'last-good');
});

test('dismissing export dialog cannot finish a different compile job',async()=>{
 const t=await setup();let close;t.control.save=()=>new Promise(r=>close=r);
 const exporting=t.run("$('export').onclick()");await new Promise(setImmediate);
 t.run('state.busy=true');close(null);await exporting;assert.equal(t.run('state.busy'),true);
});

test('superseded auto builds wait for idle typing instead of restarting on each edit',async()=>{
 const t=await setup();let finish,calls=0;t.control.compile=()=>{calls++;return new Promise(r=>finish=r)};
 t.run("$('auto').checked=true");
 const old=t.run("compile('auto')");await new Promise(setImmediate);
 t.control.onEdit({changes:[{rangeOffset:0,rangeLength:15,text:'New source'}]});await t.run('state.client.flush()');
 finish({ok:true,build_id:'old-pdf'});await old;await new Promise(r=>setTimeout(r,30));
 assert.equal(calls,1);assert.equal(t.run('state.busy'),false);assert.equal(t.run('state.result.build_id'),'last-good');
 t.run("clearTimeout(state.compileTimer);$('auto').checked=false");
});

async function probeSetup(){
 const t=await setup();t.control.resolve=async()=>({schema:1,build_id:'last-good',revision:0,file:'main.tex',expected_source:'Original source',content:'Original source',start:0,end:15,kind:'paragraph'});
 t.control.plan=async()=>({sources:{'main.tex':'Private probe','studio.sty':'Style source'},file:'main.tex'});
 t.control.prove=()=>({schema:1,build_id:'last-good',page:1,x:20,y:30,width:100,height:12,runs:[],lines:[]});
 return t;
}
test('selection probing never commits canonical source, history or the displayed PDF',async()=>{
 const t=await probeSetup();let finish,destroyed=false;t.control.prepare=()=>new Promise(r=>finish=r);
 const pending=t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})");await new Promise(setImmediate);
 assert.equal(t.run("state.sources['main.tex']"),'Original source');assert.equal(t.native.history.length,0);assert.equal(t.run('state.result.build_id'),'last-good');
 finish({pdf:{destroy:async()=>{destroyed=true}}});const result=await pending;
 assert.equal(result.proof.schema,1);assert.equal(destroyed,true);assert.equal(t.native.pending.size,0);assert.equal(t.native.history.length,0);assert.equal(t.run('state.busy'),false);
});
test('a selection proof failure destroys the private PDF and discards its source token',async()=>{
 const t=await probeSetup();let destroyed=false;t.control.prepare=async()=>({pdf:{destroy:async()=>{destroyed=true}}});t.control.prove=()=>{throw new Error('Wrong source occurrence')};
 await assert.rejects(t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})"),/Wrong source occurrence/);
 assert.equal(destroyed,true);assert.equal(t.native.pending.size,0);assert.equal(t.native.history.length,0);assert.equal(t.run('state.result.build_id'),'last-good');
});
test('cancelling while the selection PDF is prepared cannot grant a selection',async()=>{
 const t=await probeSetup();let finish,destroyed=false;t.control.prepare=()=>new Promise(r=>finish=r);
 const pending=t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})");await new Promise(setImmediate);await t.run('cancelCompile()');
 finish({pdf:{destroy:async()=>{destroyed=true}}});await assert.rejects(pending,/source changed/);
 assert.equal(destroyed,true);assert.equal(t.native.pending.size,0);assert.equal(t.native.history.length,0);
});
test('a cached selection proof still checks baseline dependencies and avoids another compiler',async()=>{
 const t=await probeSetup();let builds=0;t.control.compile=async()=>{builds++;return {ok:true,build_id:'probe'}};
 await t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})");
 await t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})");assert.equal(builds,1);
 t.control.verify=async()=>{throw new Error('Changed asset')};
 await assert.rejects(t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})"),/Changed asset/);assert.equal(builds,1);
});

test('an unsupported first SyncTeX hint retries bounded samples before granting a proof',async()=>{
 const t=await probeSetup();let hints=0;t.control.resolve=async()=>{if(++hints===1)throw new Error('Structural source');return {schema:1,build_id:'last-good',revision:0,file:'main.tex',expected_source:'Original source',content:'Original source',start:0,end:15,kind:'paragraph'}};
 const result=await t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})");assert.equal(result.proof.schema,1);assert.equal(hints,2);assert.equal(t.native.history.length,0);
});
test('duplicate failed source hints do not trigger repeated private builds',async()=>{
 const t=await probeSetup();let builds=0;t.control.compile=async()=>{builds++;return {ok:true,build_id:'probe'}};t.control.prove=()=>{throw new Error('Pixel identity failed')};
 await assert.rejects(t.run("resolveSourceSelection({buildId:'last-good',page:1,points:[],hits:[]})"),/Pixel identity failed/);assert.equal(builds,1);assert.equal(t.native.pending.size,0);
});

async function literalSetup(){
 const t=await setup();t.control.describeText=async()=>({runs:[{id:0,text:'An',styles:[]},{id:1,text:'office',styles:['Bold']}],protected:['\\textbf']});
 await t.run("editText({kind:'source',file:'main.tex',content:'An \\\\textbf{office}',plain_text:null,start:0,end:18})");return t;
}
test('the protected editor submits only changed run IDs and plain text',async()=>{
 const t=await literalSetup();assert.equal(t.run('state.dialog.literalMode'),true);
 t.run("state.dialog.literalInputs[1].input.value='archive & files';textOperation=async(_selection,options)=>{self.literalSubmission=options;return true}");
 await t.run("$('edit-form').onsubmit({preventDefault(){}})");
 const value=t.run('self.literalSubmission');assert.equal(value.literalEdits.length,1);assert.equal(value.literalEdits[0].id,1);assert.equal(value.literalEdits[0].text,'archive & files');assert.equal(value.content,undefined);
});
test('switching to LaTeX retains protected field edits through the native preview',async()=>{
 const t=await literalSetup();let edits;t.control.previewText=async args=>{edits=args.edits;return 'An \\textbf{archive}'};
 t.run("state.dialog.literalInputs[1].input.value='archive'");await t.run("$('edit-latex').onclick()");
 assert.equal(edits[0].id,1);assert.equal(t.run('state.dialog.literalMode'),false);assert.equal(t.run("$('edit-text').value"),'An \\textbf{archive}');
});
test('a stale text model cannot open a dialog against a newer revision',async()=>{
 const t=await setup();let finish;t.control.describeText=()=>new Promise(r=>finish=r);
 const pending=t.run("editText({kind:'source',file:'main.tex',content:'Original source',plain_text:null})");await new Promise(setImmediate);t.run('state.revision++');finish({runs:[]});await pending;
 assert.equal(t.run('state.dialog'),null);
});
test('mode switching blocks submission until edits are transferred',async()=>{
 const t=await literalSetup();let finish;t.control.previewText=()=>new Promise(r=>finish=r);const pending=t.run("$('edit-latex').onclick()");await new Promise(setImmediate);
 assert.equal(t.run('state.dialog.switching'),true);assert.equal(t.run('state.dialog.literalInputs[0].input.disabled'),true);
 await t.run("$('edit-form').onsubmit({preventDefault(){}})");assert.equal(t.run("$('edit-dialog').open"),true);
 finish('An \\textbf{office}');await pending;assert.equal(t.run('state.dialog.switching'),false);
});
