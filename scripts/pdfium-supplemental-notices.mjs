// Local reviewed license text only; no downloads or upstream code execution.
import {readFile,lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {join} from 'node:path';
const repositories={brotli:'chromium/src/third_party/brotli',libyuv:'libyuv/libyuv'};
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function supplementalPdfiumNotices(directory){
 async function boundedFile(name){
  const path=join(directory,name),info=await lstat(path);
  if(!info.isFile()||info.isSymbolicLink()||info.size>65536)throw Error('Invalid supplemental notice input: '+name);
  return readFile(path);
 }
 const manifest=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(await boundedFile('manifest.json')));
 if(manifest.schema!==1||manifest.kind!=='supplemental_pdfium_component_notices'||
    typeof manifest.source_scope!=='string'||!manifest.source_scope.trim()||
    !Array.isArray(manifest.notices)||manifest.notices.length!==2||
    JSON.stringify(manifest.notices.map(n=>n.component).sort())!==JSON.stringify(Object.keys(repositories)))
  throw Error('Incomplete supplemental PDFium notice inventory');
 const texts=[],metadata=[];
 for(const item of manifest.notices){
  if(item.file!==item.component+'-LICENSE.txt'||!Number.isSafeInteger(item.bytes)||item.bytes<=0||
     !/^[a-f0-9]{64}$/.test(item.sha256)||!/^[a-f0-9]{40}$/.test(item.source_revision)||
     item.source_url!==`https://chromium.googlesource.com/${repositories[item.component]}/+/${item.source_revision}/LICENSE?format=TEXT`)
   throw Error('Invalid supplemental PDFium notice declaration');
  const bytes=await boundedFile(item.file);
  if(bytes.length!==item.bytes||hash(bytes)!==item.sha256)throw Error('Supplemental PDFium notice checksum mismatch: '+item.component);
  const text=new TextDecoder('utf-8',{fatal:true}).decode(bytes);
  texts.push(`===== Supplemental ${item.component} / ${item.source_revision} =====\nSource: ${item.source_url}\n\n${text}`);
  metadata.push({...item});
 }
 return {bytes:Buffer.from('===== Supplemental component notice scope =====\n'+manifest.source_scope+'\n\n'+texts.join('\n\n')),
  metadata,source_scope:manifest.source_scope};
}
