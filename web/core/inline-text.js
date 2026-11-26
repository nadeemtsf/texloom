// Map the text that TeX actually painted to UTF-16 positions in the native
// source projection. Each offset is a caret boundary, including soft wraps.
const space = character => character === ' ' || character === '\t';
const whitespace = character => space(character) || character === '\n';
const ligatures = {'ﬀ':'ff','ﬁ':'fi','ﬂ':'fl','ﬃ':'ffi','ﬄ':'ffl'};

// Native ranges use the editor's normalized offsets. The expected bytes must
// still match the original source, including a BOM and CRLF line endings.
function sourceReader(source) {
  const bom=Number(source.startsWith('\uFEFF'));
  const breaks=Array.from(source.matchAll(/\r\n/gu),(match,i)=>match.index-bom-i);
  const raw=offset=>{
    let lo=0,hi=breaks.length;
    while(lo<hi){const mid=(lo+hi)>>>1;if(breaks[mid]<offset)lo=mid+1;else hi=mid}
    return bom+offset+lo;
  };
  return (start,end)=>source.slice(raw(start),raw(end));
}

export function inlineRows(lines, text, first = 0) {
  let index = 0;
  const rows = [];
  for (let n = first; n < lines.length; n++) {
    const line = lines[n];
    if (!line.text) return null;
    while (whitespace(text[index])) index++;
    const start = index, offsets = [start];
    for (let col = 0; col < line.text.length; col++) {
      const char = line.text[col], literal = ligatures[char] || char;
      if (space(char)) {
        // PDF.js sometimes splits a word at a font change. The geometry
        // groups the runs with a synthetic space; it has no source offset.
        while (space(text[index])) index++;
      } else if ((char==='’'&&text[index]==="'")||(char==='‘'&&text[index]==='\u0060')) {
        index++;
      } else if (text.startsWith(literal,index)) {
        index += literal.length;
      } else if (!(char === '-' && col === line.text.length-1 && n+1 < lines.length
        && /\p{L}/u.test(text[index] || ' '))) return null;
      offsets.push(index);
    }
    rows.push({line,start,end:index,offsets});
    let next = index;
    while (whitespace(text[next])) next++;
    if (next === text.length) return rows;
    if (next === index && !line.text.endsWith('-')) return null;
  }
  return null;
}

export function buildInlineIndex(view, groups, sources, measure=null) {
  const byLine = new Map(), bySource = new Map(), entries=[];
  const lines = (view.pages || []).flatMap(page=>
    ('inlineReason' in page?page.inlineReason:page.selectionReason)?[]:page.lines);
  const byInitial = new Map();
  for (const [index,line] of lines.entries()) {
    const initial = (ligatures[line.text?.[0]] || line.text || '').trimStart()[0];
    if (!byInitial.has(initial)) byInitial.set(initial,[]);
    byInitial.get(initial).push(index);
  }
  const match=flow=>{
    const result = [];
    for (const n of byInitial.get(flow.text.trimStart()[0]) || []) {
      const rows = inlineRows(lines,flow.text,n);
      if (rows && rows.every(row=>row.line.page===lines[n].page)) result.push({n,end:n+rows.length,rows});
    }
    return result;
  };
  const candidates=groups.flatMap(({file,primary,paragraphs})=>{
    const options=match(primary);
    if(options.length)return [{file,flow:primary,options}];
    return paragraphs.map(flow=>({file,flow,primary,options:match(flow)}));
  });
  const matches=candidates.map(item=>item.options);
  // Reading order resolves repeated occurrences when the entire sequence has
  // a unique alignment. Keep ambiguous rendered copies out of the index.
  const earliest=[],latest=[];
  let lower=0,upper=lines.length;
  for (const options of matches) {
    const first=options.find(m=>m.n>=lower);
    earliest.push(first);
    if(first) lower=first.end;
  }
  for (let i=matches.length-1;i>=0;i--) {
    const last=matches[i].findLast(m=>m.end<=upper);
    latest[i]=last;
    if(last) upper=last.n;
  }
  const slices=new Map();
  for (const [i,{file,flow,primary}] of candidates.entries()) {
    if (!earliest[i] || earliest[i]!==latest[i]) continue;
    const {rows}=earliest[i];
    const entry={file,kind:'source',start:flow.range[0],end:flow.range[1],content:flow.expected,
      expected_source:sources[file],text:flow.text,rows,page:rows[0].line.page};
    // A page break can split a source flow, but adjacent paragraphs on either
    // page still share an editable passage when both mappings are unique.
    const previous=entries.at(-1);
    if(primary&&previous&&candidates[i-1]?.primary===primary&&earliest[i-1]?.end===earliest[i].n&&
        previous.file===file&&previous.page===entry.page){
      if(!slices.has(file))slices.set(file,sourceReader(sources[file]));
      const source=slices.get(file),gap=source(previous.end,entry.start);
      if(/^[\t \r\n]*$/u.test(gap)&&gap.split('\n').length>=3){
        const shift=previous.text.length+1;
        previous.text+='\n'+entry.text;
        previous.end=entry.end;
        previous.content=source(previous.start,entry.end);
        previous.rows.push(...rows.map(row=>({...row,start:row.start+shift,end:row.end+shift,
          offsets:row.offsets.map(offset=>offset+shift)})));
        continue;
      }
    }
    entries.push(entry);
  }
  for(const entry of entries){
    const {file,rows}=entry;
    if(!bySource.has(file))bySource.set(file,new Map());
    bySource.get(file).set(entry.start,entry);
    for(const row of rows) {
      row.stops=measuredCaretStops(row.line,measure);
      byLine.set(row.line,{entry,row});
    }
  }
  return {byLine,bySource,entries};
}

export function findInlineFlow(view,policy) {
  const entry=view.inlineIndex?.bySource.get(policy.file)?.get(policy.start);
  if(entry && entry.text===policy.text)
    return {page:view.pages.find(p=>p.number===entry.page),rows:entry.rows,entry};
  throw Error('The typeset result could not be matched to the edited passage.');
}

export function measuredCaretStops(line,measure) {
  // Use the individual PDF font and width of each run, including mixed style.
  const stops=[];
  for(const run of line.runs || [line]) {
    const text=run.text??line.text,x=run.x??line.x,width=run.width??line.width;
    stops.push(x);
    if(measure) measure.font=run.size+'px '+(run.font?JSON.stringify(run.font)+', serif':'serif');
    const advance=s=>measure?.measureText(s).width??s.length,total=advance(text)||1;
    for(let i=1;i<=text.length;i++) stops.push(x+width*advance(text.slice(0,i))/total);
  }
  return stops;
}

export function caretOffset(stops,x) {
  if(!Number.isFinite(x)||!stops.length)return null;
  const right=stops.findIndex(stop=>stop>x);
  if(right<0)return stops.length-1;
  if(right===0)return 0;
  return x-stops[right-1]<=stops[right]-x?right-1:right;
}
