import {test} from 'node:test';
import assert from 'node:assert/strict';
import {CanvasController} from '../web/core/canvas-controller.js';

// Exercise the actual controller with a small event/DOM boundary, not another
// gesture implementation. Native Windows pointer acceptance remains separate.
class Element{
 constructor(){this.style={};this.events=new Map();this.classList={add(){},remove(){}};this.removed=false;this.width=0;this.height=0;}
 addEventListener(name,fn){this.events.set(name,fn)}
 removeEventListener(name,fn){if(this.events.get(name)===fn)this.events.delete(name)}
 setPointerCapture(id){this.capture=id}
 getContext(){return {drawImage(){}}}
 remove(){this.removed=true}
}
function setup({scale=1,mode='select',revision=0}={}){
 const generated=[];globalThis.document={addEventListener(){},removeEventListener(){},querySelectorAll:()=>[],createElement:()=>{const e=new Element();generated.push(e);return e}};
 const state={mode,revision,compiled:0,result:{build_id:'current'},sources:{'main.tex':'Text'}};
 const operations=[],notifications=[];
 const c=new CanvasController({getSession:()=>state,resolve:async()=>{},move:async(...args)=>operations.push(args),edit(){},selected(){},notify:t=>notifications.push(t)});
 c.drawFrame=()=>{};
 const canvas=new Element();canvas.width=1000;canvas.style.width='500px';
 const page={querySelector:()=>canvas,append(){}};const frame=new Element();frame.closest=()=>page;
 const selection={kind:'source',page:1,x:20,y:50,width:100,height:20};c.selection=selection;c.view={scale,result:state.result};
 const event={button:0,pointerId:7,clientX:10,clientY:20,preventDefault(){},stopPropagation(){}};
 return {c,state,frame,selection,event,operations,generated,notifications};
}
test('pointer movement renders only an outline and release invokes exactly one edit',async()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.capture,7);
 for(let n=0;n<20;n++)t.frame.events.get('pointermove')({clientX:30+n,clientY:40,altKey:false});
 assert.equal(t.operations.length,0);assert.equal(t.generated.length,0);assert.notEqual(t.frame.style.transform,'');
 const up=t.frame.events.get('pointerup');await up();assert.equal(t.operations.length,1);assert.equal(t.operations[0][0],t.selection);
 assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');assert.ok(t.generated.every(e=>e.removed));
});
test('click jitter creates no candidate or ghost',async()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({clientX:11,clientY:21,altKey:false});await t.frame.events.get('pointerup')();
 assert.equal(t.operations.length,0);assert.equal(t.generated.length,0);
});
test('Escape clears the gesture and restores its visual state without an edit',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({clientX:45,clientY:65,altKey:false});t.c.onEscape({key:'Escape'});
 assert.equal(t.operations.length,0);assert.equal(t.c.selection,null);assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');assert.ok(t.generated.every(e=>e.removed));
});
test('pointer cancellation never executes the pending movement',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({clientX:45,clientY:65,altKey:false});t.frame.events.get('pointercancel')();
 assert.equal(t.operations.length,0);assert.equal(t.frame.events.size,0);assert.ok(t.generated.every(e=>e.removed));
});

test('losing pointer capture cancels the outline without editing source',()=>{
 const t=setup();t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({clientX:45,clientY:65,altKey:false});
 t.frame.events.get('lostpointercapture')();assert.equal(t.operations.length,0);assert.equal(t.frame.events.size,0);assert.equal(t.frame.style.transform,'');
});
test('zoom and free movement preserve PDF-point deltas',async()=>{
 const t=setup({scale:2});t.c.drag(t.event,t.frame,t.selection);t.frame.events.get('pointermove')({clientX:31,clientY:45,altKey:true});
 assert.equal(t.operations.length,0);await t.frame.events.get('pointerup')();assert.equal(t.operations[0][1],10.5);assert.equal(t.operations[0][2],12.5);
});
test('stale and reading surfaces do not start a drag',()=>{
 for(const options of [{mode:'read'},{revision:1}]){const t=setup(options);t.c.drag(t.event,t.frame,t.selection);assert.equal(t.frame.events.size,0);assert.equal(t.frame.capture,undefined);assert.equal(t.operations.length,0);}
});
