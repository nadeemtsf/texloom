import {preparePagedPDF} from './pdf-page-view.js';
/** Long views share PDF.js's bounded raster queue, including in typing mode. */
export function prepareLongPDF(pdfjs,pdf,result,options={}){
  return preparePagedPDF(pdfjs,pdf,result,{...options,readingOnly:true});
}
