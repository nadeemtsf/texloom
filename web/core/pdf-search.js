/** Search original PDF text, including pages not currently painted. */
export class PDFSearch {
  constructor({document,getView,notify}){
    Object.assign(this,{document,getView,notify});this.serial=0;this.matches=[];this.index=-1;
    const $=id=>document.getElementById(id);this.input=$('pdf-query');this.count=$('pdf-matches');
    this.input.oninput=()=>{clearTimeout(this.timer);this.reset();this.timer=setTimeout(()=>this.find(),250)};
    this.input.onkeydown=e=>{if(e.key==='Enter'){e.preventDefault();this.step(e.shiftKey?-1:1)}else if(e.key==='Escape'){this.input.value='';this.reset()}};
    $('pdf-find-next').onclick=()=>this.step(1);$('pdf-find-prev').onclick=()=>this.step(-1);
  }
  reset(){this.serial++;this.matches=[];this.index=-1;this.count.textContent='';globalThis.CSS?.highlights?.delete('texloom-search');this.document.querySelectorAll('.pdf-search-match').forEach(e=>e.classList.remove('pdf-search-match'));}
  async find(){
    clearTimeout(this.timer);this.reset();const query=this.input.value.trim().toLocaleLowerCase(),view=this.getView(),serial=this.serial;if(!query||!view)return;
    if(query.length>256){this.count.textContent='Use at most 256 characters';return;}
    this.searchedQuery=query;
    this.count.textContent='Searching…';const matches=[];
    try{let characters=0;
      for(let n=1;n<=view.pageCount;n++){
        const page=await view.pdf.getPage(n),text=await page.getTextContent();if(serial!==this.serial||view!==this.getView())return;
        const value=text.items.filter(i=>typeof i.str==='string').map(i=>i.str).join(' ').toLocaleLowerCase();characters+=value.length;
        if(characters>8_000_000)throw Error('Search exceeds the eight-million-character limit. Narrow the project.');
        let at=0,occurrence=0;while((at=value.indexOf(query,at))>=0){matches.push({page:n,offset:at,occurrence:occurrence++});at+=query.length;if(matches.length>=10000)break;}if(matches.length>=10000)break;
      }
      if(serial!==this.serial||view!==this.getView())return;this.matches=matches;this.index=-1;
      if(matches.length)await this.step(1);else this.count.textContent='No matches';
    }catch(error){if(serial===this.serial){this.count.textContent='Search unavailable';this.notify(String(error?.message||error))}}
  }
  async step(direction){
    if(!this.matches.length||this.searchedQuery!==this.input.value.trim().toLocaleLowerCase())return this.find();const serial=this.serial,view=this.getView();this.index=(this.index+direction+this.matches.length)%this.matches.length;
    const match=this.matches[this.index];
    try{await view.ensurePage?.(match.page);if(serial!==this.serial||view!==this.getView())return;
      this.document.querySelectorAll('.pdf-search-match').forEach(e=>e.classList.remove('pdf-search-match'));
      const page=view.pages[match.page-1];page.shell.classList.add('pdf-search-match');page.shell.scrollIntoView({block:'center'});
      this.highlight(page,match.occurrence);
      this.count.textContent=(this.index+1)+' / '+this.matches.length;
    }catch(error){this.notify(String(error?.message||error))}
  }
  highlight(page,occurrence){
    globalThis.CSS?.highlights?.delete('texloom-search');
    const spans=[...page.shell.querySelectorAll('.textLayer span')].filter(span=>span.firstChild?.nodeType===3),query=this.input.value.trim().toLocaleLowerCase();
    const joined=spans.map(s=>s.textContent).join(' ').toLocaleLowerCase();let at=-query.length;
    for(let n=0;n<=occurrence;n++){at=joined.indexOf(query,at+query.length);if(at<0)return;}
    const find=offset=>{for(const span of spans){if(offset<=span.textContent.length)return [span,Math.min(offset,span.firstChild.length)];offset-=span.textContent.length+1;}return null};
    const start=find(at),end=find(at+query.length);if(!start||!end)return;
    if(globalThis.CSS?.highlights&&globalThis.Highlight){const range=this.document.createRange();range.setStart(start[0].firstChild,start[1]);range.setEnd(end[0].firstChild,end[1]);CSS.highlights.set('texloom-search',new Highlight(range));}
    start[0].scrollIntoView({block:'center'});
  }
}
