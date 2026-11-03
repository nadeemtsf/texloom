/** Ordered replica protocol. Text/history authority stays in Rust. */
export class DocumentClient {
  constructor(invoke,projectId,snapshot,{changed=()=>{},failed=()=>{}}={}) {
    this.invoke=invoke;this.projectId=projectId;this.snapshot=snapshot;
    this.changed=changed;this.failed=failed;this.pending=0;this.outstanding=0;this.unpublished=false;this.error=null;this.tail=Promise.resolve();this.compileSerial=0;
    this.assertRevision(snapshot.revision);
  }
  assertRevision(revision){if(!Number.isSafeInteger(revision)||revision<0)throw new Error('Invalid document revision');}
  accept(name,args,next) {
    this.assertRevision(next.revision);
    const previous=this.snapshot;
    if(next.revision<previous.revision)throw new Error('Document acknowledgement moved backward');
    if(next.kind===undefined)return {snapshot:{...previous,...next},sourceChanged:!!next.sources};
    const invalid=()=>{throw new Error('Invalid source edit acknowledgement; source synchronization stopped');};
    if(name!=='document_edit'||next.kind!=='source_delta'||next.base_revision!==previous.revision)invalid();
    if(next.revision!==previous.revision&&next.revision!==previous.revision+1)invalid();
    if(next.main!==previous.main||next.engine!==previous.engine||typeof next.can_undo!=='boolean'||typeof next.can_redo!=='boolean')invalid();
    if(!next.sources||typeof next.sources!=='object'||Array.isArray(next.sources))invalid();
    const files=Object.keys(next.sources),changed=next.revision!==previous.revision;
    if(changed?files.length!==1||files[0]!==args.file:files.length!==0)invalid();
    if(changed&&(!Object.prototype.hasOwnProperty.call(previous.sources,args.file)||typeof next.sources[args.file]!=='string'||next.sources[args.file]===previous.sources[args.file]))invalid();
    const range=s=>s&&Number.isSafeInteger(s.anchor)&&s.anchor>=0&&Number.isSafeInteger(s.head)&&s.head>=0;
    if(next.selection!==null&&(!range(next.selection)||!Object.prototype.hasOwnProperty.call(previous.sources,next.selection.file)||!Array.isArray(next.selection.secondary)||!next.selection.secondary.every(range)))invalid();
    const {kind,base_revision,sources,...metadata}=next;
    // A no-op must still reconcile the shell's optimistic editor revision.
    return {snapshot:{...previous,...metadata,sources:changed?{...previous.sources,...sources}:previous.sources},sourceChanged:true};
  }
  enqueue(name,args={},fatal=true,mutating=true) {
    this.outstanding++;if(mutating)this.pending++;
    const job=this.tail.then(async()=>{
      if(this.error)throw this.error;
      const next=await this.invoke(name,{projectId:this.projectId,revision:this.snapshot.revision,...args});
      const accepted=this.accept(name,args,next);
      this.snapshot=accepted.snapshot;if(accepted.sourceChanged)this.unpublished=true;return this.snapshot;
    }).catch(error=>{if(fatal&&!this.error){this.error=error;this.failed(error);}throw error;}).finally(()=>{
      this.outstanding--;if(mutating)this.pending--;
      if(!this.outstanding&&!this.error&&this.unpublished){this.unpublished=false;this.changed(this.snapshot);}
    });
    this.tail=job.catch(()=>{});return job;
  }
  async flush(){
    do {const tail=this.tail;await tail;if(this.error)throw this.error;if(tail===this.tail&&!this.outstanding)break;}while(true);
    return this.snapshot;
  }
  edit(file,changes,group){return this.enqueue('document_edit',{file,changes,group});}
  profile(main,engine){return this.enqueue('document_profile',{main,engine});}
  history(redo){return this.enqueue('document_history',{redo});}
  selection(file,anchor,head,secondary=[]){return this.enqueue('document_selection',{file,anchor,head,secondary},true,false);}
  async prepare(sources,baselineBuildId,operation=null){await this.flush();return this.invoke('document_prepare',{projectId:this.projectId,revision:this.snapshot.revision,sources,baselineBuildId,operation});}
  async compile(token=null,purpose='preview'){
    this.activeToken=token;
    const requestId=++this.compileSerial;this.assertRevision(requestId);
    try{return await this.invoke('compile_document',{projectId:this.projectId,revision:this.snapshot.revision,token,purpose,requestId});}
    finally{if(requestId!==this.compileSerial){const error=new Error('Compilation cancelled or superseded.');error.name='CompilationCancelled';throw error;}}
  }
  async cancel(){const requestId=this.compileSerial;this.compileSerial++;return this.invoke('cancel_compilation',{projectId:this.projectId,requestId,token:this.activeToken||null});}
  async verify(buildId){return this.invoke('verify_build',{buildId});}
  async pin(buildId){return this.invoke('pin_displayed_build',{projectId:this.projectId,buildId});}
  commit(token,buildId){return this.enqueue('document_commit',{token,buildId},false);}
  async discard(token){return this.invoke('document_discard',{projectId:this.projectId,token});}
}

export function editorText(source){return source.replace(/^\uFEFF/u,'').replace(/\r\n?|\n/gu,'\n');}

// Buffers contain immutable strings. Compare exact bytes without allocating a
// second JSON representation of the entire project or depending on key order.
export function sameSources(left,right){
  if(left===right)return true;
  const files=Object.keys(left);
  return files.length===Object.keys(right).length&&files.every(file=>Object.prototype.hasOwnProperty.call(right,file)&&left[file]===right[file]);
}
