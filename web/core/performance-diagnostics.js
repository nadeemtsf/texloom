const kinds=new Set(['source-feedback','source-selection','object-selection','drag-preview','text-move','text-edit','compile-manual','compile-auto']);
const phases=new Set(['source-queued','feedback-updated','mapping-resolved','planned','native-prepared','build-returned','pdf-prepared','frontend-validated','build-rechecked','native-committed','published','selection-ready','preview-updated','sync-flushed']);
const outcomes=new Set(['feedback','selected','published','refused','failed','compile-failed','stale','cancelled','superseded']);

export function performanceOutcome(error){
  if(error?.name==='StaleCandidate')return 'stale';
  if(error?.name==='CompilationCancelled')return 'cancelled';
  if(error?.name==='CompilationFailure')return 'compile-failed';
  return 'failed';
}

/** Local, bounded diagnostics. No source, paths, text, error messages or network output. */
export function createPerformanceDiagnostics({enabled=false,now=()=>performance.now(),
  requestFrame=globalThis.requestAnimationFrame?.bind(globalThis),cancelFrame=globalThis.cancelAnimationFrame?.bind(globalThis),
  document=globalThis.document,setTimer=setTimeout,clearTimer=clearTimeout,capacity=256,maxActive=16,frameTimeoutMs=2000}={}){
  capacity=Number.isInteger(capacity)&&capacity>0?Math.min(capacity,4096):256;
  maxActive=Number.isInteger(maxActive)&&maxActive>0?Math.min(maxActive,64):16;
  const records=[],active=new Map();let sequence=0,evicted=0,dropped=0,stopped=!enabled;
  const snapshot=()=>({schema:1,clock:'performance.now',endpoint:'two-animation-frames-proxy',
    enabled:!stopped,capacity,evicted,dropped,active:active.size,records:structuredClone(records),pending:structuredClone([...active.values()])});
  function begin(kind){
    if(stopped||!kinds.has(kind))return null;
    if(active.size>=maxActive){dropped++;return null}
    const started=now();if(!Number.isFinite(started))return null;
    const record={id:++sequence,kind,started_ms:started,phases:[],truncated_phases:0,outcome:null,operation_ms:null,frame_proxy_ms:null,frame_status:'not-requested'};
    let finished=false,ended=false,timer=null,frame=null,last=started,clockValid=true;
    const read=()=>{const value=now();if(!clockValid||!Number.isFinite(value)||value<last){clockValid=false;return null}last=value;return value-started};
    const release=()=>{if(timer!==null)clearTimer(timer);if(frame!==null)cancelFrame?.(frame);active.delete(span)};
    const save=status=>{
      if(ended)return;ended=true;record.frame_status=status;release();
      if(records.length>=capacity){records.shift();evicted++}records.push(record);
    };
    const span={
      mark(name){if(finished||!phases.has(name))return;if(record.phases.length>=64){record.truncated_phases++;return}const offset=read();if(offset!==null)record.phases.push({name,offset_ms:offset})},
      finish(outcome,{paint=false,isCurrent=()=>true}={}){
        if(finished||!outcomes.has(outcome))return;finished=true;record.outcome=outcome;record.operation_ms=read();
        if(!paint)return save('not-requested');
        record.frame_status='pending';
        if(!requestFrame)return save('unavailable');
        const valid=()=>{
          if(document?.visibilityState!=='visible'){save('hidden');return false}
          let current=false;try{current=isCurrent()}catch{}
          if(!current){save('superseded');return false}return true;
        };
        if(!valid())return;
        timer=setTimer(()=>save('timeout'),frameTimeoutMs);
        frame=requestFrame(()=>{if(ended||!valid())return;frame=requestFrame(()=>{
          if(ended||!valid())return;
          record.frame_proxy_ms=read();save(record.frame_proxy_ms===null?'invalid-clock':'observed');
        })});
      },
      discard(){if(ended)return;finished=true;ended=true;release()},
      stop(){finished=true;save('stopped')}
    };
    active.set(span,record);return span;
  }
  return Object.freeze({begin,snapshot,clear(){for(const span of active.keys())span.discard();records.length=0;evicted=0;dropped=0},
    stop(){stopped=true;for(const span of active.keys())span.stop()}});
}

/** A separately identified diagnostic build opts in; normal builds expose no API. */
export function installPerformanceDiagnostics({host=globalThis.window,enabled=import.meta.env?.VITE_TEXLOOM_PERFORMANCE==='1',...options}={}){
  const diagnostics=createPerformanceDiagnostics({enabled,...options});
  if(enabled&&host)Object.defineProperty(host,'texloomPerformance',{configurable:true,value:Object.freeze({
    snapshot:diagnostics.snapshot,clear:diagnostics.clear,stop:diagnostics.stop
  })});
  return diagnostics;
}
