/** Recovery presentation/orchestration. Native code owns all source and disk checks. */
export class TypingRecoveryController {
  constructor({invoke,document,getState,isDirty,adopt,lock,unlock,notify,confirm,saveCopy}) {
    Object.assign(this,{invoke,document,getState,isDirty,adopt,lock,unlock,notify,confirm,saveCopy});
    this.inventory={drafts:[],errors:[]};this.serial=0;
    const $=id=>document.getElementById(id);
    this.ui={trigger:$('recover-work'),dialog:$('draft-dialog'),choice:$('draft-choice'),details:$('draft-details'),
      errors:$('draft-errors'),restore:$('draft-restore'),discard:$('draft-discard'),close:$('draft-close'),copy:$('draft-copy'),
      status:$('draft-status'),warning:$('draft-warning'),warningText:$('draft-warning-text'),retry:$('draft-retry')};
    this.ui.trigger.onclick=()=>this.show();this.ui.choice.onchange=()=>this.renderDetails();
    this.ui.restore.onclick=()=>this.restore();this.ui.discard.onclick=()=>this.discard();
    this.ui.retry.onclick=()=>this.retry();this.ui.copy.onclick=saveCopy;
    this.ui.close.onclick=()=>{if(!getState().saving)this.ui.dialog.close()};
    this.ui.dialog.addEventListener('cancel',event=>{if(getState().saving)event.preventDefault()});
  }
  renderProtection() {
    const state=this.getState(),protection=state.recovery;
    const paused=!!state.project&&(!protection||protection.error||protection.protected_revision==null);
    this.ui.status.textContent=!state.project?'':paused?'Recovery paused':
      protection.protected_revision<state.revision?'Protecting changes…':'Recovery copy protected';
    const warnings=[state.project?.draft_warning,paused?'Automatic recovery is paused. Save your work or save a project copy. '+(protection?.error||'Protection is unavailable.'):''].filter(Boolean);
    this.ui.warning.classList.toggle('hidden',!warnings.length);
    this.ui.warningText.textContent=warnings.join(' ');
    this.ui.retry.classList.toggle('hidden',!paused);this.ui.retry.disabled=!!state.saving||!!state.opening||!!state.composing;
  }
  selected() { return this.inventory.drafts.find(draft=>draft.id===this.ui.choice.value); }
  render() {
    const project=this.getState().project?.id;
    const selected=this.choiceProject===project?this.ui.choice.value:'';this.choiceProject=project;
    this.ui.choice.replaceChildren();
    const placeholder=this.document.createElement('option');placeholder.value='';placeholder.textContent='Choose a project to recover';
    this.ui.choice.append(placeholder);
    for(const draft of this.inventory.drafts){
      const option=this.document.createElement('option');option.value=draft.id;
      option.textContent=draft.main+' — revision '+draft.revision+' — '+draft.project;
      this.ui.choice.append(option);
    }
    if(this.inventory.drafts.some(draft=>draft.id===selected))this.ui.choice.value=selected;
    this.ui.trigger.classList.toggle('hidden',!this.inventory.drafts.length&&!this.inventory.errors.length);
    this.ui.errors.textContent=this.inventory.errors.join('\n');
    this.renderDetails();
  }
  renderDetails() {
    const draft=this.selected();
    this.ui.restore.disabled=this.ui.discard.disabled=!draft;
    this.ui.details.textContent=draft?draft.project+'\n'+draft.files+' source files, '+draft.source_bytes+' bytes\n'+
      (draft.warning||'The last complete verified recovery state will open for review.'):'';
  }
  async refresh() {
    const serial=++this.serial;
    try {const inventory=await this.invoke('typing_drafts');if(serial===this.serial){this.inventory=inventory;this.render()}}
    catch(error){if(serial===this.serial){this.inventory={drafts:[],errors:[String(error?.message||error)]};this.render()}}
    return this.inventory;
  }
  blocked() {const state=this.getState();return state.saving||state.opening||state.mutating||state.composing||state.busy;}
  async show() {
    if(this.blocked())return this.notify('Wait for the current operation before reviewing recovery.');
    this.ui.choice.value='';
    await this.refresh();
    if(this.blocked())return;
    if(!this.inventory.drafts.length&&!this.inventory.errors.length)return this.notify('No unsaved-work drafts need recovery.');
    if(!this.ui.dialog.open)this.ui.dialog.showModal();
  }
  async restore() {
    const draft=this.selected();if(!draft||this.blocked())return;
    if(this.isDirty()&&!await this.confirm('Open this recovered project and discard the current unsaved editor version? Save a project copy first to retain it.',{title:'Open recovered work',kind:'warning'}))return;
    const client=this.getState().client;let project=null,adopted=false;this.lock();
    try {
      await client?.flush();
      project=await this.invoke('open_typing_draft',{draftId:draft.id});
      if(this.getState().client!==client)throw Error('The open project changed. Recovered work was retained.');
      this.ui.dialog.close();await this.adopt(project);
      adopted=this.getState().project?.id===project.id;
      if(!adopted)throw Error('Recovered work could not be displayed. Its draft was retained.');
      this.notify('Recovered work opened with undo history. Review it before saving. The earlier draft is retained.');
    }catch(error){this.notify(String(error?.message||error))}finally{
      if(project&&!adopted)await this.invoke('close_document',{projectId:project.id}).catch(()=>{});
      this.unlock();await this.refresh();
    }
  }
  async discard() {
    const draft=this.selected();if(!draft||this.blocked())return;
    if(!await this.confirm('Permanently discard this unsaved-work draft? Original project files and the current editor are kept.',{title:'Discard recovery draft',kind:'warning'}))return;
    this.lock();
    try {
      this.inventory=await this.invoke('discard_typing_draft',{draftId:draft.id});this.render();
      if(!this.inventory.drafts.length&&!this.inventory.errors.length)this.ui.dialog.close();
      this.notify('Recovery draft discarded.');
    }catch(error){this.notify(String(error?.message||error))}finally{this.unlock()}
  }
  async retry() {
    if(this.blocked()||!this.getState().client)return;
    this.lock();
    try {await this.getState().client.retryRecovery();this.renderProtection();}
    catch(error){this.notify(String(error?.message||error))}finally{this.unlock()}
  }
}
