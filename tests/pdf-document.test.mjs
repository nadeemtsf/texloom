import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {loadPDFDocument} from '../web/core/pdf-document.js';
const {createCanvas}=createRequire(import.meta.url)('@napi-rs/canvas');

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
