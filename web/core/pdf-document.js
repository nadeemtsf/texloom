// Reserve a worker without sending it a document. The reservation owns it
// until the first load or until the compile is discarded.
export function startPDFWorker(pdfjs) {
  let worker;
  try{worker=pdfjs.PDFWorker.create()}catch{return null}
  const ready=worker.promise.then(()=>true,()=>false);
  function dispose(){
    const pending=worker;worker=null;pending?.destroy();
  }
  return {dispose,async take(){
    if(!await ready){dispose();return undefined}
    const owned=worker;worker=null;return owned||undefined;
  }};
}

/** Own the loading task: PDF.js 6 disposes documents through that task. */
export async function loadPDFDocument(pdfjs,options,ownedWorker) {
  let loading,disposal;
  const destroy=()=>disposal??=Promise.resolve().then(async()=>{
    try{await loading?.destroy()}finally{ownedWorker?.destroy()}
  });
  try{
    loading=pdfjs.getDocument({...options,isEvalSupported:false,...(ownedWorker?{worker:ownedWorker}:{})});
    const document=await loading.promise;
    // Keep the narrow handle already consumed by the surface and transaction
    // lifetime. Do not modify or add methods to a third-party document proxy.
    return {numPages:document.numPages,getPage:number=>document.getPage(number),destroy};
  }catch(error){await destroy().catch(()=>{});throw error}
}
