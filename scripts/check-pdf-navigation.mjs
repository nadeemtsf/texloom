/** Opt-in real-browser renderer checks, no desktop app or physical input.
 * Supply PLAYWRIGHT_MODULE if Playwright is not installed in the local project. */
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {mkdir,writeFile} from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'vite';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE?pathToFileURL(process.env.PLAYWRIGHT_MODULE).href:'playwright');
const server=await createServer({configFile:false,root:process.cwd(),optimizeDeps:{noDiscovery:true,entries:['tests/browser/pdf-navigation.html']},server:{host:'127.0.0.1',port:0,watch:null},logLevel:'error'});
let browser;const checks=[];const output=process.env.PDF_TEST_OUTPUT_DIR;
if(output)await mkdir(output,{recursive:true});
try{
 console.error("starting fixture server");await server.listen();console.error("starting headless browser");const port=server.httpServer.address().port;
 browser=await chromium.launch({headless:true,channel:process.env.PDF_TEST_BROWSER||'msedge',timeout:20000});console.error('browser ready');
 for(const deviceScaleFactor of [1,1.5,2]){
  console.error('creating context',deviceScaleFactor);const context=await browser.newContext({viewport:{width:1100,height:800},deviceScaleFactor});
  console.error('creating page');const page=await context.newPage(),errors=[];console.error('navigating');page.setDefaultTimeout(15000);page.on('pageerror',error=>{errors.push(error.message);console.error(error)});page.on('console',m=>{if(m.type()==='error')console.error(m.text())});
  await page.goto(`http://127.0.0.1:${port}/tests/browser/pdf-navigation.html`);console.error('loaded');await page.waitForFunction(()=>window.ready);
  for(const count of [6,25]){
   console.error('opening',deviceScaleFactor,count);
   await page.evaluate(n=>Promise.race([fixture.open(n),new Promise((_,reject)=>setTimeout(()=>reject(Error('Open timed out')),20000))]),count);
   console.error('opened');
   if(count===6){
    const initialProof=await page.evaluate(()=>fixture.proveSelection());assert.equal(initialProof.page,3);
    checks.push({deviceScaleFactor,pages:count,case:'selection immediately after initial compilation/render',passed:true});
   }
   await page.evaluate(async()=>{await fixture.view.ensurePage(3);const p=fixture.view.pages[2].shell;fixture.elements.scroller.scrollTop=p.offsetTop+100});
   await page.waitForTimeout(100);
   const origin=[450,310],before=await page.evaluate(o=>fixture.point(o),origin);
   const canvasBefore=await page.evaluate(()=>{window.oldCanvas=fixture.view.pages[2].pageView.canvas;return fixture.status().paintCalls});
   await page.evaluate(o=>fixture.zoom(1.8,o),origin);
   const immediate=await page.evaluate(o=>({point:fixture.point(o),status:fixture.status(),same:window.oldCanvas===fixture.view.pages[2].pageView.canvas}),origin);
   assert.equal(immediate.same,true,'zoom reuses the current bitmap');
   assert.equal(immediate.status.paintCalls,canvasBefore,'fast zoom must not paint');
   assert.equal(immediate.point.page,before.page);
   assert.ok(Math.abs(immediate.point.pdf[0]-before.pdf[0])<1,'cursor horizontal PDF point');
   assert.ok(Math.abs(immediate.point.pdf[1]-before.pdf[1])<1,'cursor vertical PDF point');
   await page.evaluate(()=>fixture.finish());
   const after=await page.evaluate(()=>fixture.status());assert.ok(after.text.includes('anchor line'));
   assert.ok(after.paintCalls-canvasBefore<count,'zoom paints nearby pages rather than the whole PDF');
   checks.push({deviceScaleFactor,pages:count,case:'cached-canvas pointer anchor and nearby sharp redraw',passed:true,
    pointError:immediate.point.pdf.map((v,i)=>Math.abs(v-before.pdf[i])),draws:after.paintCalls-canvasBefore});
   const serial=await page.evaluate(()=>fixture.status().refinements);
   await page.evaluate(o=>{for(let i=0;i<30;i++)fixture.wheel({ctrlKey:true,deltaY:i%2?-8:4,clientX:o[0],clientY:o[1]})},origin);
   const during=await page.evaluate(()=>fixture.status());assert.equal(during.refinements,serial);
   await page.waitForTimeout(550);await page.evaluate(()=>fixture.waitRefinement());
   const settled=await page.evaluate(()=>fixture.status());assert.equal(settled.refinements,serial+1);
   await page.evaluate(()=>{const s=fixture.elements.scroller;window.oldTop=s.scrollTop;fixture.wheel({deltaY:87.5})});
   assert.ok(Math.abs(await page.evaluate(()=>fixture.elements.scroller.scrollTop-window.oldTop)-87.5)<=1,'browser scroll coordinates may round to a device pixel');
   checks.push({deviceScaleFactor,pages:count,case:'rapid Ctrl zoom coalesces; ordinary scrolling resumes',passed:true});
   const dimensions=await page.evaluate(()=>{const p=fixture.view.pages[2],r=p.pageView.div.getBoundingClientRect();return {canvas:p.pageView.canvas.width,css:r.width,dpr:devicePixelRatio,outputScale:p.pageView.outputScale,styles:p.pageView.div.style.cssText,outer:p.shell.style.cssText,round:parseFloat(getComputedStyle(p.pageView.div).getPropertyValue('--scale-round-x')),scale:fixture.view.scale,pdf:fixture.view.pageSizes.get(3).width}});
   assert.ok(Math.abs(dimensions.css-dimensions.pdf*dimensions.scale)<dimensions.round+0.05,JSON.stringify(dimensions));assert.ok(dimensions.canvas>=dimensions.css,'sharp canvas respects device scale: '+JSON.stringify(dimensions));
   const textRects=await page.evaluate(()=>{const span=fixture.view.pages[2].pageView.div.querySelector('.textLayer span'),p=fixture.view.pages[2].pageView.div.getBoundingClientRect(),r=span.getBoundingClientRect();return {left:r.left-p.left,top:r.top-p.top,width:r.width,scale:fixture.view.scale}});
   assert.ok(Math.abs(textRects.left-70*textRects.scale)<=2,'text layer remains aligned with PDF coordinates');assert.ok(textRects.width>0);
   if(output&&deviceScaleFactor===1&&count===6)await page.screenshot({path:path.join(output,'pdfjs-zoom-refined.png')});
   // Fit uses the viewport center, while zoom-out at the edges clamps naturally.
   await page.evaluate(async()=>{const width=fixture.elements.scroller.clientWidth;fixture.zoom(Math.min(1.5,(width-48)/600));await fixture.finish();fixture.elements.scroller.scrollTop=0;fixture.zoom(0.2,[450,310]);await fixture.finish()});
   assert.ok(await page.evaluate(()=>fixture.elements.scroller.scrollTop>=0&&fixture.elements.scroller.scrollLeft>=0));
   checks.push({deviceScaleFactor,pages:count,case:'text-layer alignment, fit and edge-clamped zoom-out',passed:true});
   if(count===6){
    // All pages participate in source-identity proof, including offscreen cached
    // canvases that did not need sharp repaint during the preceding zoom.
    await page.evaluate(async()=>{fixture.zoom(1.15,[450,310]);await fixture.finish()});
    const proof=await page.evaluate(()=>fixture.proveSelection());assert.equal(proof.schema,1);assert.equal(proof.page,3);
    await assert.rejects(page.evaluate(()=>fixture.proveSelection({inkFault:true})),/appearance/);
    checks.push({deviceScaleFactor,pages:count,case:'selection after zoom checks all pages and rejects offscreen non-text ink change',passed:true});
    const beforeInput=await page.evaluate(()=>({scale:fixture.view.scale,width:innerWidth,pageScale:visualViewport.scale}));
    await page.mouse.move(450,310);await page.keyboard.down('Control');await page.mouse.wheel(0,-60);await page.keyboard.up('Control');
    await page.waitForTimeout(550);await page.evaluate(()=>fixture.waitRefinement());
    assert.ok(await page.evaluate(()=>fixture.view.scale)>beforeInput.scale);
    const afterPDF=await page.evaluate(()=>({scale:fixture.view.scale,width:innerWidth,pageScale:visualViewport.scale}));
    assert.equal(afterPDF.width,beforeInput.width);assert.equal(afterPDF.pageScale,beforeInput.pageScale);
    await page.mouse.move(25,25);await page.keyboard.down('Control');await page.mouse.wheel(0,-60);await page.keyboard.up('Control');
    await page.keyboard.press('Control+=');await page.waitForTimeout(80);
    assert.deepEqual(await page.evaluate(()=>({scale:fixture.view.scale,width:innerWidth,pageScale:visualViewport.scale})),afterPDF);
    checks.push({deviceScaleFactor,case:'browser-injected Ctrl-wheel zooms only PDF; outside wheel and keyboard cannot zoom shell',passed:true});
    const cdp=await context.newCDPSession(page);
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1100,height:800,mobile:false,deviceScaleFactor:deviceScaleFactor+0.25});
    const densityProof=await page.evaluate(()=>fixture.proveSelection());assert.equal(densityProof.page,3);
    checks.push({deviceScaleFactor,case:'selection repaints stale cached density after emulated DPR change',passed:true});
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1100,height:800,mobile:false,deviceScaleFactor});await cdp.detach();
   }
  }
  // An old queued refinement must not paint into a newly opened document.
  await page.evaluate(async()=>{fixture.zoom(1.5,[450,310]);const old=fixture.finish().catch(()=>false);await fixture.open(3);await old});
  assert.equal(await page.evaluate(()=>fixture.status().pages),3);assert.deepEqual(errors,[]);
  checks.push({deviceScaleFactor,case:'document replacement during pending redraw',passed:true});
  await page.evaluate(()=>fixture.close());await context.close();
 }
 const result={kind:'headless-browser-renderer-development-check',browser:browser.version(),checks,physicalInput:'unexecuted; owner-only'};
 if(output)await writeFile(path.join(output,'browser-results.json'),JSON.stringify(result,null,2)+'\n');
 console.log(JSON.stringify(result,null,2));
}catch(error){console.error(JSON.stringify({completedChecks:checks},null,2));throw error}finally{await browser?.close();await server.close()}
