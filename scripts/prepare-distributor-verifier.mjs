// Build-time preparation only. Runtime verification uses no Node or Python.
import {readFile,writeFile,mkdir,copyFile,stat,rename,unlink} from 'node:fs/promises';
import {createHash,randomUUID} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {resolve,join,dirname} from 'node:path';
import {readPinnedPdfiumTar} from './pinned-pdfium-tar.mjs';
import {supplementalPdfiumNotices} from './pdfium-supplemental-notices.mjs';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const require=(ok,message)=>{if(!ok)throw Error(message)};
try{
  require(process.platform==='win32'&&process.arch==='x64','The packaged verifier targets Windows x64');
  const pin=JSON.parse(await readFile(join(root,'scripts/pdfium-pin.json'),'utf8'));
  require(pin.kind==='pdfium_pinned_distributor'&&pin.adopted===true&&pin.pdfium==='159.0.8138.0'&&
    pin.dll_sha256==='71a3e63e6ab8f5746a0fc9de9bc3d0c0dc07eb60990dfd37d9db7fc891821943'&&
    pin.archive_sha256==='8d48973e3eb472dda0f5f2435e595ac043f9f2219dd71883392f549445afa8a5',
    'Active distributor pin identity differs from the reviewed build');
  const archive=resolve(process.argv[2]||join(root,'.tools/pdfium-archives/pdfium-8138-win-x64.tgz'));
  try{await stat(archive)}catch(error){
    if(error.code!=='ENOENT')throw error;
    require(pin.archive_url==='https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F8138/pdfium-win-x64.tgz','Unexpected dependency origin');
    const response=await fetch(pin.archive_url,{signal:AbortSignal.timeout(30_000)});
    const url=new URL(response.url);
    require(response.ok&&url.protocol==='https:'&&['github.com','release-assets.githubusercontent.com'].includes(url.hostname),'Dependency download failed or redirected outside its distributor hosts');
    const chunks=[];let length=0;
    for await(const chunk of response.body){length+=chunk.length;require(length<=pin.archive_bytes,'Dependency download exceeds its exact byte budget');chunks.push(chunk)}
    const bytes=Buffer.concat(chunks);readPinnedPdfiumTar(bytes,pin);
    await mkdir(dirname(archive),{recursive:true});const temporary=archive+'.download-'+randomUUID();
    try{await writeFile(temporary,bytes,{flag:'wx'});await rename(temporary,archive)}
    finally{await unlink(temporary).catch(error=>{if(error.code!=='ENOENT')throw error})}
  }
  require((await stat(archive)).size===pin.archive_bytes,'Pinned archive size mismatch');
  const entries=readPinnedPdfiumTar(await readFile(archive),pin),dll=entries.get(pin.dll_path);
  require(dll&&hash(dll)===pin.dll_sha256,'Pinned DLL checksum mismatch');
  const expectedNotices=['LICENSE',...pin.files.map(f=>f.path).filter(name=>name.startsWith('licenses/'))].sort();
  require(JSON.stringify(pin.notices)===JSON.stringify(expectedNotices),'Incomplete distributor notice inventory');
  const supplemental=await supplementalPdfiumNotices(join(root,'resources/verification-notices'));
  const distributor=Buffer.from(pin.notices.map(name=>'===== '+name+' =====\n'+
    new TextDecoder(pin.notice_encodings[name]||'utf-8',{fatal:true}).decode(entries.get(name))).join('\n\n'));
  const notice=Buffer.concat([distributor,Buffer.from('\n\n'),supplemental.bytes]);
  const built=spawnSync('cargo.exe',['build','--locked','--release','-p','texloom-build','--bin','pdfium_worker','--target','x86_64-pc-windows-msvc'],
    {cwd:root,stdio:'inherit',windowsHide:true,timeout:180_000});
  require(!built.error&&built.status===0,'Building the native verifier failed');
  const program=join(root,'target/x86_64-pc-windows-msvc/release/pdfium_worker.exe');
  const resources=join(root,'src-tauri/resources/verification');await mkdir(resources,{recursive:true});
  await writeFile(join(resources,'pdfium.dll'),dll);await writeFile(join(resources,'PDFIUM-NOTICE.txt'),notice);
  await copyFile(program,join(resources,'pdfium_worker.exe'));
  const manifest={schema:2,protocol:3,purpose:'Pinned isolated layout verification dependency; development build',platform:'windows-x86_64',
    pdfium:pin.pdfium,binding_source:pin.distributor+'@'+pin.tag,archive_sha256:pin.archive_sha256,
    build_repository_commit:pin.build_repository_commit,dll_sha256:pin.dll_sha256,
    worker_sha256:hash(await readFile(program)),notice_sha256:hash(notice),
    supplemental_notices:supplemental.metadata,supplemental_notice_scope:supplemental.source_scope};
  await writeFile(join(resources,'verifier-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
  console.log('Prepared checked PDFium '+pin.pdfium+' verifier, '+pin.notices.length+' distributor and '+supplemental.metadata.length+' supplemental notices.');
}catch(error){console.error(error.message);process.exitCode=1}
