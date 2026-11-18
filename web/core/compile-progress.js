/** The channel belongs to one invocation; late/optional failures never replace its result. */
export function createCompileInvoke(native,makeChannel,hooks){
  return async(name,args={})=>{
    if(name!=='compile_document'||args.token!==null||args.purpose!=='preview')return native(name,args);
    const request={...args};let ticket,channel,accepting=true;
    try{
      ticket=hooks.start(request);
      if(!ticket)return native(name,args);
      channel=makeChannel(raw=>{
        if(!accepting)return;
        try{Promise.resolve(hooks.message(raw,ticket)).catch(()=>{})}catch{}
      });
    }catch{
      if(ticket)try{hooks.end(ticket)}catch{}
      return native(name,args);
    }
    try{return await native(name,{...request,onPreview:channel})}
    finally{
      accepting=false;channel.onmessage=()=>{};
      try{hooks.end(ticket)}catch{}
    }
  };
}
