// Local accidental-mismatch protection, not a signed build attestation.
import {readFile,writeFile,stat,lstat,rename} from 'node:fs/promises';
import {createHash,randomUUID} from 'node:crypto';
import {join} from 'node:path';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const names=['PDFIUM-NOTICE.txt','pdfium.dll','pdfium_worker.exe','verifier-manifest.json'];
const require=(ok,message)=>{if(!ok)throw Error(message)};
export async function fileReceipt(path){
  const before=await lstat(path);require(before.isFile()&&!before.isSymbolicLink(),'Build input must be an ordinary file');
  const raw=await readFile(path),after=await stat(path);
  require(before.size===raw.length&&before.size===after.size&&before.mtimeMs===after.mtimeMs&&before.ino===after.ino,'Build input changed while reading');
  return {bytes:raw.length,sha256:sha(raw)};
}
export async function verifierSnapshot(root){
  const folder=join(root,'src-tauri/resources/verification'),resources={};
  for(const name of names)resources[name]=await fileReceipt(join(folder,name));
  const manifest=JSON.parse(await readFile(join(folder,'verifier-manifest.json'),'utf8'));
  require(manifest.schema===2&&manifest.protocol===3,'Unsupported verifier manifest');
  for(const [name,key]of [['pdfium.dll','dll_sha256'],['pdfium_worker.exe','worker_sha256'],['PDFIUM-NOTICE.txt','notice_sha256']])
    require(resources[name].sha256===manifest[key],'Verifier inputs disagree with their manifest');
  return resources;
}
export async function recordedDesktopBuild({root,executable,build,traceEnabled=false,sourceHead=null}){
  const receiptPath=executable+'.texloom-build.json';
  // A failed new build must not leave a prior successful receipt at this path.
  try{await rename(receiptPath,receiptPath+'.previous-'+randomUUID())}catch(error){if(error.code!=='ENOENT')throw error}
  const before=await verifierSnapshot(root);
  await build();
  const after=await verifierSnapshot(root);
  require(JSON.stringify(before)===JSON.stringify(after),'Verifier resources changed during the desktop build');
  const desktop=await fileReceipt(executable);
  const receipt={schema:1,kind:'desktop_build_pairing_receipt',status:'complete',recorded_utc:new Date().toISOString(),
    scope:'Local successful build invocation and unchanged verifier resources; not signed provenance or runtime acceptance',
    source_head:sourceHead,trace_enabled:traceEnabled,desktop,resources:after};
  const temporary=receiptPath+'.pending-'+randomUUID();
  await writeFile(temporary,JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});await rename(temporary,receiptPath);
  return receipt;
}
