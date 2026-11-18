import {loadPDFDocument} from './pdf-document.js';

/** Render one page into a temporary image, with no text-hit or editing surface. */
export async function renderPreliminaryPDF(pdfjs,preview,{document,width,page=1,signal,resources={}}){
  const pdf=await loadPDFDocument(pdfjs,{...resources,data:new Uint8Array(preview.pdf)});
  let task=null;
  const cancel=()=>{try{task?.cancel()}catch{}};
  try{
    signal?.throwIfAborted();
    const number=Math.min(Math.max(1,page),pdf.numPages),source=await pdf.getPage(number);
    signal?.throwIfAborted();
    const base=source.getViewport({scale:1}),scale=Math.max(.1,Math.min(2,(width-48)/base.width));
    const viewport=source.getViewport({scale}),canvas=document.createElement('canvas');
    if(viewport.width*viewport.height>20_000_000)throw Error('Preliminary page is too large');
    canvas.width=Math.ceil(viewport.width);canvas.height=Math.ceil(viewport.height);
    canvas.setAttribute('role','img');canvas.setAttribute('aria-label',`Early preview, page ${number} of ${pdf.numPages}`);
    signal?.addEventListener('abort',cancel,{once:true});
    task=source.render({canvas,canvasContext:canvas.getContext('2d'),viewport});
    await task.promise;signal?.throwIfAborted();
    return {canvas,page:number,pageCount:pdf.numPages,dispose:()=>pdf.destroy()};
  }catch(error){await pdf.destroy().catch(()=>{});throw error}
  finally{signal?.removeEventListener('abort',cancel)}
}

/** Cover the viewport without replacing or disposing the earlier settled view. */
export class PreliminaryPDFSurface {
  constructor({container,pages,document,dismiss}){
    Object.assign(this,{container,pages,document,dismiss});this.overlay=null;
    this.position=()=>{if(this.overlay){this.overlay.style.top=container.scrollTop+'px';this.overlay.style.height=container.clientHeight+'px'}};
    container.addEventListener('scroll',this.position,{passive:true});
    this.resize=typeof ResizeObserver==='undefined'?null:new ResizeObserver(this.position);this.resize?.observe(container);
  }
  show(prepared){
    this.hide();
    const overlay=this.document.createElement('section');overlay.className='preliminary-preview';
    overlay.setAttribute('aria-label','Early PDF preview');
    const heading=this.document.createElement('div');heading.className='preliminary-heading';
    const status=this.document.createElement('span');status.setAttribute('role','status');
    status.textContent=`Early preview · page ${prepared.page} of ${prepared.pageCount} · finishing checks`;
    const dismiss=this.document.createElement('button');dismiss.type='button';dismiss.textContent='Show earlier PDF';dismiss.onclick=this.dismiss;
    heading.append(status,dismiss);
    const body=this.document.createElement('div');body.className='preliminary-pages';body.append(prepared.canvas);
    overlay.append(heading,body);
    this.previousInert=this.pages.inert;this.previousHidden=this.pages.getAttribute('aria-hidden');
    this.overlay=overlay;this.pages.inert=true;this.pages.setAttribute('aria-hidden','true');
    this.container.append(overlay);this.position();
  }
  hide(){
    if(!this.overlay)return;
    this.overlay.remove();this.overlay=null;this.pages.inert=this.previousInert;
    if(this.previousHidden===null)this.pages.removeAttribute('aria-hidden');else this.pages.setAttribute('aria-hidden',this.previousHidden);
  }
}
