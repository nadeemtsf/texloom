import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {extractTextGeometry,validateObjectLayout} from '../web/core/geometry.js';

async function geometry(name){
 const data=new Uint8Array(await readFile(new URL(`./fixtures/objects/${name}.pdf`,import.meta.url)));
 const pdf=await pdfjs.getDocument({data,isEvalSupported:false}).promise;
 try{
  const page=await pdf.getPage(1),viewport=page.getViewport({scale:1}),text=await page.getTextContent({includeMarkedContent:true});
  const g=extractTextGeometry(text.items,text.styles,1,viewport.height);
  for(const object of g.objects.values())object.pages=new Set([1]);
  return {...g,pageCount:pdf.numPages,pageSizes:new Map([[1,viewport]]),pages:[{runs:g.runs}]};
 }finally{await pdf.destroy()}
}
test('real PDF regression: direct graphics literals must not shift unselected text',async()=>{
 const baseline=await geometry('baseline'),broken=await geometry('single-move');
 const unchangedRuns=baseline.runs.filter(run=>run.text!=='Dr. Example Researcher');
 const id=[...broken.objects.keys()][0];
 assert.throws(()=>validateObjectLayout(broken,{id,kind:'move',unchangedRuns},1),/surrounding text/);
});
