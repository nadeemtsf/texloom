const general={
  title:'Set up LaTeX for Texloom',
  steps:[
    'Install one LaTeX distribution: MiKTeX or TeX Live. Texloom uses its pdfLaTeX, XeLaTeX or LuaLaTeX compiler.',
    'Finish the distribution setup and install the packages your document needs. Then reopen Texloom so it can find newly installed programs.',
    'Choose the main .tex document and its engine, then choose Compile. Auto detect respects a supported % !TeX program directive near the start of the main source.',
    'For English, Arabic and Hebrew together, choose XeLaTeX or LuaLaTeX and install fontspec, polyglossia and the fonts named in your document.',
    'Visual source selection also needs SyncTeX from the same LaTeX installation. Source editing and Read text remain available when selection is unsupported.'
  ]
};

/** Classify known setup failures only; an arbitrary TeX error is not an installation diagnosis. */
export function setupAdvice(error){
  const codes=new Set((Array.isArray(error?.diagnostics)?error.diagnostics:[]).map(d=>d?.code));
  const message=String(typeof error==='string'?error:error?.message||'').slice(0,8192);
  const missing=/^(pdflatex|xelatex|lualatex) was not found\./i.exec(message);
  if(missing)return {kind:'compiler',title:'LaTeX compiler not found',steps:[
    `Texloom could not find ${missing[1]}. Install MiKTeX or TeX Live, or finish your existing distribution setup.`,
    'If LaTeX is already installed in a custom folder, add its bin folder to your Windows user PATH. Reopen Texloom afterward.',
    'Choose the engine required by your document, then choose Compile again.'
  ]};
  if([...codes].some(code=>typeof code==='string'&&code.startsWith('compiler_preflight_'))||/^Compiler preflight failed:/i.test(message))return {kind:'initialization',title:'LaTeX could not start',steps:[
    'Open Compiler output to read the initialization error. A timeout or access error does not establish that LaTeX is missing.',
    'Open your distribution manager (MiKTeX Console or TeX Live Manager) and finish setup or repair its reported problem for your Windows account.',
    'Close any setup window waiting for an answer. Reopen Texloom after repairs and choose Compile again.'
  ]};
  if(codes.has('missing_package'))return {kind:'package',title:'A LaTeX package or class is missing',steps:[
    'Open Compiler output and note the missing .sty, .cls or .ldf file.',
    'For a distribution package, install it using MiKTeX Console or TeX Live Manager. Texloom disables automatic MiKTeX package installation during compilation.',
    'For a custom class or style supplied with your project, restore that supporting file beside the source instead.',
    'Choose Compile again after the missing files are available.'
  ]};
  if(codes.has('missing_glyph')||/fontspec.*(?:requires|cannot)|font[^\r\n]{0,160}(?:cannot be found|not found)/i.test(message))return {kind:'font',title:'Check the engine and fonts',steps:[
    'For fontspec or mixed English, Arabic and Hebrew, choose XeLaTeX or LuaLaTeX.',
    'Install the fonts named by your document and its language packages. A missing character can also mean the selected font has no glyph for that character.',
    'Reopen Texloom after installing fonts, then compile and check the displayed text.'
  ]};
  return null;
}

/** Read-only guidance: no compiler, download, settings change or project write. */
export class SetupHelp {
  constructor({document}){
    this.document=document;this.current=null;
    const $=id=>document.getElementById(id);
    this.dialog=$('setup-dialog');this.context=$('setup-context');
    $('setup-help').onclick=()=>this.show();
    $('setup-error-help').onclick=()=>this.show();
    $('setup-close').onclick=()=>this.dialog.close();
    for(const input of document.querySelectorAll('#setup-dialog input[readonly]'))input.onclick=()=>input.select();
  }
  update(error){
    this.current=setupAdvice(error);
    this.document.getElementById('setup-error-help').classList.toggle('hidden',!this.current);
  }
  show(){
    const advice=this.current||general;
    this.context.textContent=advice.title;
    const list=this.document.getElementById('setup-steps');list.replaceChildren();
    for(const text of advice.steps){const item=this.document.createElement('li');item.textContent=text;list.append(item)}
    if(!this.dialog.open)this.dialog.showModal();
  }
}
