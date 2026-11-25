function replacement(before,after) {
  let start=0, oldEnd=before.length, end=after.length;
  while(start<oldEnd&&start<end&&before[start]===after[start])start++;
  while(oldEnd>start&&end>start&&before[oldEnd-1]===after[end-1]){oldEnd--;end--}
  return {start,oldEnd,text:after.slice(start,end)};
}

// Keep ordinary character edits on the verified line. Breaks, tabs and edits
// that cannot fit use the browser's normal multiline draft.
export function paintInlineDraft(canvas,page,entry,value,scale,preferredRow) {
  const edit=replacement(entry.text,value);
  if(/[\r\n\t]/u.test(edit.text))return null;
  const contains=row=>row.start<=edit.start&&edit.oldEnd<=row.end;
  const row=entry.rows.find(row=>row.line===preferredRow?.line&&contains(row))||entry.rows.find(contains);
  if(!row)return null;
  const line=row.line, context=canvas.getContext('2d');
  if(!context)return null;
  const runs=[];
  let column=0;
  for(const [index,run] of line.runs.entries()){
    if(index)column++;
    const start=row.offsets[column];
    column+=(run.text??line.text).length;
    const end=row.offsets[column], text=entry.text.slice(start,end);
    if(!text||/[\t\n]/u.test(text))return null;
    runs.push({run,start,end,text,font:run.size+'px '+(run.font?JSON.stringify(run.font)+', serif':'serif')});
  }
  const affected=runs.find(item=>item.start<=edit.start&&edit.oldEnd<=item.end);
  if(!affected)return null;

  // Use the embedded fonts and preserve the PDF's horizontal scale per run.
  // Never position other rows using this draft's browser line boxes.
  const delta=value.length-entry.text.length;
  let advance=0, sourceDelta=0, right=line.x+line.width;
  for(const item of runs){
    const {run,font,text}=item;
    context.font=font;
    const measured=context.measureText(text).width;
    if(!measured||!Number.isFinite(measured))return null;
    item.ratio=run.width/measured;
    item.x=run.x+advance;
    item.offset=item.start+sourceDelta;
    item.draft=text;
    if(item===affected){
      item.draft=text.slice(0,edit.start-item.start)+edit.text+text.slice(edit.oldEnd-item.start);
      advance+=context.measureText(item.draft).width*item.ratio-run.width;
      sourceDelta=delta;
    }
    right=Math.max(right,item.x+context.measureText(item.draft).width*item.ratio);
  }
  const left=Math.max(0,line.x-0.5), top=Math.max(0,line.y-0.5);
  const bottom=line.y+line.height+0.5, edge=Number(page.shell.dataset.pdfWidth);
  const width=right+0.5-left, height=bottom-top;
  const overlaps=other=>other!==line&&other.x<right+0.5&&other.x+other.width>left&&
    other.y<bottom&&other.y+other.height>top;
  if(!Number.isFinite(edge)||right+0.5>edge||page.lines.some(overlaps))return null;

  const ratio=globalThis.devicePixelRatio||1;
  const pixelsPerPoint=ratio*scale;
  canvas.width=Math.ceil(width*pixelsPerPoint);
  canvas.height=Math.ceil(height*pixelsPerPoint);
  Object.assign(canvas.style,{left:left*scale+'px',top:top*scale+'px',width:width*scale+'px',height:height*scale+'px'});
  context.setTransform(pixelsPerPoint,0,0,pixelsPerPoint,-left*pixelsPerPoint,-top*pixelsPerPoint);
  context.fillStyle='#fff';
  context.fillRect(left,top,width,height);
  context.fillStyle='#111';
  context.textBaseline='alphabetic';
  const offsets=[],stops=[];
  for(const item of runs){
    context.font=item.font;
    context.save();
    context.translate(item.x,item.run.baseline);
    context.scale(item.ratio,1);
    context.fillText(item.draft,0,0);
    context.restore();
    for(let index=0;index<=item.draft.length;index++){
      offsets.push(item.offset+index);
      stops.push(item.x+context.measureText(item.draft.slice(0,index)).width*item.ratio);
    }
  }
  return entry.rows.map(item=>{
    if(item===row)return {...item,end:item.end+delta,offsets,stops};
    if(item.start<row.end)return item;
    return {...item,start:item.start+delta,end:item.end+delta,offsets:item.offsets.map(offset=>offset+delta)};
  });
}
