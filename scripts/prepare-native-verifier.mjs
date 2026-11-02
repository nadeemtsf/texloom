// Windows build dependency preparation. End users need neither Node nor Python.
import {readFile,writeFile,mkdir,copyFile,stat,rename,unlink} from 'node:fs/promises';
import {createHash,randomUUID} from 'node:crypto';
import {inflateRawSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {resolve,join,dirname} from 'node:path';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const wheelName='pypdfium2-4.30.0-py3-none-win_amd64.whl';
const wheelSha='90dbb2ac07be53219f56be09961eb95cf2473f834d01a42d901d13ccfad64b4c';
const dllSha='b2c15cd4537d8efaf23b85b3353abe7a0b863e9c52b2f017dd87bb86a55f9c72';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const require=(ok,message)=>{if(!ok)throw new Error(message)};
async function boundedFetch(url,max,host){
  const response=await fetch(url,{signal:AbortSignal.timeout(30_000)});
  require(response.ok&&new URL(response.url).hostname===host&&new URL(response.url).protocol==='https:','Dependency download failed or redirected outside its official host');
  const chunks=[];let size=0;
  for await(const chunk of response.body){size+=chunk.length;require(size<=max,'Dependency download exceeds byte budget');chunks.push(chunk)}
  return Buffer.concat(chunks);
}
// Only exact, bounded entries of a SHA-256-pinned archive are read. This is not
// a general archive extractor: no archive path is used as an output filename.
function pinnedEntries(bytes,names){
  require(hash(bytes)===wheelSha,'Pinned PDFium wheel checksum mismatch');
  let end=-1;
  for(let p=bytes.length-22;p>=Math.max(0,bytes.length-65_557);p--){if(bytes.readUInt32LE(p)===0x06054b50&&p+22+bytes.readUInt16LE(p+20)===bytes.length){end=p;break}}
  require(end>=0&&bytes.readUInt16LE(end+4)===0&&bytes.readUInt16LE(end+6)===0,'Invalid pinned ZIP directory');
  const count=bytes.readUInt16LE(end+10),start=bytes.readUInt32LE(end+16),size=bytes.readUInt32LE(end+12);
  require(count<400&&start+size===end&&bytes.readUInt16LE(end+8)===count,'Unsupported pinned ZIP layout');
  const found=new Map();let at=start;
  for(let i=0;i<count;i++){
    require(at+46<=end&&bytes.readUInt32LE(at)===0x02014b50,'Invalid pinned ZIP entry');
    const nameLength=bytes.readUInt16LE(at+28),extra=bytes.readUInt16LE(at+30),comment=bytes.readUInt16LE(at+32);
    const next=at+46+nameLength+extra+comment;require(next<=end,'Truncated pinned ZIP entry');
    const name=bytes.toString('utf8',at+46,at+46+nameLength);
    if(names.has(name)){
      require(!found.has(name),'Duplicate pinned ZIP entry');
      const compressed=bytes.readUInt32LE(at+20),length=bytes.readUInt32LE(at+24),local=bytes.readUInt32LE(at+42),method=bytes.readUInt16LE(at+10);
      require(length<=30_000_000&&compressed<=30_000_000&&local+30<=start&&bytes.readUInt32LE(local)===0x04034b50,'Invalid pinned dependency size/header');
      const localName=bytes.readUInt16LE(local+26),localExtra=bytes.readUInt16LE(local+28),dataStart=local+30+localName+localExtra;
      require(dataStart+compressed<=start&&bytes.toString('utf8',local+30,local+30+localName)===name&&bytes.readUInt16LE(local+8)===method,'Invalid pinned dependency range');
      const data=bytes.subarray(dataStart,dataStart+compressed);
      const decoded=method===0?Buffer.from(data):method===8?inflateRawSync(data,{maxOutputLength:30_000_000}):null;
      require(decoded&&decoded.length===length,'Unsupported pinned compression');found.set(name,decoded);
    }
    at=next;
  }
  require(at===end&&found.size===names.size,'Pinned archive entries are missing');return found;
}
try{
  require(process.platform==='win32'&&process.arch==='x64','The packaged verifier currently targets Windows x64');
  const wheel=resolve(process.argv[2]||join(root,'.tools/pdfium-wheels',wheelName));
  try{await stat(wheel)}catch(error){
    if(error.code!=='ENOENT')throw error;
    const index=JSON.parse((await boundedFetch('https://pypi.org/pypi/pypdfium2/4.30.0/json',2_000_000,'pypi.org')).toString('utf8'));
    const assets=index.urls.filter(a=>a.filename===wheelName&&a.digests?.sha256===wheelSha);
    require(assets.length===1&&new URL(assets[0].url).hostname==='files.pythonhosted.org'&&new URL(assets[0].url).protocol==='https:','Pinned wheel is missing from the official package index');
    const bytes=await boundedFetch(assets[0].url,30_000_000,'files.pythonhosted.org');
    require(hash(bytes)===wheelSha,'Downloaded wheel checksum mismatch');
    await mkdir(dirname(wheel),{recursive:true});const temp=wheel+'.download-'+randomUUID();
    try{await writeFile(temp,bytes,{flag:'wx'});await rename(temp,wheel)}finally{await unlink(temp).catch(error=>{if(error.code!=='ENOENT')throw error})}
  }
  require((await stat(wheel)).size<=30_000_000,'Pinned wheel exceeds size budget');
  const entries=pinnedEntries(await readFile(wheel),new Set(['pypdfium2_raw/pdfium.dll','pypdfium2-4.30.0.dist-info/LicenseRef-PdfiumThirdParty.txt']));
  const dll=entries.get('pypdfium2_raw/pdfium.dll'),notice=entries.get('pypdfium2-4.30.0.dist-info/LicenseRef-PdfiumThirdParty.txt');
  require(hash(dll)===dllSha,'Pinned DLL checksum mismatch');
  const built=spawnSync('cargo.exe',['build','--locked','--release','-p','texloom-build','--bin','pdfium_worker','--target','x86_64-pc-windows-msvc'],{cwd:root,stdio:'inherit',windowsHide:true,timeout:180_000});
  require(!built.error&&built.status===0,'Building the native layout verifier failed');
  const program=join(root,'target/x86_64-pc-windows-msvc/release/pdfium_worker.exe');
  const resources=join(root,'src-tauri/resources/verification');await mkdir(resources,{recursive:true});
  await writeFile(join(resources,'pdfium.dll'),dll);await writeFile(join(resources,'PDFIUM-NOTICE.txt'),notice);
  await copyFile(program,join(resources,'pdfium_worker.exe'));
  const manifest={schema:2,protocol:2,purpose:'Development verification dependency; release review pending',platform:'windows-x86_64',pdfium:'126.0.6462.0',binding_source:'pypdfium2-4.30.0',
    wheel_sha256:wheelSha,dll_sha256:dllSha,worker_sha256:hash(await readFile(program)),notice_sha256:hash(notice)};
  await writeFile(join(resources,'verifier-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
  console.log('Prepared checked native layout verifier and PDFium notices (development dependency).');
}catch(error){console.error(error.message);process.exitCode=1}
