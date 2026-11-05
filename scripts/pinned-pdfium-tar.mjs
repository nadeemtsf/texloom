// Source-only dependency preparation draft. Not imported by the active builder.
// Read an exactly pinned distributor archive in memory; never extract paths,
// invoke commands, write resources, load a DLL or declare compatibility.
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';

const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const require=(condition,message)=>{if(!condition)throw Error(message)};
const digest=value=>typeof value==='string'&&/^[0-9a-f]{64}$/.test(value);
const integer=value=>Number.isSafeInteger(value)&&value>=0;
function portable(value,{directory=false}={}){
  require(typeof value==='string'&&value.length>0&&value.length<=240&&/^[\x20-\x7e]+$/.test(value),'Invalid pinned archive path');
  if(directory&&value.endsWith('/'))value=value.slice(0,-1);
  require(!/[\\:<>"|?*]/.test(value)&&value.split('/').every(part=>part!==''&&part!=='.'&&part!=='..'&&
    !/[. ]$/.test(part)&&!/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)),'Nonportable pinned archive path');
  return value;
}
function textField(header,start,length){
  const field=header.subarray(start,start+length),end=field.indexOf(0),used=end<0?field:field.subarray(0,end);
  require(end<0||field.subarray(end).every(byte=>byte===0),'Invalid tar text padding');
  require(used.every(byte=>byte>=32&&byte<=126),'Non-ASCII tar field');
  return used.toString('ascii');
}
function octal(header,start,length){
  const field=header.subarray(start,start+length);
  require(field.every(byte=>byte<128),'Unsupported tar numeric encoding');
  const value=field.toString('ascii');
  require(/^[ 0-7]*[\x00 ]*$/.test(value),'Unsupported tar numeric encoding');
  const number=parseInt(value.replace(/[\x00 ]+$/g,'').trim()||'0',8);
  require(integer(number),'Invalid tar numeric value');return number;
}

/** Review-only manifest and complete file list; no caller-selected output names. */
export function readPinnedPdfiumTar(bytes,manifest){
  require(Buffer.isBuffer(bytes)&&bytes.length<=32*1024*1024,'Compressed dependency byte budget exceeded');
  require(manifest?.schema===1&&manifest.kind==='pdfium_distributor_review_candidate'&&manifest.adopted===false&&
    digest(manifest.archive_sha256)&&integer(manifest.archive_bytes)&&bytes.length===manifest.archive_bytes&&
    sha(bytes)===manifest.archive_sha256,'Pinned distributor archive identity mismatch');
  require(Array.isArray(manifest.files)&&manifest.files.length>0&&manifest.files.length<=256,'Invalid pinned entry inventory');
  const expected=new Map(),names=new Set();let declared=0;
  for(const entry of manifest.files){
    require(entry&&typeof entry==='object'&&Object.keys(entry).sort().join(',')==='bytes,path,sha256','Invalid pinned file declaration');
    const name=portable(entry.path),key=name.toLowerCase();
    require(!names.has(key)&&integer(entry.bytes)&&entry.bytes<=32*1024*1024&&digest(entry.sha256),'Invalid or duplicate pinned file');
    names.add(key);expected.set(name,entry);declared+=entry.bytes;
  }
  require(declared<=100*1024*1024,'Declared dependency byte budget exceeded');
  // The compressed digest is checked before decompression and parsing.
  const tar=gunzipSync(bytes,{maxOutputLength:128*1024*1024}),found=new Map(),seen=new Set();
  require(tar.length%512===0,'Truncated tar block layout');
  let at=0,headers=0,ended=false;
  while(at+512<=tar.length){
    const header=tar.subarray(at,at+512);
    if(header.every(byte=>byte===0)){
      require(at+1024<=tar.length&&tar.subarray(at).every(byte=>byte===0),'Invalid tar end blocks');
      ended=true;break;
    }
    require(++headers<=512,'Tar header count budget exceeded');
    const gnu=header.subarray(257,265).equals(Buffer.from('ustar  \0','ascii'));
    const posix=header.subarray(257,265).equals(Buffer.from([117,115,116,97,114,0,48,48]));
    require(gnu||posix,'Unsupported tar format');
    const checksum=octal(header,148,8);
    let sum=0;for(let i=0;i<512;i++)sum+=i>=148&&i<156?32:header[i];
    require(sum===checksum,'Invalid tar header checksum');
    const type=header[156],directory=type===53;
    require(type===48||type===0||directory,'Links, special files and tar extensions are not accepted');
    let name=textField(header,0,100);
    if(posix){const prefix=textField(header,345,155);if(prefix)name=prefix+'/'+name}
    name=portable(name,{directory});const key=name.toLowerCase();
    require(!seen.has(key),'Duplicate or case-colliding tar member');seen.add(key);
    const size=octal(header,124,12),dataStart=at+512,next=dataStart+Math.ceil(size/512)*512;
    require(size<=32*1024*1024&&Number.isSafeInteger(next)&&next<=tar.length,'Invalid tar data range');
    if(directory)require(size===0,'Directory contains data');
    else{
      const entry=expected.get(name);
      require(entry&&entry.bytes===size,'Unexpected pinned archive member or size');
      const value=tar.subarray(dataStart,dataStart+size);
      require(sha(value)===entry.sha256,'Pinned archive member digest mismatch');
      found.set(name,Buffer.from(value));
    }
    at=next;
  }
  require(ended&&found.size===expected.size,'Missing pinned archive file or end marker');
  return found;
}
