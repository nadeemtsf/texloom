/** Run with Node 22.13+ and the path to a packaged Windows Texloom.exe.
 * Uses normal WebView pointer and keyboard events on disposable project copies. */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {mkdir,mkdtemp,readFile,writeFile} from 'node:fs/promises';
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
    const r=line.getBoundingClientRect();return {x:r.left+Math.max(1,r.width*${fraction}),y:r.top+r.height/2};
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
  await button('projects');await until(control('project-dialog')+'.open','Projects dialog');
  await evaluate(control('project-path')+'.value='+JSON.stringify(file));
  await button('project-open-path');
  await until('!'+control('project-dialog')+'.open && '+control('project-name')+'.textContent==='+JSON.stringify(name),'project adoption');
  await until('document.querySelector(".line-hit")!==null && !'+control('compile')+'.disabled','initial PDF');
  return file;
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
}catch(error){
  if(socket?.readyState===1){
    const status=await evaluate("['status','selection-hint','banner','toast'].map(id=>document.getElementById(id)?.textContent)");
    console.error(status,await evaluate("Array.from(document.querySelectorAll('.line-hit'),e=>e.ariaLabel)"));
  }
  throw error;
}finally{
  socket?.close();
  if(child.pid&&child.exitCode===null)child.kill();
}
