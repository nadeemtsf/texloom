/** Use the installed Mozilla PDF.js page renderer, including its cached-canvas
 * CSS zoom / delayed sharp redraw path. PDFPageView uses CSS_UNITS internally;
 * Texloom's source geometry and public view.scale remain in PDF points. */
const CSS_UNITS=96/72;
export const PDF_ZOOM_DELAY=400;

export function capturePDFZoomAnchor(view,scroller,origin){
  const bounds=scroller.getBoundingClientRect();
  const x=Number.isFinite(origin?.[0])?origin[0]:bounds.left+scroller.clientWidth/2;
  const y=Number.isFinite(origin?.[1])?origin[1]:bounds.top+scroller.clientHeight/2;
  let nearest=null,distance=Infinity;
  for(const item of view.pages){
    const rect=item.pageView.div.getBoundingClientRect();
    if(!rect.width||!rect.height)continue;
    const d=Math.max(rect.top-y,0,y-rect.bottom);
    if(d<distance){nearest={item,rect};distance=d;if(!d)break}
  }
  if(!nearest)return null;
  const {item,rect}=nearest;
  return {item,x,y,point:item.pageView.getPagePoint(x-rect.left,y-rect.top)};
}
export function restorePDFZoomAnchor(scroller,anchor){
  if(!anchor)return;
  const {item,x,y,point}=anchor,viewport=item.pageView.viewport,rect=item.pageView.div.getBoundingClientRect();
  const [px,py]=viewport.convertToViewportPoint(...point);
  // Browser scrolling clamps naturally at physical document edges.
  scroller.scrollLeft+=rect.left+px-x;
  scroller.scrollTop+=rect.top+py-y;
}

/** Short candidate views still paint completely before publication. Reading
 * views use the same page renderer with a two-job queue and six-page cache. */
export async function preparePagedPDF(pdfjs,pdf,result,{width,zoom=0,mode='select',signal,readingOnly=false,geometry=null}={}){
  if(pdf.numPages>5000)throw Error('This PDF exceeds the 5,000-page reading limit. Source editing is available.');
  const first=await pdf.getPage(1),scale=zoom||Math.max(0.2,Math.min(1.5,(width-48)/first.getViewport({scale:1}).width));
  const fragment=document.createDocumentFragment(),pages=[],pageSizes=new Map(),objects=new Map(),unclosed=new Set();
  const bus=new pdfjs.EventBus(),cache=new Map(),running=new Map(),waiting=new Map(),queue=[],visible=new Set();
  let disposed=false,observer=null,scroller=null,notify=()=>{},clock=0,version=0,zooming=false;
  const view={pdf,result,fragment,pages,objects,unclosed,pageSizes,pageCount:pdf.numPages,scale,mode,
    lazy:readingOnly,ensurePage,mount,dispose,zoomTo,finishZoom,prepareSelectionProof};
  const check=()=>{if(disposed||signal?.aborted)throw Error('PDF view was closed.')};
  const pixelRatio=()=>globalThis.devicePixelRatio||1;
  const syncSize=item=>{
    const {viewport}=item.pageView;
    Object.assign(item.shell.style,{width:viewport.width+'px',height:viewport.height+'px'});
    item.shell.dataset.scale=String(view.scale);
  };
  signal?.addEventListener('abort',dispose,{once:true});
  try{
    for(let n=1;n<=pdf.numPages;n++){
      check();const page=n===1?first:await pdf.getPage(n),base=page.getViewport({scale:1});
      const shell=document.createElement('div');shell.className='pdf-page pending-page';
      Object.assign(shell.dataset,{page:String(n),scale:String(scale),pdfWidth:String(base.width),pdfHeight:String(base.height)});
      shell.setAttribute('aria-label','PDF page '+n);
      const linkService=new pdfjs.SimpleLinkService();linkService.externalLinkEnabled=false;
      const pageView=new pdfjs.PDFPageView({container:shell,id:n,eventBus:bus,scale:scale/CSS_UNITS,
        defaultViewport:page.getViewport({scale}),enableDetailCanvas:false,enableAutoLinking:false,
        // Use the declared budgets below, not a screen-area heuristic that can
        // allocate unequal proof canvases after a display/metrics transition.
        capCanvasAreaFactor:-1,
        maxCanvasPixels:readingOnly?16_000_000:Math.min(16_000_000,Math.floor(64_000_000/pdf.numPages)),
        annotationMode:pdfjs.AnnotationMode.ENABLE,
        layerProperties:{linkService,enableScripting:false,annotationEditorUIManager:null},
        optionalContentConfigPromise:pdf.getOptionalContentConfig?.({intent:'display'})});
      pageView.setPdfPage(page);
      const item={number:n,shell,pageView,lines:[],runs:[],paintedVersion:-1,paintedDpr:0,
        selectionReason:readingOnly?'This long document supports reading and source editing. Layout verification is limited to 20 pages.':null};
      pages.push(item);pageSizes.set(n,{width:base.width,height:base.height});syncSize(item);fragment.append(shell);
      if(geometry)await geometry(page,base,item,view);
    }
    // A rejected candidate never replaces the visible accepted view.
    if(readingOnly)await ensurePage(1);
    else for(const item of pages)await ensurePage(item.number);
    check();return view;
  }catch(error){dispose();throw error}

  async function paint(item,atVersion){
    check();const pv=item.pageView,ratio=pixelRatio();
    pv.update({scale:view.scale/CSS_UNITS});
    // draw() resolves the raster before every text-layer promise; require both.
    let settle;const textReady=new Promise((resolve,reject)=>{settle={resolve,reject}});
    const onText=event=>{if(event.source===pv){event.error?settle.reject(event.error):settle.resolve()}};
    const cancel=()=>settle.reject(Error('PDF rendering was superseded.'));
    bus.on('textlayerrendered',onText);const task=running.get(item.number);task.cancel=cancel;
    try{
      await Promise.all([pv.draw(),textReady]);check();
      if(atVersion!==version||ratio!==pixelRatio())throw Error('PDF rendering was superseded.');
      item.paintedVersion=atVersion;item.paintedDpr=ratio;item.shell.classList.remove('pending-page');cache.set(item.number,++clock);evict(item.number);
      return item;
    }finally{bus.off('textlayerrendered',onText);cancel()}
  }
  function evict(keep=null){
    if(!readingOnly)return;
    const selection=document.getSelection?.();
    const candidates=[...cache].filter(([n])=>n!==keep&&!running.has(n)&&!visible.has(n)&&!pages[n-1].shell.contains(selection?.anchorNode)&&!pages[n-1].shell.contains(selection?.focusNode)).sort((a,b)=>a[1]-b[1]);
    while(cache.size>6&&candidates.length){const [n]=candidates.shift(),item=pages[n-1];item.pageView.reset();item.paintedVersion=-1;item.shell.classList.add('pending-page');cache.delete(n)}
  }
  function pump(){
    while(!disposed&&running.size<2&&queue.length){
      const work=queue.shift();if(work.version!==version){work.reject(Error('PDF rendering was superseded.'));continue}
      if(running.has(work.number)){queue.unshift(work);return}
      running.set(work.number,{});
      paint(pages[work.number-1],work.version).then(work.resolve,work.reject).finally(()=>{running.delete(work.number);evict();pump()});
    }
  }
  function ensurePage(number){
    try{check()}catch(error){return Promise.reject(error)}
    const item=pages[number-1];if(!item)return Promise.reject(Error('PDF page does not exist.'));
    if(item.paintedVersion===version&&item.paintedDpr===pixelRatio()){cache.set(number,++clock);return Promise.resolve(item)}
    const key=version+':'+number;if(waiting.has(key))return waiting.get(key);
    const promise=new Promise((resolve,reject)=>queue.push({number,version,resolve,reject}));waiting.set(key,promise);
    promise.finally(()=>waiting.delete(key)).catch(()=>{});pump();return promise;
  }
  function nearPages(){
    if(!scroller)return [1];
    const rect=scroller.getBoundingClientRect(),margin=scroller.clientHeight;
    return pages.filter(item=>{const p=item.shell.getBoundingClientRect();return p.bottom>=rect.top-margin&&p.top<=rect.bottom+margin}).map(item=>item.number);
  }
  function report(error){if(!disposed&&!zooming&&!/superseded|Rendering cancelled/i.test(error?.message))notify(error)}
  function mount(elements){
    if(disposed)return;scroller=elements.scroller;notify=elements.onError||(()=>{});
    observer=new IntersectionObserver(entries=>{
      for(const entry of entries){const n=Number(entry.target.dataset.page);if(entry.isIntersecting){visible.add(n);if(!zooming)ensurePage(n).catch(report)}else visible.delete(n)}evict();
    },{root:scroller,rootMargin:'100% 0px'});
    for(const item of pages)observer.observe(item.shell);
  }
  function zoomTo(next,elements,origin){
    check();if(!Number.isFinite(next)||next<=0)throw Error('Invalid PDF zoom.');
    scroller=elements.scroller;const anchor=capturePDFZoomAnchor(view,scroller,origin);
    version++;zooming=true;view.scale=next;
    for(const task of running.values())task.cancel?.();
    for(const work of queue.splice(0))work.reject(Error('PDF rendering was superseded.'));
    for(const item of pages){item.pageView.update({scale:next/CSS_UNITS,drawingDelay:PDF_ZOOM_DELAY});syncSize(item)}
    restorePDFZoomAnchor(scroller,anchor);
  }
  async function finishZoom(){
    check();const atVersion=version;zooming=false;
    // Offscreen pages retain their scaled bitmap until approached; no full-PDF repaint.
    await Promise.all(nearPages().map(ensurePage));check();return atVersion===version;
  }
  async function prepareSelectionProof(){
    check();
    if(readingOnly)throw Error('This long document supports reading and source editing only.');
    const atVersion=version,ratio=pixelRatio();
    // Zoom deliberately leaves offscreen bitmaps at their previous resolution.
    // An identity proof compares EVERY page, so first paint the entire bounded
    // short view at its current scale. No resampling or relaxed pixel comparison.
    await Promise.all(pages.map(item=>ensurePage(item.number)));check();
    if(atVersion!==version||ratio!==pixelRatio())throw Error('PDF rendering was superseded.');
  }
  function dispose(){
    if(disposed)return;disposed=true;observer?.disconnect();signal?.removeEventListener('abort',dispose);
    for(const work of queue.splice(0))work.reject(Error('PDF view was closed.'));
    for(const task of running.values())task.cancel?.();
    for(const item of pages)item.pageView.destroy();cache.clear();
    // The loaded document belongs to the app/candidate, not to a zoom view.
  }
}
