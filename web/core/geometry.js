/** Geometry is expressed in PDF points with a top-left origin. */
export function union(rectangles) {
  if (!rectangles.length) return null;
  const x = Math.min(...rectangles.map(r => r.x));
  const y = Math.min(...rectangles.map(r => r.y));
  return {x,y,width:Math.max(...rectangles.map(r=>r.x+r.width))-x,height:Math.max(...rectangles.map(r=>r.y+r.height))-y};
}
export function intersects(a,b,padding=0) {
  return a.x<=b.x+b.width+padding && a.x+a.width>=b.x-padding && a.y<=b.y+b.height+padding && a.y+a.height>=b.y-padding;
}
export function inside(rect,page,padding=0.5) {
  return rect && [rect.x,rect.y,rect.width,rect.height].every(Number.isFinite) && rect.width>0 && rect.height>0 && rect.x>=-padding && rect.y>=-padding && rect.x+rect.width<=page.width+padding && rect.y+rect.height<=page.height+padding;
}
export function normalizedRectangle(a,b) {return {x:Math.min(a.x,b.x),y:Math.min(a.y,b.y),width:Math.abs(b.x-a.x),height:Math.abs(b.y-a.y)}}

/** Preserve columns and separate fragments interrupted by a large horizontal gap. */
export function groupTextRuns(runs,page) {
  const lines=[];
  for(const run of [...runs].sort((a,b)=>a.y-b.y||a.x-b.x)) {
    if(!run.text.trim()||run.width<=0||run.height<=0)continue;
    const line=lines.find(l=>Math.abs(l.baseline-run.baseline)<=Math.max(1,Math.min(l.height,run.height)*0.25) && run.x<=l.x+l.width+Math.max(12,run.height*1.5) && run.x+run.width>=l.x-2);
    if(line){const bounds=union([line,run]);Object.assign(line,bounds);line.runs.push(run);line.text=line.runs.sort((a,b)=>a.x-b.x).map(r=>r.text).join(' ')}
    else lines.push({...run,page,runs:[run]});
  }
  return lines.map((l,index)=>({...l,id:`p${page}-line${index}`}));
}

/** Read source object identities from PDF marked-content, including nested tags. */
export function extractTextGeometry(items,styles,page,pageHeight) {
  const stack=[],runs=[],objects=new Map(),unclosed=new Set();
  for(const item of items){
    if(item.type==='beginMarkedContent'||item.type==='beginMarkedContentProps'){
      const id=item.tag?.startsWith('Texloom_')?item.tag.slice(8):null;
      stack.push(id);if(id)unclosed.add(id);continue;
    }
    if(item.type==='endMarkedContent'){const id=stack.pop();if(id)unclosed.delete(id);continue}
    if(typeof item.str!=='string'||!item.str.trim())continue;
    const [a,b,c,d,e,f]=item.transform;
    if(Math.abs(b)>0.01||Math.abs(c)>0.01)continue; // Rotated text needs its own adapter.
    const height=Math.hypot(c,d),ascent=styles[item.fontName]?.ascent??0.8;
    const run={text:item.str,x:e,y:pageHeight-f-height*ascent,width:Math.abs(item.width),height,baseline:pageHeight-f,font:item.fontName,size:height,page};
    runs.push(run);
    for(const id of stack.filter(Boolean)){if(!objects.has(id))objects.set(id,[]);objects.get(id).push(run)}
  }
  return {runs,lines:groupTextRuns(runs,page),objects:new Map([...objects].map(([id,r])=>[id,{...union(r),page,runs:r}])),unclosed};
}

export function linesForSourceRectangles(lines,rectangles,page) {
  return lines.filter(line=>rectangles.some(r=>r.page===page && intersects(line,r,1.5)));
}

export function validateObjectLayout(prepared,policy,previousPageCount) {
  if(!policy)return;
  if(policy.kind==='move'&&prepared.pageCount!==previousPageCount)throw new Error('The move changed the number of pages. The original document was preserved.');
  if(policy.kind==='detach')return;
  const object=prepared.objects.get(policy.id);
  if(!object)throw new Error('The selected text did not produce a verifiable object in the compiled PDF.');
  if(object.pages.size!==1||prepared.unclosed.has(policy.id))throw new Error('This text crosses a page boundary. Move a smaller group.');
  const page=prepared.pageSizes.get(object.page);
  if(!inside(object,page))throw new Error('That position would place text outside the page. The original document was preserved.');
  if(policy.kind==='move'&&policy.unchangedRuns){
    const moved=new Set(object.runs);
    const remaining=prepared.pages.flatMap(p=>p.runs).filter(run=>!moved.has(run));
    const key=run=>`${run.page}:${run.text}:${run.x.toFixed(1)}:${run.y.toFixed(1)}:${run.width.toFixed(1)}:${run.size.toFixed(1)}`;
    const before=policy.unchangedRuns.map(key).sort(),after=remaining.map(key).sort();
    if(before.length!==after.length||before.some((value,index)=>value!==after[index]))
      throw new Error('This move changed surrounding text or layout. The original source and PDF were preserved.');
  }
}
