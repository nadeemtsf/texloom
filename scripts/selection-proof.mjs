// Run after movement-proof.mjs; all inputs and reports are synthetic fixtures.
import {spawnSync} from 'node:child_process';
import {readFile,writeFile,mkdir,cp} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {extractTextGeometry,union} from '../web/core/geometry.js';
import {validateSelectionProbe} from '../web/core/selection-proof.js';
const require=createRequire(import.meta.url);
// The pinned optional Node renderer shipped with pdfjs-dist, already in lockfile.
const {createCanvas}=require('@napi-rs/canvas');
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),base=join(root,'artifacts/proof');
const manifest=JSON.parse(await readFile(join(root,'tests/fixtures/proof/manifest.json'),'utf8'));
const movement=JSON.parse(await readFile(join(base,'report.json'),'utf8'));
const output=join(root,'artifacts/selection');await mkdir(output,{recursive:true});
const hint=join(root,'target/debug/examples/selection_hint'+(process.platform==='win32'?'.exe':''));
const report={schema:1,createdAt:new Date().toISOString(),status:'pending',cases:[],limitations:['Synthetic source/SyncTeX/probe corpus; native mouse acceptance remains open.','Rotated/cropped/UserUnit pages, arbitrary TeX and independent runtime inspection are not certified.']};
function run(exe,args,cwd=root){const r=spawnSync(exe,args,{cwd,encoding:'utf8',windowsHide:true,timeout:90_000,maxBuffer:8*1024*1024});if(r.error||r.status!==0)throw new Error(r.error?.message||(r.stderr+r.stdout).slice(-1500));return r.stdout}
async function view(path,id){
 const pdf=await pdfjs.getDocument({data:new Uint8Array(await readFile(path)),isEvalSupported:false}).promise;
 const prepared={result:{build_id:id},pageCount:pdf.numPages,pages:[],pageSizes:new Map(),objects:new Map(),unclosed:new Set()};
 try{
  for(let n=1;n<=pdf.numPages;n++){
   const page=await pdf.getPage(n),viewport=page.getViewport({scale:1}),canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height));
   await page.render({canvasContext:canvas.getContext('2d'),viewport}).promise;
   const text=await page.getTextContent({includeMarkedContent:true}),g=extractTextGeometry(text.items,text.styles,n,viewport.height);
   prepared.pages.push({number:n,runs:g.runs,lines:g.lines,shell:{querySelector:()=>canvas}});prepared.pageSizes.set(n,{width:viewport.width,height:viewport.height});
   for(const id of g.unclosed)prepared.unclosed.add(id);
   for(const [id,bounds] of g.objects){const old=prepared.objects.get(id);if(old){old.pages.add(n);old.runs.push(...bounds.runs);old.instances+=bounds.instances;Object.assign(old,union([old,bounds]));}else prepared.objects.set(id,{...bounds,pages:new Set([n])});}
  }
  return prepared;
 }finally{await pdf.destroy()}
}
function equivalent(a,b){return a.page===b.page&&a.text===b.text&&['x','y','width','height','size','baseline'].every(k=>Math.abs(a[k]-b[k])<=1e-6)}
try{
 if(movement.status!=='pass')throw new Error('The movement fixture preparation/engine matrix must pass first');
 for(const engine of movement.engines){
  for(const fixture of manifest.cases){
   const row={engine:engine.name,fixture:fixture.id,status:'pending'};report.cases.push(row);
   try{
    const sourceRoot=join(root,'tests/fixtures/proof',fixture.id),source=await readFile(join(sourceRoot,fixture.targetFile),'utf8');
    const expectedStart=Buffer.byteLength(source.slice(0,source.indexOf('% proof:begin\n')+'% proof:begin\n'.length));
    const expectedEnd=Buffer.byteLength(source.slice(0,source.indexOf('% proof:end')).replace(/\n$/u,''));
    if(fixture.expect==='refuse'){
     const line=source.slice(0,Buffer.from(source).subarray(0,expectedStart).toString().length).split('\n').length;
     let refused=false;try{run(hint,[join(sourceRoot,fixture.targetFile),join(sourceRoot,fixture.main),String(line)])}catch{refused=true}
     if(!refused)throw new Error('Unsupported linked source was accepted');row.status='expected-refusal';continue;
    }
    const compiled=join(base,'compiled',engine.name,fixture.id),baseline=await view(join(compiled,'baseline/main.pdf'),'baseline');
    let zero=join(compiled,'zero/main.pdf');
    if(fixture.expect==='cross-page'){
     const work=join(output,engine.name,fixture.id);await mkdir(work,{recursive:true});await cp(join(base,fixture.id,'zero'),work,{recursive:true});
     const args=['-interaction=nonstopmode','-halt-on-error','-file-line-error','-no-shell-escape','-synctex=1'];if(engine.miktex)args.push('--disable-installer');
     run(engine.name,[...args,'main.tex'],work);run(engine.name,[...args,'main.tex'],work);zero=join(work,'main.pdf');
    }
    const probe=await view(zero,'probe'),id='proof_'+fixture.id,target=probe.objects.get(id);
    if(!target)throw new Error('Probe target is missing');
    if(fixture.expect==='cross-page'){
     let refused=false;try{validateSelectionProbe(baseline,probe,id,[],target.page)}catch(e){refused=/one complete text object|page boundary/.test(e.message);row.reason=e.message}
     if(!refused)throw new Error('Cross-page source was not refused by the runtime validator');row.status='expected-refusal';continue;
    }
    const page=baseline.pages.find(p=>p.number===target.page);
    const targetRuns=target.runs.map(r=>{const matches=page.runs.filter(b=>equivalent(b,r));if(matches.length!==1)throw new Error('Fixture target is ambiguous');return matches[0]});
    const hits=page.lines.filter(l=>l.runs.some(r=>targetRuns.includes(r)));
    const lines=[];
    for(const hit of hits){
     const x=hit.x+Math.min(hit.width/2,15),y=hit.baseline-hit.height*.35;
     const response=run('synctex',['edit','-o',target.page+':'+x+':'+y+':'+join(compiled,'baseline/main.pdf')]);
     const input=response.match(/^Input:(.+)$/m)?.[1]?.trim(),line=response.match(/^Line:(\d+)$/m)?.[1];
     if(!input||!line||!resolve(input).endsWith(fixture.targetFile))throw new Error('SyncTeX returned a different source file');lines.push(line);
    }
    const unit=JSON.parse(run(hint,[join(sourceRoot,fixture.targetFile),join(sourceRoot,fixture.main),lines.join(',')]));
    if(unit.start!==expectedStart||unit.end!==expectedEnd)throw new Error('SyncTeX hint resolves a different source range: '+JSON.stringify(unit));
    const proof=validateSelectionProbe(baseline,probe,id,hits,target.page);
    row.status='pass';row.lines=hits.length;row.runs=proof.runs.length;row.kind=unit.kind;
   }catch(e){row.status='fail';row.reason=e.message}
  }
 }
 report.status=report.cases.some(r=>r.status==='fail')?'fail':'pass';
}catch(e){report.status='fail';report.error=e.message}
await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log('TEXLOOM_SELECTION_REPORT_BEGIN\n'+JSON.stringify(report,null,2)+'\nTEXLOOM_SELECTION_REPORT_END');process.exitCode=report.status==='pass'?0:1;
