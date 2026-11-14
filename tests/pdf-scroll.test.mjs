import {test} from 'node:test';
import assert from 'node:assert/strict';
import {bindPDFScrolling,pdfWheelDelta} from '../web/core/pdf-scroll.js';

function fixture(options){
 const handlers=new Map();
 const viewport={clientWidth:400,clientHeight:600,scrollWidth:1400,scrollHeight:3000,scrollTop:100,scrollLeft:20,
  addEventListener(name,fn,options){handlers.set(name,fn);this.options=options},removeEventListener(name,fn){if(handlers.get(name)===fn)handlers.delete(name)}};
 const binding=bindPDFScrolling(viewport,options),dispose=()=>binding.dispose();
 const send=values=>{const event={deltaMode:0,deltaX:0,deltaY:0,cancelable:true,preventDefault(){this.prevented=true},...values};handlers.get('wheel')?.(event);return event};
 return {viewport,handlers,send,dispose,binding};
}
test('mouse wheel scrolls the PDF viewport once and clamps both ends',()=>{
 const {viewport,send}=fixture();
 assert.equal(viewport.options.passive,false);
 assert.equal(send({deltaY:3,deltaMode:1}).prevented,true);assert.equal(viewport.scrollTop,160);
 send({deltaY:10000});assert.equal(viewport.scrollTop,2400);
 assert.equal(send({deltaY:50}).prevented,undefined);
 send({deltaY:-10000});assert.equal(viewport.scrollTop,0);
});
test('trackpad fractions, two axes and repeated momentum events are retained',()=>{
 const {viewport,send}=fixture();
 send({deltaX:0.25,deltaY:0.5});send({deltaX:0.125,deltaY:0.25});
 assert.equal(viewport.scrollLeft,20.375);assert.equal(viewport.scrollTop,100.75);
 send({deltaX:-0.25,deltaY:-0.5});assert.equal(viewport.scrollTop,100.25);
});
test('page units and Shift-wheel use the correct viewport axis',()=>{
 const {viewport,send}=fixture();
 send({deltaY:1,deltaMode:2});assert.equal(viewport.scrollTop,700);
 send({deltaY:1,deltaMode:2,shiftKey:true});assert.equal(viewport.scrollLeft,420);assert.equal(viewport.scrollTop,700);
 assert.deepEqual(pdfWheelDelta({deltaX:2,deltaY:3,shiftKey:true},viewport),{x:2,y:3});
});
test('zoom gestures, handled/noncancelable input and invalid deltas are untouched',()=>{
 const {viewport,send}=fixture();
 for(const values of [{ctrlKey:true},{metaKey:true},{defaultPrevented:true},{cancelable:false},{deltaY:NaN},{deltaX:Infinity},{deltaMode:99}]){
  const event=send({deltaY:100,...values});assert.equal(event.prevented,undefined);
  assert.equal(viewport.scrollTop,100);assert.equal(viewport.scrollLeft,20);
 }
});
test('disposal removes the listener and a PDF without overflow is not moved',()=>{
 const {viewport,handlers,send,dispose}=fixture();
 viewport.scrollTop=0;viewport.scrollLeft=0;viewport.scrollHeight=600;viewport.scrollWidth=400;
 assert.equal(send({deltaY:30}).prevented,undefined);assert.equal(viewport.scrollTop,0);
 dispose();dispose();assert.equal(handlers.size,0);
});

test('Ctrl-wheel changes only PDF scale and coalesces rapid inputs into one render',()=>{
 let scale=1,renders=0,serial=0;const timers=new Map();
 const {viewport,handlers,binding}=fixture({getScale:()=>scale,onZoom:next=>{scale=next},renderZoom:()=>renders++,
  setTimer:fn=>{timers.set(++serial,fn);return serial},clearTimer:id=>timers.delete(id)});
 const send=deltaY=>{const e={ctrlKey:true,deltaY,deltaX:0,deltaMode:0,cancelable:true,preventDefault(){this.prevented=true}};handlers.get('wheel')(e);assert.equal(e.prevented,true)};
 send(-50);const first=scale;send(-30);assert.ok(scale>first&&first>1);assert.equal(renders,0);assert.equal(timers.size,1);
 for(const fn of timers.values())fn();timers.clear();assert.equal(renders,1);
 assert.equal(viewport.scrollTop,100);assert.equal(viewport.scrollLeft,20);
 send(30);binding.cancelZoom();assert.equal(timers.size,0);
 send(30);binding.dispose();assert.equal(timers.size,0);
});

test('Ctrl-wheel suppresses application zoom at PDF scale limits or without a loaded PDF',()=>{
 let scale=2.5,calls=0;
 const {viewport,handlers}=fixture({getScale:()=>scale,onZoom:()=>calls++});
 const send=()=>{const e={ctrlKey:true,deltaY:-100,deltaMode:0,cancelable:true,preventDefault(){this.prevented=true}};handlers.get('wheel')(e);assert.equal(e.prevented,true)};
 send();assert.equal(calls,0);scale=null;send();assert.equal(calls,0);
 assert.equal(viewport.scrollTop,100);
});

test('navigation cancels an in-progress gesture only when the viewport actually changes',()=>{
 let cancellations=0;
 const {send,viewport}=fixture({onNavigate:()=>cancellations++});
 send({deltaY:20});assert.equal(cancellations,1);
 viewport.scrollTop=2400;send({deltaY:20});assert.equal(cancellations,1);
 send({deltaY:-20});assert.equal(cancellations,2);
});


test('Ctrl zoom forwards the cursor origin immediately and defers sharp drawing for 400 ms',()=>{
 let observed,delay,draws=0;const {send,binding}=fixture({getScale:()=>1,
  onZoom:(scale,origin)=>observed={scale,origin},renderZoom:()=>draws++,setTimer:(_fn,ms)=>{delay=ms;return 1},clearTimer(){}});
 send({ctrlKey:true,deltaY:-40,clientX:351.5,clientY:204.25});
 assert.deepEqual(observed.origin,[351.5,204.25]);assert.ok(observed.scale>1);assert.equal(delay,400);assert.equal(draws,0);binding.dispose();
});

test('gesture gain is 25 percent stronger than r11 and remains reversible',()=>{
 let scale=1;const {send,dispose}=fixture({getScale:()=>scale,onZoom:value=>{scale=value}});
 send({ctrlKey:true,deltaY:-40});assert.ok(Math.abs(scale-1.0778841508846315)<1e-12);
 send({ctrlKey:true,deltaY:40});assert.ok(Math.abs(scale-1)<1e-12);dispose();
});

test('browser zoom input is routed to the PDF and blocked elsewhere without swallowing ordinary scroll',()=>{
 const handlers=new Map(),eventRoot={addEventListener(n,fn){handlers.set(n,fn)},removeEventListener(n){handlers.delete(n)}};
 const {viewport,send,binding}=fixture({eventRoot,getScale:()=>1,onZoom(){}});
 const outside={ctrlKey:true,cancelable:true,target:{},preventDefault(){this.prevented=true}};
 handlers.get('wheel')(outside);assert.equal(outside.prevented,true);
 const inside={...outside,prevented:false,composedPath:()=>[viewport,eventRoot]};
 handlers.get('wheel')(inside);assert.equal(inside.prevented,false);assert.equal(send({ctrlKey:true,deltaY:-20}).prevented,true);
 const ordinary={...outside,ctrlKey:false,prevented:false};handlers.get('wheel')(ordinary);assert.equal(ordinary.prevented,false);
 for(const key of ['+','=','-','0']){const e={...outside,key,prevented:false};handlers.get('keydown')(e);assert.equal(e.prevented,true)}
 const find={...outside,key:'f',prevented:false};handlers.get('keydown')(find);assert.equal(find.prevented,false);
 binding.dispose();assert.equal(handlers.size,0);
});
