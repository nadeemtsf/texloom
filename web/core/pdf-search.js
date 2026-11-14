/** Compare shaped PDF letters and ligatures with ordinary typed Unicode. */
export const pdfSearchKey=value=>value.normalize('NFKC').toLocaleLowerCase();
const graphemes=new Intl.Segmenter(undefined,{granularity:'grapheme'});
/** Return original UTF-16 boundaries, even when normalization expands a glyph. */
export function pdfSearchRange(value,query,occurrence=0){
  const key=pdfSearchKey(value),needle=pdfSearchKey(query);if(!needle)return null;
  let at=-needle.length;for(let n=0;n<=occurrence;n++){at=key.indexOf(needle,at+needle.length);if(at<0)return null;}
  let offset=0,start;
  for(const {segment,index} of graphemes.segment(value)){
    const end=offset+pdfSearchKey(segment).length;
    if(start===undefined&&at<end)start=index;
    if(start!==undefined&&at+needle.length<=end)return [start,index+segment.length];
    offset=end;
  }
  return null;
}
/** Search original PDF text, including pages not currently painted. */
export class PDFSearch {
  constructor({document,getView,notify}){
    Object.assign(this,{document,getView,notify});this.serial=0;this.navigation=0;this.matches=[];this.index=-1;
    const $=id=>document.getElementById(id);this.input=$('pdf-query');this.count=$('pdf-matches');
    this.input.oninput=()=>{clearTimeout(this.timer);this.reset();this.timer=setTimeout(()=>this.find(),250)};
    this.input.onkeydown=e=>{if(e.key==='Enter'){e.preventDefault();this.step(e.shiftKey?-1:1)}else if(e.key==='Escape'){this.input.value='';this.reset()}};
    $('pdf-find-next').onclick=()=>this.step(1);$('pdf-find-prev').onclick=()=>this.step(-1);
  }
  reset(){this.serial++;this.navigation++;this.matches=[];this.index=-1;this.count.textContent='';globalThis.CSS?.highlights?.delete('texloom-search');this.document.querySelectorAll('.pdf-search-match').forEach(e=>e.classList.remove('pdf-search-match'));}
  async find(){
    clearTimeout(this.timer);this.reset();const query=pdfSearchKey(this.input.value.trim()),view=this.getView(),serial=this.serial;if(!query||!view)return;
    if(query.length>256){this.count.textContent='Use at most 256 characters';return;}
    this.searchedQuery=query;
    this.count.textContent='Searching…';const matches=[];
    try{let characters=0;
      for(let n=1;n<=view.pageCount;n++){
        const page=await view.pdf.getPage(n),text=await page.getTextContent();if(serial!==this.serial||view!==this.getView())return;
        const value=pdfSearchKey(text.items.filter(i=>typeof i.str==='string').map(i=>i.str).join(' '));characters+=value.length;
        if(characters>8_000_000)throw Error('Search exceeds the eight-million-character limit. Narrow the project.');
        let at=0,occurrence=0;while((at=value.indexOf(query,at))>=0){matches.push({page:n,offset:at,occurrence:occurrence++});at+=query.length;if(matches.length>=10000)break;}if(matches.length>=10000)break;
      }
      if(serial!==this.serial||view!==this.getView())return;this.matches=matches;this.index=-1;
      if(matches.length)await this.step(1);else this.count.textContent='No matches';
    }catch(error){if(serial===this.serial){this.count.textContent='Search unavailable';this.notify(String(error?.message||error))}}
  }
  async step(direction){
    if(!this.matches.length||this.searchedQuery!==pdfSearchKey(this.input.value.trim()))return this.find();
    const serial=this.serial,navigation=++this.navigation,view=this.getView();this.index=(this.index+direction+this.matches.length)%this.matches.length;
    const index=this.index,match=this.matches[index];
    const current=()=>serial===this.serial&&navigation===this.navigation&&view===this.getView();
    try{await view.ensurePage?.(match.page);if(!current())return;
      this.document.querySelectorAll('.pdf-search-match').forEach(e=>e.classList.remove('pdf-search-match'));
      const page=view.pages[match.page-1];page.shell.classList.add('pdf-search-match');page.shell.scrollIntoView({block:'center'});
      this.highlight(page,match.occurrence);
      this.count.textContent=(index+1)+' / '+this.matches.length;
    }catch(error){if(current())this.notify(String(error?.message||error))}
  }
  highlight(page,occurrence){
    globalThis.CSS?.highlights?.delete('texloom-search');
    const spans=[...page.shell.querySelectorAll('.textLayer span')].filter(span=>span.firstChild?.nodeType===3);
    const bounds=pdfSearchRange(spans.map(s=>s.textContent).join(' '),this.input.value.trim(),occurrence);if(!bounds)return;
    const find=offset=>{for(const span of spans){if(offset<=span.textContent.length)return [span,Math.min(offset,span.firstChild.length)];offset-=span.textContent.length+1;}return null};
    const start=find(bounds[0]),end=find(bounds[1]);if(!start||!end)return;
    if(globalThis.CSS?.highlights&&globalThis.Highlight){const range=this.document.createRange();range.setStart(start[0].firstChild,start[1]);range.setEnd(end[0].firstChild,end[1]);CSS.highlights.set('texloom-search',new Highlight(range));}
    start[0].scrollIntoView({block:'center'});
  }
}
