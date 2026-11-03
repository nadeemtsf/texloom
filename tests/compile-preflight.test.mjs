import {test} from 'node:test';
import assert from 'node:assert/strict';
import {CompilationFailure,stageCandidate} from '../web/core/transactions.js';

test('compiler initialization failures show their short diagnostic and retain the captured trace',()=>{
  const log='Synthetic initialization trace\n'.repeat(1000)+'Access is denied.';
  const diagnostics=[{code:'compiler_preflight_access_denied',severity:'error',message:'The compiler could not initialize because access was denied. Open Compiler output for details.'}];
  const error=new CompilationFailure(log,diagnostics);
  assert.equal(error.message,diagnostics[0].message);assert.equal(error.log,log);assert.equal(error.diagnostics,diagnostics);
  assert.equal(new CompilationFailure('! Undefined control sequence.\n').message,'! Undefined control sequence.');
});

test('a compiler preflight failure never prepares or verifies a candidate PDF',async()=>{
  const sources={'main.tex':'Original source'},snapshot=structuredClone(sources);let prepared=0,verified=0;
  const diagnostics=[{code:'compiler_preflight_output_truncated',severity:'error',message:'Compiler version output exceeded its limit.'}];
  await assert.rejects(stageCandidate({sources,revision:4,identity:'current',getIdentity:()=>'current',
    compile:async()=>({ok:false,verification_eligible:false,revision:4,pdf:[],log:'Captured version output',diagnostics}),
    prepare:async()=>{prepared++},verify:async()=>{verified++}}),error=>{
      assert.equal(error.message,diagnostics[0].message);assert.equal(error.log,'Captured version output');return true;
    });
  assert.equal(prepared,0);assert.equal(verified,0);assert.deepEqual(sources,snapshot);
});
