import {test} from 'node:test';
import assert from 'node:assert/strict';
import {checkTranslation,checkRaster,targetRecords} from '../scripts/proof/layout-check.mjs';
import {inspectPdfjs} from '../scripts/proof/pdfjs-inspector.mjs';

const record=(text,x,y)=>({text,x,y,width:1,height:1,page:1});
const snapshot=records=>({pages:[{width:10,height:10}],records,graphics:[]});
test('proof oracle rejects wrong movement, changed neighbors, graphics and ambiguous occurrences',()=>{
  const a=snapshot([record('A',1,1),record('B',7,7)]);
  const b=snapshot([record('A',1,3),record('B',7,7)]);
  assert.deepEqual([...checkTranslation(a,b,{text:'A',dx:0,dy:2})],[0]);
  assert.throws(()=>checkTranslation(a,b,{text:'A',dx:0,dy:1}),/Target/);
  assert.throws(()=>checkTranslation(a,{...b,records:[b.records[0],record('B',7,7.03)]},{text:'A',dx:0,dy:2}),/Neighbor/);
  assert.throws(()=>checkTranslation(a,{...b,graphics:[{type:2}]},{text:'A',dx:0,dy:2}),/Non-text/);
  assert.throws(()=>targetRecords([record('A',1,1),record('A',2,2)],'A'),/Ambiguous/);
  assert.throws(()=>targetRecords([record('AB',1,1)],'A'),/crosses/);
  assert.throws(()=>checkTranslation(a,b,{text:'A',dx:NaN,dy:2}),/Invalid/);
});
test('pixel oracle detects unrelated drawing changes and refuses masks covering other content',()=>{
  const a={...snapshot([record('A',1,1),record('B',7,7)]),rasters:[{width:10,height:10,stride:10,channels:1,scale:1}]};
  const b={...a,records:[record('A',1,3),record('B',7,7)]};
  const old=Buffer.alloc(100),next=Buffer.alloc(100);next[11]=1;next[31]=2;
  assert.ok(checkRaster(a,b,[old],[next],new Set([0]),{padding:0}).unmaskedPixelsChecked>0);
  next[99]=1;assert.throws(()=>checkRaster(a,b,[old],[next],new Set([0]),{padding:0}),/Unselected pixels/);
  assert.throws(()=>checkRaster(a,b,[old],[old],new Set([0]),{identity:true,padding:NaN}),/padding/);
  const overlapping={...a,records:[record('A',1,1),record('B',1,1)]};
  assert.throws(()=>checkRaster(overlapping,overlapping,[old],[old],new Set([0])),/overlaps/);
});
test('independent proof inspector detects the real direct-literal regression',async()=>{
  const a=await inspectPdfjs(new URL('./fixtures/objects/baseline.pdf',import.meta.url));
  const b=await inspectPdfjs(new URL('./fixtures/objects/single-move.pdf',import.meta.url));
  assert.throws(()=>checkTranslation(a,b,{text:'Dr. Example Researcher',dx:0,dy:98,tolerance:0.25}),/Neighbor/);
});
