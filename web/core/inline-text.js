// One literal ASCII line keeps PDF caret and source offsets in the same index
// space. The native verifier also checks the marked glyphs.
export function inlineTextSupported(text){
  return text.length>0&&text.length<=160&&text.trim()===text&&!text.includes('  ')&&!text.includes('--')&&/^[A-Za-z0-9 .,!?;:-]+$/u.test(text);
}

export function findInlineLine(view,{page,line,text}){
  const target=view.pages.find(candidate=>candidate.number===page);
  const matches=target?.lines.filter(candidate=>candidate.text===text
    &&Math.abs(candidate.x-line.x)<1&&Math.abs(candidate.baseline-line.baseline)<1);
  if(!matches||matches.length!==1)throw Error('The edit does not fit on the same PDF line. Edit it in the source editor.');
  return {page:target,line:matches[0]};
}

export function caretOffset(stops,x){
  if(!Number.isFinite(x)||x<stops[0]-2||x>stops.at(-1)+2)return null;
  const right=stops.findIndex(stop=>stop>x);
  if(right<0)return stops.length-1;
  if(right===0)return 0;
  return x-stops[right-1]<=stops[right]-x?right-1:right;
}
