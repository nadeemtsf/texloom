import {spawnSync} from 'node:child_process';
import {mkdir,readFile,writeFile,cp,access} from 'node:fs/promises';
import {resolve,join,dirname,delimiter} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {inspectPdfjs} from './proof/pdfjs-inspector.mjs';
import {checkTranslation,checkRaster,checkTargetRaster,targetRecords} from './proof/layout-check.mjs';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const output=join(root,'artifacts','proof');
const python=process.env.PYTHON ?? (process.platform==='win32'?'python':'python3');
const manifest=JSON.parse(await readFile(join(root,'tests/fixtures/proof/manifest.json'),'utf8'));
const report={schema:1,createdAt:new Date().toISOString(),status:'pending',baselineCommit:'e7b6d1eb334c879a4aa28c4f22b984fb3c17532b',
  sourceChecks:[],engines:[],comparisons:[],limitations:['Fixture oracle, not live operation authorization.',
    'Crop/rotation/UserUnit and arbitrary graphics appearance need stronger inspection.',
    'Independent character/ink bounds are compared within each renderer, not across renderers.']};
await mkdir(output,{recursive:true});
function run(file,args,options={}) {
  const r=spawnSync(file,args,{cwd:root,encoding:'utf8',windowsHide:true,timeout:90_000,maxBuffer:8*1024*1024,...options});
  if(r.error || r.status!==0) throw new Error(r.error?.message || (r.stderr+r.stdout).slice(-3000) || 'Process failed');
  return r.stdout;
}
async function inspect(path,label) {
  const json=join(output,label+'.json');
  const env={...process.env,PYTHONPATH:[join(root,'.tools/proof-python'),process.env.PYTHONPATH].filter(Boolean).join(delimiter)};
  run(python,[join(root,'scripts/proof/inspect_pdf.py'),path,json],{env});
  return {...JSON.parse(await readFile(json,'utf8')),evidencePath:json};
}
let policySequence=0;
async function rustPolicy(before,after,beforePdf,afterPdf,fixture,operation) {
  const request=join(output,'policy-'+(++policySequence)+'.json');
  await writeFile(request,JSON.stringify({before:before.evidencePath,after:after.evidencePath,before_pdf:beforePdf,after_pdf:afterPdf,
    before_target:[...targetRecords(before.records,fixture.text,fixture.occurrence)],
    after_target:[...targetRecords(after.records,fixture.text,fixture.occurrence)],operation}));
  const exe=join(root,'target/debug/examples/displacement_check'+(process.platform==='win32'?'.exe':''));
  return JSON.parse(run(exe,[request]));
}
async function rasterPair(a,b,target,identity) {
  const old=await Promise.all(a.rasters.map(r=>readFile(r.path)));
  const next=await Promise.all(b.rasters.map(r=>readFile(r.path)));
  const result=checkRaster(a,b,old,next,target,{identity});
  if(!identity)checkTargetRaster(a,b,old,next,target,{dx:manifest.dx,dy:manifest.dy});
  return result;
}
function attempt(fn) {try{return {status:'pass',result:fn()}}catch(e){return {status:'fail',reason:e.message}}}
async function regression() {
  const before=join(root,'tests/fixtures/objects/baseline.pdf'),after=join(root,'tests/fixtures/objects/single-move.pdf');
  const policy={text:'Dr. Example Researcher',dx:0,dy:98,tolerance:0.25};
  const a=await inspectPdfjs(before),b=await inspectPdfjs(after);
  const pdfjs=attempt(()=>checkTranslation(a,b,policy));
  let pdfium,rust;
  try {const x=await inspect(before,'regression-baseline'),y=await inspect(after,'regression-broken');
    report.rust_identity_smoke=await rustPolicy(x,x,before,before,policy,{kind:'identity'});
    pdfium=attempt(()=>checkTranslation(x,y,policy));
    try {rust=await rustPolicy(x,y,before,after,policy,{kind:'displace',dx:0,dy:98})}
    catch(e){rust={status:/Target or neighbor has no match|Raster appearance changed beyond/u.test(e.message)?'fail':'blocked',reason:e.message}}
  } catch(e) {pdfium={status:'blocked',reason:e.message};rust={status:'blocked',reason:e.message};}
  report.regression={pdfjs,pdfium,rust,rejected:pdfjs.status==='fail'&&rust.status==='fail'};
  if(!report.regression.rejected) throw new Error('Known broken PDF was accepted by the comparison pipeline');
}
function selection(source) {
  const startMarker='% proof:begin\n',endMarker='% proof:end';
  const start=source.indexOf(startMarker)+startMarker.length,end=source.indexOf(endMarker);
  if(start<startMarker.length || end<start)throw new Error('Missing fixture selection delimiters');
  return [Buffer.byteLength(source.slice(0,start)),Buffer.byteLength(source.slice(0,end).replace(/\n$/u,''))];
}
async function prepare() {
  const exe=process.env.PROOF_CANDIDATE_BIN ?? join(root,'target/debug/examples/proof_candidate'+(process.platform==='win32'?'.exe':''));
  await access(exe);
  for(const fixture of manifest.cases) {
    const sourceDir=join(root,'tests/fixtures/proof',fixture.id);
    const sourcePath=join(sourceDir,fixture.targetFile),mainPath=join(sourceDir,fixture.main);
    const source=await readFile(sourcePath),text=source.toString('utf8'),[start,end]=selection(text);
    const candidate=(dx,dy,action)=>run(exe,[sourcePath,mainPath,String(start),String(end),'proof_'+fixture.id,String(dx),String(dy),action]);
    if(fixture.expect==='refuse') {
      let refused=false;
      try{candidate(manifest.dx,manifest.dy,'wrap')}catch(e){refused=true;report.sourceChecks.push({id:fixture.id,status:'refused',reason:e.message})}
      if(!refused)throw new Error(fixture.id+' must be refused by the actual Rust source planner');
      continue;
    }
    const detached=Buffer.from(candidate(manifest.dx,manifest.dy,'detach'));
    if(!detached.equals(source))throw new Error(fixture.id+' failed exact wrap/unwrap source identity');
    report.sourceChecks.push({id:fixture.id,status:'pass',sourceSha256:createHash('sha256').update(source).digest('hex')});
    for(const variant of ['baseline','zero','moved']) {
      const dir=join(output,fixture.id,variant);await mkdir(dir,{recursive:true});
      await cp(sourceDir,dir,{recursive:true});
      if(variant!=='baseline') {
        await writeFile(join(dir,fixture.targetFile),candidate(variant==='zero'?0:manifest.dx,variant==='zero'?0:manifest.dy,'wrap'));
        const main=await readFile(join(dir,fixture.main),'utf8');
        await writeFile(join(dir,fixture.main),main.replace('\\begin{document}','\\usepackage{texloom-objects}\n\\begin{document}'));
        await cp(join(root,'resources/texloom-objects.sty'),join(dir,'texloom-objects.sty'));
      }
    }
  }
}
async function compile(engine,dir) {
  const args=['-interaction=nonstopmode','-halt-on-error','-file-line-error','-no-shell-escape','-synctex=1'];
  if(engine.miktex)args.push('--disable-installer');
  for(let pass=0;pass<2;pass++) {
    const log=run(engine.name,[...args,'main.tex'],{cwd:dir,env:{...process.env,SOURCE_DATE_EPOCH:'1767225600',FORCE_SOURCE_DATE:'1'}});
    await writeFile(join(dir,'pass-'+pass+'.txt'),log);
  }
  return join(dir,'main.pdf');
}
async function matrix() {
  for(const name of ['pdflatex','lualatex','xelatex']) {
    let engine;
    try {
      const version=run(name,['--version']);
      if(/RegOpenKey|Access is denied|fatal error/iu.test(version))throw new Error('Engine configuration is inaccessible');
      engine={name,status:'ready',version,miktex:version.includes('MiKTeX')};
    } catch(e){report.engines.push({name,status:'blocked',reason:e.message});continue;}
    report.engines.push(engine);
    for(const fixture of manifest.cases.filter(f=>f.expect!=='refuse')) {
      const row={engine:name,fixture:fixture.id,status:'pending'};
      report.comparisons.push(row);
      try {
        const variants={};
        for(const variant of ['baseline','zero','moved']) {
          const prepared=join(output,fixture.id,variant),dir=join(output,'compiled',name,fixture.id,variant);
          await mkdir(dir,{recursive:true});await cp(prepared,dir,{recursive:true});
          const pdf=await compile(engine,dir);
          variants[variant]={pdf,pdfjs:await inspectPdfjs(pdf),pdfium:await inspect(pdf,name+'-'+fixture.id+'-'+variant)};
          if(variant==='baseline' && fixture.expect==='cross-page') {
            const ids=targetRecords(variants.baseline.pdfium.records,fixture.text);
            const pages=new Set([...ids].map(i=>variants.baseline.pdfium.records[i].page));
            if(pages.size<2)throw new Error('Cross-page fixture did not actually cross pages');
            try {await rustPolicy(variants.baseline.pdfium,variants.baseline.pdfium,pdf,pdf,fixture,{kind:'displace',dx:manifest.dx,dy:manifest.dy});throw new Error('Rust accepted a cross-page target')}
            catch(e){if(!/Target crosses physical pages/u.test(e.message))throw e;row.rust_movement={status:'expected-refusal',reason:e.message}}
            row.status='expected-refusal';row.reason='Target occupies multiple pages';break;
          }
        }
        if(row.status==='expected-refusal')continue;
        const baseline=variants.baseline,zero=variants.zero,moved=variants.moved;
        row.rust_identity=await rustPolicy(baseline.pdfium,zero.pdfium,baseline.pdf,zero.pdf,fixture,{kind:'identity'});
        try {row.rust_movement=await rustPolicy(baseline.pdfium,moved.pdfium,baseline.pdf,moved.pdf,fixture,{kind:'displace',dx:manifest.dx,dy:manifest.dy})}
        catch(e){if(fixture.expect!=='overlap-refuse'||!/Target raster overlaps/u.test(e.message))throw e;row.rust_movement={status:'expected-refusal',reason:e.message}}
        if(fixture.expect==='overlap-refuse'&&row.rust_movement.status!=='expected-refusal')throw new Error('Rust accepted target overlap');
        for(const renderer of ['pdfjs','pdfium']) {
          const policy={text:fixture.text,occurrence:fixture.occurrence,tolerance:manifest.toleranceBp};
          checkTranslation(baseline[renderer],zero[renderer],{...policy,dx:0,dy:0});
          const target=checkTranslation(baseline[renderer],moved[renderer],{...policy,dx:manifest.dx,dy:manifest.dy});
          if(renderer==='pdfjs') {
            for(const i of target.after)if(!moved.pdfjs.records[i].markers.includes('Texloom_proof_'+fixture.id))
              throw new Error('The moved PDF target lost its exact object marker');
          } else {
            await rasterPair(baseline.pdfium,zero.pdfium,target,true);
            row.raster=await rasterPair(baseline.pdfium,moved.pdfium,target,false);
          }
        }
        if(fixture.expect==='overlap-refuse')throw new Error('Expected overlap was not refused');
        row.status='pass';
      } catch(e) {
        row.status=fixture.expect==='overlap-refuse' && /Target mask overlaps/u.test(e.message)?'expected-refusal':'fail';
        row.reason=e.message;
      }
    }
  }
}
try {
  await regression();
  if(process.argv.includes('--regression'))report.status=report.regression.pdfium.status==='blocked'?'blocked':'pass';
  else {
    await prepare();
    if(process.argv.includes('--prepare'))report.status=report.regression.pdfium.status==='blocked'?'blocked':'prepared';
    else {
      await matrix();
      report.status=report.engines.some(e=>e.status==='blocked')?'blocked':
        report.comparisons.some(c=>c.status==='fail')?'fail':'pass';
    }
  }
} catch(e) {report.status='fail';report.error=e.message;}
await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode=['pass','prepared'].includes(report.status)?0:report.status==='blocked'?2:1;
