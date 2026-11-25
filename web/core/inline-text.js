function plainParagraph(text){
  return text.trim()===text&&!text.includes('  ')&&!text.includes('--')&&/^[A-Za-z0-9 .,!?;:-]+$/u.test(text);
}

export function inlineTextSupported(text){
  const paragraphs=text.split('\n');
  return text.length<=4096&&paragraphs.length<=24&&paragraphs.every(plainParagraph);
}

export function inlineSourceText(source){
  const paragraphs=source.split(/\r?\n\r?\n/u);
  const text=paragraphs.join('\n');
  return paragraphs.every(plainParagraph)&&inlineTextSupported(text)?text:null;
}

export function nearbyInlineLines(lines,clicked){
  const index=lines.indexOf(clicked);
  if(index<0||!plainParagraph(clicked.text))return [clicked];
  const font=clicked.runs[0];
  const compatible=(line,adjacent)=>plainParagraph(line.text)&&line.runs.every(run=>run.font===font.font&&Math.abs(run.size-font.size)<0.1)
    &&Math.abs(line.x-clicked.x)<=font.size*2.5&&Math.abs(line.baseline-adjacent.baseline)<=font.size*2.2;
  let start=index,end=index+1;
  while(start>0&&end-start<24&&compatible(lines[start-1],lines[start]))start--;
  while(end<lines.length&&end-start<24&&compatible(lines[end],lines[end-1]))end++;
  return lines.slice(start,end);
}

export function inlineRows(lines,text){
  let start=0;
  const rows=[];
  for(const line of lines){
    const end=start+line.text.length;
    if(!plainParagraph(line.text)||!text.startsWith(line.text,start))return null;
    rows.push({line,start,end});
    if(end===text.length)return rows;
    if(![' ','\n'].includes(text[end]))return null;
    start=end+1;
  }
  return null;
}

export function findInlineFlow(view,{page,line,text}){
  const target=view.pages.find(candidate=>candidate.number===page);
  const start=target?.lines.findIndex(candidate=>Math.abs(candidate.x-line.x)<1&&Math.abs(candidate.baseline-line.baseline)<1)??-1;
  const rows=start<0?null:inlineRows(target.lines.slice(start),text);
  if(!rows)throw Error('The edited text does not fit in the verified area on this page. Shorten it or use the source editor.');
  return {page:target,rows};
}

export function caretOffset(stops,x){
  if(!Number.isFinite(x)||x<stops[0]-2||x>stops.at(-1)+2)return null;
  const right=stops.findIndex(stop=>stop>x);
  if(right<0)return stops.length-1;
  if(right===0)return 0;
  return x-stops[right-1]<=stops[right]-x?right-1:right;
}
