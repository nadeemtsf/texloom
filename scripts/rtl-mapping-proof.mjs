// Execute research/RTL-MAPPING-PILOT.md on a fresh synthetic native workspace.
import {spawn} from 'node:child_process';
import {createWriteStream} from 'node:fs';
import {access,mkdir,writeFile} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createInterface} from 'node:readline';
import {once} from 'node:events';
import {proofPDFView} from './proof-pdf-view.mjs';
import {validateSelectionProbe} from '../web/core/selection-proof.js';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
if(!process.env.TEXLOOM_RTL_MAPPING_DIR)throw Error('Set TEXLOOM_RTL_MAPPING_DIR to a fresh directory');
const executable=resolve(process.env.TEXLOOM_SELECTION_HARNESS||join(root,'target/debug/examples/native_selection_development.exe'));
await access(executable); // Refuse to launch a missing or unfinished build.
const output=resolve(process.env.TEXLOOM_RTL_MAPPING_DIR);await mkdir(output,{recursive:false});
const log=createWriteStream(join(output,'native-stderr.log'),{flags:'wx'});
const child=spawn(executable,[join(output,'native')],{cwd:root,windowsHide:true,stdio:['pipe','pipe','pipe']});
child.stderr.pipe(log);
const exited=once(child,'exit').then(([code])=>({code}),error=>({error:String(error)}));
const lines=createInterface({input:child.stdout})[Symbol.asyncIterator]();
const report={schema:1,protocol:'RTL-MAPPING-PILOT-v1',status:'pending',created_at:new Date().toISOString(),cases:[],limitations:['Reused synthetic template, not held-out final evaluation.','Actual native resolver/planner/compile plus PDF.js probe checks; no frontend candidate loop or native pointer input.','No move commit, Save, latency, general RTL or usability result.']};
const labels=[['arabic','مرحبا بالعالم. هذا نص عربي للتجربة.'],['hebrew','שלום עולם. זהו טקסט בעברית לבדיקה.'],['english','This editable project keeps the engine choice and language commands in its source.']];
async function message(){
 let timer;
 try{
  const next=await Promise.race([lines.next(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('Native harness response timeout')),120_000)})]);
  if(next.done)throw Error('Native harness exited before its reply; see stderr');
  const value=JSON.parse(next.value);if(value.status==='fatal')throw Error(value.reason);return value;
 }finally{clearTimeout(timer)}
}
try{
 const ready=await message();if(ready.status!=='ready'||ready.engine!=='xelatex')throw Error('Unexpected native baseline');
 const baseline=await proofPDFView(ready.baseline_pdf,'native-baseline');
 const hitsFor=text=>baseline.pages.flatMap(page=>page.lines.filter(line=>line.text.normalize('NFKC')===text.normalize('NFKC')));
 for(const [language,text] of labels){
  const hits=hitsFor(text);if(hits.length!==1)throw Error(`${language}: intended baseline label is not unique`);
  const line=hits[0],row={language,status:'pending',point:{x:line.x+line.width/2,y:line.baseline-line.height*.35},attempts:[]};report.cases.push(row);
  for(let candidate=0;candidate<16;candidate++){
   child.stdin.write(JSON.stringify({language,candidate,...row.point})+'\n');
   const result=await message();await writeFile(join(output,`${language}-${candidate}-native.json`),JSON.stringify(result,null,2)+'\n',{flag:'wx'});
   if(result.canonical_and_disk_unchanged!==true)throw Error('Native canonical/disk preservation failed');
   const attempt={candidate,native_status:result.status};row.attempts.push(attempt);
   if(result.status==='refused'){attempt.reason=result.reason;continue}
   if(result.status!=='probe')throw Error('Unexpected native query reply');
   const probe=await proofPDFView(result.pdf,`${language}-${candidate}`);
   try{validateSelectionProbe(baseline,probe,result.id,hits,1)}catch(error){attempt.probe_accepted=false;attempt.reason=error.message;continue}
   attempt.probe_accepted=true;row.source_file=result.unit.file;row.source_lines=[result.unit.first_line,result.unit.last_line];row.wrong_target_controls_refused=0;
   for(const [other,otherText] of labels.filter(([name])=>name!==language)){
    const wrong=hitsFor(otherText);if(wrong.length!==1)throw Error(`${other}: negative label not unique`);
    let refused=false;try{validateSelectionProbe(baseline,probe,result.id,wrong,1)}catch{refused=true}
    if(!refused)throw Error(`${language}: accepted wrong ${other} target`);row.wrong_target_controls_refused++;
   }
   row.status='pass';break;
  }
  if(row.status!=='pass')throw Error(`${language}: no intended candidate passed within16 attempts`);
 }
 child.stdin.end();const exit=await exited;if(exit.code!==0)throw Error(`Native harness exit: ${JSON.stringify(exit)}`);
 report.status='pass';
}catch(error){report.status='fail';report.error=String(error?.stack||error);process.exitCode=1;child.stdin.end();}
finally{
 await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});
 console.log(JSON.stringify(report));
}
