import {test} from 'node:test';
import assert from 'node:assert/strict';
import {prepareLongPDF} from '../web/core/long-pdf.js';
import {PDFSearch,pdfSearchKey,pdfSearchRange} from '../web/core/pdf-search.js';
function element(){return {children:[],dataset:{},style:{setProperty(){}},classList:{add(){},remove(){}},setAttribute(){},append(...items){this.children.push(...items)},replaceChildren(...items){this.children=items},contains:()=>false,getContext:()=>({}),scrollIntoView(){this.scrolled=true},querySelectorAll:()=>[]};}
test('a 100-page view paints on demand, evicts old canvases and disposes without destroying the shared PDF',async()=>{
 const previous=globalThis.document;let painted=0,destroyed=0;globalThis.document={createElement:element,createDocumentFragment:element,getSelection:()=>null};globalThis.devicePixelRatio=1;
 const pdf={numPages:100,destroy(){destroyed++},getPage:async number=>({getViewport:({scale})=>({width:300*scale,height:400*scale}),getTextContent:async()=>({items:[{str:'Page '+number}]}),render(){painted++;return {promise:Promise.resolve(),cancel(){}}}})};
 try{const view=await prepareLongPDF({TextLayer:class{render(){return Promise.resolve()}cancel(){}}},pdf,{}, {width:400});
  assert.equal(view.pages.length,100);assert.equal(painted,1);assert.ok(view.pages.every(p=>p.selectionReason));
  for(let n=2;n<=15;n++)await view.ensurePage(n);assert.equal(painted,15);
  assert.equal(view.pages.filter(p=>p.shell.children.length).length,6,'only six raster/text pairs are retained');
  await view.ensurePage(15);assert.equal(painted,15);await view.ensurePage(1);assert.equal(painted,16,'an evicted page repaints');
  view.dispose();view.dispose();assert.equal(destroyed,0);await assert.rejects(view.ensurePage(90),/closed/);
 }finally{globalThis.document=previous;delete globalThis.devicePixelRatio;}
});
test('PDF search reaches unpainted pages and ignores an old document result',async()=>{
 const elements=new Map(['pdf-query','pdf-matches','pdf-find-next','pdf-find-prev'].map(id=>[id,{...element(),value:'needle'}]));let ensured=[];
 const pages=Array.from({length:30},(_,i)=>({shell:element(),number:i+1}));let current={pageCount:30,pages,ensurePage:async n=>ensured.push(n),pdf:{getPage:async n=>({getTextContent:async()=>({items:[{str:n===25?'needle twice needle':'ordinary text'}]})})}};
 const search=new PDFSearch({document:{getElementById:id=>elements.get(id),querySelectorAll:()=>[]},getView:()=>current,notify:error=>{throw Error(error)}});
 await search.find();assert.equal(search.matches.length,2);assert.deepEqual(ensured,[25]);assert.equal(elements.get('pdf-matches').textContent,'1 / 2');await search.step(1);assert.equal(elements.get('pdf-matches').textContent,'2 / 2');
 elements.get('pdf-query').value='ordinary';await search.step(1);assert.equal(search.matches.length,29);assert.equal(search.matches[0].page,1,'Enter after changing a query must search the new text');
 let finish;current.pdf.getPage=async()=>({getTextContent:()=>new Promise(resolve=>finish=resolve)});const pending=search.find();await new Promise(setImmediate);current={...current};finish({items:[{str:'needle'}]});await pending;assert.equal(search.matches.length,0);
});
test('ordinary Arabic finds shaped PDF letters, with original highlight boundaries',async()=>{
 const shaped='ﻣﺭﺣﺑﺎ ﺑﺎﻟﻌﺎﻟﻡ';
 assert.equal(pdfSearchKey(shaped),'مرحبا بالعالم');
 assert.deepEqual(pdfSearchRange(shaped,'مرحبا'),[0,5]);
 const elements=new Map(['pdf-query','pdf-matches','pdf-find-next','pdf-find-prev'].map(id=>[id,{...element(),value:'مرحبا'}]));
 const view={pageCount:1,pages:[{shell:element()}],pdf:{getPage:async()=>({getTextContent:async()=>({items:[{str:shaped}]})})}};
 const search=new PDFSearch({document:{getElementById:id=>elements.get(id),querySelectorAll:()=>[]},getView:()=>view,notify:error=>{throw Error(error)}});
 await search.find();assert.equal(search.matches.length,1);assert.equal(elements.get('pdf-matches').textContent,'1 / 1');
});
test('normalized search highlights complete ligatures and combining clusters without offset drift',()=>{
 assert.deepEqual(pdfSearchRange('x ﻻ y ﻻ','لا',1),[6,7]);
 assert.deepEqual(pdfSearchRange('x ﬃ y','ffi'),[2,3]);
 assert.deepEqual(pdfSearchRange('x ﬃ y','fi'),[2,3]);
 assert.deepEqual(pdfSearchRange('Cafe\u0301 cafe\u0301','café',1),[6,11]);
 assert.deepEqual(pdfSearchRange('😀 שלום','שלום'),[3,7]);
 assert.equal(pdfSearchRange('שלום','missing'),null);
 assert.equal(pdfSearchRange('x',''),null);
});
