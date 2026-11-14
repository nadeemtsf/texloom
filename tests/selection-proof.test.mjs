import {test} from 'node:test';
import assert from 'node:assert/strict';
import {validateSelectionProbe,canvasRaster} from '../web/core/selection-proof.js';
import {extractTextGeometry} from '../web/core/geometry.js';

const run=(text,x,y=50)=>({text,x,y,width:80,height:12,baseline:y+10,size:12,page:1});
function fixture(){
  const a=run('Repeated name',20),b=run('Repeated name',20,80),c=run('Department',20,100);
  const lines=[a,b,c].map((r,i)=>({...r,id:'line'+i,runs:[r]}));
  const before={result:{build_id:'baseline'},pageCount:1,pages:[{number:1,runs:[a,b,c],lines,raster:{width:1,height:1,data:new Uint8Array([0,0,0,255])}}],pageSizes:new Map([[1,{width:600,height:800}]])};
  const after=structuredClone(before);after.objects=new Map([['target',{...a,page:1,pages:new Set([1]),instances:1,runs:[after.pages[0].runs[0]]}]]);after.unclosed=new Set();
  return {before,after,hits:[lines[0]],options:{readRaster:p=>p.raster}};
}
test('a zero-offset probe identifies the actual occurrence without mutating either view',()=>{
 const {before,after,hits,options}=fixture(),original=structuredClone(before);
 const proof=validateSelectionProbe(before,after,'target',hits,1,options);
 assert.equal(proof.runs[0],before.pages[0].runs[0]);assert.equal(proof.build_id,'baseline');assert.equal(proof.schema,1);assert.deepEqual(before,original);
});
test('identical text at another position cannot authorize the clicked occurrence',()=>{
 const {before,after,options}=fixture();assert.throws(()=>validateSelectionProbe(before,after,'target',[before.pages[0].lines[1]],1,options),/clicked text/);
});
test('repeated instances of one source fragment are refused even on the same page',()=>{
 const {before,after,hits,options}=fixture();after.objects.get('target').instances=2;
 assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options),/more than once/);
});
test('a whole paragraph may expand the clicked line while retaining its exact runs',()=>{
 const {before,after,hits,options}=fixture();after.objects.get('target').runs.push(after.pages[0].runs[1]);
 const proof=validateSelectionProbe(before,after,'target',hits,1,options);assert.equal(proof.lines.length,2);assert.equal(proof.runs.length,2);
});
test('a tiny unrelated position change and a text change both refuse selection',()=>{
 for(const change of [r=>r.x+=0.001,r=>r.text='Changed']){
  const {before,after,hits,options}=fixture();change(after.pages[0].runs[2]);
  assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options),/text or layout/);
 }
});
test('non-text raster changes are checked across the entire page',()=>{
 const {before,after,hits,options}=fixture();after.pages[0].raster.data[0]=1;
 assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options),/appearance/);
});
test('page changes, missing identities, unclosed markers and page-spanning targets are refused',()=>{
 for(const mutate of [p=>p.pageCount++,p=>p.objects.clear(),p=>p.unclosed.add('target'),p=>p.objects.get('target').pages.add(2)]){
  const {before,after,hits,options}=fixture();mutate(after);assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options));
 }
});
test('overlapping duplicate glyph records never get a guessed identity',()=>{
 const {before,after,hits,options}=fixture();before.pages[0].runs.push({...before.pages[0].runs[0]});after.pages[0].runs.push({...after.pages[0].runs[0]});
 assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options),/ambiguous/);
});
test('unsupported page coordinates and verification memory limits fail explicitly',()=>{
 const {before,after,hits,options}=fixture();before.pages[0].selectionReason='Rotated page';
 assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,options),/Rotated/);delete before.pages[0].selectionReason;
 assert.throws(()=>validateSelectionProbe(before,after,'target',hits,1,{...options,maxBytes:1}),/verification limit/);
});
test('PDF marker extraction counts distinct occurrences instead of silently merging them',()=>{
 const items=[];for(let i=0;i<2;i++)items.push({type:'beginMarkedContent',tag:'Texloom_repeat'},{str:'Text',transform:[12,0,0,12,20,700-i*20],width:30,fontName:'a'},{type:'endMarkedContent'});
 assert.equal(extractTextGeometry(items,{},1,800).objects.get('repeat').instances,2);
});

test('identity reads the page renderer canvas and refuses an unpainted zero-sized bitmap',()=>{
 const canvas={width:2,height:1,getContext:()=>({getImageData:()=>({data:new Uint8Array(8)})})};
 const page={pageView:{canvas},shell:{querySelector(){throw Error('must not select an auxiliary canvas')}}};
 assert.equal(canvasRaster(page).width,2);canvas.width=0;assert.throws(()=>canvasRaster(page),/verification limit/);
});
