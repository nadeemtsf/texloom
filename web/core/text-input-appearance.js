/** Bounded editing hints, not a TeX renderer. Values never come from source CSS. */
export function applyTextInputAppearance(input,{styles=[],plain=true}={}){
  const names=new Set(styles);
  input.dir=plain?'auto':'ltr';
  input.style.fontWeight=plain&&names.has('Bold')?'700':'';
  input.style.fontStyle=plain&&names.has('Italic')?'italic':plain&&names.has('Slanted')?'oblique':'';
  input.style.fontVariantCaps=plain&&names.has('Small caps')?'small-caps':'';
  input.style.textDecorationLine=plain&&names.has('Underlined')?'underline':'';
  input.style.fontFamily=!plain?'Consolas, monospace':names.has('Monospace')?'Consolas, monospace':
    names.has('Roman')?'Georgia, serif':names.has('Sans serif')?'Segoe UI, sans-serif':'';
}

export function renderProtectedCommands(document,container,commands){
  container.replaceChildren();
  const unique=[...new Set(commands)];container.classList.toggle('hidden',!unique.length);
  if(!unique.length)return;
  const title=document.createElement('p');title.textContent='Kept unchanged:';container.append(title);
  for(const value of unique.slice(0,8)){
    const token=document.createElement('code');token.className='protected-command';token.textContent=value.length>80?value.slice(0,77)+'…':value;container.append(token);
  }
  if(unique.length>8){const more=document.createElement('span');more.textContent=` and ${unique.length-8} more`;container.append(more)}
}
