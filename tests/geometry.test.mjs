import {test} from 'node:test';
import assert from 'node:assert/strict';
import {extractTextGeometry,validateObjectLayout,groupTextRuns,linesForSourceRectangles} from '../web/core/geometry.js';
const text=(str,x=20,y=700)=>({str,transform:[12,0,0,12,x,y],width:60,fontName:'bold'});
test('nested PDF marked content retains exact object identity and transformed glyph bounds',()=>{
 const result=extractTextGeometry([{type:'beginMarkedContent',tag:'Texloom_name'},{type:'beginMarkedContent',tag:'Span'},text('Name'),{type:'endMarkedContent'},{type:'endMarkedContent'},text('Outside',100)],{bold:{ascent:0.8}},1,800);
 assert.equal(result.objects.get('name').runs[0].text,'Name');assert.equal(result.objects.get('name').y,90.4);assert.equal(result.unclosed.size,0);
 assert.equal(result.objects.get('name').runs.length,1);
});
test('page-spanning unclosed marked content is detectable',()=>{
 const result=extractTextGeometry([{type:'beginMarkedContent',tag:'Texloom_name'},text('Name')],{},1,800);assert.ok(result.unclosed.has('name'));
});
test('text on separate columns remains separate selection targets',()=>{
 const lines=groupTextRuns([{text:'Left',x:20,y:10,width:80,height:12,baseline:20},{text:'Right',x:300,y:10,width:80,height:12,baseline:20}],1);assert.equal(lines.length,2);
 assert.equal(linesForSourceRectangles(lines,[{page:1,x:18,y:8,width:90,height:15}],1).length,1);
});
test('layout checks reject absent, clipped and cross-page objects',()=>{
 const view={pageCount:1,objects:new Map(),unclosed:new Set(),pageSizes:new Map([[1,{width:600,height:800}]])},policy={id:'name',kind:'move'};
 assert.throws(()=>validateObjectLayout(view,policy,1),/verifiable/);
 view.objects.set('name',{page:1,pages:new Set([1]),x:590,y:10,width:100,height:12});assert.throws(()=>validateObjectLayout(view,policy,1),/outside/);
 view.objects.get('name').pages.add(2);assert.throws(()=>validateObjectLayout(view,policy,1),/boundary/);
});
test('a valid in-page object passes without altering its geometry',()=>{
 const bounds={page:1,pages:new Set([1]),x:50,y:500,width:200,height:40};
 validateObjectLayout({pageCount:1,objects:new Map([['name',bounds]]),unclosed:new Set(),pageSizes:new Map([[1,{width:600,height:800}]])},{id:'name',kind:'move'},1);assert.equal(bounds.y,500);
});
