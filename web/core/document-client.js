/** Ordered replica protocol. Text/history authority stays in Rust. */
export class DocumentClient {
  constructor(invoke,projectId,snapshot,{changed=()=>{},failed=()=>{}}={}) {
    this.invoke=invoke;this.projectId=projectId;this.snapshot=snapshot;
    this.changed=changed;this.failed=failed;this.pending=0;this.outstanding=0;this.unpublished=false;this.error=null;this.tail=Promise.resolve();
    this.assertRevision(snapshot.revision);
  }
  assertRevision(revision){if(!Number.isSafeInteger(revision)||revision<0)throw new Error('Invalid document revision');}
  enqueue(name,args={},fatal=true,mutating=true) {
    this.outstanding++;if(mutating)this.pending++;
    const job=this.tail.then(async()=>{
      if(this.error)throw this.error;
      const next=await this.invoke(name,{projectId:this.projectId,revision:this.snapshot.revision,...args});
      this.assertRevision(next.revision);
      if(next.revision<this.snapshot.revision)throw new Error('Document acknowledgement moved backward');
      this.snapshot={...this.snapshot,...next};if(next.sources)this.unpublished=true;return this.snapshot;
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
  async prepare(sources){await this.flush();return this.invoke('document_prepare',{projectId:this.projectId,revision:this.snapshot.revision,sources});}
  async compile(token=null){return this.invoke('compile_document',{projectId:this.projectId,revision:this.snapshot.revision,token});}
  commit(token,buildId){return this.enqueue('document_commit',{token,buildId},false);}
  async discard(token){return this.invoke('document_discard',{projectId:this.projectId,token});}
}

export function editorText(source){return source.replace(/^\uFEFF/u,'').replace(/\r\n?|\n/gu,'\n');}
