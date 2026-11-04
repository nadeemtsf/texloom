import {test} from 'node:test';
import assert from 'node:assert/strict';
import {setupAdvice} from '../web/core/setup-help.js';
import {CompilationFailure} from '../web/core/transactions.js';

test('only a known missing-compiler failure recommends compiler installation',()=>{
  assert.equal(setupAdvice('xelatex was not found. Install MiKTeX or TeX Live, then reopen Texloom.').kind,'compiler');
  const advice=setupAdvice(new CompilationFailure('Access is denied.',[{code:'compiler_preflight_access_denied',severity:'error',message:'The compiler could not initialize because access was denied.'}]));
  assert.equal(advice.kind,'initialization');assert.match(advice.steps.join(' '),/does not establish that LaTeX is missing/);
  assert.equal(setupAdvice(new CompilationFailure('! Undefined control sequence.')),null);
  assert.equal(setupAdvice(Error('The source changed during this operation.')),null);
});

test('missing packages distinguish distribution packages from project-owned files',()=>{
  const advice=setupAdvice(new CompilationFailure("! LaTeX Error: File `custom.cls' not found.",[{code:'missing_package',severity:'error',message:'custom.cls not found'}]));
  assert.equal(advice.kind,'package');assert.match(advice.steps.join(' '),/custom class or style/);
  assert.match(advice.steps.join(' '),/disables automatic MiKTeX/);
  assert.equal(setupAdvice({diagnostics:[{code:'missing_glyph'}]}).kind,'font');
  assert.equal(setupAdvice(Error('The font "Synthetic font" cannot be found.')).kind,'font');
});
