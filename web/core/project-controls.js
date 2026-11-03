/** Project tools. Native services own path, byte, revision and disk checks. */
export class ProjectControls {
  constructor({document,invoke,getState,flush,isDirty,lock,unlock,confirm,open,save,adopt,compile,notify,cursor,insertImage}) {
    Object.assign(this,{document,invoke,getState,flush,isDirty,lock,unlock,confirm,open,save,adopt,compile,notify,cursor,insertImage});
    this.$=id=>document.getElementById(id);this.review=null;this.choices=new Map();this.serial=0;
    const bind=(id,fn)=>{this.$(id).onclick=()=>this.run(fn)};
    bind('projects',()=>this.show());bind('project-open-path',()=>this.openPath());
    bind('project-create',()=>this.create());bind('project-choose-parent',()=>this.chooseParent());
    bind('project-browse-file',()=>this.browse(false));bind('project-browse-folder',()=>this.browse(true));
    bind('project-import',()=>this.importZip());bind('project-export',()=>this.exportZip());
    bind('project-review-encoding',()=>this.reviewEncoding());bind('encoding-convert',()=>this.convert());
    bind('import-image',()=>this.image());
    this.$('project-close').onclick=()=>{if(!this.getState().saving)this.$('project-dialog').close()};
    this.$('encoding-close').onclick=()=>{if(!this.getState().saving)this.$('encoding-dialog').close()};
    for(const id of ['project-dialog','encoding-dialog'])this.$(id).addEventListener('cancel',event=>{if(this.getState().saving)event.preventDefault()});
    this.$('project-path').addEventListener('keydown',event=>{if(event.key==='Enter'){event.preventDefault();this.run(()=>this.openPath())}});
  }
  async run(fn) {try{await fn()}catch(error){this.notify(String(error?.message||error))}}
  blocked() {const s=this.getState();return s.saving||s.opening||s.mutating||s.composing||s.busy;}
  ready() {if(this.blocked())throw Error('Wait for the current operation to finish.');}
  async show() {
    this.ready();this.$('project-export').disabled=!this.getState().project;
    this.$('project-dialog').showModal();const serial=++this.serial;
    const recent=await this.invoke('recent_projects',{});if(serial!==this.serial)return;
    const list=this.$('project-recents');list.replaceChildren();
    for(const entry of recent){const button=this.document.createElement('button');button.type='button';
      button.textContent=entry.name+' · '+entry.main;button.title=entry.path;
      button.onclick=()=>this.run(()=>this.switchProject(()=>this.invoke('open_recent',{path:entry.path})));list.append(button)}
    if(!recent.length)list.textContent='Projects you open will appear here.';
  }
  path(){const path=this.$('project-path').value.trim().replace(/^"(.*)"$/u,'$1');if(!path)throw Error('Enter a project file or folder path.');return path;}
  async switchProject(load) {
    this.ready();await this.flush();
    if(this.isDirty()&&!await this.confirm('Open another project and discard the current unsaved editor version? Save a copy first to retain it.',{title:'Unsaved source',kind:'warning'}))return;
    this.lock();let project=null,adopted=false;
    try{project=await load();await this.adopt(project);adopted=this.getState().project?.id===project.id;
      if(!adopted)throw Error('The project could not be displayed.');this.$('project-dialog').close();this.$('encoding-dialog').close();
    }finally{if(project&&!adopted)await this.invoke('close_document',{projectId:project.id}).catch(()=>{});this.unlock()}
    if(adopted&&!project.source_only)this.notify('Project source opened. Review it, then choose Compile to build its PDF.');
  }
  openPath(){const path=this.path();return this.switchProject(()=>this.invoke('open_project',{path}));}
  async chooseParent(){this.ready();const parent=await this.open({directory:true,multiple:false,title:'Choose a parent folder for a new project'});if(typeof parent==='string')this.$('project-parent').value=parent;}
  async create(){
    const template=this.$('project-template').value,name=this.$('project-new-name').value.trim(),parent=this.$('project-parent').value.trim()||null;
    if(!name)throw Error('Enter a name for the new project.');
    await this.switchProject(async()=>{const created=await this.invoke('create_project',{template,name,parent});this.$('project-path').value=created.path;return this.invoke('open_project',{path:created.path})});
  }
  async browse(directory) {
    this.ready();const path=await this.open({directory,multiple:false,title:directory?'Open project folder':'Open LaTeX project',...(!directory?{filters:[{name:'LaTeX source',extensions:['tex']}]}:{})});
    if(typeof path!=='string')return;this.$('project-path').value=path;await this.openPath();
  }
  async importZip() {
    this.ready();const path=await this.open({multiple:false,filters:[{name:'LaTeX project ZIP',extensions:['zip']}]});if(typeof path!=='string')return;
    await this.switchProject(async()=>{const imported=await this.invoke('import_project_archive',{path});this.$('project-path').value=imported.path;return this.invoke('open_project',{path:imported.path})});
  }
  async exportZip() {
    this.ready();if(!this.getState().project)return;this.lock();
    try{await this.flush();const s=this.getState();const path=await this.save({defaultPath:s.project.name+'.zip',filters:[{name:'LaTeX project ZIP',extensions:['zip']}]});if(!path)return;
      await this.invoke('export_project_archive',{projectId:s.project.id,revision:s.revision,path});this.notify('Project ZIP exported with the current source, images and engine choice.');
    }finally{this.unlock()}
  }
  async reviewEncoding() {
    this.ready();const path=this.path();const review=await this.invoke('review_encoding',{path});this.review=review;this.choices.clear();
    const list=this.$('encoding-files');list.replaceChildren();this.$('encoding-preview').value='';this.$('encoding-message').textContent='';
    for(const issue of review.files){const row=this.document.createElement('label');row.className='parameter-row';const name=this.document.createElement('span');name.textContent=issue.file;
      const select=this.document.createElement('select');select.setAttribute('aria-label',issue.file+' encoding');
      for(const [value,label] of [['','Choose original encoding'],['utf-8','UTF-8'],['windows-1252','Western Windows'],['iso-8859-1','Latin-1'],['windows-1256','Arabic Windows'],['windows-1255','Hebrew Windows'],['utf-16le','UTF-16 little endian'],['utf-16be','UTF-16 big endian'],['shift_jis','Japanese Shift JIS'],['gb18030','Chinese GB18030'],['big5','Chinese Big5']]){
        const option=this.document.createElement('option');option.value=value;option.textContent=label;select.append(option)}
      select.value=issue.detected||'';select.onchange=()=>{this.choices.delete(issue.file);this.conversionReady()};
      const preview=this.document.createElement('button');preview.type='button';preview.textContent='Preview';preview.onclick=()=>this.run(async()=>{
        if(!select.value)throw Error('Choose the original source encoding.');const encoding=select.value,context=this.review;
        const text=await this.invoke('preview_encoding',{path:context.root,file:issue.file,encoding});
        if(this.review!==context||select.value!==encoding)return;
        this.$('encoding-preview').value=text;this.$('encoding-message').textContent=issue.file+' · '+encoding+' · read-only preview';
        this.choices.set(issue.file,{file:issue.file,encoding,sha256:issue.sha256});this.conversionReady();
      });row.append(name,select,preview);list.append(row);
    }
    if(!review.files.length)this.$('encoding-message').textContent='All source files are already valid UTF-8. Open the project normally.';
    this.conversionReady();this.$('project-dialog').close();this.$('encoding-dialog').showModal();
  }
  conversionReady(){this.$('encoding-convert').disabled=!this.review?.files.length||this.choices.size!==this.review.files.length;}
  async convert() {
    this.ready();if(!this.review?.files.length||this.choices.size!==this.review.files.length)return;
    const review=this.review,choices=[...this.choices.values()];const parent=await this.open({directory:true,multiple:false,title:'Choose a folder for the UTF-8 copy, outside the original project'});if(!parent)return;
    await this.switchProject(async()=>{const copy=await this.invoke('convert_encoding',{path:review.root,parent,fingerprint:review.fingerprint,choices});return this.invoke('open_project',{path:copy.path})});
  }
  async image() {
    this.ready();const initial=this.getState();if(!initial.project)return;
    if(initial.compileApproved===false)throw Error('Review and compile the project before importing an image.');
    await this.flush();const context=this.cursor();if(!context)throw Error('Place the source cursor where the image should appear.');
    const client=initial.client,path=await this.open({multiple:false,title:'Import image',filters:[{name:'Image',extensions:['png','jpg','jpeg','pdf']}]});if(typeof path!=='string')return;
    this.ready();if(this.getState().client!==client||this.getState().sources[context.file]!==context.source)throw Error('The source changed. Choose the image again.');
    this.lock();let image;
    try{await this.flush();image=await this.invoke('import_image',{projectId:initial.project.id,revision:this.getState().revision,path})}finally{this.unlock()}
    const before=this.getState().result?.build_id;await this.compile();
    if(this.getState().client!==client||this.getState().result?.build_id===before){this.notify('Image copied to '+image.asset+'. Compile the project before inserting it.');return}
    if(this.getState().sources[context.file]!==context.source){this.notify('Image copied to '+image.asset+'. The source cursor changed; insert its path in the current source.');return}
    await this.insertImage(context,image.asset);
  }
}
