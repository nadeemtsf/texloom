import {editorText} from './document-protocol.js';

function boundary(text,offset){
  if(!Number.isSafeInteger(offset)||offset<0||offset>text.length)return false;
  const before=text.charCodeAt(offset-1),after=text.charCodeAt(offset);
  return !(before>=0xd800&&before<=0xdbff&&after>=0xdc00&&after<=0xdfff);
}

/** Native selections use offsets in the BOM-free, LF-normalized UTF-16 view. */
export function selectionSourceRange(selection,sources){
  if(!selection||!['source','object'].includes(selection.kind)||!Object.hasOwn(sources,selection.file))return null;
  const raw=sources[selection.file];if(typeof raw!=='string')return null;
  if(selection.kind==='source'&&selection.expected_source!==raw)return null;
  const text=editorText(raw),{start,end}=selection;
  if(!boundary(text,start)||!boundary(text,end)||start>=end)return null;
  return {file:selection.file,text,start,end};
}

/** Presentation-only source link; no selection/cursor IPC, source patch or focus change. */
export class SourceHighlight {
  constructor({editor,getState,isCurrent,activateFile}){
    Object.assign(this,{editor,getState,isCurrent,activateFile});
    this.decorations=editor.createDecorationsCollection?.();
  }
  clear(){try{this.decorations?.clear()}catch{}}
  update(selection){
    this.clear();if(!this.decorations||!this.isCurrent())return false;
    const target=selectionSourceRange(selection,this.getState().sources);if(!target)return false;
    // The caller owns safe file activation/rollback. A failed activation must
    // not leave later typing pointed at a different file than the editor shows.
    try{
      if(this.getState().file!==target.file)this.activateFile(target.file);
      if(!this.isCurrent()||this.getState().file!==target.file)return false;
      const model=this.editor.getModel();if(!model||model.getValue()!==target.text)return false;
      const start=model.getPositionAt(target.start),end=model.getPositionAt(target.end);
      const range={startLineNumber:start.lineNumber,startColumn:start.column,endLineNumber:end.lineNumber,endColumn:end.column};
      this.decorations.set([{range,options:{description:'Selected PDF text source',className:'texloom-source-match'}}]);
      this.editor.revealRangeInCenterIfOutsideViewport(range);
      return true;
    }catch{this.clear();return false}
  }
}
