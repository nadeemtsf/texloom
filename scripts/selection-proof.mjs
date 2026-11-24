// Run after movement-proof.mjs; all inputs and reports are synthetic fixtures.
import {spawnSync} from 'node:child_process';
import {readFile,writeFile,mkdir,cp} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {loadPDFDocument} from '../web/core/pdf-document.js';
import {extractTextGeometry,union} from '../web/core/geometry.js';
import {validateSelectionProbe} from '../web/core/selection-proof.js';
const require=createRequire(import.meta.url);
// The pinned optional Node renderer shipped with pdfjs-dist, already in lockfile.
const {createCanvas}=require('@napi-rs/canvas');
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),base=process.env.TEXLOOM_PROOF_REPORT_DIR?resolve(process.env.TEXLOOM_PROOF_REPORT_DIR):join(root,'artifacts/proof');
const manifest=JSON.parse(await readFile(join(root,'tests/fixtures/proof/manifest.json'),'utf8'));
const movement=JSON.parse(await readFile(join(base,'report.json'),'utf8'));
const output=process.env.TEXLOOM_SELECTION_REPORT_DIR ? resolve(process.env.TEXLOOM_SELECTION_REPORT_DIR) : join(root,'artifacts/selection');await mkdir(output,{recursive:true});
const examples=resolve(root,process.env.CARGO_TARGET_DIR||'target','debug/examples');
const executable=process.platform==='win32'?'.exe':'';
const hint=join(examples,'selection_hint'+executable);
const candidateProgram=join(examples,'proof_candidate'+executable);
const report={schema:1,createdAt:new Date().toISOString(),status:'pending',cases:[],limitations:['Synthetic source/SyncTeX/probe corpus; native mouse acceptance remains open.','Rotated/cropped/UserUnit pages, arbitrary TeX and independent runtime inspection are not certified.']};
function run(exe,args,cwd=root){const r=spawnSync(exe,args,{cwd,encoding:'utf8',windowsHide:true,timeout:90_000,maxBuffer:8*1024*1024});if(r.error||r.status!==0)throw new Error(r.error?.message||(r.stderr+r.stdout).slice(-1500));return r.stdout}
async function view(path,id){
 const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(await readFile(path))});
 const prepared={result:{build_id:id},pageCount:pdf.numPages,pages:[],pageSizes:new Map(),objects:new Map(),unclosed:new Set()};
 try{
  for(let n=1;n<=pdf.numPages;n++){
   const page=await pdf.getPage(n),viewport=page.getViewport({scale:1}),canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height));
   await page.render({canvas,canvasContext:canvas.getContext('2d'),viewport}).promise;
   const text=await page.getTextContent({includeMarkedContent:true}),g=extractTextGeometry(text.items,text.styles,n,viewport.height);
   prepared.pages.push({number:n,runs:g.runs,lines:g.lines,shell:{querySelector:()=>canvas}});prepared.pageSizes.set(n,{width:viewport.width,height:viewport.height});
   for(const id of g.unclosed)prepared.unclosed.add(id);
   for(const [id,bounds] of g.objects){const old=prepared.objects.get(id);if(old){old.pages.add(n);old.runs.push(...bounds.runs);old.instances+=bounds.instances;Object.assign(old,union([old,bounds]));}else prepared.objects.set(id,{...bounds,pages:new Set([n])});}
  }
  return prepared;
 }finally{await pdf.destroy()}
}
function equivalent(a,b){return a.page===b.page&&a.text===b.text&&['x','y','width','height','size','baseline'].every(k=>Math.abs(a[k]-b[k])<=1e-6)}
async function checkBulletSelection(engine) {
 const fixture=join(root,'tests/fixtures/proof/bullet-selection/main.tex');
 const source=await readFile(fixture,'utf8'),opening='\\item Alpha opens a longer list item.';
 const command=source.indexOf(opening);
 if(command<0)throw Error('The expected bullet is missing from the fixture');
 const item=command+'\\item '.length;
 const lineEnd=source.indexOf('\n',item);
 if(lineEnd<0)throw Error('The bullet fixture requires a complete source line');
 const itemEnd=source[lineEnd-1]==='\r'?lineEnd-1:lineEnd;
 const start=Buffer.byteLength(source.slice(0,item)),end=Buffer.byteLength(source.slice(0,itemEnd));
 const marker='proof_bullet';
 const work=join(output,engine.name,'bullet-selection');
 const baselineDir=join(work,'baseline'),probeDir=join(work,'tagged');
 await mkdir(baselineDir,{recursive:true});
 await mkdir(probeDir,{recursive:true});
 await cp(fixture,join(baselineDir,'main.tex'));
 const marked=run(candidateProgram,[fixture,fixture,String(start),String(end),marker,'0','0','bullet-probe']);
 await writeFile(join(probeDir,'main.tex'),marked);
 await cp(join(root,'resources/texloom-objects.sty'),join(probeDir,'texloom-objects.sty'));
 const args=['-interaction=nonstopmode','-halt-on-error','-file-line-error','-no-shell-escape','-synctex=1'];
 if(engine.miktex)args.push('--disable-installer');
 for(const dir of [baselineDir,probeDir]){
  const log=run(engine.name,[...args,'main.tex'],dir);
  await writeFile(join(dir,'compiler.log'),log);
 }
 const original=join(baselineDir,'main.pdf');
 const baseline=await view(original,'bullet-baseline'),probe=await view(join(probeDir,'main.pdf'),'bullet-probe');
 const page=baseline.pages[0];
 const first=page.lines.find(line=>line.text.includes('Alpha opens a longer list item.'));
 const wrapped=page.lines.find(line=>line!==first&&line.text.includes('will wrap at the line boundary.'));
 const other=page.lines.find(line=>line.text.includes('Bravo is short.'));
 if(!first||!wrapped||!other)throw Error('Bullet fixture must wrap and show the neighboring item');
 for(const hit of [first,wrapped]){
  const x=hit.x+Math.min(hit.width/2,15),y=hit.baseline-hit.height*.35;
  const response=run('synctex',['edit','-o','1:'+x+':'+y+':'+original],baselineDir);
  const input=response.match(/^Input:(.+)$/m)?.[1]?.trim(),line=response.match(/^Line:(\d+)$/m)?.[1];
  if(!input||!line||resolve(baselineDir,input)!==join(baselineDir,'main.tex'))throw Error('SyncTeX did not return the bullet source line');
  const unit=JSON.parse(run(hint,[fixture,fixture,line]));
  if(unit.kind!=='bullet_item'||unit.start!==start||unit.end!==end)throw Error('The clicked line did not resolve the complete bullet item');
 }
 const row={engine:engine.name,fixture:'bullet-selection'};
 try{
  const proof=validateSelectionProbe(baseline,probe,marker,[first,wrapped],1);
  if(proof.lines.length<2||proof.lines.includes(other))throw Error('Selection did not isolate the wrapped item');
 }catch(error){
  if(engine.name==='xelatex'&&/probe changed the page appearance/.test(error.message)){
   // The marker can alter XeLaTeX pixels even when the source hint is exact.
   return {...row,status:'expected-refusal',reason:error.message};
  }
  throw error;
 }
 let refused=false;
 try{validateSelectionProbe(baseline,probe,marker,[other],1)}
 catch(error){if(!/does not produce all the clicked text/.test(error.message))throw error;refused=true}
 if(!refused)throw Error('Bullet marker authorized the neighboring item');
 return {...row,status:'pass',kind:'bullet_item'};
}

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
    let unit=null,lastError=null;
    for(const fraction of [null,0.5,0.85]){
     row.hintAttempts=(row.hintAttempts||0)+1;
     try{
      const lines=[];
      for(const hit of hits){
       const x=hit.x+(fraction===null?Math.min(hit.width/2,15):hit.width*fraction),y=hit.baseline-hit.height*.35;
       const response=run('synctex',['edit','-o',target.page+':'+x+':'+y+':'+join(compiled,'baseline/main.pdf')]);
       const input=response.match(/^Input:(.+)$/m)?.[1]?.trim(),line=response.match(/^Line:(\d+)$/m)?.[1];
       if(!input||!line||!resolve(input).endsWith(fixture.targetFile))throw new Error('SyncTeX returned a different source file: '+response.slice(-700));lines.push(line);
      }
      const candidate=JSON.parse(run(hint,[join(sourceRoot,fixture.targetFile),join(sourceRoot,fixture.main),lines.join(',')]));
      if(candidate.start!==expectedStart||candidate.end!==expectedEnd)throw new Error('SyncTeX hint resolves a different source range: '+JSON.stringify(candidate));
      unit=candidate;break;
     }catch(e){lastError=e}
    }
    if(!unit)throw lastError;
    const proof=validateSelectionProbe(baseline,probe,id,hits,target.page);
    row.status='pass';row.lines=hits.length;row.runs=proof.runs.length;row.kind=unit.kind;
    if(engine.name==='lualatex'&&fixture.id==='minipage'){
     // LuaTeX's box hint can identify the closing environment for every line.
     // Exercise actual private builds: wrong candidate units must be refused
     // before the correct name acquires a capability. No golden range selects it.
     const clicked=hits[0],x=clicked.x+Math.min(clicked.width/2,15),y=clicked.baseline-clicked.height*.35;
     const response=run('synctex',['edit','-o',target.page+':'+x+':'+y+':'+join(compiled,'baseline/main.pdf')]);
     const actualLine=response.match(/^Line:(\d+)$/m)?.[1];
     const line=String(source.split('\n').findIndex(row=>row.trim()==='\\end{minipage}')+1);let accepted=null,rejections=0;
     for(let ordinal=0;ordinal<16;ordinal++){
      let hinted;try{hinted=JSON.parse(run(hint,[join(sourceRoot,fixture.targetFile),join(sourceRoot,fixture.main),line,String(ordinal)]))}catch{break}
      const work=join(output,'minipage-single-'+ordinal);await mkdir(work,{recursive:true});await cp(sourceRoot,work,{recursive:true});
      const marked=run(join(examples,'proof_candidate'+(process.platform==='win32'?'.exe':'')),
       [join(sourceRoot,fixture.targetFile),join(sourceRoot,fixture.main),String(hinted.start),String(hinted.end),'single_probe','0','0','wrap']);
      await writeFile(join(work,'main.tex'),marked.replace('\\begin{document}','\\usepackage{texloom-objects}\n\\begin{document}'));
      await cp(join(root,'resources/texloom-objects.sty'),join(work,'texloom-objects.sty'));
      const args=['-interaction=nonstopmode','-halt-on-error','-file-line-error','-no-shell-escape'];if(engine.miktex)args.push('--disable-installer');
      run(engine.name,[...args,'main.tex'],work);run(engine.name,[...args,'main.tex'],work);
      const staged=await view(join(work,'main.pdf'),'single-probe-'+ordinal);
      try{const capability=validateSelectionProbe(baseline,staged,'single_probe',[clicked],target.page);accepted={hinted,runs:capability.runs.length,ordinal};break}
      catch(error){if(!/mapped source does not produce all the clicked text/u.test(error.message))throw error;rejections++}
     }
     if(!accepted||accepted.runs!==clicked.runs.length||rejections<2)throw Error('Single-name minipage candidate isolation was not proven');
     report.cases.push({engine:engine.name,fixture:'minipage-single-name',status:'pass',private_builds:accepted.ordinal+1,wrong_candidates_rejected:rejections,actual_synctex_line:Number(actualLine),regression_box_hint_line:Number(line)});
    }
   }catch(e){
    row.status=engine.name==='pdflatex'&&fixture.id==='formatting'&&/probe changed the page appearance/.test(e.message)?'expected-refusal':'fail';row.reason=e.message;
    if(row.status==='expected-refusal')row.limitation='Measured unsupported pdfLaTeX mixed-format probe; the runtime pixel check remains strict.';
   }
  }
  try{report.cases.push(await checkBulletSelection(engine))}
  catch(error){report.cases.push({engine:engine.name,fixture:'bullet-selection',status:'fail',reason:error.message})}
 }
 report.status=report.cases.some(r=>r.status==='fail')?'fail':'pass';
}catch(e){report.status='fail';report.error=e.message}
await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log('TEXLOOM_SELECTION_REPORT_BEGIN\n'+JSON.stringify(report,null,2)+'\nTEXLOOM_SELECTION_REPORT_END');process.exitCode=report.status==='pass'?0:1;
