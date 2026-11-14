import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
import {resolve,dirname,join} from 'node:path';
import {recordedDesktopBuild} from './desktop-build-receipt.mjs';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),args=process.argv.slice(2);
try{
  // Restrict path-changing overrides rather than guessing where Tauri wrote an EXE.
  for(const flag of args)if(flag==='-c'||/^--(?:profile|config|runner)(?:=|$)/.test(flag))throw Error('Custom profile/config/runner builds need their own artifact binding; this command refuses to guess.');
  const targets=args.flatMap((arg,i)=>arg==='--target'||arg==='-t'?[args[i+1]]:arg.startsWith('--target=')?[arg.slice(9)]:[]);
  if(targets.length>1||targets.some(t=>t!=='x86_64-pc-windows-msvc'))throw Error('Recorded desktop packaging supports one Windows x64 target');
  const target=targets[0]||process.env.CARGO_BUILD_TARGET;
  if(target&&target!=='x86_64-pc-windows-msvc')throw Error('Unsupported CARGO_BUILD_TARGET');
  const base=resolve(root,process.env.CARGO_TARGET_DIR||'target');
  const executable=join(base,...(target?[target]:[]),args.includes('--debug')?'debug':'release','texloom-desktop.exe');
  const git=spawnSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8',windowsHide:true});
  const clean=spawnSync('git',['diff','HEAD','--quiet'],{cwd:root,windowsHide:true});
  const cli=createRequire(import.meta.url).resolve('@tauri-apps/cli/tauri.js');
  // A pre-commit build includes staged changes; the old HEAD is not its source.
  // The gate records its tested tree separately, so don't mislabel that binary.
  const receipt=await recordedDesktopBuild({root,executable,sourceHead:git.status===0&&clean.status===0?git.stdout.trim():null,
    traceEnabled:process.env.VITE_TEXLOOM_PERFORMANCE==='1',build:async()=>{
      const result=spawnSync(process.execPath,[cli,'build',...args],{cwd:root,stdio:'inherit',windowsHide:true,timeout:1_800_000});
      if(result.error||result.status!==0)throw Error(result.error?.message||'Desktop build failed');
    }});
  console.log('Recorded desktop/verifier pairing '+receipt.desktop.sha256);
}catch(error){console.error(error.message);process.exitCode=1}
