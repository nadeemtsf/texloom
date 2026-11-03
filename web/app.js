import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import 'monaco-editor/esm/vs/editor/contrib/find/browser/findController.js';
import 'monaco-editor/esm/vs/editor/contrib/contextmenu/browser/contextmenu.js';
import 'monaco-editor/esm/vs/editor/contrib/multicursor/browser/multicursor.js';
import 'monaco-editor/esm/vs/editor/contrib/clipboard/browser/clipboard.js';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import * as pdfjs from 'pdfjs-dist';
import pdfWorkerURL from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {open,save,confirm} from '@tauri-apps/plugin-dialog';
import {preparePDFView,prepareLoadedPDF,publishPDFView} from './core/pdf-surface.js';
import {CanvasController} from './core/canvas-controller.js';
import {stageCandidate,sessionIdentity,StaleCandidate,CompilationFailure,compileDiagnostic} from './core/transactions.js';
import {inside,intersects} from './core/geometry.js';
import {DocumentClient,editorText,sameSources} from './core/document-client.js';
import {TypingRecoveryController} from './core/typing-recovery.js';
import {validateSelectionProbe} from './core/selection-proof.js';

self.MonacoEnvironment={getWorker:()=>new EditorWorker()};
pdfjs.GlobalWorkerOptions.workerSrc=pdfWorkerURL;
monaco.languages.register({id:'latex'});
monaco.languages.setMonarchTokensProvider('latex',{tokenizer:{root:[
  [/%.*$/,'comment'],[/\\[a-zA-Z@]+\*?|\\./,'keyword'],[/[{}\[\]]/,'delimiter'],[/\$[^$]*\$/,'string']
]}});
monaco.languages.setLanguageConfiguration('latex',{comments:{lineComment:'%'},brackets:[['{','}'],['[',']']],
  autoClosingPairs:[{open:'{',close:'}'},{open:'[',close:']'}]});
monaco.editor.defineTheme('texloom',{base:'vs',inherit:true,rules:[
  {token:'keyword',foreground:'426F4B'},{token:'comment',foreground:'89947C',fontStyle:'italic'},{token:'delimiter',foreground:'9C824D'}
],colors:{'editor.background':'#FCFCF8','editor.foreground':'#40543E','editorLineNumber.foreground':'#A7B398',
  'editor.selectionBackground':'#BCD5A880','editor.lineHighlightBackground':'#EEF4E6','editorCursor.foreground':'#365D3E'}});

const $=id=>document.getElementById(id);
const clone=value=>structuredClone(value);
const message=error=>typeof error==='string'?error:error?.message||String(error);
const PT_PER_MM=72/25.4;
const model=monaco.editor.createModel('','latex');
const editor=monaco.editor.create($('editor'),{model,theme:'texloom',automaticLayout:true,
  minimap:{enabled:false},fontFamily:'Consolas, monospace',fontSize:13,lineHeight:23,wordWrap:'on',
  scrollBeyondLastLine:false,renderWhitespace:'selection',padding:{top:18},
  unicodeHighlight:{ambiguousCharacters:false},accessibilitySupport:'auto'});
const state={
  project:null,sources:{},saved:{},file:'',engine:'auto',revision:0,compiled:-1,result:null,pdf:null,view:null,
  selected:null,objectSelection:null,lastHistory:0,typingGroup:0,suppress:false,client:null,
  canUndo:false,canRedo:false,opening:false,mutating:false,composing:false,
  buildEpoch:0,cancelRequested:false,
  recovery:null,restoredDirty:false,
  busy:false,pending:false,zoom:0,mode:'select',compileTimer:null,renderSerial:0,renderAbort:null,dialog:null
};
// Dedicated keys follow native history even when setValue clears Monaco's stack.
const undoContext=editor.createContextKey('texloomCanUndo',false);
const redoContext=editor.createContextKey('texloomCanRedo',false);
const surfaceElements={pages:$('pages'),scroller:$('preview-scroll'),count:$('page-count'),empty:$('empty')};
const canvas=new CanvasController({
  getSession:()=>state,
  resolve:args=>resolveSourceSelection(args),
  move:(selection,dx,dy)=>moveText(selection,dx,dy),
  edit:selection=>editText(selection),
  selected:selection=>showObjectInspector(selection),
  notify:text=>toast(text)
});
const typingRecovery=new TypingRecoveryController({invoke,document,getState:()=>state,isDirty:dirty,
  adopt:project=>openProject(null,project),notify:toast,confirm,saveCopy:saveProjectCopy,
  lock:()=>{state.saving=true;editorReadOnly();updateControls()},
  unlock:()=>{state.saving=false;editorReadOnly();updateControls()}});

function toast(text){
  $('toast').textContent=text;$('toast').classList.remove('hidden');clearTimeout(toast.timer);
  toast.timer=setTimeout(()=>$('toast').classList.add('hidden'),5500);
}
function dirty(){return state.restoredDirty||!!state.client?.pending||!!state.client?.error||!sameSources(state.sources,state.saved)}
function status(text){$('status').textContent=text||(dirty()?'Unsaved changes':'All changes saved')}
function acceptDocument(snapshot){
  state.sources={...snapshot.sources};state.revision=snapshot.revision;
  state.project.main=snapshot.main;state.engine=snapshot.engine;
  state.canUndo=snapshot.can_undo;state.canRedo=snapshot.can_redo;
  state.recovery=snapshot.recovery??state.recovery;
  status();updateControls();
}
function editorReadOnly(){editor.updateOptions({readOnly:state.opening||state.mutating||state.saving||!!state.client?.error});}
function syncFailed(error){
  editorReadOnly();
  $('banner').textContent='Source synchronization paused. Your editor text is retained; copy it before reopening the project. '+message(error);
  $('banner').classList.remove('hidden');updateControls();
}
function syncEditor(){
  state.suppress=true;model.setValue(editorText(state.sources[state.file]||''));
  model.setEOL(monaco.editor.EndOfLineSequence.LF);state.suppress=false;
  $('active-file').textContent=state.file;renderFiles();
}
function fresh(){return !!state.result&&state.result.verification_eligible!==false&&state.revision===state.compiled&&!state.busy&&!state.client?.pending&&!state.client?.error&&!state.mutating&&!state.saving&&!state.composing&&!state.opening}
function updateControls(){
  typingRecovery.renderProtection();
  const ready=fresh(),legacy=ready&&!!state.selected,object=ready&&!!state.objectSelection;
  for(const id of ['add-text','pin'])$(id).disabled=!ready;
  for(const id of ['apply-position','apply-size','apply-content','edit-block','delete-block'])$(id).disabled=!legacy;
  for(const id of ['object-apply','object-edit','object-source'])$(id).disabled=!object;
  $('object-detach').disabled=!object||state.objectSelection?.kind!=='object';
  $('compile').disabled=state.busy||!!state.saving;
  $('save').disabled=!!state.saving||state.busy||state.composing;
  $('open').disabled=!!state.saving;
  $('engine').disabled=!!state.saving;
  $('main-file').disabled=!!state.saving;
  $('cancel-compile').disabled=!state.busy||state.mutating;
  $('undo').disabled=!state.canUndo||state.mutating||state.saving||state.composing||!!state.client?.error;
  $('redo').disabled=!state.canRedo||state.mutating||state.saving||state.composing||!!state.client?.error;
  undoContext.set(!$('undo').disabled);redoContext.set(!$('redo').disabled);
  $('compile-state').textContent=state.busy?'Compiling candidate…':'';
  $('pages').classList.toggle('stale',state.revision!==state.compiled||state.busy);
  $('selection-hint').textContent=state.mode==='read'?'Drag over text to highlight and copy it.':
    state.result&&!state.result.objects?.length&&!state.result.blocks?.length?
      'Click text to select it. Shift-click or draw a box for a group.':
      'Select text or an object. Drag its frame; double-click to edit.';
}
function sourceChanged(revision){
  state.revision=revision??state.revision+1;canvas.clear();state.selected=null;renderInspector();status();scheduleCompile();updateControls();
}
function scheduleCompile(){clearTimeout(state.compileTimer);if($('auto').checked)state.compileTimer=setTimeout(()=>compile('auto'),800)}
function showFailure(error){
  if(state.cancelRequested){toast('Compilation cancelled.');return}
  if(error.name==='CompilationCancelled'){toast(error.message);return}
  if(error instanceof StaleCandidate){toast(error.message);if(state.pending!=='manual')state.pending=false;scheduleCompile();return}
  renderDiagnostics(error.diagnostics||[]);
  if(error.log)$('log').textContent=error.log;
  $('banner').textContent=message(error)+' Visual editing is paused until the current source compiles. Read text is still available.';
  $('banner').classList.remove('hidden');
}
function renderDiagnostics(diagnostics){
  $('diagnostics').replaceChildren();
  for(const d of diagnostics){
    const el=document.createElement(d.file?'button':'p');
    el.className='diagnostic '+d.severity;el.textContent=(d.file?d.file+(d.line?':'+d.line:'')+' — ':'')+d.message;
    if(d.file)el.onclick=async()=>{try{await state.client.flush();if(state.sources[d.file]===undefined)return;
      state.file=d.file;state.lastHistory=0;syncEditor();editor.setPosition({lineNumber:Math.min(d.line||1,model.getLineCount()),column:1});editor.focus();sendSelection();
    }catch(error){syncFailed(error)}};
    $('diagnostics').append(el);
  }
}
async function cancelCompile(){
  state.cancelRequested=true;state.buildEpoch++;state.pending=false;clearTimeout(state.compileTimer);
  try{await state.client?.cancel()}catch(error){toast(message(error))}
}
function renderFiles(){
  if(!state.project)return;
  $('file-list').replaceChildren();
  for(const name of Object.keys(state.sources).sort()){
    const button=document.createElement('button');button.textContent=name;button.title=name;
    button.classList.toggle('active',name===state.file);
    button.onclick=async()=>{try{await state.client.flush();state.lastHistory=0;state.file=name;syncEditor();sendSelection()}catch(error){syncFailed(error)}};$('file-list').append(button);
  }
  $('main-file').replaceChildren();
  for(const name of Object.keys(state.sources).filter(n=>n.endsWith('.tex'))){
    const option=document.createElement('option');option.value=name;option.textContent=name;$('main-file').append(option);
  }
  $('main-file').value=state.project.main;$('engine').value=state.engine;
}
async function openProject(path=null,supplied=null){
  if((state.busy||state.opening||state.saving)&&!supplied)return toast('Wait for the current operation before opening another project.');
  if(state.client&&!state.client.error)await state.client.flush();
  if(!supplied&&state.project&&dirty()&&!await confirm('Discard the current unsaved edits before opening another project?',{title:'Unsaved source',kind:'warning'}))return;
  state.opening=true;editorReadOnly();
  try{
    const project=supplied||await invoke('open_project',{path});
    const snapshot=await invoke('document_snapshot',{projectId:project.id});
    state.renderAbort?.abort();canvas.clear();const previous=state.pdf,previousProject=state.project?.id,previousRevision=state.client?.snapshot.revision;
    Object.assign(state,{project,sources:{...project.sources},saved:{...project.sources},file:project.main,
      revision:snapshot.revision,compiled:-1,result:null,pdf:null,view:null,selected:null,lastHistory:0,canUndo:false,canRedo:false,
      recovery:snapshot.recovery,restoredDirty:!!project.restored});
    const client=new DocumentClient(invoke,project.id,snapshot,{changed:next=>{if(state.client===client)acceptDocument(next)},failed:syncFailed});
    state.client=client;acceptDocument(snapshot);
    $('project-name').textContent=project.name;$('pages').replaceChildren();$('empty').classList.remove('hidden');
    $('banner').classList.add('hidden');$('log-panel').classList.add('hidden');$('log').textContent='';
    renderDiagnostics([]);
    syncEditor();renderInspector();status();updateControls();
    if(previous)await previous.destroy();
    if(previousProject)await invoke('close_document',{projectId:previousProject,revision:previousRevision,discardRecovery:true})
      .then(reply=>{if(reply?.warning)toast(reply.warning)}).catch(error=>toast(message(error)));
    state.opening=false;editorReadOnly();sendSelection();
    renderSaveRecoveries();
    await typingRecovery.refresh();
    if(project.restored)$('save-copy').classList.remove('hidden');
    if(project.source_only){
      $('banner').textContent='The original project folder is missing. Recovered source is editable. Save a copy writes source files; images and other missing assets must be restored separately.';
      $('banner').classList.remove('hidden');return;
    }
    if(!supplied&&(project.recoveries?.length||project.recovery_error)){
      $('save-copy').classList.remove('hidden');await showSaveRecovery();return;
    }
    await compile();
  }catch(error){
    toast(message(error));
    if(!state.project){
      status('Could not open project');
      $('banner').textContent='Could not open a project: '+message(error)+'. Choose Open project to try again.';
      $('banner').classList.remove('hidden');
    }
  }finally{state.opening=false;editorReadOnly();updateControls()}
}
async function saveProject(){
  if(!state.project||state.saving||state.opening||state.mutating||state.composing)return;
  if(state.busy)return toast('Wait for the current compilation before saving.');
  state.saving=true;editorReadOnly();updateControls();
  try{await state.client.flush();const sources={...state.sources};
    await state.client.save();state.saved=sources;state.restoredDirty=false;state.project.draft_warning=null;status();toast('Project source saved.')}
  catch(error){toast(message(error));$('save-copy').classList.remove('hidden');await refreshSaveRecoveries().catch(()=>{})}
  finally{state.saving=false;editorReadOnly();updateControls()}
}
async function saveProjectCopy(){
  if(!state.project||state.saving||state.opening||state.mutating||state.composing)return;
  if(state.busy)return toast('Wait for the current compilation before saving a copy.');
  state.saving=true;editorReadOnly();updateControls();
  try{
    await state.client.flush();
    const parent=await open({directory:true,multiple:false,title:'Choose a folder outside this project for its copy'});
    if(!parent)return;
    const path=await invoke('copy_document',{projectId:state.project.id,revision:state.revision,parent});
    toast((state.project.source_only?'Recovered source files saved to ':'Project copy saved to ')+path+'. The original project remains open.');
  }catch(error){toast(message(error))}
  finally{state.saving=false;editorReadOnly();updateControls()}
}
function renderSaveRecoveries(){
  const saves=state.project?.recoveries||[];
  $('recover-save').classList.toggle('hidden',!saves.length&&!state.project?.recovery_error);
  $('recovery-choice').replaceChildren();
  saves.forEach((draft,index)=>{const option=document.createElement('option');option.value=draft.id;
    option.textContent='Save draft '+(index+1)+' · '+draft.files.length+' file'+(draft.files.length===1?'':'s');$('recovery-choice').append(option)});
  renderRecoveryFiles();
}
function renderRecoveryFiles(){
  const draft=state.project?.recoveries?.find(item=>item.id===$('recovery-choice').value);
  $('recovery-files').textContent=draft?.files.join('\n')||'';
  $('recovery-warning').textContent=dirty()?
    'Recovery reopens files from disk. Save a project copy first if you want to retain the current unsaved editor version.':
    'Choose whether to finish the previous Save or restore its earlier files. Your project will reopen afterward.';
}
async function refreshSaveRecoveries(){
  if(!state.project)return [];
  const client=state.client,project=state.project;
  try{
    const saves=await invoke('project_recoveries',{projectId:project.id});
    if(state.client!==client)return [];
    project.recoveries=saves;project.recovery_error=null;renderSaveRecoveries();return saves;
  }catch(error){
    if(state.client===client){project.recovery_error=message(error);renderSaveRecoveries();$('save-copy').classList.remove('hidden')}
    throw error;
  }
}
async function showSaveRecovery(){
  if(!state.project||state.saving)return;
  try{
    await state.client.flush();const saves=await refreshSaveRecoveries();
    if(!saves.length)return toast('No interrupted Save needs recovery.');
    renderRecoveryFiles();if(!$('recovery-dialog').open)$('recovery-dialog').showModal();
  }catch(error){toast('Save recovery could not be read. Your editor text is retained. '+message(error))}
}
async function applySaveRecovery(finish){
  const draft=state.project?.recoveries?.find(item=>item.id===$('recovery-choice').value);
  if(!draft||state.saving)return;
  if(dirty()&&!await confirm('Recovery will reopen files from disk and discard the current unsaved editor version. Save a project copy first to retain it. Continue?',{title:'Reopen recovered files',kind:'warning'}))return;
  const client=state.client;let adopted=false;state.saving=true;editorReadOnly();updateControls();
  try{
    await client.flush();
    const project=await invoke('recover_document',{projectId:state.project.id,revision:state.revision,transactionId:draft.id,finish});
    if(state.client!==client){await invoke('close_document',{projectId:project.id});return}
    $('recovery-dialog').close();await openProject(null,project);adopted=true;
    toast(finish?'The interrupted Save was completed.':'Earlier files restored. The recovery draft is retained until you discard it.');
  }catch(error){toast(message(error))}
  finally{state.saving=false;editorReadOnly();updateControls()}
  if(adopted)await compile();
}
async function discardSaveRecovery(){
  const draft=state.project?.recoveries?.find(item=>item.id===$('recovery-choice').value);
  if(!draft||state.saving)return;
  if(!await confirm('Keep the current project files and discard this recovery draft? Edits stored only in the draft will be removed.',{title:'Discard save draft',kind:'warning'}))return;
  state.saving=true;editorReadOnly();updateControls();
  try{
    await state.client.flush();
    state.project.recoveries=await invoke('discard_document_recovery',{projectId:state.project.id,revision:state.revision,transactionId:draft.id});
    renderSaveRecoveries();if(!state.project.recoveries.length)$('recovery-dialog').close();
    toast('Save recovery draft discarded. Project files were kept.');
  }catch(error){toast(message(error))}
  finally{state.saving=false;editorReadOnly();updateControls()}
}
function compileCandidate(){return state.client.compile();}
const selectionProofs=new WeakMap();
async function resolveSourceSelection({buildId,page,points,hits}){
  if(!fresh()||buildId!==state.result.build_id)throw new StaleCandidate();
  const identity=sessionIdentity(state),baseline=state.view,client=state.client,seen=new Set();
  const assertCurrent=()=>{if(sessionIdentity(state)!==identity||state.view!==baseline)throw new StaleCandidate()};
  const variants=[points,...[0.5,0.85].map(fraction=>hits.map(h=>({x:h.x+h.width*fraction,y:h.baseline-h.height*0.35})))];
  state.busy=true;state.cancelRequested=false;clearTimeout(state.compileTimer);updateControls();
  try{
    let lastError=null;
    for(const sample of variants){
      let token=null,prepared=null;
      try{
        assertCurrent();
        const resolved=await invoke('resolve_text_selection',{projectId:state.project.id,revision:state.revision,buildId,page,points:sample});
        assertCurrent();
        if(resolved.schema!==1||resolved.build_id!==buildId||resolved.revision!==state.revision)throw new StaleCandidate();
        const key=JSON.stringify([resolved.file,resolved.start,resolved.end]);if(seen.has(key))continue;seen.add(key);
        const cached=selectionProofs.get(baseline)?.get(key);
        if(cached&&hits.every(h=>h.runs.every(r=>cached.runs.includes(r)))){
          await client.verify(buildId);assertCurrent();return {...resolved,proof:cached};
        }
        const id='selection-'+crypto.randomUUID().slice(0,8);
        const plan=await invoke('plan_text_object',{request:{sources:clone(state.sources),main:state.project.main,file:resolved.file,
          action:'wrap',id,selection:[resolved.start,resolved.end],expected:resolved.content,dx:0,dy:0}});
        assertCurrent();
        const candidate=await client.prepare(plan.sources,buildId);token=candidate.token;
        const staged=await stageCandidate({sources:plan.sources,revision:candidate.revision,identity,getIdentity:()=>sessionIdentity(state),
          compile:()=>client.compile(token),prepare:result=>preparePDFView(pdfjs,result,{width:$('preview-scroll').clientWidth,zoom:baseline.scale,mode:state.mode}),
          requireVerified:true,verify:result=>client.verify(result.build_id)});
        prepared=staged.prepared;
        const proof=validateSelectionProbe(baseline,prepared,id,hits,page);
        await client.verify(buildId);assertCurrent();
        let cache=selectionProofs.get(baseline);if(!cache){cache=new Map();selectionProofs.set(baseline,cache)}
        if(cache.size>=64)cache.delete(cache.keys().next().value);cache.set(key,proof);
        return {...resolved,proof};
      }catch(error){
        if(error instanceof StaleCandidate||error.name==='CompilationCancelled')throw error;
        assertCurrent();lastError=error;
      }finally{
        if(prepared?.pdf)await prepared.pdf.destroy().catch(()=>{});
        if(token)await client.discard(token).catch(()=>{});
      }
    }
    throw lastError||new Error('This text could not be mapped safely. Use source editing.');
  }finally{finishJob()}
}
function prepareResult(result){
  return preparePDFView(pdfjs,result,{width:$('preview-scroll').clientWidth,zoom:state.zoom,mode:state.mode});
}
function publishPrepared(prepared,result){
  const previous=state.pdf;
  state.renderSerial++;state.renderAbort?.abort();
  state.pdf=prepared.pdf;state.result={...result,pdf:[]};state.compiled=state.revision;
  prepared.result=state.result;prepared.mode=state.mode;state.view=prepared;
  publishPDFView(prepared,surfaceElements);canvas.bind(prepared);
  $('fit').textContent=state.zoom?Math.round(prepared.scale*100)+'%':'Fit';
  $('log').textContent=result.log;$('banner').classList.add('hidden');
  renderDiagnostics(result.diagnostics||[]);
  if(result.verification_eligible===false){$('banner').textContent='PDF compiled with unresolved diagnostics. Resolve them before visual editing or export.';$('banner').classList.remove('hidden')}
  renderOverlays();renderInspector();
  status((dirty()?'Unsaved':'Saved')+' · '+result.seconds.toFixed(2)+'s · '+result.engine);
  if(previous&&previous!==prepared.pdf)previous.destroy().catch(error=>toast(message(error)));
  state.client.pin(result.build_id).catch(error=>toast(message(error)));
}
async function compile(reason='manual'){
  if(!state.project)return;clearTimeout(state.compileTimer);
  if(state.composing||state.saving)return;
  if(state.busy){if(reason==='manual'||!state.pending)state.pending=reason;return}
  try{await state.client.flush()}catch(error){return syncFailed(error)}
  if(state.busy){if(reason==='manual'||!state.pending)state.pending=reason;return}
  const identity=sessionIdentity(state),revision=state.revision;state.busy=true;state.cancelRequested=false;updateControls();
  try{
    const candidate=await stageCandidate({sources:clone(state.sources),revision,identity,getIdentity:()=>sessionIdentity(state),
        compile:compileCandidate,prepare:prepareResult,policy:null,previousPageCount:state.view?.pageCount,verify:result=>state.client.verify(result.build_id)});
    publishPrepared(candidate.prepared,candidate.result);
  }catch(error){showFailure(error)}
  finally{finishJob()}
}
function finishJob(){
  state.busy=false;updateControls();
  if(state.pending){const reason=state.pending;state.pending=false;if(reason==='manual')setTimeout(()=>compile(),0);else scheduleCompile()}
}
async function commitSources(sources,{file=state.file,policy=null,operation=null,expectedIdentity=sessionIdentity(state)}={}){
  if(!fresh()||sessionIdentity(state)!==expectedIdentity)return toast('Compile the latest source before applying this edit.');
  const client=state.client;let nativeCandidate=null,prepared=null;
  state.busy=true;state.cancelRequested=false;clearTimeout(state.compileTimer);updateControls();
  try{
    nativeCandidate=await client.prepare(sources,state.result.build_id,operation);
    const candidate=await stageCandidate({sources,revision:nativeCandidate.revision,identity:expectedIdentity,getIdentity:()=>sessionIdentity(state),
      compile:()=>client.compile(nativeCandidate.token),prepare:prepareResult,policy,previousPageCount:state.view.pageCount,requireVerified:true,verify:result=>client.verify(result.build_id)});
    prepared=candidate.prepared;
    if(sessionIdentity(state)!==expectedIdentity)throw new StaleCandidate();
    state.mutating=true;editorReadOnly();
    await client.commit(nativeCandidate.token,candidate.result.build_id);
    state.file=file;state.lastHistory=0;syncEditor();publishPrepared(prepared,candidate.result);prepared=null;
    return true;
  }catch(error){showFailure(error);return false}
  finally{
    if(prepared?.pdf)await prepared.pdf.destroy();
    if(nativeCandidate)await client.discard(nativeCandidate.token).catch(()=>{});
    state.mutating=false;editorReadOnly();finishJob();
  }
}
function visualTransaction(file,source,request){
  const candidate=clone(state.sources);candidate[file]=source;
  return commitSources(candidate,{file,operation:{kind:'legacy',file,request}});
}
async function textOperation(selection,{dx,dy,content,literalEdits,spacingEdits,plain=false,detach=false}={}){
  if(!fresh())return toast('Compile the current source before changing the canvas.');
  if(selection.kind==='source'&&state.sources[selection.file]!==selection.expected_source)return toast('That source selection changed. Select it again.');
  const identity=sessionIdentity(state),id=selection.kind==='object'?selection.id:'text-'+crypto.randomUUID().slice(0,8);
  if((literalEdits||spacingEdits)&&!literalEdits?.length&&!spacingEdits?.length)return true;
  const isEdit=content!==undefined||literalEdits!==undefined||spacingEdits!==undefined;
  const action=detach?'detach':selection.kind==='source'?'wrap':isEdit?'edit':'move';
  try{
    const request={sources:clone(state.sources),main:state.project.main,file:selection.file,
      action,id,selection:selection.kind==='source'?[selection.start,selection.end]:null,
      expected:selection.content,
      dx:dx??selection.dx??0,dy:dy??selection.dy??0,content:content??null,literal_edits:literalEdits??null,spacing_edits:spacingEdits??null,plain};
    const plan=await invoke('plan_text_object',{request});
    if(sessionIdentity(state)!==identity)return toast('The source changed. Select the text again.');
    const kind=detach?'detach':spacingEdits?.length?'spacing':isEdit?'edit':'move';
    const selectedRuns=new Set(selection.kind==='object'?state.view.objects.get(id)?.runs:
      selection.baselineRuns||canvas.hits.flatMap(line=>line.runs));
    const unchangedRuns=state.view.pages.flatMap(page=>page.runs).filter(run=>!selectedRuns.has(run));
    const ok=await commitSources(plan.sources,{file:plan.file,expectedIdentity:identity,
      operation:{kind:'text',request},
      policy:{id,kind,unchangedRuns:kind==='move'?unchangedRuns:null}});
    if(ok&&!detach)canvas.selectObject(id);
    return ok;
  }catch(error){toast(message(error));return false}
}
function moveText(selection,dx,dy){
  if(!selection)return;
  const proposed={...selection,x:selection.x+dx,y:selection.y+dy};
  if(!inside(proposed,state.view.pageSizes.get(selection.page)))return toast('Keep the selected text inside the page.');
  return textOperation(selection,{dx:(selection.dx||0)+dx,dy:(selection.dy||0)+dy});
}
async function editText(selection=state.objectSelection){
  if(!selection||!fresh())return;
  const identity=sessionIdentity(state);let textModel=null;
  try{textModel=await invoke('describe_text_edit',{content:selection.content,document:state.sources[state.project.main]})}catch{}
  if(identity!==sessionIdentity(state)||!fresh())return;
  state.dialog={kind:'text-object',selection:clone(selection),identity,literalInputs:[],spacingInputs:[]};
  const simple=typeof selection.plain_text==='string';
  showEditor('Edit selected text',simple?selection.plain_text:selection.content,simple,!simple);
  if(textModel&&((!simple&&textModel.runs?.length)||textModel.spacing?.length)){
    state.dialog.literalMode=true;
    for(const run of textModel.runs){
      const label=document.createElement('label'),title=document.createElement('span'),input=document.createElement('textarea');
      title.textContent=run.styles.length?run.styles.join(' / '):'Text';input.value=run.text;input.rows=2;
      input.setAttribute('aria-label',title.textContent+' '+(run.id+1));label.className='literal-run-field';label.append(title,input);$('literal-fields').append(label);
      state.dialog.literalInputs.push({id:run.id,initial:run.text,input});
    }
    for(const gap of textModel.spacing||[]){
      const label=document.createElement('label'),title=document.createElement('span'),input=document.createElement('input');
      title.textContent='Extra line gap '+(gap.id+1)+(gap.value!==null?' ('+gap.unit+')':'');
      input.type=gap.value===null?'text':'number';input.step='any';input.value=gap.value===null?gap.source:String(gap.value);input.disabled=gap.value===null;
      input.setAttribute('aria-label',title.textContent);if(gap.reason)input.title=gap.reason;
      label.className='literal-run-field';label.append(title,input);$('spacing-fields').append(label);
      if(gap.value!==null)state.dialog.spacingInputs.push({id:gap.id,initial:String(gap.value),input});
    }
    $('spacing-fields').classList.toggle('hidden',!textModel.spacing?.length);
    $('literal-fields').classList.remove('hidden');$('edit-text').classList.add('hidden');$('plain-label').classList.add('hidden');$('edit-latex').classList.remove('hidden');
    $('edit-help').textContent='Formatting and macros stay unchanged. Line-gap changes adjust existing spacing and may reflow following content.';
    (state.dialog.literalInputs[0]||state.dialog.spacingInputs[0])?.input.focus();
  }
}
function literalChanges(context){return context.literalInputs.filter(run=>run.input.value!==run.initial).map(run=>({id:run.id,text:run.input.value}))}
function spacingChanges(context){return (context.spacingInputs||[]).filter(run=>!run.input.value.trim()||Number(run.input.value)!==Number(run.initial)).map(run=>({id:run.id,value:run.input.value}))}
function editInputs(context){return [...context.literalInputs,...context.spacingInputs||[]]}
function showObjectInspector(selection){
  state.objectSelection=selection;
  $('object-selection').classList.toggle('hidden',!selection);
  if(selection){
    state.selected=null;$('selection').classList.add('hidden');$('no-selection').classList.add('hidden');
    $('object-title').textContent=selection.kind==='source'?'Selected '+selection.unitKind:'Movable text';
    $('object-summary').textContent='Source: '+selection.file+'. Movement keeps its normal layout space.';
    $('object-dx').value=((selection.dx||0)/PT_PER_MM).toFixed(2);
    $('object-dy').value=((selection.dy||0)/PT_PER_MM).toFixed(2);
    $('object-preview').textContent=selection.plain_text??selection.content;
  }else if(!state.selected)$('no-selection').classList.remove('hidden');
  updateControls();
}
function showSource(selection){
  if(!selection)return;state.file=selection.file;syncEditor();
  const start=model.getPositionAt(selection.start),end=model.getPositionAt(selection.end);
  editor.setSelection(new monaco.Selection(start.lineNumber,start.column,end.lineNumber,end.column));
  editor.revealRangeInCenter(new monaco.Range(start.lineNumber,start.column,end.lineNumber,end.column));editor.focus();
}
async function renderPDF(){
  if(!state.pdf||!state.result)return;
  state.renderAbort?.abort();const abort=new AbortController();state.renderAbort=abort;
  const serial=++state.renderSerial,pdf=state.pdf,result=state.result;
  try{
    const view=await prepareLoadedPDF(pdfjs,pdf,result,{width:$('preview-scroll').clientWidth,zoom:state.zoom,mode:state.mode,signal:abort.signal});
    if(serial!==state.renderSerial||pdf!==state.pdf)return;
    state.view=view;publishPDFView(view,surfaceElements);canvas.bind(view);
    $('fit').textContent=state.zoom?Math.round(view.scale*100)+'%':'Fit';renderOverlays();updateControls();
  }catch(error){if(!abort.signal.aborted)toast(message(error))}
}

// Legacy absolute-positioned blocks retain their existing dimension controls.
async function patchBlock(changes,block=state.selected){
  if(!block||!fresh())return;
  const identity=sessionIdentity(state);
  try{
    const request={source:state.sources[block.file],action:'patch',id:block.id,changes};
    const source=await invoke('transform',{request});
    if(identity===sessionIdentity(state))await visualTransaction(block.file,source,request);
  }catch(error){toast(message(error));renderInspector()}
}
function selectBlock(block){canvas.clear();state.selected={...block};renderInspector();document.querySelectorAll('.block').forEach(el=>el.classList.toggle('selected',el.dataset.id===block.id&&el.dataset.file===block.file));updateControls()}
function renderInspector(){
  $('selection').classList.toggle('hidden',!state.selected);
  $('no-selection').classList.toggle('hidden',!!state.selected||!!state.objectSelection);
  if(!state.selected)return;
  const current=state.result?.blocks.find(b=>b.id===state.selected.id&&b.file===state.selected.file);
  if(current)state.selected={...current};
  const block=state.selected;$('block-name').textContent=block.id;
  for(const field of ['x','y','width','size','page'])$(field).value=Number(block[field].toFixed(2));
  $('content').value=block.content;
}
function renderOverlays(){
  document.querySelectorAll('.block').forEach(el=>el.remove());
  for(const block of state.result?.blocks||[]){
    const page=document.querySelector('.pdf-page[data-page="'+block.page+'"]');if(!page)continue;
    const factor=Number(page.dataset.scale),el=document.createElement('div');
    el.className='block';el.dataset.id=block.id;el.dataset.file=block.file;
    el.classList.toggle('selected',state.selected?.id===block.id&&state.selected?.file===block.file);
    Object.assign(el.style,{left:block.x*factor+'px',top:block.y*factor+'px',width:block.width*factor+'px',height:Math.max(8,block.height)*factor+'px'});
    el.tabIndex=0;el.setAttribute('role','button');el.setAttribute('aria-label','Move '+block.id);
    const label=document.createElement('span');label.className='block-title';label.textContent=block.id+' · drag to move';
    const handle=document.createElement('span');handle.className='handle';handle.title='Resize block width';el.append(label,handle);
    el.onclick=e=>{e.stopPropagation();selectBlock(block)};
    el.ondblclick=e=>{e.stopPropagation();selectBlock(block);editBlock(block)};
    el.onpointerdown=e=>startBlockDrag(e,block,el,page);
    el.onkeydown=e=>{const step=e.shiftKey?10:1,changes={ArrowLeft:{x:block.x-step},ArrowRight:{x:block.x+step},ArrowUp:{y:block.y-step},ArrowDown:{y:block.y+step}};
      if(changes[e.key]){e.preventDefault();patchBlock(changes[e.key],block)}else if(e.key==='Enter')editBlock(block)};
    page.append(el);
  }
}
function startBlockDrag(event,block,el,page){
  if(event.button!==0||state.mode==='read'||!fresh())return;
  event.preventDefault();event.stopPropagation();canvas.clear();state.selected={...block};renderInspector();
  document.querySelectorAll('.block').forEach(e=>e.classList.remove('selected'));el.classList.add('selected');
  const resize=event.target.classList.contains('handle'),factor=Number(page.dataset.scale),start=[event.clientX,event.clientY];let changes=null;
  el.setPointerCapture(event.pointerId);
  const move=e=>{const dx=(e.clientX-start[0])/factor,dy=(e.clientY-start[1])/factor;if(Math.abs(dx)+Math.abs(dy)<1)return;
    const snap=v=>e.altKey?Math.round(v*100)/100:Math.round(v/4)*4;
    if(resize){changes={width:Math.max(12,snap(block.width+dx))};el.style.width=changes.width*factor+'px'}
    else{changes={x:Math.max(0,snap(block.x+dx)),y:Math.max(0,snap(block.y+dy))};el.style.left=changes.x*factor+'px';el.style.top=changes.y*factor+'px'}};
  const clean=()=>{el.removeEventListener('pointermove',move);el.removeEventListener('pointerup',up);el.removeEventListener('pointercancel',cancel)};
  const up=()=>{clean();if(changes)patchBlock(changes,block)},cancel=()=>{clean();renderOverlays()};
  el.addEventListener('pointermove',move);el.addEventListener('pointerup',up);el.addEventListener('pointercancel',cancel);
}
function showEditor(title,value,plain,locked=false){
  $('spacing-fields').replaceChildren();$('spacing-fields').classList.add('hidden');
  $('literal-fields').replaceChildren();$('literal-fields').classList.add('hidden');$('edit-latex').classList.add('hidden');$('edit-text').classList.remove('hidden');$('plain-label').classList.remove('hidden');
  $('edit-help').textContent='Formatted content opens as LaTeX so its commands are preserved.';
  $('edit-title').textContent=title;$('edit-text').value=value;$('plain').checked=plain;$('plain').disabled=locked;$('edit-dialog').showModal();$('edit-text').focus();
}
function editBlock(block=state.selected){
  if(!block)return;state.dialog={kind:'block',block:{...block},original:state.sources[block.file]};
  const simple=!/[\\{}%]/.test(block.content);showEditor('Edit '+block.id,block.content,simple,!simple);
}
function currentPage(){const top=$('preview-scroll').getBoundingClientRect().top;return Number([...document.querySelectorAll('.pdf-page')].find(p=>p.getBoundingClientRect().bottom>top+50)?.dataset.page||1)}
async function insertBlock({content='',plain=false,selection=null,page=currentPage(),x=62,y=100,width=270,size=12}={}){
  const id='block-'+crypto.randomUUID().slice(0,8),file=state.project.main,identity=sessionIdentity(state);
  try{const request={source:state.sources[file],action:'insert',id,content,plain,selection,page,x,y,width,size};
    const source=await invoke('transform',{request});
    if(identity!==sessionIdentity(state))return toast('The source changed. Retry the insertion.');
    if(await visualTransaction(file,source,request)){const block=state.result.blocks.find(b=>b.id===id);if(block)selectBlock(block)}
  }catch(error){toast(message(error))}
}
async function pinSelection(){
  if(state.file!==state.project.main)return toast('Select a complete source fragment in the main document.');
  const selection=editor.getSelection();if(!selection||selection.isEmpty())return toast('Select complete source content, or click text on the PDF.');
  const start=model.getOffsetAt(selection.getStartPosition()),end=model.getOffsetAt(selection.getEndPosition());
  const identity=sessionIdentity(state);
  try{const mapped=await invoke('pdf_from_source',{buildId:state.result.build_id,file:state.file,line:selection.startLineNumber});
    if(identity===sessionIdentity(state))await insertBlock({selection:[start,end],page:mapped.page,x:mapped.x,y:mapped.y,width:Math.min(500,mapped.width)});
  }catch(error){toast(message(error))}
}
async function undo(redo=false){
  if(!state.client||state.mutating||state.saving||state.composing||state.client.error)return;
  state.mutating=true;editorReadOnly();updateControls();
  try{
    await state.client.flush();if(!(redo?state.canRedo:state.canUndo))return;
    const before=clone(state.sources),snapshot=await state.client.history(redo);
    const changed=Object.keys(snapshot.sources).find(name=>snapshot.sources[name]!==before[name]);
    state.file=snapshot.selection?.file||changed|| (snapshot.sources[state.file]!==undefined?state.file:snapshot.main);
    state.lastHistory=0;state.selected=null;syncEditor();sourceChanged(snapshot.revision);renderInspector();
    if(snapshot.selection){
      const selections=[snapshot.selection,...(snapshot.selection.secondary||[])].map(range=>{
        const anchor=model.getPositionAt(range.anchor),head=model.getPositionAt(range.head);
        return {selectionStartLineNumber:anchor.lineNumber,selectionStartColumn:anchor.column,
          positionLineNumber:head.lineNumber,positionColumn:head.column};
      });
      state.suppress=true;editor.setSelections(selections);state.suppress=false;
      editor.revealPositionInCenter(model.getPositionAt(snapshot.selection.head));
    }
  }catch(error){syncFailed(error)}finally{state.mutating=false;editorReadOnly();updateControls()}
}
function escapeLatex(text){
  const map={'\\':'\\textbackslash{}','{':'\\{','}':'\\}','%':'\\%','&':'\\&','#':'\\#','_':'\\_','$':'\\$','~':'\\textasciitilde{}','^':'\\textasciicircum{}'};
  return [...text].map(c=>map[c]||c).join('').replace(/\n\n/g,'\n\\par\n');
}
function setMode(value){
  state.mode=value;$('pages').classList.toggle('read-mode',value==='read');
  $('select-mode').classList.toggle('active',value==='select');$('read-mode').classList.toggle('active',value==='read');
  $('select-mode').setAttribute('aria-pressed',String(value==='select'));$('read-mode').setAttribute('aria-pressed',String(value==='read'));
  updateControls();
}

editor.onDidChangeModelContent(event=>{
  if(state.suppress||!state.project)return;
  if(Date.now()-state.lastHistory>650)state.typingGroup++;
  state.lastHistory=Date.now();
  const changes=event.changes.map(c=>({offset:c.rangeOffset,length:c.rangeLength,text:c.text}));
  state.client.edit(state.file,changes,state.typingGroup).catch(()=>{});sourceChanged();
});
// Monaco's context-menu and accessibility undo paths reach the model methods.
// Route those through the same native history as keyboard and toolbar actions.
model.undo=()=>undo();model.redo=()=>undo(true);
editor.addAction({id:'texloom.undo',label:'Undo',precondition:'texloomCanUndo',contextMenuGroupId:'1_modification',contextMenuOrder:1,run:()=>undo()});
editor.addAction({id:'texloom.redo',label:'Redo',precondition:'texloomCanRedo',contextMenuGroupId:'1_modification',contextMenuOrder:2,run:()=>undo(true)});
editor.getDomNode()?.addEventListener('compositionstart',()=>{state.composing=true;state.lastHistory=0;canvas.clear();updateControls()});
editor.getDomNode()?.addEventListener('compositionend',()=>{state.composing=false;state.lastHistory=0;scheduleCompile();updateControls()});
editor.onDidChangeCursorPosition(()=>{const position=editor.getPosition();$('cursor').textContent='Ln '+position.lineNumber+', Col '+position.column});
function sendSelection(selections=editor.getSelections()){
  if(state.suppress||!state.client||state.opening||state.mutating||state.client.error)return;
  const ranges=(selections||[]).map(s=>({
    anchor:model.getOffsetAt({lineNumber:s.selectionStartLineNumber,column:s.selectionStartColumn}),
    head:model.getOffsetAt({lineNumber:s.positionLineNumber,column:s.positionColumn})}));
  if(ranges.length)state.client.selection(state.file,ranges[0].anchor,ranges[0].head,ranges.slice(1)).catch(()=>{});
}
editor.onDidChangeCursorSelection(event=>{
  // Explicit cursor movement breaks a typing group; content-driven movement does not.
  if(event.reason===3)state.lastHistory=0;
  sendSelection([event.selection,...event.secondarySelections]);
});
editor.addCommand(monaco.KeyMod.CtrlCmd|monaco.KeyCode.KeyS,()=>saveProject());
editor.addCommand(monaco.KeyMod.CtrlCmd|monaco.KeyCode.Enter,()=>compile());
editor.addCommand(monaco.KeyMod.CtrlCmd|monaco.KeyCode.KeyZ,()=>undo());
editor.addCommand(monaco.KeyMod.CtrlCmd|monaco.KeyMod.Shift|monaco.KeyCode.KeyZ,()=>undo(true));
editor.addCommand(monaco.KeyMod.CtrlCmd|monaco.KeyCode.KeyY,()=>undo(true));
$('edit-form').onsubmit=async event=>{
  event.preventDefault();const context=state.dialog;if(context?.switching)return;
  const text=$('edit-text').value,plain=$('plain').checked;$('edit-dialog').close();
  if(context?.kind==='new')return insertBlock({content:text,plain});
  if(context?.kind==='text-object'){
    if(context.identity!==sessionIdentity(state))return toast('The source changed. Reopen the text editor.');
    if(context.literalMode)return textOperation(context.selection,{literalEdits:literalChanges(context),spacingEdits:spacingChanges(context)});
    return textOperation(context.selection,{content:text,plain});
  }
  if(context?.kind==='block'){
    if(state.sources[context.block.file]!==context.original)return toast('The source changed. Reopen the editor.');
    return patchBlock({content:plain?escapeLatex(text):text},context.block);
  }
};
$('cancel-edit').onclick=()=>$('edit-dialog').close();
$('edit-latex').onclick=async()=>{
  const context=state.dialog;if(!context?.literalMode)return;$('edit-latex').disabled=true;
  context.switching=true;for(const run of editInputs(context))run.input.disabled=true;
  try{
    const content=await invoke('preview_text_edit',{content:context.selection.content,document:state.sources[state.project.main],edits:literalChanges(context),spacingEdits:spacingChanges(context)});
    if(context!==state.dialog||context.identity!==sessionIdentity(state))throw new StaleCandidate();
    context.literalMode=false;$('edit-text').value=content;$('edit-text').classList.remove('hidden');$('literal-fields').classList.add('hidden');$('spacing-fields').classList.add('hidden');$('edit-latex').classList.add('hidden');
    $('plain-label').classList.remove('hidden');$('plain').checked=false;$('plain').disabled=true;$('edit-help').textContent='LaTeX mode: edit commands carefully. Changes are compiled before acceptance.';$('edit-text').focus();
  }catch(error){toast(message(error))}finally{context.switching=false;for(const run of editInputs(context))run.input.disabled=false;$('edit-latex').disabled=false}
};
$('compile').onclick=()=>compile();$('save').onclick=saveProject;$('save-copy').onclick=saveProjectCopy;$('recover-save').onclick=showSaveRecovery;$('undo').onclick=()=>undo();$('redo').onclick=()=>undo(true);$('pin').onclick=pinSelection;
$('cancel-compile').onclick=cancelCompile;
$('open').onclick=async()=>{try{const path=await open({multiple:false,filters:[{name:'LaTeX document',extensions:['tex']}]});if(typeof path==='string')await openProject(path)}catch(error){toast(message(error))}};
$('export').onclick=async()=>{
  if(!state.project||state.busy)return toast('Wait for the current operation before exporting.');
  let prepared=null,ownsJob=false;
  try{
    const path=await save({defaultPath:'document.pdf',filters:[{name:'PDF document',extensions:['pdf']}]});if(!path)return;
    await state.client.flush();if(state.busy)return toast('Wait for compilation before exporting.');
    state.busy=true;state.cancelRequested=false;ownsJob=true;clearTimeout(state.compileTimer);updateControls();
    const identity=sessionIdentity(state),revision=state.revision,client=state.client;
    const candidate=await stageCandidate({sources:clone(state.sources),revision,identity,getIdentity:()=>sessionIdentity(state),
      compile:()=>client.compile(null,'export'),prepare:prepareResult,policy:null,requireVerified:true,verify:result=>client.verify(result.build_id)});
    prepared=candidate.prepared;
    state.mutating=true;editorReadOnly();updateControls();
    await invoke('export_document_build',{projectId:state.project.id,revision,buildId:candidate.result.build_id,path});
    toast('PDF exported from a clean build of this source revision.');
  }catch(error){showFailure(error)}finally{if(prepared?.pdf)await prepared.pdf.destroy();if(ownsJob){state.mutating=false;editorReadOnly();finishJob()}}
};
$('engine').onchange=()=>{if(!state.project)return;state.engine=$('engine').value;state.client.profile(state.project.main,state.engine).catch(()=>{});sourceChanged()};
$('main-file').onchange=()=>{if(!state.project)return;state.project.main=$('main-file').value;state.client.profile(state.project.main,state.engine).catch(()=>{});sourceChanged()};
$('auto').onchange=()=>{if($('auto').checked&&state.compiled!==state.revision)compile('auto')};
$('add-text').onclick=()=>{state.dialog={kind:'new'};showEditor('Add a positioned text block','',true)};
$('edit-block').onclick=()=>editBlock();$('apply-content').onclick=()=>patchBlock({content:$('content').value});$('apply-size').onclick=()=>patchBlock({size:Number($('size').value)});
$('apply-position').onclick=()=>{const page=Number($('page').value);if(!Number.isInteger(page)||page<1||page>(state.pdf?.numPages||0))return toast('Choose an existing physical page.');patchBlock({x:Number($('x').value),y:Number($('y').value),width:Number($('width').value),page})};
$('delete-block').onclick=async()=>{const block=state.selected;if(!block||!fresh())return;
  const identity=sessionIdentity(state),request={source:state.sources[block.file],action:'delete',id:block.id};
  try{const source=await invoke('transform',{request});
    if(identity===sessionIdentity(state)&&await visualTransaction(block.file,source,request)){state.selected=null;renderInspector()}
  }catch(error){toast(message(error))}};
$('object-edit').onclick=()=>editText();
$('object-source').onclick=()=>showSource(state.objectSelection);
$('object-detach').onclick=()=>textOperation(state.objectSelection,{detach:true});
$('object-apply').onclick=()=>{const selection=state.objectSelection;if(!selection)return;
  moveText(selection,Number($('object-dx').value)*PT_PER_MM-(selection.dx||0),Number($('object-dy').value)*PT_PER_MM-(selection.dy||0))};
$('select-mode').onclick=()=>setMode('select');$('read-mode').onclick=()=>setMode('read');
$('zoom-in').onclick=()=>{state.zoom=Math.min(2.5,(state.zoom||state.view?.scale||1)+0.1);renderPDF()};
$('zoom-out').onclick=()=>{state.zoom=Math.max(0.2,(state.zoom||state.view?.scale||1)-0.1);renderPDF()};
$('fit').onclick=()=>{state.zoom=0;renderPDF()};
let resizeTimer;window.addEventListener('resize',()=>{clearTimeout(resizeTimer);resizeTimer=setTimeout(()=>{if(!state.zoom)renderPDF()},180)});
$('log-toggle').onclick=()=>$('log-panel').classList.toggle('hidden');$('close-log').onclick=()=>$('log-panel').classList.add('hidden');
$('banner').onclick=()=>$('log-panel').classList.remove('hidden');
let quitting=false;getCurrentWindow().onCloseRequested(async event=>{
  if(state.saving){event.preventDefault();toast('Wait for Save or recovery to finish before closing.');return}
  if(quitting||!dirty()){
    state.saving=true;editorReadOnly();updateControls();
    try {
      if(quitting&&state.client?.error){
        // An explicit discard may close an unsynchronized editor. Keep its
        // native draft because the last acknowledged revision may be stale.
        if(state.project)await invoke('close_document',{projectId:state.project.id});
      }else{
        await state.client?.flush();
        if(state.project){const reply=await invoke('close_document',{projectId:state.project.id,revision:state.client.snapshot.revision,discardRecovery:true});if(reply?.warning)toast(reply.warning)}
      }
      await invoke('mark_clean_exit');
    }catch(error){event.preventDefault();quitting=false;state.saving=false;editorReadOnly();updateControls();toast(message(error))}
    return;
  }event.preventDefault();
  if(await confirm('Close Texloom and discard unsaved source edits?',{title:'Unsaved source',kind:'warning'})){quitting=true;await getCurrentWindow().close()}
});
document.addEventListener('keydown',event=>{
  if(editor.hasTextFocus()||$('edit-dialog').open||$('recovery-dialog').open||$('draft-dialog').open||event.target.closest('input,textarea,select'))return;
  if(event.ctrlKey||event.metaKey){
    if(event.key.toLowerCase()==='s'){event.preventDefault();saveProject()}
    else if(event.key==='Enter'){event.preventDefault();compile()}
    else if(event.key.toLowerCase()==='z'){event.preventDefault();undo(event.shiftKey)}
    else if(event.key.toLowerCase()==='y'){event.preventDefault();undo(true)}
  }
});
$('recovery-choice').onchange=renderRecoveryFiles;
$('recovery-finish').onclick=()=>applySaveRecovery(true);
$('recovery-rollback').onclick=()=>applySaveRecovery(false);
$('recovery-copy').onclick=saveProjectCopy;
$('recovery-discard').onclick=discardSaveRecovery;
$('recovery-close').onclick=()=>{if(!state.saving)$('recovery-dialog').close()};
$('recovery-dialog').addEventListener('cancel',event=>{if(state.saving)event.preventDefault()});
setMode('select');
openProject();
