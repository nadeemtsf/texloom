import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {loadPDFDocument,startPDFWorker} from '../web/core/pdf-document.js';
import {selectionReasonForPage} from '../web/core/pdf-surface.js';
import {acceptCompilation,decodeCompilation} from '../web/core/build-protocol.js';
import {renderPreliminaryPDF} from '../web/core/preliminary-pdf.js';
const {createCanvas}=createRequire(import.meta.url)('@napi-rs/canvas');

test('the preliminary renderer paints a real PDF page without creating an editable or final-build surface',async()=>{
 const bytes=new Uint8Array(await readFile(new URL('./fixtures/objects/baseline.pdf',import.meta.url)));
 const attributes=new Map();
 const document={createElement(name){assert.equal(name,'canvas');const canvas=createCanvas(1,1);
  canvas.setAttribute=(key,value)=>attributes.set(key,value);return canvas}};
 const rendered=await renderPreliminaryPDF(pdfjs,{pdf:bytes},{document,width:640,page:99,
  resources:{standardFontDataUrl:new URL('../node_modules/pdfjs-dist/standard_fonts/',import.meta.url).href}});
 try{
  assert.equal(rendered.page,1);assert.equal(rendered.pageCount,1);
  assert.equal(attributes.get('role'),'img');assert.match(attributes.get('aria-label'),/Early preview/);
  assert.equal(rendered.build_id,undefined);assert.equal(rendered.objects,undefined);
  const pixels=rendered.canvas.getContext('2d').getImageData(0,0,rendered.canvas.width,rendered.canvas.height).data;
  assert.ok(pixels.some((value,index)=>index%4!==3&&value<100),'actual PDF text ink');
  assert.ok(bytes.length>0,'the PDF.js worker cannot detach the received progress bytes');
 }finally{await rendered.dispose();await rendered.dispose()}
});

test('an aborted preliminary load disposes its PDF even when it resolves after cancellation',async()=>{
 let finish,disposed=0;const abort=new AbortController();
 const pdfjs={getDocument(){return {promise:new Promise(resolve=>finish=resolve),destroy:async()=>{disposed++}}}};
 const rendering=renderPreliminaryPDF(pdfjs,{pdf:[]},{document:{},width:640,signal:abort.signal});
 abort.abort();finish({numPages:1});await assert.rejects(rendering,{name:'AbortError'});assert.equal(disposed,1);
});

test('binary compile delivery preserves real PDF text and every rendered pixel',async()=>{
 const bytes=new Uint8Array(await readFile(new URL('./fixtures/objects/baseline.pdf',import.meta.url)));
 const logical={ok:true,build_id:'binary-render',revision:1,pdf:Array.from(bytes),blocks:[],objects:[],log:'',
  engine:'pdflatex',seconds:0,verification_eligible:true,passes:2,diagnostics:[],job_id:1};
 const metadata=new TextEncoder().encode(JSON.stringify({...logical,pdf:[]}));
 const wire=new Uint8Array(8+metadata.length+bytes.length);wire.set([84,76,80,49]);
 new DataView(wire.buffer).setUint32(4,metadata.length,true);wire.set(metadata,8);wire.set(bytes,8+metadata.length);
 const decoded=acceptCompilation(decodeCompilation(wire.buffer),{revision:1,sources:{},engine:'pdflatex'});
 const outputs=[];
 for(const result of [logical,decoded]){
  // Match the production PDF surface's copy: PDF.js may transfer the buffer to its worker.
  const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(result.pdf)});
  try{
   const page=await pdf.getPage(1),viewport=page.getViewport({scale:1});
   const canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height)),context=canvas.getContext('2d');
   await page.render({canvas,canvasContext:context,viewport}).promise;
   // PDF.js assigns a different document prefix to font IDs on every load.
   outputs.push({text:(await page.getTextContent({includeMarkedContent:true})).items.map(item=>
    item.fontName?{...item,fontName:item.fontName.replace(/^g_d\d+_/u,'g_doc_')}:item),
    pixels:context.getImageData(0,0,canvas.width,canvas.height).data});
  }finally{await pdf.destroy()}
 }
 assert.deepEqual(outputs[1],outputs[0]);
 assert.deepEqual(decoded.pdf,bytes,'worker transfer cannot detach the retained compile reply');
});

function pageFixture(extra){
 const stream='BT /F1 12 Tf 50 100 Td (Page coordinate fixture) Tj ET';
 const objects=['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Kids [3 0 R] /Count 1 >>',`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] ${extra} /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>`,'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',`<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`];
 let text='%PDF-1.7\n';const offsets=[0];for(const [i,body] of objects.entries()){offsets.push(text.length);text+=`${i+1} 0 obj\n${body}\nendobj\n`;}
 const start=text.length;text+=`xref\n0 ${offsets.length}\n0000000000 65535 f \n`+offsets.slice(1).map(n=>String(n).padStart(10,'0')+' 00000 n \n').join('')+`trailer\n<< /Size ${offsets.length} /Root 1 0 R >>\nstartxref\n${start}\n%%EOF\n`;return new TextEncoder().encode(text);
}
for(const [label,extra,size,editable] of [
 ['ordinary','',[300,200],true],['cropped','/CropBox [10 20 270 180]',[260,160],false],
 ['rotated90','/CropBox [10 20 270 180] /Rotate 90',[160,260],false],
 ['rotated180','/CropBox [10 20 270 180] /Rotate 180',[260,160],false],
 ['rotated270','/CropBox [10 20 270 180] /Rotate 270',[160,260],false],
 ['scaled','/CropBox [10 20 270 180] /UserUnit 2',[520,320],false],
])test('real '+label+' page paints and round-trips its coordinates while its editing capability stays explicit',async()=>{
 const pdf=await loadPDFDocument(pdfjs,{data:pageFixture(extra),useSystemFonts:true});
 try{const page=await pdf.getPage(1),viewport=page.getViewport({scale:1});assert.deepEqual([viewport.width,viewport.height],size);
  const canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height)),context=canvas.getContext('2d');await page.render({canvas,canvasContext:context,viewport}).promise;
  const point=viewport.convertToViewportPoint(50,100),back=viewport.convertToPdfPoint(...point);assert.ok(Math.abs(back[0]-50)<1e-8&&Math.abs(back[1]-100)<1e-8);
  assert.ok((await page.getTextContent()).items.some(item=>item.str?.includes('coordinate fixture')));
  const pixels=context.getImageData(0,0,canvas.width,canvas.height).data;assert.ok(pixels.some((value,i)=>i%4!==3&&value<100),'visible text ink');
  assert.equal(selectionReasonForPage(page,viewport)===null,editable);
 }finally{await pdf.destroy()}
});

test('the PDF lifetime belongs to its loading task and is disposed exactly once without a proxy destroy method',async()=>{
 let disposed=0,received;
 const document={numPages:1,getPage(n){assert.equal(this,document);return {number:n}}};
 const handle=await loadPDFDocument({getDocument(options){received=options;return {promise:Promise.resolve(document),destroy:async()=>{disposed++}}}},{data:new Uint8Array(),isEvalSupported:true});
 assert.equal(received.isEvalSupported,false);assert.equal(handle.numPages,1);assert.deepEqual(handle.getPage(1),{number:1});
 assert.equal(document.destroy,undefined);
 const one=handle.destroy(),two=handle.destroy();assert.equal(one,two);await Promise.all([one,two]);assert.equal(disposed,1);
});

test('a failed PDF load disposes its task and preserves the original error even if cleanup also fails',async()=>{
 const original=Error('Invalid PDF input');let disposed=0;
 await assert.rejects(loadPDFDocument({getDocument:()=>({promise:Promise.reject(original),destroy:async()=>{disposed++;throw Error('Cleanup failure')}})},{data:new Uint8Array()}),error=>error===original);
 assert.equal(disposed,1);
});

test('the patched PDF.js opens, paints and extracts real fixture text through the production lifetime handle',async()=>{
 const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(await readFile(new URL('./fixtures/objects/baseline.pdf',import.meta.url)))});
 try{
  assert.equal(pdf.numPages,1);
  const page=await pdf.getPage(1),viewport=page.getViewport({scale:1});
  const canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height)),context=canvas.getContext('2d');
  await page.render({canvas,canvasContext:context,viewport}).promise;
  const text=(await page.getTextContent({includeMarkedContent:true})).items.map(item=>item.str||'').join(' ');
  assert.match(text,/Dr\. Example Researcher/);
  const image=context.getImageData(0,0,canvas.width,canvas.height).data;
  let painted=false;for(let i=0;i<image.length;i+=4){if(image[i]<240&&image[i+3]>0){painted=true;break}}
  assert.equal(painted,true,'real page ink was painted');
 }finally{await pdf.destroy()}
});


test('an unused PDF worker is disposed and a transferred worker belongs to its document',async()=>{
 const order=[];
 const worker={promise:Promise.resolve(),destroy(){order.push('worker')}};
 const pdfjs={PDFWorker:{create:()=>worker},getDocument(options){
  assert.equal(options.worker,worker);
  return {promise:Promise.resolve({numPages:1}),destroy:async()=>order.push('task')};
 }};
 const unused=startPDFWorker(pdfjs);
 unused.dispose();unused.dispose();
 assert.equal(await unused.take(),undefined);assert.deepEqual(order,['worker']);
 order.length=0;
 const startup=startPDFWorker(pdfjs);
 const owned=await startup.take();
 assert.equal(owned,worker);
 assert.equal(await startup.take(),undefined);
 startup.dispose();assert.deepEqual(order,[]);
 const pdf=await loadPDFDocument(pdfjs,{},owned);
 await Promise.all([pdf.destroy(),pdf.destroy()]);
 assert.deepEqual(order,['task','worker']);
});

test('PDF worker startup exceptions fall back to the ordinary document load',async()=>{
 const original=Error('Worker startup refused');let destroyed=0;
 const worker={promise:Promise.reject(original),destroy(){destroyed++}};
 const library={PDFWorker:{create:()=>worker},getDocument(options){
  assert.equal(Object.hasOwn(options,'worker'),false);
  return {promise:Promise.resolve({numPages:1}),destroy:async()=>{}};
 }};
 const startup=startPDFWorker(library);
 const pdf=await loadPDFDocument(library,{},await startup.take());
 await pdf.destroy();startup.dispose();
 assert.equal(destroyed,1);
 assert.equal(startPDFWorker({PDFWorker:{create(){throw original}}}),null);
});

test('PDF input refusal still destroys an owned worker after the loading task',async()=>{
 const original=Error('Invalid document');
 const order=[];
 const worker={destroy(){order.push('worker')}};
 const library={getDocument(){return {promise:Promise.reject(original),destroy:async()=>{
   order.push('task');throw Error('Cleanup failed');
 }}}};
 await assert.rejects(loadPDFDocument(library,{},worker),error=>error===original);
 assert.deepEqual(order,['task','worker']);
});
