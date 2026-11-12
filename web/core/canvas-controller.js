import {intersects,normalizedRectangle} from './geometry.js';
import {performanceOutcome} from './performance-diagnostics.js';

/** DOM gestures are isolated from source edits and compilation. */
export class CanvasController {
  constructor({getSession,resolve,move,edit,selected,notify,eventWindow=globalThis.window,diagnostics=null}) {
    Object.assign(this,{getSession,resolve,move,edit,onSelected:selected,notify,eventWindow,diagnostics});
    this.view=null;this.selection=null;this.pending=0;this.hits=[];
    this.cancelGesture=null;this.pageHandlers=new Map();
    this.onEscape=e=>{if(e.key==='Escape'){this.cancelGesture?.();this.clear()}};
    document.addEventListener('keydown',this.onEscape);
    // Switching windows need not deliver pointercancel before the next release.
    // Keep the accepted selection, but discard any in-progress drag or marquee.
    this.onBlur=()=>this.cancelGesture?.();
    eventWindow?.addEventListener('blur',this.onBlur);
  }
  clear(){this.cancelGesture?.();this.pending++;this.selection=null;this.hits=[];this.drawFrame();this.onSelected(null)}
  bind(view) {
    this.cancelGesture?.();this.unbindPages();this.view=view;this.pending++;this.hits=[];
    for(const {shell,lines,number} of view.pages){
      for(const line of lines){
        const hit=document.createElement('button');hit.type='button';hit.className='line-hit';hit.tabIndex=-1;
        hit.setAttribute('aria-label','Select text: '+line.text);
        this.position(hit,line,view.scale);shell.append(hit);
        hit.onclick=e=>{e.stopPropagation();this.lastClick={time:performance.now(),x:e.clientX,y:e.clientY};this.selectHits(e.shiftKey?[...this.hits,line]:[line],number)};
        hit.ondblclick=e=>{e.stopPropagation();this.selectHits([line],number,true)};
      }
      const handler=event=>this.marquee(event,shell,lines,number);
      shell.addEventListener('pointerdown',handler);this.pageHandlers.set(shell,handler);
    }
    if(this.selection?.kind==='object'){
      const object=(view.result.objects||[]).find(o=>o.id===this.selection.id),bounds=view.objects.get(this.selection.id);
      this.selection=object&&bounds?{...object,...bounds,kind:'object'}:null;
    }else this.selection=null;
    this.drawFrame();this.onSelected(this.selection);
  }
  unbindPages(){
    for(const [shell,handler] of this.pageHandlers){shell.removeEventListener('pointerdown',handler);shell.querySelectorAll('.line-hit').forEach(hit=>hit.remove())}
    this.pageHandlers.clear();
  }
  position(el,r,scale){Object.assign(el.style,{left:r.x*scale+'px',top:r.y*scale+'px',width:Math.max(3,r.width*scale)+'px',height:Math.max(8,r.height*scale)+'px'})}
  fresh(){const s=this.getSession();return !!this.view&&s.result?.verification_eligible!==false&&!s.previewZooming&&!s.busy&&s.revision===s.compiled&&!s.client?.pending&&!s.client?.error&&!s.mutating&&!s.composing&&!s.opening}
  async selectHits(hits,page,edit=false){
    if(this.getSession().mode==='read')return;
    if(!this.fresh())return this.notify('Compile the current source before selecting layout objects. Read text remains available.');
    const reason=this.view.pages.find(p=>p.number===page)?.selectionReason;if(reason)return this.notify(reason);
    hits=[...new Map(hits.filter(h=>h.page===page).map(h=>[h.id,h])).values()];
    if(!hits.length||hits.length>64)return this.notify('Select between one and 64 adjacent text lines.');
    const serial=++this.pending,session=this.getSession(),buildId=session.result.build_id,revision=session.revision;
    const trace=this.diagnostics?.begin('source-selection');
    this.notify('Mapping selected text to its source…');
    try{
      const resolved=await this.resolve({buildId,page,hits,trace,points:hits.map(h=>({x:h.x+Math.min(h.width/2,15),y:h.baseline-h.height*0.35}))});
      const now=this.getSession();
      if(serial!==this.pending||revision!==now.revision||now.result?.build_id!==buildId||now.mode==='read'){trace?.finish('stale');return}
      if(now.sources[resolved.file]!==resolved.expected_source)throw new Error('The source changed. Select the text again.');
      const proof=resolved.proof;
      if(proof?.schema!==1||proof.build_id!==buildId||proof.page!==page)throw new Error('This selection has no current layout proof.');
      const mapped=proof.lines;
      this.hits=mapped;this.selection={...resolved,x:proof.x,y:proof.y,width:proof.width,height:proof.height,baselineRuns:proof.runs,page,kind:'source',unitKind:resolved.kind};
      const frame=this.drawFrame();this.onSelected(this.selection);frame?.focus({preventScroll:true});
      trace?.mark('selection-ready');trace?.finish('selected',{paint:true,isCurrent:()=>serial===this.pending&&this.fresh()});
      this.notify(`Selected ${resolved.kind}${mapped.length>1?' · '+mapped.length+' visible lines':''}. Drag its frame or double-click to edit.`);
      if(edit)this.edit(this.selection);
    }catch(error){trace?.finish(performanceOutcome(error));if(serial===this.pending)this.notify(error.message||String(error))}
  }
  selectObject(id,{record=true}={}){
    const object=(this.view?.result.objects||[]).find(o=>o.id===id),bounds=this.view?.objects.get(id);
    if(!object||!bounds||!this.fresh()||this.getSession().mode==='read')return;
    const reason=this.view.pages.find(p=>p.number===bounds.page)?.selectionReason;if(reason)return this.notify(reason);
    if(bounds.instances!==1||bounds.pages.size!==1)return this.notify('This source object has multiple PDF instances. Use source editing.');
    const trace=record?this.diagnostics?.begin('object-selection'):null;
    this.pending++;this.hits=[];this.selection={...object,...bounds,kind:'object'};const frame=this.drawFrame();this.onSelected(this.selection);frame?.focus({preventScroll:true});
    const serial=this.pending;trace?.mark('selection-ready');trace?.finish('selected',{paint:true,isCurrent:()=>serial===this.pending&&this.fresh()});
    return this.selection;
  }
  drawFrame(){
    document.querySelectorAll('.source-selection-frame').forEach(el=>el.remove());
    if(!this.view)return;
    for(const [id,bounds] of this.view.objects){
      const page=this.view.pages.find(p=>p.number===bounds.page);if(!page||bounds.pages.size!==1||bounds.instances!==1||page.selectionReason)continue;
      let hit=[...page.shell.querySelectorAll('.text-object-hit')].find(el=>el.dataset.object===id);
      if(!hit){hit=document.createElement('button');hit.type='button';hit.dataset.object=id;page.shell.append(hit)}
      hit.className='text-object-hit';hit.replaceChildren();
      hit.setAttribute('aria-label','Select movable text '+id);this.position(hit,bounds,this.view.scale);
      hit.onclick=e=>{e.stopPropagation();if(this.selection?.id!==id)this.selectObject(id)};
      hit.ondblclick=e=>{e.stopPropagation();const selected=this.selectObject(id);if(selected)this.edit(selected)};
      hit.onpointerdown=null;hit.onkeydown=null;
    }
    const selection=this.selection;if(!selection)return;
    const page=this.view.pages.find(p=>p.number===selection.page);if(!page)return;
    const frame=selection.kind==='object'?[...page.shell.querySelectorAll('.text-object-hit')].find(el=>el.dataset.object===selection.id):document.createElement('div');
    if(!frame)return;
    frame.classList.add('text-object-frame');if(selection.kind==='source')frame.classList.add('source-selection-frame');frame.tabIndex=0;
    frame.setAttribute('role','button');frame.setAttribute('aria-label','Move selected text');this.position(frame,selection,this.view.scale);
    const label=document.createElement('span');label.className='object-frame-label';label.textContent=selection.kind==='source'?`Selected ${selection.unitKind} · drag to move`:'Text object · drag to move';frame.append(label);
    frame.onpointerdown=event=>this.drag(event,frame,selection);
    if(selection.kind==='source')frame.onclick=event=>{
      const last=this.lastClick;
      if(last&&performance.now()-last.time<500&&Math.hypot(event.clientX-last.x,event.clientY-last.y)<6){this.lastClick=null;this.edit(selection)}
    };
    frame.ondblclick=event=>{event.stopPropagation();this.edit(selection)};
    frame.onkeydown=event=>{
      const step=event.shiftKey?10:1,directions={ArrowLeft:[-step,0],ArrowRight:[step,0],ArrowUp:[0,-step],ArrowDown:[0,step]};
      if(directions[event.key]){event.preventDefault();this.move(selection,...directions[event.key])}
      else if(event.key==='Enter'){event.preventDefault();this.edit(selection)}
    };
    page.shell.append(frame);return frame;
  }
  drag(event,frame,selection){
    if(event.button!==0||!this.fresh()||this.getSession().mode==='read')return;
    this.cancelGesture?.();const pointerId=event.pointerId;
    event.preventDefault();event.stopPropagation();frame.focus({preventScroll:true});frame.setPointerCapture(event.pointerId);
    const origin={x:event.clientX,y:event.clientY};let delta=null,previewMeasured=false;
    const factor=this.view.scale;
    const move=e=>{
      if(e.pointerId!==pointerId)return;
      const snap=v=>e.altKey?Math.round(v*100)/100:Math.round(v/2)*2;
      if(Math.hypot(e.clientX-origin.x,e.clientY-origin.y)<3&&!delta)return;
      const trace=previewMeasured?null:this.diagnostics?.begin('drag-preview');previewMeasured=true;
      delta={x:snap((e.clientX-origin.x)/factor),y:snap((e.clientY-origin.y)/factor)};
      frame.style.transform=`translate(${delta.x*this.view.scale}px,${delta.y*this.view.scale}px)`;
      frame.classList.add('dragging');
      trace?.mark('preview-updated');trace?.finish('feedback',{paint:true,isCurrent:()=>this.cancelGesture===cancel&&this.fresh()});
    };
    // A moving outline is honest feedback until target-only paint is proven.
    // Cropping the page would copy background ink; masking would erase it.
    const clean=()=>{frame.style.transform='';frame.classList.remove('dragging');if(this.cancelGesture===cancel)this.cancelGesture=null};
    const unbind=()=>{frame.removeEventListener('pointermove',move);frame.removeEventListener('pointerup',up);frame.removeEventListener('pointercancel',cancel);frame.removeEventListener('lostpointercapture',cancel)};
    const cancel=e=>{if(e&&e.pointerId!==pointerId)return;unbind();clean()};const up=async e=>{if(e.pointerId!==pointerId)return;const trace=delta&&(delta.x||delta.y)?this.diagnostics?.begin('text-move'):null;unbind();this.lastClick=null;try{if(delta&&(delta.x||delta.y))await this.move(selection,delta.x,delta.y,trace)}finally{clean()}};
    this.cancelGesture=cancel;frame.addEventListener('pointermove',move);frame.addEventListener('pointerup',up);frame.addEventListener('pointercancel',cancel);frame.addEventListener('lostpointercapture',cancel);
  }
  marquee(event,shell,lines,page){
    if(event.button!==0||event.target.closest('button,.block,.text-object-frame')||!this.fresh()||this.getSession().mode==='read')return;
    this.cancelGesture?.();const pointerId=event.pointerId;
    const rect=shell.getBoundingClientRect(),scale=this.view.scale;
    const point=e=>({x:(e.clientX-rect.left)/scale,y:(e.clientY-rect.top)/scale}),origin=point(event);
    const box=document.createElement('div');box.className='marquee';shell.append(box);shell.setPointerCapture(event.pointerId);
    let bounds=null;
    const move=e=>{if(e.pointerId!==pointerId)return;bounds=normalizedRectangle(origin,point(e));this.position(box,bounds,scale)};
    const clean=()=>{box.remove();shell.removeEventListener('pointermove',move);shell.removeEventListener('pointerup',up);shell.removeEventListener('pointercancel',cancel);shell.removeEventListener('lostpointercapture',cancel);if(this.cancelGesture===cancel)this.cancelGesture=null};
    const cancel=e=>{if(e&&e.pointerId!==pointerId)return;clean()};const up=e=>{if(e.pointerId!==pointerId)return;clean();if(bounds&&bounds.width*scale>5&&bounds.height*scale>5)this.selectHits(lines.filter(l=>intersects(l,bounds)),page);else this.clear()};
    this.cancelGesture=cancel;shell.addEventListener('pointermove',move);shell.addEventListener('pointerup',up);shell.addEventListener('pointercancel',cancel);shell.addEventListener('lostpointercapture',cancel);
  }
  destroy(){this.cancelGesture?.();this.unbindPages();document.removeEventListener('keydown',this.onEscape);this.eventWindow?.removeEventListener('blur',this.onBlur);this.view=null;this.selection=null}
}
