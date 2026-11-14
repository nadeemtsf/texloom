export function clearDialogPosition(dialog){
  for(const name of ['position','left','top','right','bottom','margin'])dialog.style[name]='';
}

/** Keep the modal semantics; use a nearby position only when it fits visibly. */
export function positionDialogNear(dialog,target,{width=globalThis.innerWidth,height=globalThis.innerHeight,gap=12,padding=16}={}){
  clearDialogPosition(dialog);
  if(!dialog.open||!target||![width,height,gap,padding,target.left,target.top,target.right,target.bottom].every(Number.isFinite)||gap<0||padding<0)return false;
  const rect=dialog.getBoundingClientRect(),w=rect.width,h=rect.height;
  if(!Number.isFinite(w)||!Number.isFinite(h)||w<=0||h<=0||width<w+2*padding||height<h+2*padding)return false;
  const visible={left:Math.max(padding,target.left),top:Math.max(padding,target.top),
    right:Math.min(width-padding,target.right),bottom:Math.min(height-padding,target.bottom)};
  if(visible.left>=visible.right||visible.top>=visible.bottom)return false;
  const clampX=value=>Math.max(padding,Math.min(width-padding-w,value));
  const clampY=value=>Math.max(padding,Math.min(height-padding-h,value));
  const choices=[{left:visible.right+gap,top:clampY(visible.top)},
    {left:visible.left-gap-w,top:clampY(visible.top)},
    {left:clampX(visible.left),top:visible.bottom+gap},
    {left:clampX(visible.left),top:visible.top-gap-h}];
  const fit=choices.find(({left,top})=>left>=padding&&top>=padding&&left+w<=width-padding&&top+h<=height-padding);
  if(!fit)return false;
  Object.assign(dialog.style,{position:'fixed',left:fit.left+'px',top:fit.top+'px',right:'auto',bottom:'auto',margin:'0'});
  return true;
}
