import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {readPinnedPdfiumTar} from '../scripts/pinned-pdfium-tar.mjs';
const sha=raw=>createHash('sha256').update(raw).digest('hex');

test('an active distributor declaration uses the same bounded reader without implying runtime compatibility',()=>{
  const {bytes,manifest}=fixture();manifest.kind='pdfium_pinned_distributor';manifest.adopted=true;
  assert.equal(readPinnedPdfiumTar(bytes,manifest).get('bin/pdfium.dll').toString(),'Synthetic bytes, not executable');
  manifest.adopted=false;assert.throws(()=>readPinnedPdfiumTar(bytes,manifest),/identity mismatch/);
});
function fixture({name='bin/pdfium.dll',type='0',badChecksum=false}={}){
  const data=Buffer.from('Synthetic bytes, not executable'),header=Buffer.alloc(512);
  header.write(name,0,100,'ascii');header.write('0000644\0',100,'ascii');
  header.write(data.length.toString(8).padStart(11,'0')+'\0',124,'ascii');
  header.fill(32,148,156);header.write(type,156,'ascii');Buffer.from([117,115,116,97,114,0,48,48]).copy(header,257);
  const checksum=header.reduce((sum,byte)=>sum+byte,0);
  header.write(checksum.toString(8).padStart(6,'0')+'\0 ',148,'ascii');
  if(badChecksum)header[10]^=1;
  const bytes=gzipSync(Buffer.concat([header,data,Buffer.alloc(512-data.length),Buffer.alloc(1024)]));
  return {bytes,manifest:{schema:1,kind:'pdfium_distributor_review_candidate',adopted:false,archive_sha256:sha(bytes),archive_bytes:bytes.length,
    files:[{path:'bin/pdfium.dll',bytes:data.length,sha256:sha(data)}]}};
}
test('pinned archive returns only declared immutable bytes and performs no installation',()=>{
  const {bytes,manifest}=fixture(),files=readPinnedPdfiumTar(bytes,manifest);
  assert.equal(files.size,1);assert.equal(sha(files.get('bin/pdfium.dll')),manifest.files[0].sha256);
});
test('archive-byte tampering is rejected before parsing',()=>{
  const {bytes,manifest}=fixture();bytes[bytes.length-1]^=1;assert.throws(()=>readPinnedPdfiumTar(bytes,manifest),/identity mismatch/);
});
test('a valid compressed digest cannot hide unsafe tar headers, links or paths',()=>{
  for(const options of [{badChecksum:true},{type:'2'},{name:'../actual-document.tex'}]){
    const {bytes,manifest}=fixture(options);assert.throws(()=>readPinnedPdfiumTar(bytes,manifest));
  }
});
test('missing or differently hashed declarations never produce a partial successful extraction',()=>{
  const {bytes,manifest}=fixture();manifest.files[0].sha256='0'.repeat(64);
  assert.throws(()=>readPinnedPdfiumTar(bytes,manifest),/digest mismatch/);
  manifest.files.push({...manifest.files[0],path:'extra.txt'});
  assert.throws(()=>readPinnedPdfiumTar(bytes,manifest));
});
