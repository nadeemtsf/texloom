import {spawnSync} from 'node:child_process';
import {mkdir,writeFile} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {randomUUID} from 'node:crypto';

function gitWithEnv(root,env,...args){
 const r=spawnSync('git',args,{cwd:root,env,encoding:'utf8',windowsHide:true});
 if(r.error||r.status!==0)throw Error(r.error?.message||r.stderr||'Git snapshot check failed');
 return r.stdout.trim();
}
function git(root,...args){return gitWithEnv(root,process.env,...args)}
function snapshot(root,env){
 const run=(...args)=>gitWithEnv(root,env,...args);
 if(run('diff','--name-only'))throw Error('Stage all tracked changes before committing: CI must test the same files as the commit.');
 if(run('ls-files','--others','--exclude-standard'))throw Error('Stage or ignore new files before committing: CI cannot rely on untracked files.');
 return {tree:run('write-tree'),head:run('rev-parse','HEAD')};
}
export async function checkStagedTree(root,runChecks,env=process.env){
 const before=snapshot(root,env);
 if(await runChecks(before)!==0)throw Error('Local CI failed. Commit blocked; inspect the saved step logs.');
 const after=snapshot(root,env);
 if(after.tree!==before.tree||after.head!==before.head)throw Error('The staged tree or HEAD changed during CI. Commit blocked; stage and test again.');
 return before;
}
async function main(){
 const root=git(process.cwd(),'rev-parse','--show-toplevel');
 const output=join(root,'artifacts/local-ci',new Date().toISOString().replaceAll(/[:.]/g,'-')+'-'+randomUUID());
 await mkdir(output,{recursive:true});
 const receipt={schema:1,kind:'local_precommit_gate',started_utc:new Date().toISOString(),status:'running',platform:process.platform,
  scope:'Staged-tree local checks, not hosted CI, physical UI acceptance or a release attestation'};
 try{
  Object.assign(receipt,await checkStagedTree(root,async()=>{
   if(process.platform!=='win32')throw Error('The full native pre-commit gate requires Windows x64.');
   const r=spawnSync('pwsh.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/check-ci.ps1','-ArtifactRoot',output],
    {cwd:root,stdio:'inherit',windowsHide:true,timeout:3_600_000});
   if(r.error)throw r.error;return r.status;
  }));receipt.status='passed';console.log('Local CI passed for staged tree '+receipt.tree);
 }catch(error){receipt.status='failed';receipt.error=error.message;console.error(error.message);process.exitCode=1}
 receipt.finished_utc=new Date().toISOString();
 await writeFile(join(output,'pre-commit.json'),JSON.stringify(receipt,null,2)+'\n');
 console.log('Pre-commit evidence: '+output);
}
if(process.argv[1]&&pathToFileURL(resolve(process.argv[1])).href===import.meta.url)await main();
