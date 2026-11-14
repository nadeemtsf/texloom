import {test} from 'node:test';
import assert from 'node:assert/strict';
import {selectionSourceRange,SourceHighlight} from '../web/core/source-highlight.js';
import {applyTextInputAppearance,renderProtectedCommands} from '../web/core/text-input-appearance.js';
import {positionDialogNear} from '../web/core/dialog-anchor.js';

test('source highlighting uses normalized UTF-16 boundaries without guessing a stale occurrence',()=>{
  const raw='\ufeffA\r\n😀Z',sources={'other.tex':raw};
  const selection={kind:'source',file:'other.tex',expected_source:raw,start:2,end:4};
  assert.deepEqual(selectionSourceRange(selection,sources),{file:'other.tex',text:'A\n😀Z',start:2,end:4});
  for(const change of [{start:3},{end:3},{start:-1},{end:99},{expected_source:'different'},{file:'missing.tex'}])
    assert.equal(selectionSourceRange({...selection,...change},sources),null);
});

test('automatic source link activates the matching file and decorates without changing cursor or focus',()=>{
  const state={file:'main.tex',sources:{'main.tex':'First','other.tex':'Second'}},events=[];let current=true,value='First';
  const model={getValue:()=>value,getPositionAt:offset=>({lineNumber:1,column:offset+1})};
  const editor={createDecorationsCollection:()=>({clear:()=>events.push('clear'),set:r=>events.push(['range',r[0].range])}),
    getModel:()=>model,revealRangeInCenterIfOutsideViewport:r=>events.push(['reveal',r]),
    focus:()=>assert.fail('Source link stole focus'),setSelection:()=>assert.fail('Source link moved cursor')};
  const link=new SourceHighlight({editor,getState:()=>state,isCurrent:()=>current,activateFile:file=>{state.file=file;value=state.sources[file]}});
  const selection={kind:'source',file:'other.tex',expected_source:'Second',start:0,end:6};
  assert.equal(link.update(selection),true);assert.equal(state.file,'other.tex');assert.equal(events[1][0],'range');
  current=false;assert.equal(link.update(selection),false);assert.equal(events.at(-1),'clear');
  current=true;value='Local changed text';assert.equal(link.update(selection),false);
  assert.deepEqual(state.sources,{'main.tex':'First','other.tex':'Second'});
});

test('failed source activation clears the hint and grants no editing authority',()=>{
  let clear=0;const state={file:'a.tex',sources:{'a.tex':'A','b.tex':'B'}};
  const link=new SourceHighlight({editor:{createDecorationsCollection:()=>({clear(){clear++},set(){assert.fail()}})},
    getState:()=>state,isCurrent:()=>true,activateFile(){throw Error('Editor could not switch')}});
  assert.equal(link.update({kind:'source',file:'b.tex',expected_source:'B',start:0,end:1}),false);
  assert.equal(state.file,'a.tex');assert.equal(clear,2);
});

test('format hints are allowlisted and reset for raw LaTeX mode',()=>{
  const input={style:{}};applyTextInputAppearance(input,{styles:['Bold','Italic','Roman','background:url(private)']});
  assert.equal(input.dir,'auto');assert.equal(input.style.fontWeight,'700');assert.equal(input.style.fontStyle,'italic');
  assert.equal(input.style.fontFamily,'Georgia, serif');assert.equal(JSON.stringify(input).includes('private'),false);
  applyTextInputAppearance(input,{plain:false});assert.equal(input.dir,'ltr');assert.equal(input.style.fontWeight,'');assert.equal(input.style.fontStyle,'');
});

test('protected command labels are literal bounded text with duplicate entries removed',()=>{
  const container={children:[],replaceChildren(){this.children=[]},append(item){this.children.push(item)},classList:{toggle(){}}};
  renderProtectedCommands({createElement:()=>({})},container,['<img onerror=bad>',...Array.from({length:10},(_,n)=>'\\command'+n),'<img onerror=bad>']);
  assert.equal(container.children.length,10);assert.equal(container.children[1].textContent,'<img onerror=bad>');
  assert.equal(container.children.at(-1).textContent,' and 3 more');assert.equal(container.children.some(x=>'innerHTML' in x),false);
});

test('editor is anchored only beside a visible target that fits the viewport',()=>{
  const dialog={open:true,style:{left:'old'},getBoundingClientRect:()=>({width:180,height:150})};
  assert.equal(positionDialogNear(dialog,{left:100,top:100,right:200,bottom:120},{width:800,height:600}),true);
  assert.equal(dialog.style.left,'212px');assert.equal(dialog.style.top,'100px');
  assert.equal(positionDialogNear(dialog,{left:100,top:100,right:200,bottom:120},{width:150,height:100}),false);
  assert.equal(dialog.style.left,'');
  assert.equal(positionDialogNear(dialog,{left:-300,top:-300,right:-100,bottom:-100},{width:800,height:600}),false);
});
