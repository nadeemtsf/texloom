/** Run with Node 22.13+ and the path to a packaged Windows Texloom.exe.
 * Uses normal WebView pointer and keyboard events on disposable project copies. */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {cp,mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises';
import {createServer} from 'node:net';
import {basename,dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';

if(process.platform!=='win32'||!process.argv[2])throw Error('Supply the Windows portable executable path.');
const repo=fileURLToPath(new URL('../',import.meta.url));
const executable=resolve(process.argv[2]);
const artifacts=join(repo,'artifacts','native-pdf-typing');
await mkdir(artifacts,{recursive:true});
const work=await mkdtemp(join(artifacts,'run-'));
const server=createServer();server.listen(0,'127.0.0.1');await once(server,'listening');
const {port}=server.address();await new Promise(resolve=>server.close(resolve));
const child=spawn(executable,[],{cwd:dirname(executable),windowsHide:true,stdio:'ignore',env:{...process.env,
  WEBVIEW2_USER_DATA_FOLDER:join(work,'webview'),
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-address=127.0.0.1 --remote-debugging-port='+port}});
let socket,sequence=0;
const requests=new Map();
async function waitFor(read,label,timeout=90000){
  const end=Date.now()+timeout;
  do{const value=await read();if(value)return value;await sleep(50)}while(Date.now()<end);
  throw Error('Timed out waiting for '+label);
}
function send(method,params={}){
  return new Promise((resolve,reject)=>{
    const id=++sequence;requests.set(id,{resolve,reject});socket.send(JSON.stringify({id,method,params}));
  });
}
async function evaluate(expression){
  const {result,exceptionDetails}=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});
  if(exceptionDetails)throw Error(exceptionDetails.exception?.description||exceptionDetails.text);
  return result.value;
}
const until=(expr,label)=>waitFor(()=>evaluate(expr),label);
const control=id=>'document.getElementById('+JSON.stringify(id)+')';
async function button(id){
  await until('!'+control(id)+'.disabled',id+' enabled');
  await evaluate(control(id)+'.click()');
}
async function clickLine(text,fraction=0){
  const point=await evaluate(`(()=>{
    const line=Array.from(document.querySelectorAll('.line-hit')).find(node=>node.ariaLabel==='Select text: '+${JSON.stringify(text)});
    if(!line)return null;
    line.scrollIntoView({block:'nearest'});
    const r=line.getBoundingClientRect();return {x:r.left+(${fraction}<0?-3:Math.max(1,r.width*${fraction})),y:r.top+r.height/2};
  })()`);
  assert.ok(point,'Missing rendered line '+text);
  await send('Input.dispatchMouseEvent',{type:'mouseMoved',...point});
  for(const type of ['mousePressed','mouseReleased'])
    await send('Input.dispatchMouseEvent',{type,...point,button:'left',clickCount:1});
  assert.equal(await evaluate("document.activeElement?.matches('.pdf-typing')"),true,'Caret focus after clicking '+text);
}
async function lineRaster(label='Head of the Example Department',surrounding=true,scroll=true){
  const clip=await evaluate(`(()=>{
    const line=[...document.querySelectorAll('.line-hit')].find(node=>node.ariaLabel===${JSON.stringify('Select text: '+label)});
    if(!line)return null;
    if(${scroll})line.scrollIntoView({block:'nearest'});
    const rect=line.getBoundingClientRect(), viewport=document.querySelector('#preview-scroll').getBoundingClientRect();
    const x=Math.ceil(Math.max(viewport.left,rect.left-5)),y=Math.ceil(Math.max(viewport.top,rect.top-(${surrounding}?2:0)*rect.height));
    return {x,y,width:Math.min(Math.floor(viewport.right)-x,Math.ceil(rect.width*1.7)),
      height:Math.min(Math.floor(viewport.bottom)-y,Math.floor(rect.height*(${surrounding}?5:1))),scale:1};
  })()`);
  assert.ok(clip,'Missing PDF line '+label);
  // Exclude the cursor itself, so any change to the text underneath fails.
  await evaluate("document.querySelectorAll('.pdf-typing-selection').forEach(node=>node.style.visibility='hidden')");
  try{
    const {data}=await send('Page.captureScreenshot',{format:'png',clip});
    return data;
  }finally{await evaluate("document.querySelectorAll('.pdf-typing-selection').forEach(node=>node.style.removeProperty('visibility'))")}
}
async function checkDraftLine(line,neighbors){
  const painted="new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))";
  await evaluate(`[...document.querySelectorAll('.line-hit')].find(node=>node.ariaLabel===${JSON.stringify(
    'Select text: '+line)}).scrollIntoView({block:'center'})`);
  await evaluate(painted);
  const original=await lineRaster(line,false,false),before=[];
  for(const label of neighbors)before.push(await lineRaster(label,false,false));
  await clickLine(line,0.42);
  await type('x');
  await evaluate(painted);
  assert.equal(await evaluate("!!document.querySelector('.pdf-typing-line-draft:not([hidden])')"),true,
    'The character edit uses the original line position');
  const edited=await lineRaster(line,false,false);
  if(edited===original)console.error(await evaluate(`(()=>{
    const draft=document.querySelector('.pdf-typing-line-draft:not([hidden])');
    const line=[...document.querySelectorAll('.line-hit')].find(node=>node.ariaLabel===${JSON.stringify('Select text: '+line)});
    return {draft:draft?.getBoundingClientRect().toJSON(),line:line?.getBoundingClientRect().toJSON(),
      style:draft&&getComputedStyle(draft).display,value:document.activeElement.innerText};
  })()`));
  assert.equal(edited===original,false,'The edited letter appears before TeX finishes');
  for(const [i,label] of neighbors.entries()){
    const after=await lineRaster(label,false,false);
    if(after!==before[i]){
      await writeFile(join(work,'neighbor-before.png'),Buffer.from(before[i],'base64'));
      await writeFile(join(work,'neighbor-during.png'),Buffer.from(after,'base64'));
    }
    assert.equal(after===before[i],true,label+' stays in place while typing');
  }
  await key('Backspace',8);
  await key('Escape',27);
  await evaluate(painted);
  assert.equal((await lineRaster(line,false,false))===original,true,'Deleting the provisional letter restores the original pixels');
}
async function key(key,virtual,modifiers=0){
  for(const type of ['keyDown','keyUp'])
    await send('Input.dispatchKeyEvent',{type,key,windowsVirtualKeyCode:virtual,modifiers});
}
const type=text=>send('Input.insertText',{text});
const rendered=(...parts)=>until('Array.from(document.querySelectorAll(".line-hit")).some(node=>'+
  JSON.stringify(parts)+'.every(part=>node.ariaLabel.includes(part)))','rendered '+parts.join(' / '));
async function save(file){
  await button('save');
  await until(control('status')+".textContent==='All changes saved'",'save acknowledgement');
  return readFile(file,'utf8');
}
async function setup(source,name){
  const directory=join(work,name);await mkdir(directory);
  const file=join(directory,'main.tex');await writeFile(file,source);
  await openProject(file,name);
  return file;
}
async function openProject(file,name){
  await button('projects');await until(control('project-dialog')+'.open','Projects dialog');
  await evaluate(control('project-path')+'.value='+JSON.stringify(file));
  await button('project-open-path');
  await until('!'+control('project-dialog')+'.open && '+control('project-name')+'.textContent==='+JSON.stringify(name),'project adoption');
  await until('document.querySelector(".line-hit")!==null && !'+control('compile')+'.disabled','initial PDF');
}

async function fullReport(){
  const name='Full technical report',directory=join(work,name),main=join(directory,'main.tex');
  await cp(join(repo,'research','report'),directory,{recursive:true,filter:path=>!path.endsWith('.ps1')});
  const paths=(await readdir(directory,{recursive:true,withFileTypes:true}))
    .filter(entry=>entry.isFile()).map(entry=>join(entry.parentPath,entry.name));
  const snapshots=new Map();
  for(const path of paths)snapshots.set(path,await readFile(path,'utf8'));
  await openProject(main,name);
  const count=await evaluate("document.querySelectorAll('#pages > [data-page]').length");
  assert.ok(count>20,'The real report must exceed the previous edit limit');
  assert.equal(await evaluate(`!!document.querySelector('[data-page="${count}"] .line-hit')`),false,
    'The last page is still painted on demand');
  const observations=[];
  async function visibleTargets(page){
    await evaluate(`document.querySelector('[data-page="${page}"]').scrollIntoView({block:'start'})`);
    await until(`!!document.querySelector('[data-page="${page}"] .line-hit')`,'page '+page+' available for typing');
    return evaluate(`[...document.querySelectorAll('[data-page="${page}"] [data-inline-source]')].map(hit=>
      ({page:${page},file:hit.dataset.inlineSource,line:hit.ariaLabel.slice(13)}))`);
  }
  const early=(await visibleTargets(2)).find(item=>item.file==='sections/01-scope.tex'&&item.line.length>=50);
  assert.ok(early,'The early report page has mapped prose in its first included section');
  let late;
  for(let page=21;page<=count&&!late;page++)
    late=(await visibleTargets(page)).find(item=>item.file.startsWith('sections/')&&item.line.length>=50);
  assert.ok(late,'Find mapped text in an included file beyond page 20');
  for(const {page,file:partial,line} of [early,late]){
    await visibleTargets(page);
    const before=await lineRaster(line,false);
    await clickLine(line,0.42);
    assert.equal((await lineRaster(line,false))===before,true,'Caret leaves rendered text unchanged');
    const badge='Qzx';
    const started=performance.now();
    await type(badge);
    await waitFor(async()=>{
      const {refused,published}=await evaluate(`({
        refused:document.getElementById('selection-hint').textContent.includes('Could not verify')
          ||document.getElementById('selection-hint').textContent.includes('could not verify'),
        published:[...document.querySelectorAll('.line-hit')].some(hit=>hit.ariaLabel.includes(${JSON.stringify(badge)})
          && hit.dataset.inlineSource===${JSON.stringify(partial)})
      })`);
      if(refused)throw Error(await evaluate(control('banner')+'.textContent'));
      return published;
    },'verified edited text on page '+page,180000);
    const verifyMs=Math.round(performance.now()-started);
    const file=join(directory,partial),saved=await save(file);
    assert.equal(saved.split(badge).length,2,'One insertion was saved');
    assert.equal(saved.replace(badge,''),snapshots.get(file),'Save preserves all other source text');
    for(const [path,original] of snapshots)if(path!==file)assert.equal(await readFile(path,'utf8'),original,path+' stays untouched');
    const editedPage=await evaluate(`[...document.querySelectorAll('.line-hit')]
      .find(hit=>hit.ariaLabel.includes(${JSON.stringify(badge)})&&hit.dataset.inlineSource===${JSON.stringify(partial)})
      .closest('[data-page]').dataset.page`);
    const canvasSelector='[data-page="'+editedPage+'"] .canvasWrapper canvas';
    const rasterHash=`(async()=>{
      const canvas=document.querySelector(${JSON.stringify(canvasSelector)});
      if(!canvas)return null;
      const hash=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(canvas.toDataURL()));
      return [...new Uint8Array(hash)].join(',');
    })()`;
    const verified=await until(rasterHash,'verified edited page raster');
    await evaluate("window.previousReportShell=document.querySelector('#pages > [data-page]')");
    await button('compile');
    await until("document.querySelector('#pages > [data-page]')!==window.previousReportShell && !!document.querySelector('#pages > [data-page]')",
      'clean build publishes fresh PDF');
    await visibleTargets(Number(editedPage));
    assert.equal(await until(rasterHash,'clean edited page raster'),verified,'A fresh compile agrees with the verified edited page');
    await button('undo');
    await evaluate(`document.querySelector('[data-page="${page}"]').scrollIntoView({block:'start'})`);
    await waitFor(()=>evaluate(`[...document.querySelectorAll('[data-page="${page}"] .line-hit')]
      .some(hit=>hit.ariaLabel===${JSON.stringify('Select text: '+line)})`),'Undo refreshes page '+page,120000);
    assert.equal(await save(file),snapshots.get(file));
    observations.push({page,file:partial,line,caretKeepsPixels:true,saveAndUndo:true,cleanCompile:true,verifyMs});
    console.log('Verified editing, Save and Undo in '+partial+' from page '+page);
  }
  await openProject(main,name);
  for(const [path,original] of snapshots)assert.equal(await readFile(path,'utf8'),original);
  await writeFile(join(work,'long-report-result.json'),JSON.stringify({executable,pages:count,observations},null,2));
  console.log('Passed Windows editing checks on the complete report. Results in '+work);
}

async function verifyPageReflow(){
  const line='A short paragraph starts here.';
  const added=' This additional sentence is long enough to require another typeset line at this width.';
  const source=[
    '\\documentclass[12pt]{article}','\\usepackage[textheight=80pt,textwidth=250pt]{geometry}',
    '\\pagestyle{empty}','\\raggedbottom','\\clubpenalty=10000','\\widowpenalty=10000','\\begin{document}',
    ...Array.from({length:20},(_,index)=>'Ordinary body text on page '+(index+1)+'.\n\\newpage'),
    'The anchor stays in place.','','\\vspace{49pt}',line,'\\end{document}',''
  ].join('\n');
  const file=await setup(source,'Paragraph moves after page twenty');
  await evaluate("document.querySelector('[data-page=\"21\"]').scrollIntoView({block:'start'})");
  await rendered(line);
  await clickLine(line);
  await key('End',35);
  await type(added);
  await waitFor(async()=>{
    const {refused,moved}=await evaluate(`({
      refused:document.getElementById('selection-hint').textContent.includes('could not verify'),
      moved:[...document.querySelectorAll('[data-page="22"] .line-hit')].some(hit=>hit.ariaLabel.includes(${JSON.stringify(line)}))
    })`);
    if(refused)throw Error(await evaluate(control('banner')+'.textContent'));
    return moved;
  },'verified target moves to page 22',150000);
  assert.equal(await save(file),source.replace(line,line+'\\'+added));
  console.log('Verified a paragraph reflowing from page 21 onto a new page 22');
}

async function checkLineBoundaries(){
  const first='Delta starts the second page.',second='Echo is the next paragraph.';
  const source=['\\documentclass[12pt]{article}','\\usepackage[textheight=48pt]{geometry}',
    '\\pagestyle{empty}','\\begin{document}','Alpha opens the first page.','','Bravo is next.','',
    'Charlie ends the first page.','',first,'',second,'','Foxtrot ends the document.','\\end{document}',''].join('\r\n');
  const file=await setup(source,'Backspace after a natural page break');
  await until(`[...document.querySelectorAll('[data-page="2"] .line-hit')].some(hit=>hit.ariaLabel===${
    JSON.stringify('Select text: '+second)})`,'second paragraph on page two');
  const raster=await lineRaster(second,false);
  await clickLine(second,-1);
  const caretOn=label=>evaluate(`(()=>{
    const caret=document.querySelector('.pdf-typing-caret')?.getBoundingClientRect();
    const hit=[...document.querySelectorAll('.line-hit')].find(node=>node.ariaLabel===${JSON.stringify('Select text: '+label)})
      ?.getBoundingClientRect();
    return !!(caret&&hit&&Math.abs(caret.top-hit.top)<1&&Math.abs(caret.left-hit.left)<1);
  })()`);
  assert.equal(await caretOn(second),true,'A click just before the first glyph places the caret');
  assert.equal((await lineRaster(second,false))===raster,true,'The click preserves rendered text');
  await key('ArrowUp',38);assert.equal(await caretOn(first),true,'Up uses the previous typeset row');
  await key('ArrowDown',40);assert.equal(await caretOn(second),true,'Down returns to the next row');
  await key('Backspace',8);
  await rendered('second page.Echo');
  const joined=await save(file);
  assert.equal(joined.replaceAll('\r\n','\n'),source.replace(first+'\r\n\r\n'+second,first+second).replaceAll('\r\n','\n'));
  await button('undo');
  await until(`[...document.querySelectorAll('.line-hit')].some(hit=>hit.ariaLabel===${
    JSON.stringify('Select text: '+second)})`,'Undo publishes the separate second paragraph');
  assert.equal(await save(file),source,'Undo restores the original source bytes');

  const prose='This short text has many plain words that run past the width of the page and wrap into more rows of print.';
  const wrapSource=['\\documentclass[12pt]{article}','\\usepackage[textwidth=175pt]{geometry}',
    '\\pagestyle{empty}','\\begin{document}',prose,'\\end{document}',''].join('\n');
  const wrapFile=await setup(wrapSource,'Backspace across a soft wrap');
  const wrapped=await evaluate("[...document.querySelectorAll('.line-hit[data-inline-source]')][1]?.ariaLabel.slice(13)");
  assert.ok(wrapped,'The paragraph has an editable second row');
  await clickLine(wrapped,0.25);
  await key('Home',36);
  assert.equal(await caretOn(wrapped),true,'Home uses the typeset start of the wrapped row');
  const at=prose.indexOf(wrapped);
  assert.ok(at>0&&prose[at-1]===' ','This soft wrap follows a source space');
  await key('Backspace',8);
  const withoutSpace=prose.slice(0,at-1)+prose.slice(at);
  assert.equal(await save(wrapFile),wrapSource.replace(prose,withoutSpace),'Backspace deletes the source space at a soft wrap');
  console.log('Passed line-start caret, row navigation, wrapped-line Backspace, paragraph join and Undo.');
}
try{
  const target=await waitFor(async()=>{
    if(child.exitCode!==null)throw Error('Texloom exited before the test opened.');
    try{
      const list=await(await fetch('http://127.0.0.1:'+port+'/json/list')).json();
      return list.find(t=>t.type==='page'&&new URL(t.url).hostname==='tauri.localhost');
    }catch{return null}
  },'test WebView');
  socket=new WebSocket(target.webSocketDebuggerUrl);await once(socket,'open');
  socket.addEventListener('message',({data})=>{
    const message=JSON.parse(data),call=requests.get(message.id);if(!call)return;
    requests.delete(message.id);
    if(message.error)call.reject(Error(message.error.message));else call.resolve(message.result);
  });
  await until('document.querySelector(".line-hit")!==null','startup page');
  if(process.argv.includes('--line-boundaries')){
    await checkLineBoundaries();
  }else if(process.argv.includes('--full-report')){
    await fullReport();
    await verifyPageReflow();
  }else{
  await checkLineBoundaries();
  const sample=await readFile(join(repo,'resources','main.tex'),'utf8');
  const file=await setup(sample,'Macro signature');
  await evaluate(`window.caretTimes=[];document.addEventListener('pointerdown',()=>{window.pressTime=performance.now()},true);
    document.addEventListener('focusin',e=>{if(e.target.matches('.pdf-typing'))window.caretTimes.push(performance.now()-window.pressTime)},true)`);
  const last='The paragraph after the signature must stay in the same place.';
  await clickLine('Dr. Example Researcher');await key('End',35);await type(' II');
  await until(control('compile-state')+".textContent.includes('Compiling')",'first verification in flight');
  await clickLine(last);assert.equal(await evaluate('document.activeElement.innerText'),last);
  await key('End',35);await type(' Extra.');
  await rendered('Researcher II');await rendered('place. Extra.');
  let source=await save(file);
  assert.ok(source.includes('\\textbf{Dr. Example Researcher II}'));
  assert.ok(source.includes('\\newcommand{\\PersonName}{Dr. Example Researcher}'));
  assert.match(source,/same place\.(?:\\ )? ?Extra\./);
  await clickLine('Head of the Example Department');await key('Home',36);await key('Backspace',8);
  await rendered('Researcher II','Head of');
  source=await save(file);assert.match(source,/Researcher II\}Head of/);
  await clickLine('An ordinary paragraph before the signature. It remains in normal flow.');
  await key('Home',36);await key('Backspace',8);
  await rendered('heading.An ordinary');
  source=await save(file);assert.match(source,/Unchanged heading\.An ordinary/);
  await button('undo');
  await until("Array.from(document.querySelectorAll('.line-hit')).some(e=>e.ariaLabel==='Select text: Unchanged heading.')",'undo restored the separate first paragraph');
  source=await save(file);assert.match(source,/Unchanged heading\.\r?\n\r?\nAn ordinary/);
  const literal=await setup(sample.replace('\\textbf{\\PersonName}','\\textbf{Dr. Example Research}'),'Literal signature');
  const before=await lineRaster();
  await clickLine('Head of the Example Department',0.42);
  const after=await lineRaster();
  if(before!==after){
    await writeFile(join(work,'signature-before.png'),Buffer.from(before,'base64'));
    await writeFile(join(work,'signature-after.png'),Buffer.from(after,'base64'));
    assert.fail('Placing the caret moved or obscured text in the signature: '+work);
  }
  const caret=await until(`(()=>{
    const caret=document.querySelector('.pdf-typing-caret');
    if(!caret)return null;
    const hit=[...document.querySelectorAll('.line-hit')].find(node=>node.ariaLabel==='Select text: Head of the Example Department');
    const box=hit.getBoundingClientRect(), rect=caret.getBoundingClientRect();
    return {x:rect.left,y:rect.top,lineX:box.left,lineY:box.top,width:box.width};
  })()`,'visible caret over the unchanged PDF');
  assert.ok(Math.abs(caret.y-caret.lineY)<1,'Caret is on the department line');
  assert.ok(Math.abs(caret.x-(caret.lineX+0.42*caret.width))<6,'Caret is at the clicked word');
  await key('ArrowRight',39);
  await until("document.querySelector('.pdf-typing-caret')?.getBoundingClientRect().left>"+caret.x,'caret advances along the department line');
  await key('ArrowRight',39,8);
  await until("document.querySelector('.pdf-typing-range')!==null",'keyboard range on original text');
  await key('Escape',27);
  assert.equal(await evaluate("document.querySelector('.pdf-typing-caret')"),null);
  assert.equal(await readFile(literal,'utf8'),sample.replace('\\textbf{\\PersonName}','\\textbf{Dr. Example Research}'));
  await checkDraftLine('Head of the Example Department',['Dr. Example Research','Example University']);
  assert.equal(await lineRaster(),before);
  await clickLine('Dr. Example Research');await key('End',35);await key('Tab',9);await type('A');
  assert.ok((await evaluate('document.activeElement.innerText')).includes('Research\tA'));
  source=await save(literal);
  assert.ok(source.includes('\\textbf{Dr. Example Research\\mbox{\\ \\ \\ \\ }A}'));
  await until("Array.from(document.querySelectorAll('.line-hit'),e=>e.ariaLabel.slice(13)).join(' ').replace(/\\s+/gu,' ').includes('Dr. Example Research A')",'typeset tab and following letter');

  const report=await readFile(join(repo,'research','report','main.tex'),'utf8');
  const section=await readFile(join(repo,'research','report','sections','01-scope.tex'),'utf8');
  const documentStart='\\begin{document}';
  const preamble=report.slice(0,report.indexOf(documentStart)+documentStart.length);
  const first=section.slice(0,section.indexOf('\\begin{quote}'));
  const second=section.slice(section.indexOf('Three boundaries must be distinguished.'),
    section.indexOf('\\subsection{Purpose of this report}'));
  const excerpt=preamble+'\n'+first+'\n\\clearpage\n'+second+'\n\\end{document}\n';
  const reportFile=await setup(excerpt,'Technical report excerpt');
  const wrapping=await evaluate(`(()=>{
    const rows=[...document.querySelectorAll('[data-page="2"] .line-hit')].map(node=>node.ariaLabel.slice(13));
    const start=rows.findIndex(text=>text.startsWith('Three boundaries must be distinguished.'));
    return start<0?[]:rows.slice(start,start+3);
  })()`);
  assert.equal(wrapping.length,3,'A wrapped report paragraph is visible on page two');
  await checkDraftLine(wrapping[1],[wrapping[0],wrapping[2]]);
  assert.equal(await readFile(reportFile,'utf8'),excerpt);

  const timings=await evaluate('window.caretTimes');
  assert.ok(timings.length>=4);
  const result={executable,fixtures:[basename(dirname(file)),basename(dirname(literal)),basename(dirname(reportFile))],
    tests:['type into macro use','switch passages during verification','join hard line','join paragraphs','Undo and Save',
      'unchanged Windows raster on signature click','unchanged neighboring lines while typing',
      'literal bold name and Tab','unchanged wrapped report lines on page two'],
    pointerDownToFocusMs:timings};
  await writeFile(join(work,'result.json'),JSON.stringify(result,null,2));
  console.log('Passed Windows typing checks. Results in '+work);
  }
}catch(error){
  if(socket?.readyState===1){
    const status=await evaluate("['status','selection-hint','banner','toast'].map(id=>document.getElementById(id)?.textContent)");
    console.error(status,await evaluate("Array.from(document.querySelectorAll('.line-hit')).slice(0,12).map(e=>e.ariaLabel)"));
  }
  throw error;
}finally{
  socket?.close();
  if(child.pid&&child.exitCode===null)child.kill();
}
