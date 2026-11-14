import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {checkStagedTree as checkTree} from '../scripts/pre-commit.mjs';
// A hook can export absolute GIT_DIR/GIT_INDEX_FILE for a linked worktree.
// Synthetic repositories must never inherit the owning commit's authority.
const isolatedGitEnv=(env)=>Object.fromEntries(Object.entries(env).filter(([key])=>!key.toUpperCase().startsWith('GIT_')));
const fixtureEnv=isolatedGitEnv(process.env);
const checkStagedTree=(root,check)=>checkTree(root,check,fixtureEnv);
function git(root,...args){const r=spawnSync('git',args,{cwd:root,env:fixtureEnv,encoding:'utf8',windowsHide:true});assert.equal(r.status,0,r.stderr);return r.stdout.trim()}
async function fixture(fn){
 const root=await mkdtemp(join(tmpdir(),'texloom-gate-'));
 try{
  git(root,'init','--quiet');
  await writeFile(join(root,'source.js'),'original\n');git(root,'add','source.js');
  git(root,'-c','core.hooksPath=empty-hooks','-c','user.name=Gate Test','-c','user.email=gate@example.invalid','commit','--quiet','-m','Synthetic baseline');
  await writeFile(join(root,'source.js'),'staged\n');git(root,'add','source.js');await fn(root);
 }finally{await rm(root,{recursive:true,force:true})}
}
test('pre-commit binds a successful check to the exact staged tree',()=>fixture(async root=>{
 let calls=0;const result=await checkStagedTree(root,async()=>{calls++;return 0});
 assert.equal(calls,1);assert.equal(result.tree,git(root,'write-tree'));assert.equal(result.head,git(root,'rev-parse','HEAD'));
}));
test('pre-commit blocks a failed checker instead of returning a passing receipt',()=>fixture(async root=>{
 await assert.rejects(checkStagedTree(root,async()=>1),/Local CI failed/);
}));
test('pre-commit refuses partial staging and missing untracked inputs before tests run',()=>fixture(async root=>{
 const unexpected=async()=>assert.fail('Must refuse before executing checks');
 await writeFile(join(root,'source.js'),'unstaged\n');await assert.rejects(checkStagedTree(root,unexpected),/Stage all tracked/);
 git(root,'add','source.js');await writeFile(join(root,'new.js'),'untracked\n');await assert.rejects(checkStagedTree(root,unexpected),/Stage or ignore/);
}));
test('pre-commit blocks staged changes made during an otherwise successful check',()=>fixture(async root=>{
 await assert.rejects(checkStagedTree(root,async()=>{await writeFile(join(root,'source.js'),'changed during test\n');git(root,'add','source.js');return 0}),/changed during CI/);
}));
test('synthetic gate repositories retain their own index and HEAD under inherited worktree hook settings',()=>fixture(async parent=>{
 const before={tree:git(parent,'write-tree'),head:git(parent,'rev-parse','HEAD')};
 const inherited={...fixtureEnv,GIT_DIR:join(parent,'.git'),GIT_WORK_TREE:parent,GIT_INDEX_FILE:join(parent,'.git','index')};
 await fixture(async child=>{
  await writeFile(join(child,'source.js'),'different child staged source\n');git(child,'add','source.js');
  const checked=await checkTree(child,async()=>0,isolatedGitEnv(inherited));
  assert.equal(checked.tree,git(child,'write-tree'));assert.notEqual(checked.tree,before.tree);
 });
 assert.deepEqual({tree:git(parent,'write-tree'),head:git(parent,'rev-parse','HEAD')},before);
}));
test('Windows CI uses the runtime pinned verifier preparation, never the obsolete Python DLL',async()=>{
 for(const name of ['ci.yml','proof.yml']){
  const yaml=await readFile(new URL('../.github/workflows/'+name,import.meta.url),'utf8');
  assert.ok(yaml.includes('node scripts/prepare-native-verifier.mjs'),name);
  assert.ok(yaml.includes("'src-tauri/resources/verification/pdfium.dll'"),name);
  assert.equal(yaml.includes('.tools/native-pdfium'),false,name);
  assert.equal(yaml.includes('python scripts/proof/prepare_pdfium.py'),false,name);
 }
});
test('the executable hook retains Unix line endings on Windows checkouts',async()=>{
 const attributes=await readFile(new URL('../.gitattributes',import.meta.url),'utf8');
 assert.ok(attributes.includes('.githooks/* text eol=lf'));
 const hook=await readFile(new URL('../.githooks/pre-commit',import.meta.url),'utf8');
 assert.ok(hook.startsWith('#!/bin/sh\n'));assert.ok(hook.includes('exec node scripts/pre-commit.mjs'));
});
test('local CI uses PowerShell 7 consistently across its launcher and packaging child',async()=>{
 const launcher=await readFile(new URL('../scripts/pre-commit.mjs',import.meta.url),'utf8');
 const runner=await readFile(new URL('../scripts/check-ci.ps1',import.meta.url),'utf8');
 assert.ok(launcher.includes("spawnSync('pwsh.exe'"));
 assert.ok(runner.includes("(Join-Path $PSHOME 'pwsh.exe')"));
 assert.equal(launcher.includes("spawnSync('powershell.exe'"),false);
});
