import {spawnSync} from 'node:child_process';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),output=join(root,'artifacts/protected-text');await mkdir(output,{recursive:true});
function run(file,args,cwd=root){const r=spawnSync(file,args,{cwd,encoding:'utf8',windowsHide:true,timeout:90_000,maxBuffer:8*1024*1024});if(r.error||r.status!==0)throw new Error(r.error?.message||(r.stderr+r.stdout).slice(-1500));return r.stdout}
const file=join(root,'tests/fixtures/proof/formatting/main.tex'),original=await readFile(file,'utf8');
const candidate=run(join(root,'target/debug/examples/literal_candidate'+(process.platform==='win32'?'.exe':'')),[file,file,'1','strong & stable']);
if(candidate!==original.replace('\\textbf{bold}','\\textbf{strong \\& stable}'))throw new Error('The literal editor changed formatting or unrelated source');
const report={schema:1,status:'pending',cases:[],limitations:['Three synthetic direct-engine compile cases; native edit/reflow acceptance remains open.']};
for(const engine of ['pdflatex','lualatex','xelatex']){
 const row={engine,status:'pending'};report.cases.push(row);
 try{
  const dir=join(output,engine);await mkdir(dir,{recursive:true});await writeFile(join(dir,'main.tex'),candidate);
  const version=run(engine,['--version']);const args=['-interaction=nonstopmode','-halt-on-error','-file-line-error','-no-shell-escape'];if(version.includes('MiKTeX'))args.push('--disable-installer');
  run(engine,[...args,'main.tex'],dir);run(engine,[...args,'main.tex'],dir);
  const pdf=await pdfjs.getDocument({data:new Uint8Array(await readFile(join(dir,'main.pdf'))),isEvalSupported:false}).promise;
  let text='';try{for(let n=1;n<=pdf.numPages;n++)text+=(await (await pdf.getPage(n)).getTextContent()).items.map(i=>i.str||'').join('');}finally{await pdf.destroy()}
  if(!text.replace(/\s/gu,'').includes('strong&stable'))throw new Error('The compiled PDF did not contain the protected edit');row.status='pass';row.source_scaffold_preserved=true;
 }catch(e){row.status='fail';row.reason=e.message}
}
report.status=report.cases.every(r=>r.status==='pass')?'pass':'fail';await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
console.log('TEXLOOM_TEXT_REPORT_BEGIN\n'+JSON.stringify(report,null,2)+'\nTEXLOOM_TEXT_REPORT_END');process.exitCode=report.status==='pass'?0:1;
