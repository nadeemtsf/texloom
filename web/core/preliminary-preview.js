/** Optional rendering owns its lifetime, independently of the settled PDF. */
export class PreliminaryPreview {
  constructor({current,render,show,hide}){
    Object.assign(this,{current,render,show,hide});this.active=null;
  }
  get visible(){return !!this.active?.prepared}
  start(request){
    this.clear();const identity=Object.freeze({...request});
    if(!this.current(identity))return null;
    return this.active={request:identity,abort:new AbortController(),seen:false,accepting:true,prepared:null};
  }
  async offer(ticket,progress){
    if(!ticket||this.active!==ticket||!ticket.accepting||ticket.seen||!this.current(ticket.request))return false;
    ticket.seen=true;let prepared=null;
    try{
      prepared=await this.render(progress,{signal:ticket.abort.signal});
      if(this.active!==ticket||!ticket.accepting||!this.current(ticket.request)){
        this.dispose(prepared);return false;
      }
      ticket.prepared=prepared;this.show(prepared);return true;
    }catch{
      if(this.active===ticket)this.clear(ticket);
      else if(prepared)this.dispose(prepared);
      return false;
    }
  }
  end(ticket){if(ticket&&this.active===ticket)ticket.accepting=false}
  clear(ticket=null){
    if(!this.active||ticket&&this.active!==ticket)return;
    const previous=this.active;this.active=null;previous.accepting=false;previous.abort.abort();
    if(previous.prepared){try{this.hide(previous.prepared)}catch{}finally{this.dispose(previous.prepared)}}
  }
  dispose(prepared){try{Promise.resolve(prepared.dispose()).catch(()=>{})}catch{}}
}
