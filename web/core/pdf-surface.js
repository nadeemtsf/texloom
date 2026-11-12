import {extractTextGeometry,union} from './geometry.js';
import {loadPDFDocument} from './pdf-document.js';
import {prepareLongPDF} from './long-pdf.js';
import {preparePagedPDF} from './pdf-page-view.js';

export function selectionReasonForPage(page,viewport){
  return viewport.rotation!==0||page.view[0]!==0||page.view[1]!==0||page.userUnit!==1?
    'Visual selection for rotated, cropped or scaled PDF pages is not verified. Read text and source editing remain available.':null;
}

/** Prepare a complete PDF view offscreen. Publishing it is a separate operation. */
export async function preparePDFView(pdfjs,result,{width,zoom=0,mode='select',signal,lazy=false}={}) {
  const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(result.pdf),
    cMapUrl:new URL('/pdfjs/cmaps/',location.href).href,cMapPacked:true,
    standardFontDataUrl:new URL('/pdfjs/standard_fonts/',location.href).href,wasmUrl:new URL('/pdfjs/wasm/',location.href).href});
  try{return await prepareLoadedPDF(pdfjs,pdf,result,{width,zoom,mode,signal,lazy})}
  catch(error){try{await pdf.destroy()}catch{}throw error}
}

export async function prepareLoadedPDF(pdfjs,pdf,result,{width,zoom=0,mode='select',signal,lazy=false}={}) {
  if(lazy&&pdf.numPages>20)return prepareLongPDF(pdfjs,pdf,result,{width,zoom,mode,signal});
  return preparePagedPDF(pdfjs,pdf,result,{width,zoom,mode,signal,geometry:async(page,base,item,view)=>{
    const text=await page.getTextContent({includeMarkedContent:true});
    const geometry=extractTextGeometry(text.items,text.styles,item.number,base.height);
    for(const id of geometry.unclosed)view.unclosed.add(id);
    for(const [id,bounds] of geometry.objects){
      const existing=view.objects.get(id);
      if(existing){existing.pages.add(item.number);Object.assign(existing,union([existing,bounds]));existing.runs.push(...bounds.runs);existing.instances+=bounds.instances}
      else view.objects.set(id,{...bounds,pages:new Set([item.number])});
    }
    Object.assign(item,{lines:geometry.lines,runs:geometry.runs,selectionReason:selectionReasonForPage(page,base)});
  }});
}

export function capturePDFScrollAnchor(elements){
  const viewport=elements.scroller.getBoundingClientRect(),center=(viewport.top+viewport.bottom)/2;
  for(const shell of elements.pages.children){
    const rect=shell.getBoundingClientRect();
    if(rect.bottom>=center&&rect.height>0)return {page:shell.dataset.page,fraction:Math.max(0,Math.min(1,(center-rect.top)/rect.height))};
  }
  return null;
}
export function restorePDFScrollAnchor(elements,anchor){
  if(!anchor)return;
  const shell=[...elements.pages.children].find(page=>page.dataset.page===anchor.page);if(!shell)return;
  const rect=shell.getBoundingClientRect(),viewport=elements.scroller.getBoundingClientRect();
  elements.scroller.scrollTop+=rect.top+rect.height*anchor.fraction-(viewport.top+viewport.bottom)/2;
}
export function publishPDFView(prepared,elements,{preserveAnchor=false}={}) {
  const anchor=preserveAnchor?capturePDFScrollAnchor(elements):null;
  const scroll=elements.scroller.scrollTop;
  elements.pages.replaceChildren(prepared.fragment);
  elements.pages.classList.toggle('read-mode',prepared.mode==='read');
  elements.scroller.scrollTop=scroll;
  restorePDFScrollAnchor(elements,anchor);
  elements.count.textContent=prepared.pageCount+' '+(prepared.pageCount===1?'page':'pages');
  elements.empty.classList.add('hidden');
  prepared.mount?.({...elements,onError:error=>{elements.count.textContent=String(error?.message||error)}});
}
