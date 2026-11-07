// Node-only rendered PDF view for development proofs; not a product renderer.
import {readFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {loadPDFDocument} from '../web/core/pdf-document.js';
import {extractTextGeometry,union} from '../web/core/geometry.js';
const {createCanvas}=createRequire(import.meta.url)('@napi-rs/canvas');
export async function proofPDFView(path,id){
 const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(await readFile(path))});
 const view={result:{build_id:id},pageCount:pdf.numPages,pages:[],pageSizes:new Map(),objects:new Map(),unclosed:new Set()};
 try{
  for(let n=1;n<=pdf.numPages;n++){
   const page=await pdf.getPage(n),viewport=page.getViewport({scale:1});
   const canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height));
   await page.render({canvas,canvasContext:canvas.getContext('2d'),viewport}).promise;
   const text=await page.getTextContent({includeMarkedContent:true});
   const geometry=extractTextGeometry(text.items,text.styles,n,viewport.height);
   view.pages.push({number:n,runs:geometry.runs,lines:geometry.lines,shell:{querySelector:()=>canvas}});
   view.pageSizes.set(n,{width:viewport.width,height:viewport.height});
   for(const marker of geometry.unclosed)view.unclosed.add(marker);
   for(const [marker,bounds] of geometry.objects){
    const old=view.objects.get(marker);
    if(old){old.pages.add(n);old.runs.push(...bounds.runs);old.instances+=bounds.instances;Object.assign(old,union([old,bounds]));}
    else view.objects.set(marker,{...bounds,pages:new Set([n])});
   }
  }
  return view;
 }finally{await pdf.destroy()}
}
