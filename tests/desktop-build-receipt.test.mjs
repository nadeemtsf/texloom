import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,access,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,sep} from 'node:path';
import {createHash} from 'node:crypto';
import {recordedDesktopBuild} from '../scripts/desktop-build-receipt.mjs';
const hash=raw=>createHash('sha256').update(raw).digest('hex');
async function fixture(fn){
  const root=await mkdtemp(join(tmpdir(),'texloom-build-binding-')),dir=join(root,'src-tauri/resources/verification'),executable=join(root,'Texloom.exe');
  try{
    await mkdir(dir,{recursive:true});
    const values={'pdfium_worker.exe':'synthetic worker','pdfium.dll':'synthetic library','PDFIUM-NOTICE.txt':'synthetic notice'};
    for(const [name,value]of Object.entries(values))await writeFile(join(dir,name),value);
    await writeFile(join(dir,'verifier-manifest.json'),JSON.stringify({schema:2,protocol:3,worker_sha256:hash(values['pdfium_worker.exe']),dll_sha256:hash(values['pdfium.dll']),notice_sha256:hash(values['PDFIUM-NOTICE.txt'])}));
    await fn({root,dir,executable});
  }finally{assert.ok(resolve(root).startsWith(resolve(tmpdir())+sep));await rm(root,{recursive:true,force:true});}
}
test('successful build binds exact executable and all verifier inputs',()=>fixture(async({root,dir,executable})=>{
  const result=await recordedDesktopBuild({root,executable,build:()=>writeFile(executable,'new synthetic app')});
  assert.equal(result.desktop.sha256,hash('new synthetic app'));assert.equal(Object.keys(result.resources).length,4);
  assert.equal(result.resources['pdfium_worker.exe'].sha256,hash('synthetic worker'));
  assert.deepEqual(JSON.parse(await readFile(executable+'.texloom-build.json','utf8')),result);
}));
test('resource mutation during build refuses a completed pairing receipt',()=>fixture(async({root,dir,executable})=>{
  await assert.rejects(recordedDesktopBuild({root,executable,build:async()=>{await writeFile(executable,'app');await writeFile(join(dir,'PDFIUM-NOTICE.txt'),'changed');}}),/disagree|changed/);
  await assert.rejects(access(executable+'.texloom-build.json'));
}));
test('failed build removes the old successful receipt from the canonical path',()=>fixture(async({root,executable})=>{
  await writeFile(executable+'.texloom-build.json','earlier receipt');
  await assert.rejects(recordedDesktopBuild({root,executable,build:async()=>{throw Error('controlled build failure')}}),/controlled build failure/);
  await assert.rejects(access(executable+'.texloom-build.json'));
}));
test('inconsistent inputs fail before the build callback can run',()=>fixture(async({root,dir,executable})=>{
  await writeFile(join(dir,'pdfium.dll'),'changed');let called=false;
  await assert.rejects(recordedDesktopBuild({root,executable,build:async()=>{called=true}}),/disagree/);assert.equal(called,false);
}));
