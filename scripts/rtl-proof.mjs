// Development regression, not a held-out evaluation or native UI acceptance.
// Compiles only the repository's synthetic .tex template in a fresh directory.
import {spawnSync} from 'node:child_process';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import * as pdfjs from 'pdfjs-dist/legacy/build/pdf.mjs';
import {loadPDFDocument} from '../web/core/pdf-document.js';
import {extractTextGeometry,union} from '../web/core/geometry.js';
import {validateSelectionProbe} from '../web/core/selection-proof.js';
const {createCanvas}=createRequire(import.meta.url)('@napi-rs/canvas');
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
if(!process.env.TEXLOOM_RTL_PROOF_DIR)throw Error('Set TEXLOOM_RTL_PROOF_DIR to a fresh output directory');
const output=resolve(process.env.TEXLOOM_RTL_PROOF_DIR);
await mkdir(output,{recursive:false}); // Refuse to overwrite an earlier run.
const template=await readFile(join(root,'resources/templates/multilingual.tex'),'utf8');
const style=await readFile(join(root,'resources/texloom-objects.sty'),'utf8');
const cases=[['arabic','مرحبا بالعالم. هذا نص عربي للتجربة.'],['hebrew','שלום עולם. זהו טקסט בעברית לבדיקה.'],['english','This editable project keeps the engine choice and language commands in its source.']];
const report={schema:1,status:'pending',created_at:new Date().toISOString(),cases:[],limitations:['Synthetic development regression on this Windows/XeLaTeX machine.','Does not exercise the native pointer, SyncTeX candidate search or owner installation.']};
async function compile(name,source){
 const directory=join(output,name);await mkdir(directory);
 await writeFile(join(directory,'main.tex'),source);await writeFile(join(directory,'texloom-objects.sty'),style);
 for(let pass=1;pass<=2;pass++){
  const result=spawnSync('xelatex',['--disable-installer','-no-shell-escape','-interaction=nonstopmode','-halt-on-error','-synctex=1','main.tex'],{cwd:directory,encoding:'utf8',windowsHide:true,timeout:90_000,maxBuffer:8*1024*1024});
  await writeFile(join(directory,`compile-${pass}.log`),(result.stdout||'')+(result.stderr||''));
  if(result.error||result.status!==0)throw Error(`${name}: ${result.error?.message||'XeLaTeX failed; see log'}`);
 }
 return join(directory,'main.pdf');
}
async function view(path,id){
 const pdf=await loadPDFDocument(pdfjs,{data:new Uint8Array(await readFile(path))});
 const prepared={result:{build_id:id},pageCount:pdf.numPages,pages:[],pageSizes:new Map(),objects:new Map(),unclosed:new Set()};
 try{
  for(let n=1;n<=pdf.numPages;n++){
   const page=await pdf.getPage(n),viewport=page.getViewport({scale:1}),canvas=createCanvas(Math.ceil(viewport.width),Math.ceil(viewport.height));
   await page.render({canvas,canvasContext:canvas.getContext('2d'),viewport}).promise;
   const text=await page.getTextContent({includeMarkedContent:true}),geometry=extractTextGeometry(text.items,text.styles,n,viewport.height);
   prepared.pages.push({number:n,runs:geometry.runs,lines:geometry.lines,shell:{querySelector:()=>canvas}});
   prepared.pageSizes.set(n,{width:viewport.width,height:viewport.height});
   for(const id of geometry.unclosed)prepared.unclosed.add(id);
   for(const [id,bounds] of geometry.objects){
    const old=prepared.objects.get(id);
    if(old){old.pages.add(n);old.runs.push(...bounds.runs);old.instances+=bounds.instances;Object.assign(old,union([old,bounds]));}
    else prepared.objects.set(id,{...bounds,pages:new Set([n])});
   }
  }
  return prepared;
 }finally{await pdf.destroy()}
}
try{
 const baseline=await view(await compile('baseline',template),'baseline');
 // Label the clicked line from the intended baseline text, independently of
 // whichever glyphs the candidate marker happens to own. Arabic extraction
 // uses presentation forms; NFKC is only for this synthetic label lookup.
 const hitsFor=text=>baseline.pages.flatMap(page=>page.lines.filter(line=>line.text.normalize('NFKC')===text.normalize('NFKC')));
 for(const [name,text] of cases){
  const row={language:name,status:'pending'};report.cases.push(row);
  if(template.split(text).length!==2)throw Error(`${name}: expected one exact synthetic target`);
  const id=`probe-${name}`;
  const source=template.replace('\\begin{document}','\\usepackage{texloom-objects}\n\\begin{document}').replace(text,`\\TexloomObject{${id}}{0}{0}{${text}}`);
  const probe=await view(await compile(name,source),name);
  const object=probe.objects.get(id);if(!object)throw Error(`${name}: marker does not own text`);
  const hits=hitsFor(text);if(hits.length!==1)throw Error(`${name}: baseline target label is not unique`);
  const proof=validateSelectionProbe(baseline,probe,id,hits,1);
  row.status='pass';row.mapped_lines=proof.lines.length;row.mapped_runs=proof.runs.length;
  row.wrong_target_controls_refused=0;
  for(const [other,otherText] of cases.filter(([other])=>other!==name)){
   const wrong=hitsFor(otherText);if(wrong.length!==1)throw Error(`${other}: negative-control label is not unique`);
   let refused=false;
   try{validateSelectionProbe(baseline,probe,id,wrong,1)}catch{refused=true}
   if(!refused)throw Error(`${name}: marker accepted the ${other} target`);
   row.wrong_target_controls_refused++;
  }
 }
 report.status='pass';
}catch(error){report.status='fail';report.error=String(error?.stack||error);process.exitCode=1}
finally{await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report))}
