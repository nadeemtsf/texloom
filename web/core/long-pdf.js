import {preparePagedPDF} from './pdf-page-view.js';
/** Reading-only preparation shares PDF.js's page renderer and bounded queue. */
export function prepareLongPDF(pdfjs,pdf,result,options={}){
  return preparePagedPDF(pdfjs,pdf,result,{...options,readingOnly:true});
}
