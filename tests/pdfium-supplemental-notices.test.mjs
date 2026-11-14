import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,cp,readFile,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {supplementalPdfiumNotices} from '../scripts/pdfium-supplemental-notices.mjs';
const source=new URL('../resources/verification-notices/',import.meta.url);
async function fixture(fn){
 const root=await mkdtemp(join(tmpdir(),'texloom-notices-'));
 try{await cp(source,root,{recursive:true});await fn(root)}finally{
  assert.ok(resolve(root).startsWith(resolve(tmpdir())+sep+'texloom-notices-'));
  await rm(root,{recursive:true,force:true});
 }
}
test('reviewed supplemental notices retain exact upstream text and provenance',async()=>{
 const result=await supplementalPdfiumNotices(fileURLToPath(source));
 assert.deepEqual(result.metadata.map(n=>n.component),['brotli','libyuv']);
 for(const row of result.metadata){
  const raw=await readFile(new URL(row.file,source));assert.ok(result.bytes.includes(raw));
 }
 assert.match(result.bytes.toString(),/exact DLL nested checkout remains unresolved/);
});
test('changed notice bytes refuse packaging before any worker build',async()=>fixture(async root=>{
 await writeFile(join(root,'brotli-LICENSE.txt'),'Changed license text');
 await assert.rejects(supplementalPdfiumNotices(root),/checksum mismatch/);
}));
test('a missing component or duplicate replacement cannot silently reduce notices',async()=>fixture(async root=>{
 const path=join(root,'manifest.json'),original=JSON.parse(await readFile(path,'utf8'));
 for(const rows of [[original.notices[0]],[original.notices[0],original.notices[0]]]){
  await writeFile(path,JSON.stringify({...original,notices:rows}));
  await assert.rejects(supplementalPdfiumNotices(root),/Incomplete/);
 }
}));
test('path escape and mismatched source identity are rejected',async()=>fixture(async root=>{
 const path=join(root,'manifest.json'),original=JSON.parse(await readFile(path,'utf8'));
 for(const patch of [{file:'../outside.txt'},{source_url:'https://example.invalid/LICENSE'},{source_revision:'not-a-commit'}]){
  const changed=structuredClone(original);Object.assign(changed.notices[0],patch);
  await writeFile(path,JSON.stringify(changed));await assert.rejects(supplementalPdfiumNotices(root),/Invalid/);
 }
}));
