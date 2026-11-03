// The desktop package contains its dependency inventory and source notices.
// This is generated from the locked Windows graph, never an unrelated cache scan.
import {readFile,writeFile,mkdir,readdir,lstat} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const out=join(root,'src-tauri/resources/distribution');
const run=spawnSync('cargo',['metadata','--locked','--format-version','1','--filter-platform','x86_64-pc-windows-msvc'],
  {cwd:root,encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024});
if(run.error||run.status!==0)throw Error(run.error?.message||run.stderr||'Dependency inventory failed');
const metadata=JSON.parse(run.stdout),active=new Set(metadata.resolve.nodes.map(n=>n.id));
const dependencies=[],texts=[];
async function notices(folder,label){
  const pending=[{folder,depth:0}];let found=0;
  while(pending.length){const current=pending.pop();
    for(const entry of await readdir(current.folder,{withFileTypes:true})){
      const path=join(current.folder,entry.name);
      if(entry.isSymbolicLink())throw Error('Linked license input: '+label);
      if(entry.isDirectory()&&current.depth===0&&/^(licenses?|notices?)$/iu.test(entry.name))pending.push({folder:path,depth:1});
      else if(entry.isFile()&&(/^(license|licence|notice|copying|thirdpartynotice)/iu.test(entry.name)||current.depth===1)){
        if((await lstat(path)).size>4_000_000)throw Error('License input exceeds budget: '+label);
        texts.push('===== '+label+' / '+entry.name+' =====\n'+await readFile(path,'utf8'));found++;
      }
    }
  }
  if(!found)texts.push('===== '+label+' =====\nNo separate license text in the locked package; see its declared license and source in DEPENDENCIES.json.');
}
for(const pkg of metadata.packages.filter(p=>active.has(p.id)&&p.source).sort((a,b)=>(a.name+a.version).localeCompare(b.name+b.version))){
  if(!pkg.license&&!pkg.license_file)throw Error('Missing dependency license: '+pkg.name);
  const source=`https://crates.io/api/v1/crates/${encodeURIComponent(pkg.name)}/${encodeURIComponent(pkg.version)}/download`;
  dependencies.push({ecosystem:'cargo',name:pkg.name,version:pkg.version,license:pkg.license,source});
  await notices(dirname(pkg.manifest_path),pkg.name+' '+pkg.version);
}
const lock=JSON.parse(await readFile(join(root,'package-lock.json'),'utf8'));
for(const [path,pkg] of Object.entries(lock.packages).filter(([path,p])=>path&&!p.dev).sort(([a],[b])=>a.localeCompare(b))){
  let manifest;
  try{manifest=JSON.parse(await readFile(join(root,path,'package.json'),'utf8'))}
  catch(error){if(error.code==='ENOENT'&&pkg.optional)continue;throw error}
  const license=manifest.license||pkg.license;
  if(!license)throw Error('Missing dependency license: '+path);
  dependencies.push({ecosystem:'npm',name:manifest.name,version:pkg.version,license,source:pkg.resolved||manifest.repository});
  await notices(join(root,path),manifest.name+' '+pkg.version);
}
await mkdir(out,{recursive:true});
await writeFile(join(out,'DEPENDENCIES.json'),JSON.stringify({schema:1,target:'x86_64-pc-windows-msvc',
  scope:'Locked Windows Cargo graph (including build tools), and installed production JavaScript packages. PDFium notices are supplied separately in verification/PDFIUM-NOTICE.txt.',dependencies},null,2)+'\n');
const mpl=dependencies.filter(p=>String(p.license).includes('MPL-2.0'));
await writeFile(join(out,'SOURCE-AVAILABILITY.txt'),[
  'Texloom third-party source availability',
  'The dependencies listed below are supplied under the Mozilla Public License 2.0, possibly with alternative licenses. Their source files are unchanged by Texloom. Corresponding source for each exact version is available at the listed public download URL. Full license texts and other notices accompany this package. The complete locked dependency inventory is in DEPENDENCIES.json.',
  ...mpl.map(p=>`${p.name} ${p.version}\nLicense: ${p.license}\nSource: ${p.source}`),
  'Texloom source: https://github.com/nadeemtsf/texloom (access may require repository-owner permission).',
].join('\n\n')+'\n');
await writeFile(join(out,'THIRD-PARTY-NOTICES.txt'),texts.join('\n\n')+'\n');
console.log(JSON.stringify({dependencies:dependencies.length,mpl:mpl.length,output:out}));
