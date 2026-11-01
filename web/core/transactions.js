import {validateObjectLayout} from './geometry.js';

export class StaleCandidate extends Error {
  constructor(){super('The source changed during this operation. The newer source was preserved.');this.name='StaleCandidate'}
}
export class CompilationFailure extends Error {
  constructor(log,diagnostics=[]){super(compileDiagnostic(log));this.name='CompilationFailure';this.log=log;this.diagnostics=diagnostics}
}
export function compileDiagnostic(log='') {
  const lines=String(log).split(/\r?\n/);
  const index=lines.findIndex(line=>/\.tex:\d+:.*(?:error|undefined|missing|extra|runaway)/i.test(line)||/^!\s/.test(line));
  if(index<0)return 'Compilation failed. Open compiler output for details.';
  let message=lines[index].trim();
  for(let n=index+1;n<Math.min(lines.length,index+4);n++){
    if(!lines[n].trim()||/^(See |Type |l\.\d|\s*\.\.\.)/.test(lines[n]))break;
    message+=(lines[n-1].length>=78?'':' ')+lines[n].trim();
  }
  return message;
}
export function sessionIdentity(session) {
  return `${session.project?.id}\0${session.project?.main}\0${session.revision}\0${session.buildEpoch||0}`;
}

/** No source, history, or PDF state is mutated before the complete candidate passes. */
export async function stageCandidate({sources,revision,identity,getIdentity,compile,prepare,policy,previousPageCount,verify=async()=>{},requireVerified=false}) {
  let prepared=null;
  try{
    if(getIdentity()!==identity)throw new StaleCandidate();
    const result=await compile(structuredClone(sources),revision);
    if(getIdentity()!==identity)throw new StaleCandidate();
    if(!result.ok||requireVerified&&result.verification_eligible===false)throw new CompilationFailure(result.log,result.diagnostics);
    if(result.revision!==revision)throw new StaleCandidate();
    prepared=await prepare(result);
    if(getIdentity()!==identity)throw new StaleCandidate();
    validateObjectLayout(prepared,policy,previousPageCount);
    await verify(result);
    if(getIdentity()!==identity)throw new StaleCandidate();
    return {sources,result,prepared};
  }catch(error){if(prepared?.pdf)await prepared.pdf.destroy();if(error.name!=='CompilationCancelled'&&getIdentity()!==identity)throw new StaleCandidate();throw error}
}
