import {inside,union} from './geometry.js';

const fields=['x','y','width','height','baseline','size'];
function sameRun(a,b){return a.page===b.page&&a.text===b.text&&fields.every(k=>Number.isFinite(a[k])&&Number.isFinite(b[k])&&Math.abs(a[k]-b[k])<=1e-6)}
function pairRuns(before,after){
  if(before.length!==after.length)throw new Error('The selection probe changed the document text. Use source editing.');
  // Preserve reading order; no coordinate rounding or large target masks.
  for(let i=0;i<before.length;i++)if(!sameRun(before[i],after[i]))throw new Error('The selection probe changed text or layout. Use source editing.');
}
export function canvasRaster(page){
  const canvas=page.pageView?.canvas||page.shell.querySelector('canvas');
  if(!canvas||!canvas.width||!canvas.height||canvas.width*canvas.height>16*1024*1024)throw new Error('This page exceeds the selection verification limit. Use source editing.');
  return {width:canvas.width,height:canvas.height,data:canvas.getContext('2d',{willReadFrequently:true}).getImageData(0,0,canvas.width,canvas.height).data};
}

/** A SyncTeX hint gains a capability only after a zero-offset private build.
 * Canonical source and the displayed PDF are never changed by this function.
 * This is same-renderer identity verification; independent validation is P6.
 */
export function validateSelectionProbe(before,probe,id,hits,page,{readRaster=canvasRaster,maxBytes=96*1024*1024}={}){
  if(before.pageCount!==probe.pageCount||before.pages.length!==probe.pages.length)throw new Error('The selection probe changed the page count.');
  const target=probe.objects.get(id);
  if(!target||target.pages.size!==1||target.page!==page||probe.unclosed.has(id))throw new Error('This source does not identify one complete text object on this page.');
  if(target.instances!==1)throw new Error('This source is used more than once in the PDF. Select it in the source editor.');
  if(!inside(target,probe.pageSizes.get(page)))throw new Error('The selected text crosses the page edge.');
  let bytes=0;
  for(let i=0;i<before.pages.length;i++){
    const a=before.pages[i],b=probe.pages[i],ap=before.pageSizes.get(a.number),bp=probe.pageSizes.get(b.number);
    if(a.selectionReason||b.selectionReason)throw new Error(a.selectionReason||b.selectionReason);
    if(a.number!==b.number||ap.width!==bp.width||ap.height!==bp.height)throw new Error('The selection probe changed page geometry.');
    pairRuns(a.runs,b.runs);
    const x=readRaster(a),y=readRaster(b);bytes+=x.data.length;
    if(bytes>maxBytes)throw new Error('This document exceeds the selection verification limit. Use source editing.');
    if(x.width!==y.width||x.height!==y.height||x.data.length!==y.data.length)throw new Error('The preview scale changed. Select the text again.');
    for(let n=0;n<x.data.length;n++)if(x.data[n]!==y.data[n])throw new Error('The selection probe changed the page appearance. Use source editing.');
  }
  const baseline=before.pages.find(p=>p.number===page);
  const runs=target.runs.map(run=>{
    const matches=baseline.runs.filter(r=>sameRun(r,run));
    if(matches.length!==1)throw new Error('Overlapping or repeated text makes this selection ambiguous.');
    return matches[0];
  });
  if(!hits.length||hits.some(h=>h.page!==page||!h.runs.length||h.runs.some(r=>!runs.includes(r))))throw new Error('The mapped source does not produce all the clicked text. Select it in the source editor.');
  const lines=baseline.lines.filter(line=>line.runs.some(run=>runs.includes(run)));
  return {schema:1,build_id:before.result.build_id,page,...union(runs),runs,lines};
}
