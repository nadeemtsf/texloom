/** Own the loading task: PDF.js 6 disposes documents through that task. */
export async function loadPDFDocument(pdfjs,options) {
  const loading=pdfjs.getDocument({...options,isEvalSupported:false});
  let disposal=null;
  const destroy=()=>disposal??=Promise.resolve().then(()=>loading.destroy());
  try{
    const document=await loading.promise;
    // Keep the narrow handle already consumed by the surface and transaction
    // lifetime. Do not modify or add methods to a third-party document proxy.
    return {numPages:document.numPages,getPage:number=>document.getPage(number),destroy};
  }catch(error){await destroy().catch(()=>{});throw error}
}
