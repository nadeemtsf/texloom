// The browser owns the caret, IME, selection and keyboard editing. Keep each
// logical break as an actual newline so Backspace can traverse styled spans.
export class InlineSurface {
  constructor(element) { this.element=element; }
  get value() { return this.element.innerText.replace(/\r\n?/gu,'\n'); }
  get selection() {
    const node=this.element,selection=node.ownerDocument.getSelection();
    if(!selection?.rangeCount||!node.contains(selection.focusNode))return null;
    const offset=(child,end)=>{
      // Count the semantic breaks between contenteditable blocks as well as text.
      const range=document.createRange();range.selectNodeContents(node);range.setEnd(child,end);
      const box=document.createElement('div');box.style.cssText='position:fixed;left:-100000px;white-space:pre-wrap';
      box.append(range.cloneContents());document.body.append(box);
      try{return box.innerText.length}finally{box.remove()}
    };
    return {anchor:offset(selection.anchorNode,selection.anchorOffset),head:offset(selection.focusNode,selection.focusOffset)};
  }
  select(offset) {
    const node=this.element,selection=node.ownerDocument.getSelection();
    node.focus({preventScroll:true});
    const range=document.createRange();
    for(const block of node.children){
      const length=block.textContent.length;
      if(offset>length){offset-=length+1;continue}
      const walker=document.createTreeWalker(block,NodeFilter.SHOW_TEXT);
      let text;
      while((text=walker.nextNode())&&offset>text.length)offset-=text.length;
      range.setStart(text||block,text?offset:0);range.collapse(true);
      selection.removeAllRanges();selection.addRange(range);return;
    }
    range.selectNodeContents(node);range.collapse(false);selection.removeAllRanges();selection.addRange(range);
  }
  initialize(entry,scale) {
    const node=this.element,fragment=document.createDocumentFragment(),rows=entry.rows;
    const left=Math.min(...rows.map(r=>r.line.x));
    const newBlock=()=>{const block=document.createElement('div');fragment.append(block);return block};
    let block=newBlock(),cursor=0;
    const literal=end=>{
      const parts=entry.text.slice(cursor,end).split('\n');
      for(const [i,part] of parts.entries()){
        if(i){if(!block.hasChildNodes())block.append(document.createElement('br'));block=newBlock()}
        if(part)block.append(document.createTextNode(part));
      }
      cursor=end;
    };
    for(const [i,row] of rows.entries()){
      literal(row.start);
      if(!block.hasChildNodes()){
        block.style.textIndent=(row.line.x-left)*scale+'px';
        if(i){
          const previous=rows[i-1].line;
          block.style.paddingTop=Math.max(0,(row.line.y-previous.y-previous.height*1.2)*scale)+'px';
        }
      }
      let column=0;
      for(const [j,run] of row.line.runs.entries()){
        if(j)column++;
        const end=row.offsets[column+(run.text??row.line.text).length];
        const span=document.createElement('span');
        span.textContent=entry.text.slice(cursor,end);cursor=end;
        span.style.fontFamily=run.font?JSON.stringify(run.font)+', serif':'serif';
        span.style.fontSize=run.size*scale+'px';
        block.append(span);column+=(run.text??row.line.text).length;
      }
    }
    literal(entry.text.length);
    if(!block.hasChildNodes())block.append(document.createElement('br'));
    node.replaceChildren(fragment);
  }
  insert(text) {
    // Chromium dispatches a real input event and keeps this operation in the
    // browser's typing history (and replaces any active selection).
    document.execCommand('insertText',false,text);
  }
  position(page,entry,scale,scroller) {
    const node=this.element,rows=entry.rows,lines=rows.map(r=>r.line);
    const first=lines[0],last=lines.at(-1),rect=page.shell.getBoundingClientRect(),viewport=scroller.getBoundingClientRect();
    const left=Math.min(...lines.map(l=>l.x)),natural=first.height*1.2;
    const soft=rows.filter(r=>entry.text[r.end]===' ').map(r=>r.line.x+r.line.width);
    const right=soft.length?Math.max(...soft,...lines.map(l=>l.x+l.width)):Math.max(rect.width/scale-left,...lines.map(l=>l.x+l.width));
    const next=page.lines.find(l=>l.y>last.y+1);
    const maxHeight=(next?next.y:rect.height/scale)-first.y;
    Object.assign(node.style,{left:rect.left-viewport.left+scroller.scrollLeft+left*scale+'px',
      top:rect.top-viewport.top+scroller.scrollTop+first.y*scale+'px',
      width:Math.max(1,(right-left)*scale+1)+'px',minHeight:Math.min(maxHeight,last.y-first.y+natural)*scale+'px',maxHeight:Math.max(natural,maxHeight)*scale+'px',
      fontSize:first.height*scale+'px',lineHeight:natural*scale+'px'});
  }
}
