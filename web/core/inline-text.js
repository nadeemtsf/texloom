const TAB_SOURCE=String.raw`\mbox{\ \ \ \ }`;
const horizontal=character=>character===' '||character==='\t';
function plainParagraph(text){
  return !!text.trim()&&!text.includes('--')&&/^[A-Za-z0-9 \t.,!?;:-]+$/u.test(text);
}

export function inlineTextSupported(text){
  const paragraphs=text.split('\n');
  return text.length<=4096&&paragraphs.length<=24&&paragraphs.every(plainParagraph);
}

export function inlineSourceText(source){
  const text=source.replaceAll('\r\n','\n').split(/\n[ \t]*\n/u).map(paragraph=>
    paragraph.replace(/\n[ \t]*/gu,' ').replaceAll('\t',' ').replaceAll(TAB_SOURCE,'\t').replaceAll('\\ ',' ')
  ).join('\n');
  return inlineTextSupported(text)?text:null;
}

export function inlineSourceReplacement(text,separator){
  return text.split('\n').map(paragraph=>[...paragraph].map((character,index)=>{
    if(character==='\t')return TAB_SOURCE;
    if(character===' '&&(index===0||index+1===paragraph.length||paragraph[index-1]===' '))return '\\ ';
    return character;
  }).join('')).join(separator);
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
  let index=0;
  const rows=[];
  for(const line of lines){
    if(!plainParagraph(line.text))return null;
    while(horizontal(text[index])&&!horizontal(line.text[0]))index++;
    const start=index,offsets=[start];
    for(let i=0;i<line.text.length;i++){
      const character=line.text[i];
      if(horizontal(character)){
        if(!horizontal(text[index])&&!horizontal(line.text[i-1]))return null;
        while(horizontal(text[index]))index++;
      }else if(text[index++]!==character)return null;
      offsets.push(index);
    }
    rows.push({line,start,end:index,offsets});
    if(index===text.length)return rows;
    if(text[index]!=='\n'&&!horizontal(text[index]))return null;
    while(horizontal(text[index]))index++;
    if(text[index]==='\n')index++;
    if(index===text.length)return rows;
  }
  return null;
}

export function findInlineFlow(view,{page,line,text}){
  const target=view.pages.find(candidate=>candidate.number===page);
  const start=target?.lines.findIndex(candidate=>Math.abs(candidate.x-line.x)<=line.runs[0].size*3&&Math.abs(candidate.baseline-line.baseline)<1)??-1;
  const rows=start<0?null:inlineRows(target.lines.slice(start),text);
  if(!rows)throw Error('The edited text does not fit in the verified area on this page. Shorten it or use the source editor.');
  return {page:target,rows};
}

export function measuredCaretStops(line,measure){
  if(measure){
    const font=line.runs[0],family=font.font?JSON.stringify(font.font)+', Georgia, serif':'Georgia, serif';
    measure.font=font.size+'px '+family;
  }
  const advance=text=>measure?.measureText(text).width??text.length;
  const length=advance(line.text)||1;
  return Array.from({length:line.text.length+1},(_,i)=>line.x+line.width*advance(line.text.slice(0,i))/length);
}

export function documentBody(source){
  const start=/^\\begin\{document\}[ \t]*\r?$/mu.exec(source);
  if(!start)return null;
  const end=source.indexOf('\\end{document}',start.index+start[0].length);
  if(end<0)return null;
  const bodyStart=start.index+start[0].length;
  const raw=source.slice(bodyStart,end);
  const content=raw.trim();
  return {content,start:bodyStart+raw.length-raw.trimStart().length};
}

export function caretOffset(stops,x){
  if(!Number.isFinite(x)||x<stops[0]-2||x>stops.at(-1)+2)return null;
  const right=stops.findIndex(stop=>stop>x);
  if(right<0)return stops.length-1;
  if(right===0)return 0;
  return x-stops[right-1]<=stops[right]-x?right-1:right;
}
