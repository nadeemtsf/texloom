import {test} from 'node:test';
import assert from 'node:assert/strict';
import {PDFSearch} from '../web/core/pdf-search.js';

function setup(){
  const elements=new Map(),events=[],errors=[],pending=new Map();
  const element=id=>elements.get(id)||elements.set(id,{value:'',textContent:''}).get(id);
  const view={pages:[1,2].map(n=>({shell:{classList:{add:()=>events.push(n)},scrollIntoView(){}}})),
    ensurePage:n=>new Promise((resolve,reject)=>pending.set(n,{resolve,reject}))};
  const search=new PDFSearch({document:{getElementById:element,querySelectorAll:()=>[]},getView:()=>view,notify:e=>errors.push(e)});
  search.highlight=(_page,occurrence)=>events.push('highlight '+occurrence);
  search.input.value='target';search.searchedQuery='target';search.matches=[{page:1,occurrence:0},{page:2,occurrence:1}];
  return {search,events,errors,pending};
}
test('a slower earlier navigation cannot replace the newest PDF match',async()=>{
  const {search,events,pending}=setup();
  const first=search.step(1),second=search.step(1);
  pending.get(2).resolve();await second;pending.get(1).resolve();await first;
  assert.deepEqual(events,[2,'highlight 1']);assert.equal(search.count.textContent,'2 / 2');
});
test('reset while a page loads prevents stale highlighting and match counts',async()=>{
  const {search,events,pending}=setup();const first=search.step(1);search.reset();
  pending.get(1).resolve();await first;
  assert.deepEqual(events,[]);assert.equal(search.count.textContent,'');
});
test('failure of superseded navigation does not report a new error',async()=>{
  const {search,errors,pending}=setup();const first=search.step(1),second=search.step(1);
  pending.get(2).resolve();await second;pending.get(1).reject(Error('old page unavailable'));await first;
  assert.deepEqual(errors,[]);assert.equal(search.count.textContent,'2 / 2');
});
