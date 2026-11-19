import {test} from 'node:test';
import assert from 'node:assert/strict';
import {prepareLongPDF} from '../web/core/long-pdf.js';
import {PDFSearch,pdfSearchKey,pdfSearchRange} from '../web/core/pdf-search.js';
import {capturePDFScrollAnchor,restorePDFScrollAnchor} from '../web/core/pdf-surface.js';
import {preparePagedPDF} from '../web/core/pdf-page-view.js';
function element(){return {children:[],dataset:{},style:{setProperty(){}},classList:{add(){},remove(){}},setAttribute(){},append(...items){this.children.push(...items)},replaceChildren(...items){this.children=items},contains:()=>false,getContext:()=>({}),scrollIntoView(){this.scrolled=true},querySelectorAll:()=>[]};}
test('zoom preserves the page and fraction at the reading viewport center',()=>{
 const scroller={scrollTop:74000,getBoundingClientRect:()=>({top:200,bottom:800})};
 const shell=(page,top,height)=>({dataset:{page:String(page)},getBoundingClientRect:()=>({top,bottom:top+height,height})});
 const elements={scroller,pages:{children:[shell(74,-700,1000),shell(75,320,1000),shell(76,1340,1000)]}};
 const anchor=capturePDFScrollAnchor(elements);assert.deepEqual(anchor,{page:'75',fraction:0.18});
 elements.pages.children=[shell(74,6000,1150),shell(75,7170,1150),shell(76,8340,1150)];
 restorePDFScrollAnchor(elements,anchor);assert.equal(scroller.scrollTop,80877);
 assert.equal(7170-(scroller.scrollTop-74000)+1150*anchor.fraction,500,'same position on page75 remains centered');
 const before=scroller.scrollTop;restorePDFScrollAnchor(elements,{page:'missing',fraction:0.5});assert.equal(scroller.scrollTop,before);
 elements.pages.children=[];assert.equal(capturePDFScrollAnchor(elements),null);
});
const fakeViewer={AnnotationMode:{ENABLE:1},SimpleLinkService:class{},
 EventBus:class{
  constructor(){this.events=new Map()}
  on(n,fn){if(!this.events.has(n))this.events.set(n,new Set());this.events.get(n).add(fn)}
  off(n,fn){this.events.get(n)?.delete(fn)}
  dispatch(n,event){for(const fn of this.events.get(n)||[])fn(event)}
 },
 PDFPageView:class{
  constructor(o){this.o=o;this.div=element();this.viewport=o.defaultViewport;this.renderingState=0;o.container.append(this.div)}
  setPdfPage(p){this.page=p}update(){this.renderingState=0}reset(){this.div.replaceChildren();this.renderingState=0}
  async draw(){await this.page.render().promise;this.div.append(element());this.renderingState=3;this.o.eventBus.dispatch('textlayerrendered',{source:this})}
  destroy(){this.reset()}
 }};
function deferredPDF(){
 const started=[],paint=new Map();
 const pdf={numPages:3,getPage:async number=>({
  getViewport:({scale})=>({width:300*scale,height:400*scale}),
  render(){
   started.push(number);
   return {promise:new Promise((resolve,reject)=>paint.set(number,{resolve,reject}))};
  }
 })};
 return {pdf,started,paint};
}
function withFakeDOM(t){
 const previous=globalThis.document;
 globalThis.document={createElement:element,createDocumentFragment:element,getSelection:()=>null};
 t.after(()=>{globalThis.document=previous});
}
function nextTurn(){return new Promise(setImmediate)}

test('a short PDF overlaps two page jobs and waits for every page to finish',async t=>{
 withFakeDOM(t);
 const {pdf,started,paint}=deferredPDF();
 const preparation=preparePagedPDF(fakeViewer,pdf,{}, {width:400});
 let prepared=false;
 preparation.then(()=>{prepared=true},()=>{});
 await nextTurn();
 assert.deepEqual(started,[1,2]);
 paint.get(2).resolve();
 await nextTurn();
 assert.deepEqual(started,[1,2,3]);
 paint.get(3).resolve();
 await nextTurn();
 assert.equal(prepared,false);
 paint.get(1).resolve();
 const view=await preparation;
 assert.ok(view.pages.every(item=>item.pageView.div.children.length===1));
 view.dispose();
});

test('a failed short-view paint disposes the overlapping and queued work',async t=>{
 withFakeDOM(t);
 const {pdf,started,paint}=deferredPDF();
 let destroyed=0;
 const viewer={...fakeViewer,PDFPageView:class extends fakeViewer.PDFPageView{
  destroy(){destroyed++;super.destroy()}
 }};
 const preparation=preparePagedPDF(viewer,pdf,{}, {width:400});
 await nextTurn();
 assert.deepEqual(started,[1,2]);
 const rejected=assert.rejects(preparation,/synthetic page failure/);
 paint.get(2).reject(Error('synthetic page failure'));
 await rejected;
 assert.equal(destroyed,3);
 assert.deepEqual(started,[1,2]);
});

test('a 100-page view fetches bounded metadata batches, paints on demand and disposes without destroying the PDF',async()=>{
 const previous=globalThis.document;let painted=0,destroyed=0,active=0,peak=0;
 globalThis.document={createElement:element,createDocumentFragment:element,getSelection:()=>null};globalThis.devicePixelRatio=1;
 const pdf={numPages:100,destroy(){destroyed++},getPage:async number=>{
  peak=Math.max(peak,++active);
  await Promise.resolve();
  active--;
  return {getViewport:({scale})=>({width:300*scale,height:400*scale}),getTextContent:async()=>({items:[{str:'Page '+number}]}),render(){painted++;return {promise:Promise.resolve(),cancel(){}}}};
 }};
 try{const view=await prepareLongPDF(fakeViewer,pdf,{}, {width:400});
  assert.equal(view.pages.length,100);assert.equal(painted,1);assert.ok(view.pages.every(p=>p.selectionReason));
  assert.equal(peak,8);
  assert.deepEqual(view.pages.map(item=>item.number),Array.from({length:100},(_,i)=>i+1));
  for(let n=2;n<=15;n++)await view.ensurePage(n);assert.equal(painted,15);
  assert.equal(view.pages.filter(p=>p.pageView.div.children.length).length,6,'only six raster/text pairs are retained');
  await view.ensurePage(15);assert.equal(painted,15);await view.ensurePage(1);assert.equal(painted,16,'an evicted page repaints');
  view.dispose();view.dispose();assert.equal(destroyed,0);await assert.rejects(view.ensurePage(90),/closed/);
 }finally{globalThis.document=previous;delete globalThis.devicePixelRatio;}
});

test('a failed reading batch settles pending lookups and reports the first failing page',async t=>{
 withFakeDOM(t);
 const {pdf,started}=deferredPDF(),fetch=pdf.getPage;
 let release;
 const delayed=new Promise(resolve=>{release=resolve});
 pdf.numPages=40;
 const requested=[];
 pdf.getPage=async number=>{
  requested.push(number);
  if(number===3){await delayed;throw Error('page 3 refused')}
  if(number===4)throw Error('page 4 refused');
  return fetch(number);
 };
 const failed=assert.rejects(prepareLongPDF(fakeViewer,pdf,{}, {width:400}),/page 3 refused/);
 await nextTurn();
 assert.deepEqual(requested,[1,2,3,4,5,6,7,8,9]);
 release();
 await failed;
 assert.deepEqual(started,[]);
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
