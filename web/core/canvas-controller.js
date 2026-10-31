import {intersects,normalizedRectangle,union,linesForSourceRectangles} from './geometry.js';

/** DOM gestures are isolated from source edits and compilation. */
export class CanvasController {
  constructor({getSession,resolve,move,edit,selected,notify}) {
    Object.assign(this,{getSession,resolve,move,edit,onSelected:selected,notify});
    this.view=null;this.selection=null;this.pending=0;this.hits=[];
    this.cancelGesture=null;
    this.onEscape=e=>{if(e.key==='Escape'){this.cancelGesture?.();this.clear()}};
    document.addEventListener('keydown',this.onEscape);
  }
  clear(){this.cancelGesture?.();this.pending++;this.selection=null;this.hits=[];this.drawFrame();this.onSelected(null)}
  bind(view) {
    this.cancelGesture?.();const sameBuild=this.view?.result.build_id===view.result.build_id;this.view=view;this.pending++;
    for(const {shell,lines,number} of view.pages){
      for(const line of lines){
        const hit=document.createElement('button');hit.type='button';hit.className='line-hit';hit.tabIndex=-1;
        hit.setAttribute('aria-label','Select text: '+line.text);
        this.position(hit,line,view.scale);shell.append(hit);
        hit.onclick=e=>{e.stopPropagation();this.lastClick={time:performance.now(),x:e.clientX,y:e.clientY};this.selectHits(e.shiftKey?[...this.hits,line]:[line],number)};
        hit.ondblclick=e=>{e.stopPropagation();this.selectHits([line],number,true)};
      }
      shell.addEventListener('pointerdown',event=>this.marquee(event,shell,lines,number));
    }
    if(this.selection?.kind==='object'){
      const object=(view.result.objects||[]).find(o=>o.id===this.selection.id),bounds=view.objects.get(this.selection.id);
      this.selection=object&&bounds?{...object,...bounds,kind:'object'}:null;
    }else if(!sameBuild)this.selection=null;
    this.drawFrame();this.onSelected(this.selection);
  }
  position(el,r,scale){Object.assign(el.style,{left:r.x*scale+'px',top:r.y*scale+'px',width:Math.max(3,r.width*scale)+'px',height:Math.max(8,r.height*scale)+'px'})}
  fresh(){const s=this.getSession();return !!this.view&&!s.busy&&s.revision===s.compiled}
  async selectHits(hits,page,edit=false){
    if(!this.fresh())return this.notify('Compile the current source before selecting layout objects. Read text remains available.');
    hits=[...new Map(hits.filter(h=>h.page===page).map(h=>[h.id,h])).values()];
    if(!hits.length||hits.length>64)return this.notify('Select between one and 64 adjacent text lines.');
    const serial=++this.pending,session=this.getSession(),buildId=session.result.build_id,revision=session.revision;
    this.notify('Mapping selected text to its source…');
    try{
      const resolved=await this.resolve({buildId,page,points:hits.map(h=>({x:h.x+Math.min(h.width/2,15),y:h.baseline-h.height*0.35}))});
      const now=this.getSession();
      if(serial!==this.pending||revision!==now.revision||now.result?.build_id!==buildId)return;
      if(now.sources[resolved.file]!==resolved.expected_source)throw new Error('The source changed. Select the text again.');
      const pageLines=this.view.pages.find(p=>p.number===page)?.lines||[];
      const mapped=linesForSourceRectangles(pageLines,resolved.rectangles,page);
      if(!mapped.length)throw new Error('The complete source unit could not be matched to visible text. Select it in the source editor.');
      const bounds=union(mapped);
      this.hits=mapped;this.selection={...resolved,...bounds,page,kind:'source',unitKind:resolved.kind};
      this.drawFrame();this.onSelected(this.selection);
      this.notify(`Selected ${resolved.kind}${mapped.length>1?' · '+mapped.length+' visible lines':''}. Drag its frame or double-click to edit.`);
      if(edit)this.edit(this.selection);
    }catch(error){if(serial===this.pending)this.notify(error.message||String(error))}
  }
  selectObject(id){
    const object=(this.view?.result.objects||[]).find(o=>o.id===id),bounds=this.view?.objects.get(id);
    if(!object||!bounds)return;
    this.pending++;this.hits=[];this.selection={...object,...bounds,kind:'object'};this.drawFrame();this.onSelected(this.selection);
  }
  drawFrame(){
    document.querySelectorAll('.source-selection-frame').forEach(el=>el.remove());
    if(!this.view)return;
    for(const [id,bounds] of this.view.objects){
      const page=this.view.pages.find(p=>p.number===bounds.page);if(!page||bounds.pages.size!==1)continue;
      let hit=[...page.shell.querySelectorAll('.text-object-hit')].find(el=>el.dataset.object===id);
      if(!hit){hit=document.createElement('button');hit.type='button';hit.dataset.object=id;page.shell.append(hit)}
      hit.className='text-object-hit';hit.replaceChildren();
      hit.setAttribute('aria-label','Select movable text '+id);this.position(hit,bounds,this.view.scale);
      hit.onclick=e=>{e.stopPropagation();if(this.selection?.id!==id)this.selectObject(id)};hit.ondblclick=e=>{e.stopPropagation();this.selectObject(id);this.edit(this.selection)};
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
    page.shell.append(frame);
  }
  drag(event,frame,selection){
    if(event.button!==0||!this.fresh()||this.getSession().mode==='read')return;
    event.preventDefault();event.stopPropagation();frame.setPointerCapture(event.pointerId);
    const origin={x:event.clientX,y:event.clientY};let delta=null,ghost=null,mask=null;
    const page=frame.closest('.pdf-page'),canvas=page.querySelector('canvas'),factor=this.view.scale;
    const preview=()=>{
      if(ghost)return;
      mask=document.createElement('div');mask.className='drag-mask';this.position(mask,selection,factor);
      ghost=document.createElement('canvas');ghost.className='drag-ghost';this.position(ghost,selection,factor);
      const ratio=canvas.width/parseFloat(canvas.style.width);
      ghost.width=Math.ceil(selection.width*factor*ratio);ghost.height=Math.ceil(selection.height*factor*ratio);
      ghost.getContext('2d').drawImage(canvas,selection.x*factor*ratio,selection.y*factor*ratio,ghost.width,ghost.height,0,0,ghost.width,ghost.height);
      page.append(mask,ghost);
    };
    const move=e=>{
      const snap=v=>e.altKey?Math.round(v*100)/100:Math.round(v/2)*2;
      if(Math.hypot(e.clientX-origin.x,e.clientY-origin.y)<3&&!delta)return;
      delta={x:snap((e.clientX-origin.x)/factor),y:snap((e.clientY-origin.y)/factor)};preview();
      frame.style.transform=`translate(${delta.x*this.view.scale}px,${delta.y*this.view.scale}px)`;
      ghost.style.transform=frame.style.transform;
      frame.classList.add('dragging');
    };
    const clean=()=>{ghost?.remove();mask?.remove();frame.style.transform='';frame.classList.remove('dragging');this.cancelGesture=null};
    const unbind=()=>{frame.removeEventListener('pointermove',move);frame.removeEventListener('pointerup',up);frame.removeEventListener('pointercancel',cancel)};
    const cancel=()=>{unbind();clean()};const up=async()=>{unbind();this.lastClick=null;try{if(delta&&(delta.x||delta.y))await this.move(selection,delta.x,delta.y)}finally{clean()}};
    this.cancelGesture=cancel;frame.addEventListener('pointermove',move);frame.addEventListener('pointerup',up);frame.addEventListener('pointercancel',cancel);
  }
  marquee(event,shell,lines,page){
    if(event.button!==0||event.target.closest('button,.block,.text-object-frame')||!this.fresh()||this.getSession().mode==='read')return;
    const rect=shell.getBoundingClientRect(),scale=this.view.scale;
    const point=e=>({x:(e.clientX-rect.left)/scale,y:(e.clientY-rect.top)/scale}),origin=point(event);
    const box=document.createElement('div');box.className='marquee';shell.append(box);shell.setPointerCapture(event.pointerId);
    let bounds=null;
    const move=e=>{bounds=normalizedRectangle(origin,point(e));this.position(box,bounds,scale)};
    const clean=()=>{box.remove();shell.removeEventListener('pointermove',move);shell.removeEventListener('pointerup',up);shell.removeEventListener('pointercancel',cancel);this.cancelGesture=null};
    const cancel=()=>clean();const up=()=>{clean();if(bounds&&bounds.width*scale>5&&bounds.height*scale>5)this.selectHits(lines.filter(l=>intersects(l,bounds)),page);else this.clear()};
    this.cancelGesture=cancel;shell.addEventListener('pointermove',move);shell.addEventListener('pointerup',up);shell.addEventListener('pointercancel',cancel);
  }
  destroy(){this.cancelGesture?.();document.removeEventListener('keydown',this.onEscape);this.view=null;this.selection=null}
}
