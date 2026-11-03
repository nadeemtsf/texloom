/** Bounded painting for long documents. Visual transactions retain their
 * separate complete-document verification budget. */
export async function prepareLongPDF(pdfjs,pdf,result,{width,zoom=0,mode='select',signal}={}){
  if(pdf.numPages>5000)throw Error('This PDF exceeds the 5,000-page reading limit. Source editing is available.');
  const first=await pdf.getPage(1),scale=zoom||Math.max(0.2,Math.min(1.5,(width-48)/first.getViewport({scale:1}).width));
  const fragment=document.createDocumentFragment(),pages=[],pageSizes=new Map(),cache=new Map(),running=new Map(),visible=new Set(),queue=[];
  let disposed=false,observer=null,clock=0,notify=()=>{};
  const view={pdf,result,fragment,pages,objects:new Map(),unclosed:new Set(),pageSizes,pageCount:pdf.numPages,scale,mode,
    lazy:true,ensurePage:renderPage,mount,dispose};
  for(let n=1;n<=pdf.numPages;n++){
    if(signal?.aborted)throw Error('PDF rendering was superseded.');
    const page=n===1?first:await pdf.getPage(n),base=page.getViewport({scale:1}),viewport=page.getViewport({scale});
    const shell=document.createElement('div');shell.className='pdf-page pending-page';
    Object.assign(shell.dataset,{page:String(n),scale:String(scale),pdfWidth:String(base.width),pdfHeight:String(base.height)});
    Object.assign(shell.style,{width:viewport.width+'px',height:viewport.height+'px'});
    shell.style.setProperty('--total-scale-factor',String(scale));shell.style.setProperty('--scale-factor',String(scale));
    shell.setAttribute('aria-label','PDF page '+n);pageSizes.set(n,{width:base.width,height:base.height});
    pages.push({number:n,shell,lines:[],runs:[],selectionReason:'This long document supports reading and source editing. Layout verification is limited to 20 pages.'});fragment.append(shell);
  }
  async function paint(number){
    const item=pages[number-1];if(!item||disposed||signal?.aborted)throw Error('PDF view was closed.');
    const page=await pdf.getPage(number);if(disposed||signal?.aborted)throw Error('PDF view was closed.');
    const viewport=page.getViewport({scale}),ratio=Math.min(2.5,devicePixelRatio||1),canvas=document.createElement('canvas');
    const pixelWidth=Math.ceil(viewport.width*ratio),pixelHeight=Math.ceil(viewport.height*ratio);
    if(pixelWidth*pixelHeight>16_000_000)throw Error('Page exceeds the reading raster limit. Reduce the zoom.');
    canvas.width=pixelWidth;canvas.height=pixelHeight;
    Object.assign(canvas.style,{width:viewport.width+'px',height:viewport.height+'px'});
    const rendering=page.render({canvas,canvasContext:canvas.getContext('2d'),viewport,transform:[ratio,0,0,ratio,0,0]});
    running.set(number,{rendering,canvas});
    await rendering.promise;const text=await page.getTextContent();
    if(disposed||signal?.aborted)throw Error('PDF view was closed.');
    const layer=document.createElement('div');layer.className='textLayer';
    const textLayer=new pdfjs.TextLayer({textContentSource:text,container:layer,viewport});running.get(number).textLayer=textLayer;await textLayer.render();
    if(disposed||signal?.aborted){textLayer.cancel();canvas.width=canvas.height=0;throw Error('PDF view was closed.');}
    item.shell.replaceChildren(canvas,layer);item.shell.classList.remove('pending-page');cache.set(number,{canvas,layer,at:++clock});evict(number);
    return item;
  }
  function evict(keep=null){
    const selection=document.getSelection?.();
    const candidates=[...cache].filter(([n,e])=>n!==keep&&!visible.has(n)&&!e.layer.contains(selection?.anchorNode)&&!e.layer.contains(selection?.focusNode)).sort((a,b)=>a[1].at-b[1].at);
    while(cache.size>6&&candidates.length){const [n,e]=candidates.shift();e.canvas.width=e.canvas.height=0;pages[n-1].shell.replaceChildren();pages[n-1].shell.classList.add('pending-page');cache.delete(n);}
  }
  function pump(){while(!disposed&&running.size<2&&queue.length){const work=queue.shift();running.set(work.number,{});paint(work.number).then(work.resolve,work.reject).finally(()=>{if(!cache.has(work.number)){const canvas=running.get(work.number)?.canvas;if(canvas)canvas.width=canvas.height=0;}running.delete(work.number);pump()})}}
  const waiting=new Map();
  function renderPage(number){
    if(cache.has(number)){cache.get(number).at=++clock;return Promise.resolve(pages[number-1]);}
    if(disposed||signal?.aborted)return Promise.reject(Error('PDF view was closed.'));
    if(waiting.has(number))return waiting.get(number);
    const promise=new Promise((resolve,reject)=>queue.push({number,resolve,reject}));waiting.set(number,promise);promise.finally(()=>waiting.delete(number)).catch(()=>{});pump();return promise;
  }
  function mount({scroller,onError=()=>{}}){
    if(disposed)return;notify=onError;
    observer=new IntersectionObserver(entries=>{for(const entry of entries){const n=Number(entry.target.dataset.page);if(entry.isIntersecting){visible.add(n);renderPage(n).catch(e=>{if(!disposed)notify(e)})}else visible.delete(n)}evict();},{root:scroller,rootMargin:'100% 0px'});
    for(const page of pages)observer.observe(page.shell);
  }
  function dispose(){if(disposed)return;disposed=true;observer?.disconnect();for(const work of queue.splice(0))work.reject(Error('PDF view was closed.'));for(const task of running.values()){task.rendering?.cancel();task.textLayer?.cancel();if(task.canvas)task.canvas.width=task.canvas.height=0;}for(const e of cache.values())e.canvas.width=e.canvas.height=0;cache.clear();}
  signal?.addEventListener('abort',dispose,{once:true});
  try{await renderPage(1);return view}catch(e){dispose();throw e}
}
