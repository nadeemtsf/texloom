import {test} from 'node:test';
import assert from 'node:assert/strict';
import {CanvasController} from '../web/core/canvas-controller.js';

// Exercise the actual controller with a small event/DOM boundary, not another
// gesture implementation. Native Windows pointer acceptance remains separate.
class Element{
 constructor(){this.style={};this.events=new Map();this.classList={add(){},remove(){}};this.removed=false;this.width=0;this.height=0;}
 addEventListener(name,fn){this.events.set(name,fn)}
 removeEventListener(name,fn){if(this.events.get(name)===fn)this.events.delete(name)}
 setAttribute(){}
 setPointerCapture(id){this.capture=id}
 focus(options){this.focused=options}
 getContext(){return {drawImage(){}}}
 remove(){this.removed=true}
}
function setup({scale=1,mode='select',revision=0}={}){
 const generated=[];globalThis.document={addEventListener(){},removeEventListener(){},querySelectorAll:()=>[],createElement:()=>{const e=new Element();generated.push(e);return e}};
 const state={mode,revision,compiled:0,result:{build_id:'current'},sources:{'main.tex':'Text'}};
 const operations=[],notifications=[],eventWindow=new Element();
 const c=new CanvasController({getSession:()=>state,resolve:async()=>{},move:async(...args)=>operations.push(args),edit(){},selected(){},notify:t=>notifications.push(t),eventWindow});
 c.drawFrame=()=>{};
 const canvas=new Element();canvas.width=1000;canvas.style.width='500px';
 const page={querySelector:()=>canvas,append(){}};const frame=new Element();frame.closest=()=>page;
 const selection={kind:'source',page:1,x:20,y:50,width:100,height:20};c.selection=selection;c.view={scale,result:state.result};
 const event={button:0,pointerId:7,clientX:10,clientY:20,preventDefault(){},stopPropagation(){}};
 return {state,c,frame,selection,event,operations,generated,notifications,eventWindow};
}
test('pointer movement renders only an outline and release invokes exactly one edit',async()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.capture,7);assert.deepEqual(t.frame.focused,{preventScroll:true});
 for(let n=0;n<20;n++)t.frame.events.get('pointermove')({pointerId:7,clientX:30+n,clientY:40,altKey:false});
 assert.equal(t.operations.length,0);assert.equal(t.generated.length,0);assert.notEqual(t.frame.style.transform,'');
 const up=t.frame.events.get('pointerup');await up({pointerId:7});assert.equal(t.operations.length,1);assert.equal(t.operations[0][0],t.selection);
 assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');assert.ok(t.generated.every(e=>e.removed));
});
test('click jitter creates no candidate or ghost',async()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({pointerId:7,clientX:11,clientY:21,altKey:false});await t.frame.events.get('pointerup')({pointerId:7});
 assert.equal(t.operations.length,0);assert.equal(t.generated.length,0);
});
test('Escape clears the gesture and restores its visual state without an edit',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({pointerId:7,clientX:45,clientY:65,altKey:false});t.c.onEscape({key:'Escape'});
 assert.equal(t.operations.length,0);assert.equal(t.c.selection,null);assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');assert.ok(t.generated.every(e=>e.removed));
});
test('pointer cancellation never executes the pending movement',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({pointerId:7,clientX:45,clientY:65,altKey:false});t.frame.events.get('pointercancel')();
 assert.equal(t.operations.length,0);assert.equal(t.frame.events.size,0);assert.ok(t.generated.every(e=>e.removed));
});

test('losing pointer capture cancels the outline without editing source',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({pointerId:7,clientX:45,clientY:65,altKey:false});
 t.frame.events.get('lostpointercapture')();assert.equal(t.operations.length,0);assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');
});
test('switching windows cancels a held drag and a later deliberate drag still works',async()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);
 t.frame.events.get('pointermove')({pointerId:7,clientX:45,clientY:65,altKey:false});
 t.eventWindow.events.get('blur')();
 assert.equal(t.operations.length,0);assert.equal(t.frame.events.size,0);
 assert.equal(t.frame.style.transform,'');assert.equal(t.c.selection,t.selection);
 assert.equal(t.c.cancelGesture,null);
 t.c.drag(t.event,t.frame,t.selection);
 t.frame.events.get('pointermove')({pointerId:7,clientX:30,clientY:40,altKey:false});
 await t.frame.events.get('pointerup')({pointerId:7});assert.equal(t.operations.length,1);
});

test('disposing a controller releases its window listener and active gesture',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.c.destroy();
 assert.equal(t.eventWindow.events.has('blur'),false);assert.equal(t.frame.events.size,0);
 assert.equal(t.operations.length,0);assert.equal(t.c.view,null);
});
test('switching windows removes a marquee without resolving or replacing the selection',()=>{
 const t=setup(),shell=new Element();let resolutions=0;
 shell.getBoundingClientRect=()=>({left:0,top:0});shell.append=()=>{};
 t.c.resolve=async()=>{resolutions++};
 t.c.marquee({...t.event,target:{closest:()=>null}},shell,[],1);
 shell.events.get('pointermove')({pointerId:7,clientX:80,clientY:90});
 assert.equal(t.generated.length,1);assert.equal(t.generated[0].removed,false);
 t.eventWindow.events.get('blur')();
 assert.equal(t.generated[0].removed,true);assert.equal(shell.events.size,0);
 assert.equal(t.c.selection,t.selection);assert.equal(resolutions,0);assert.equal(t.operations.length,0);
});
test('zoom and free movement preserve PDF-point deltas',async()=>{
 const t=setup({scale:2});t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({pointerId:7,clientX:31,clientY:45,altKey:true});
 assert.equal(t.operations.length,0);await t.frame.events.get('pointerup')({pointerId:7});assert.equal(t.operations[0][1],10.5);assert.equal(t.operations[0][2],12.5);
});
test('stale and reading surfaces do not start a drag',()=>{
 for(const options of [{mode:'read'},{revision:1}]){const t=setup(options);t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.events.size,0);assert.equal(t.frame.capture,undefined);assert.equal(t.operations.length,0);}
});
test('selecting the verified movable object restores keyboard focus after a PDF publication',()=>{
 const t=setup(),object={id:'text-current',kind:'object',page:1};
 t.c.view.result.objects=[object];t.c.view.objects=new Map([[object.id,{page:1,instances:1,pages:new Set([1])}]]);t.c.view.pages=[{number:1}];
 t.c.drawFrame=()=>t.frame;t.c.selectObject(object.id);
 assert.equal(t.c.selection.id,object.id);assert.deepEqual(t.frame.focused,{preventScroll:true});
});

test('another pointer cannot move, release or cancel the active drag',async()=>{
  const t=setup();t.c.drag(t.event,t.frame,t.selection);
  t.frame.events.get('pointermove')({pointerId:8,clientX:100,clientY:100});
  await t.frame.events.get('pointerup')({pointerId:8});t.frame.events.get('pointercancel')({pointerId:8});
  assert.equal(t.operations.length,0);assert.equal(t.frame.events.has('pointerup'),true);
  t.frame.events.get('pointermove')({pointerId:7,clientX:30,clientY:40,altKey:false});
  await t.frame.events.get('pointerup')({pointerId:7});assert.equal(t.operations.length,1);
});

test('marquee lost capture removes its listeners and cannot resolve a later release',()=>{
  const t=setup(),shell=new Element();let resolved=0;shell.getBoundingClientRect=()=>({left:0,top:0});shell.append=()=>{};
  t.c.resolve=async()=>{resolved++};t.c.marquee({...t.event,target:{closest:()=>null}},shell,[],1);
  shell.events.get('pointermove')({pointerId:7,clientX:80,clientY:90});
  shell.events.get('lostpointercapture')({pointerId:7});assert.equal(shell.events.size,0);
  assert.equal(t.generated[0].removed,true);assert.equal(resolved,0);assert.equal(t.c.cancelGesture,null);
});


test('zoom refinement fences held drags and selection until the view is ready',()=>{
 const t=setup();t.state.previewZooming=true;
 t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.events.size,0);assert.equal(t.operations.length,0);
 t.state.previewZooming=false;t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.events.has('pointermove'),true);t.c.destroy();
});

test('rebinding reused PDF page shells removes old hits and pointer handlers',()=>{
 const t=setup(),shell=new Element(),hits=[];let removed=0,oldHandler;
 shell.append=hit=>hits.push(hit);shell.querySelectorAll=()=>hits.splice(0).map(hit=>({remove(){removed++}}));
 const view={scale:1,pages:[{number:1,shell,lines:[{text:'same line',x:10,y:20,width:50,height:15}]}]};
 // bind must own one handler and one hit per source line on every zoom.
 t.generated.length=0;t.c.bind(view);oldHandler=shell.events.get('pointerdown');
 t.c.bind(view);assert.notEqual(shell.events.get('pointerdown'),oldHandler);assert.equal(hits.length,1);assert.equal(removed,1);
 t.c.destroy();assert.equal(shell.events.has('pointerdown'),false);assert.equal(hits.length,0);
});
