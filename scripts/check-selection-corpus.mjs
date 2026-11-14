/** Real compiler PDF pairs through the production PDF.js selection proof.
 * Headless synthetic regression only; no Texloom window, source commit or Save. */
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createServer} from 'vite';
const corpus=JSON.parse(await readFile(process.env.PDF_SELECTION_CORPUS,'utf8'));
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE?pathToFileURL(process.env.PLAYWRIGHT_MODULE).href:'playwright');
const server=await createServer({configFile:false,root:process.cwd(),optimizeDeps:{noDiscovery:true,entries:['tests/browser/pdf-navigation.html']},server:{host:'127.0.0.1',port:0,watch:null},logLevel:'error'});
let browser;const checks=[];
try{
 await server.listen();browser=await chromium.launch({headless:true,channel:'msedge'});
 const context=await browser.newContext({viewport:{width:1100,height:800},deviceScaleFactor:1.25});
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(`http://127.0.0.1:${server.httpServer.address().port}/tests/browser/pdf-navigation.html`);await page.waitForFunction(()=>window.ready);
 // Smoke check for the unchanged synthetic path in the shared fixture.
 await page.evaluate(()=>fixture.open(6));assert.equal((await page.evaluate(()=>fixture.proveSelection())).page,3);
 for(const row of corpus.rows){
  await page.evaluate(url=>fixture.open(6,{url}),row.baseline_url);
  await page.evaluate(async()=>{fixture.zoom(1.1,[450,310]);await fixture.finish()});
  const proof=await page.evaluate(row=>fixture.proveSelection({url:row.probe_url,id:row.id,page:row.page,hitText:row.hitText}),row);
  assert.equal(proof.page,row.page);assert.ok(proof.lines>0);checks.push({...row,passed:true});
 }
 assert.deepEqual(errors,[]);await page.evaluate(()=>fixture.close());
 const result={kind:'headless_three_engine_selection_identity_regression',browser:browser.version(),deviceScaleFactor:1.25,checks,
  scope:'Actual core source resolver and zero-offset wrapper, compiled baseline/probes, strict full-page PDF.js identity after zoom. No SyncTeX/UI selection, native commit, physical input or accuracy claim.'};
 if(process.env.PDF_TEST_OUTPUT_DIR){await mkdir(process.env.PDF_TEST_OUTPUT_DIR,{recursive:true});await writeFile(path.join(process.env.PDF_TEST_OUTPUT_DIR,'results.json'),JSON.stringify(result,null,2)+'\n')}
 console.log(JSON.stringify(result,null,2));
}catch(error){console.error(JSON.stringify({completed:checks},null,2));throw error}
finally{await browser?.close();await server.close()}
