import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {readFile} from 'node:fs/promises';
import {loadPDFDocument} from '../../web/core/pdf-document.js';

export async function inspectPdfjs(path) {
  const doc = await loadPDFDocument(pdfjs,{data:new Uint8Array(await readFile(path))});
  const result = {renderer:'pdfjs', version:pdfjs.version, pages:[], records:[], graphics:[]};
  try {
    for (let p=1; p<=doc.numPages; p++) {
      const page=await doc.getPage(p), viewport=page.getViewport({scale:1});
      if (viewport.rotation !== 0 || page.view[0] !== 0 || page.view[1] !== 0 || page.userUnit !== 1)
        throw new Error('Proof PDF.js inspector needs the crop/rotation/UserUnit adapter');
      result.pages.push({width:viewport.width,height:viewport.height});
      const text=await page.getTextContent({includeMarkedContent:true, disableNormalization:true});
      const stack=[];
      for (const item of text.items) {
        if (item.type === 'beginMarkedContent' || item.type === 'beginMarkedContentProps') {stack.push(item.tag);continue;}
        if (item.type === 'endMarkedContent') {if(!stack.length)throw new Error('Unbalanced PDF marker');stack.pop();continue;}
        if (!item.str?.trim()) continue;
        const [a,b,c,d,e,f]=item.transform;
        if (Math.abs(b)>1e-5 || Math.abs(c)>1e-5 || a<0 || d<0) throw new Error('Unsupported text transform in proof');
        const height=Math.hypot(c,d),ascent=text.styles[item.fontName]?.ascent ?? 0.8;
        result.records.push({text:item.str,page:p,x:e,y:viewport.height-f-height*ascent,
          width:item.width,height,markers:stack.filter(x=>x?.startsWith('Texloom_'))});
      }
      if(stack.length)throw new Error('Unclosed PDF marker');
    }
    return result;
  } finally {await doc.destroy();}
}
