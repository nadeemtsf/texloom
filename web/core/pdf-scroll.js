import {PDF_ZOOM_DELAY} from './pdf-page-view.js';
/** Wheel/trackpad input belongs to the persistent PDF viewport, not a page or
 * source edit. Preserve fractional deltas and let the device supply momentum. */
function wheelDelta(event,viewport,lineSize=20){
  if(event.defaultPrevented||event.cancelable===false)return null;
  let x=event.deltaX??0,y=event.deltaY??0;
  if(!Number.isFinite(x)||!Number.isFinite(y))return null;
  if(event.shiftKey&&x===0){x=y;y=0}
  const mode=event.deltaMode??0;
  if(mode===1){x*=lineSize;y*=lineSize}
  else if(mode===2){x*=viewport.clientWidth;y*=viewport.clientHeight}
  else if(mode!==0)return null;
  return {x,y};
}
export function pdfWheelDelta(event,viewport,lineSize=20){
  return event.ctrlKey||event.metaKey?null:wheelDelta(event,viewport,lineSize);
}

export function bindPDFScrolling(viewport,{getScale=()=>null,onZoom=null,renderZoom=()=>{},onNavigate=()=>{},eventRoot=viewport.ownerDocument,
  setTimer=(fn,delay)=>globalThis.setTimeout(fn,delay),clearTimer=id=>globalThis.clearTimeout(id)}={}){
  let zoomTimer=null;
  const cancelZoom=()=>{if(zoomTimer!==null){clearTimer(zoomTimer);zoomTimer=null}};
  // WebView2 must allow zoom input to deliver precision-touchpad pinch events.
  // Cancel its browser defaults so only the PDF handler consumes those events.
  const withinPDF=event=>event.composedPath?.().includes(viewport)||event.target===viewport||viewport.contains?.(event.target);
  const shellWheel=event=>{
    if((event.ctrlKey||event.metaKey)&&event.cancelable!==false&&!withinPDF(event))event.preventDefault();
  };
  const shellKey=event=>{
    if((event.ctrlKey||event.metaKey)&&!event.altKey&&['+','=','-','0','Add','Subtract'].includes(event.key)&&event.cancelable!==false)event.preventDefault();
  };
  const wheel=event=>{
    if(event.ctrlKey&&onZoom){
      const delta=wheelDelta(event,viewport);if(!delta||!delta.y)return;
      event.preventDefault(); // Ctrl/pinch must zoom this PDF, never the app shell.
      const scale=getScale();if(!Number.isFinite(scale)||scale<=0)return;
      const next=Math.max(0.2,Math.min(2.5,scale*Math.exp(-delta.y*0.0015)));
      if(next===scale)return;
      onNavigate();onZoom(next,[event.clientX,event.clientY]);cancelZoom();
      zoomTimer=setTimer(()=>{zoomTimer=null;renderZoom()},PDF_ZOOM_DELAY);
      return;
    }
    const delta=pdfWheelDelta(event,viewport);if(!delta)return;
    const top=viewport.scrollTop,left=viewport.scrollLeft;
    const nextTop=Math.max(0,Math.min(Math.max(0,viewport.scrollHeight-viewport.clientHeight),top+delta.y));
    const nextLeft=Math.max(0,Math.min(Math.max(0,viewport.scrollWidth-viewport.clientWidth),left+delta.x));
    if(nextTop===top&&nextLeft===left)return;
    event.preventDefault(); // The browser must not apply the same delta again.
    onNavigate();
    viewport.scrollTop=nextTop;viewport.scrollLeft=nextLeft;
  };
  viewport.addEventListener('wheel',wheel,{passive:false});
  eventRoot?.addEventListener('wheel',shellWheel,{capture:true,passive:false});
  eventRoot?.addEventListener('keydown',shellKey,{capture:true});
  return {cancelZoom,dispose(){cancelZoom();viewport.removeEventListener('wheel',wheel);eventRoot?.removeEventListener('wheel',shellWheel,true);eventRoot?.removeEventListener('keydown',shellKey,true)}};
}
