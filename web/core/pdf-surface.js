import {extractTextGeometry,union} from './geometry.js';

/** Prepare a complete PDF view offscreen. Publishing it is a separate operation. */
export async function preparePDFView(pdfjs,result,{width,zoom=0,mode='select',signal}={}) {
  const loading=pdfjs.getDocument({data:new Uint8Array(result.pdf),isEvalSupported:false,
    cMapUrl:new URL('/pdfjs/cmaps/',location.href).href,cMapPacked:true,
    standardFontDataUrl:new URL('/pdfjs/standard_fonts/',location.href).href,wasmUrl:new URL('/pdfjs/wasm/',location.href).href});
  const pdf=await loading.promise;
  try{return await prepareLoadedPDF(pdfjs,pdf,result,{width,zoom,mode,signal})}
  catch(error){await pdf.destroy();throw error}
}

export async function prepareLoadedPDF(pdfjs,pdf,result,{width,zoom=0,mode='select',signal}={}) {
  const fragment=document.createDocumentFragment(),pages=[],objects=new Map(),unclosed=new Set(),pageSizes=new Map();
  const first=await pdf.getPage(1),scale=zoom||Math.max(0.2,Math.min(1.5,(width-48)/first.getViewport({scale:1}).width));
  for(let n=1;n<=pdf.numPages;n++){
    if(signal?.aborted)throw new Error('PDF rendering was superseded.');
    const page=await pdf.getPage(n),base=page.getViewport({scale:1}),viewport=page.getViewport({scale});
    pageSizes.set(n,{width:base.width,height:base.height});
    const shell=document.createElement('div');shell.className='pdf-page';
    Object.assign(shell.dataset,{page:String(n),scale:String(scale),pdfWidth:String(base.width),pdfHeight:String(base.height)});
    Object.assign(shell.style,{width:viewport.width+'px',height:viewport.height+'px'});
    shell.style.setProperty('--total-scale-factor',String(scale));shell.style.setProperty('--scale-factor',String(scale));
    const canvas=document.createElement('canvas'),ratio=Math.min(2.5,devicePixelRatio||1);
    canvas.width=Math.ceil(viewport.width*ratio);canvas.height=Math.ceil(viewport.height*ratio);
    canvas.style.width=viewport.width+'px';canvas.style.height=viewport.height+'px';shell.append(canvas);
    const rendering=page.render({canvasContext:canvas.getContext('2d'),viewport,transform:[ratio,0,0,ratio,0,0]});
    const abort=()=>rendering.cancel();signal?.addEventListener('abort',abort,{once:true});
    try{await rendering.promise}finally{signal?.removeEventListener('abort',abort)}
    const text=await page.getTextContent({includeMarkedContent:true});
    const layer=document.createElement('div');layer.className='textLayer';shell.append(layer);
    await new pdfjs.TextLayer({textContentSource:text,container:layer,viewport}).render();
    const geometry=extractTextGeometry(text.items,text.styles,n,base.height);
    for(const id of geometry.unclosed)unclosed.add(id);
    for(const [id,bounds] of geometry.objects){
      const existing=objects.get(id);
      if(existing){existing.pages.add(n);Object.assign(existing,union([existing,bounds]));existing.runs.push(...bounds.runs);existing.instances+=bounds.instances}
      else objects.set(id,{...bounds,pages:new Set([n])});
    }
    const selectionReason=base.rotation!==0||page.view[0]!==0||page.view[1]!==0||page.userUnit!==1?
      'Visual selection for rotated, cropped or scaled PDF pages is not verified. Read text and source editing remain available.':null;
    pages.push({number:n,shell,lines:geometry.lines,runs:geometry.runs,selectionReason});fragment.append(shell);
  }
  if(signal?.aborted)throw new Error('PDF rendering was superseded.');
  return {pdf,result,fragment,pages,objects,unclosed,pageSizes,pageCount:pdf.numPages,scale,mode};
}

export function publishPDFView(prepared,elements) {
  const scroll=elements.scroller.scrollTop;
  elements.pages.replaceChildren(prepared.fragment);
  elements.pages.classList.toggle('read-mode',prepared.mode==='read');
  elements.scroller.scrollTop=scroll;
  elements.count.textContent=prepared.pageCount+' '+(prepared.pageCount===1?'page':'pages');
  elements.empty.classList.add('hidden');
}
