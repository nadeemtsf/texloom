// IPC boundary double for frontend integration tests. The actual native state
// machine and byte-preservation behavior are tested separately in Rust.
class NativeDocumentDouble {
 constructor(control){this.control=control;this.sources={'main.tex':'Original source','studio.sty':'Style source'};
  this.revision=0;this.main='main.tex';this.engine='auto';this.history=[];this.redo=[];this.pending=new Map();}
 snapshot(){return structuredClone({sources:this.sources,revision:this.revision,main:this.main,engine:this.engine,
  can_undo:!!this.history.length,can_redo:!!this.redo.length});}
 record(){this.history.push(this.snapshot());this.redo=[];}
 async invoke(name,args){
  if(name==='resolve_text_selection')return this.control.resolve(args);
  if(name==='plan_text_object')return this.control.plan(args.request);
  if(name==='mark_clean_exit'){this.control.cleanExits=(this.control.cleanExits??0)+1;return;}
  if(name==='verify_build')return this.control.verify?.(args);
  if(name==='cancel_compilation')return;
  if(name==='document_selection')return {revision:this.revision,selection:{file:args.file,anchor:args.anchor,head:args.head,secondary:args.secondary}};
  if(name==='document_prepare'){
   if(args.revision!==this.revision)throw new Error('Stale document');
   const token='candidate-'+(this.pending.size+1);this.pending.set(token,{sources:structuredClone(args.sources),base:this.revision});
   return {token,revision:this.revision+1};
  }
  if(name==='compile_document'){
   if(args.revision!==this.revision)throw new Error('Stale compilation');
   const pending=args.token?this.pending.get(args.token):null;
   const revision=pending?pending.base+1:this.revision;
   const result=await this.control.compile({sources:pending?.sources??this.sources,revision});
   if(pending&&result.ok)pending.build=result.build_id;
   return {...result,revision:result.revision??revision};
  }
  if(name==='document_commit'){
   const pending=this.pending.get(args.token);
   if(!pending||pending.base!==this.revision||pending.build!==args.buildId)throw new Error('Invalid native candidate');
   this.record();this.sources=structuredClone(pending.sources);this.revision++;this.pending.delete(args.token);return this.snapshot();
  }
  if(name==='document_discard'){this.pending.delete(args.token);return;}
  if(name==='document_edit'){
   if(args.revision!==this.revision)throw new Error('Stale source');
   this.record();let source=this.sources[args.file];
   for(const c of [...args.changes].sort((a,b)=>b.offset-a.offset))source=source.slice(0,c.offset)+c.text+source.slice(c.offset+c.length);
   this.sources[args.file]=source;this.revision++;this.pending.clear();return this.snapshot();
  }
  if(name==='document_history'){
   if(args.revision!==this.revision)throw new Error('Stale history');
   const from=args.redo?this.redo:this.history,to=args.redo?this.history:this.redo;
   if(from.length){to.push(this.snapshot());const old=from.pop();this.sources=old.sources;this.main=old.main;this.engine=old.engine;this.revision++;}
   this.pending.clear();return this.snapshot();
  }
  throw new Error('Unexpected command '+name);
 }
}
module.exports={NativeDocumentDouble};
