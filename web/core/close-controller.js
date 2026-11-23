/** Confirm unsaved changes before releasing the current native document. */
export class CloseController {
  constructor({getState,isDirty,confirm,closeWindow,invoke,lock,unlock,notify,chooseUnsavedAction,saveProject}) {
    Object.assign(this,{getState,isDirty,confirm,closeWindow,invoke,lock,unlock,notify,chooseUnsavedAction,saveProject});
    this.prompting=false;this.approvedClose=false;
  }
  async confirmWithoutRecovery() {
    const action=await this.chooseUnsavedAction();
    if(action==='discard')return true;
    if(action!=='save')return false;
    await this.saveProject();
    return !this.isDirty();
  }
  async handle(event) {
    const state=this.getState();
    if(state.saving||state.opening||state.mutating||state.composing||this.prompting){
      event.preventDefault();this.notify('Wait for the current edit, Save or recovery to finish before closing.');return;
    }
    if(!this.approvedClose&&this.isDirty()){
      event.preventDefault();this.prompting=true;
      try{
        const yes=this.chooseUnsavedAction?await this.confirmWithoutRecovery():await this.confirm('Close without saving? Your unsaved changes will stay in Recover work. Original project files will not change. To permanently remove a draft, use Discard selected draft in Recover work.',
          {title:'Keep a recovery copy and close?',kind:'warning',okLabel:'Keep draft and close',cancelLabel:'Keep editing'});
        if(yes){this.approvedClose=true;this.prompting=false;await this.closeWindow()}
      }catch(error){this.approvedClose=false;this.notify(String(error?.message||error))}
      finally{this.prompting=false}
      return;
    }
    const client=state.client,project=state.project?.id;let closed=false;this.lock();
    try{
      await client?.flush();
      const current=this.getState();
      if(client!==current.client||project!==current.project?.id)throw Error('The project changed before closing. Review it before trying again.');
      const dirty=this.isDirty(),protection=current.recovery;
      if(!this.chooseUnsavedAction&&dirty&&(client?.error||protection?.error||protection?.protected_revision!==current.revision))
        throw Error('The latest changes do not have a complete recovery copy. Keep this window open and Save, save a copy, or retry recovery before closing.');
      if(project){
        const reply=await this.invoke('close_document',{projectId:project,revision:client.snapshot.revision,discardRecovery:!!this.chooseUnsavedAction||!dirty});
        if(reply?.warning)this.notify(reply.warning);
      }
      closed=true;
      await this.invoke('mark_clean_exit');
    }catch(error){
      // Once native close succeeded, reopening this now-closed document replica
      // would be misleading. An exit-marker failure must not do that.
      if(!closed){event.preventDefault();this.approvedClose=false;this.unlock()}
      this.notify(String(error?.message||error));
    }
  }
}
