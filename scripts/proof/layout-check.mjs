/** Independent fixture oracle. These checks do not authorize live app edits. */
const finite = n => typeof n === 'number' && Number.isFinite(n);
const compact = s => s.normalize('NFKC').replace(/\s/gu, '');
export class ProofFailure extends Error {}
const requireProof = (value, reason) => { if (!value) throw new ProofFailure(reason); };

export function targetRecords(records, text, occurrence) {
  const needle = compact(text);
  requireProof(needle.length > 0, 'Empty target');
  const haystack = records.map(r => compact(r.text)).join('');
  const matches = [];
  for (let pos = haystack.indexOf(needle); pos >= 0; pos = haystack.indexOf(needle, pos + 1)) matches.push(pos);
  requireProof(matches.length > 0, 'Target is absent from output');
  requireProof(occurrence !== undefined || matches.length === 1, 'Ambiguous repeated target');
  const start = matches[occurrence ?? 0];
  requireProof(start !== undefined, 'Target occurrence is absent');
  const end = start + needle.length, selected = new Set();
  let offset = 0;
  records.forEach((r, i) => {
    const next = offset + compact(r.text).length;
    if (offset < end && next > start) {
      requireProof(offset >= start && next <= end, 'A renderer record crosses the target boundary');
      selected.add(i);
    }
    offset = next;
  });
  return selected;
}

export function checkTranslation(before, after, {text, occurrence, dx, dy, tolerance = 0.02}) {
  requireProof([dx, dy, tolerance].every(finite) && tolerance >= 0, 'Invalid movement policy');
  requireProof(JSON.stringify(before.pages) === JSON.stringify(after.pages), 'Page geometry changed');
  requireProof(before.records.length === after.records.length, 'Output record count changed');
  const target = targetRecords(before.records, text, occurrence);
  const remaining = new Set(after.records.map((_,i)=>i)), selectedAfter = new Set(), mapping = [];
  before.records.forEach((old, i) => {
    const selected=target.has(i);
    requireProof(['x','y','width','height'].every(k=>finite(old[k])), 'Nonfinite output geometry');
    const candidates=[...remaining].filter(j=>{
      const next=after.records[j];
      if(old.text!==next.text || old.page!==next.page)return false;
      return ['x','y','width','height'].every(key=>{
        const delta=selected?(key==='x'?dx:key==='y'?dy:0):0;
        return finite(next[key]) && Math.abs(next[key]-old[key]-delta)<=tolerance;
      });
    });
    requireProof(candidates.length===1, (selected?'Target':'Neighbor')+' has '+candidates.length+
      ' matches at its expected position (record '+i+')');
    const j=candidates[0];remaining.delete(j);mapping.push(j);if(selected)selectedAfter.add(j);
  });
  requireProof(JSON.stringify(before.graphics) === JSON.stringify(after.graphics), 'Non-text object geometry changed');
  return {before:target,after:selectedAfter,mapping};
}

export function checkRaster(before, after, a, b, target, {identity = false, padding = 1} = {}) {
  requireProof(Number.isInteger(padding) && padding >= 0 && padding <= 4, 'Invalid raster padding');
  requireProof(before.rasters.length === after.rasters.length, 'Raster page count changed');
  let checked = 0;
  before.rasters.forEach((old, page) => {
    const next = after.rasters[page];
    requireProof(['width', 'height', 'stride', 'channels', 'scale'].every(k => old[k] === next[k]), 'Raster format changed');
    requireProof(a[page].length === old.stride * old.height && b[page].length === a[page].length, 'Truncated raster');
    const mask = new Uint8Array(old.width * old.height);
    if (!identity) {
      for (const [phase,snapshot] of [before, after].entries()) {
        const selected=target instanceof Set?target:(phase===0?target.before:target.after);
        for (const i of selected) {
        const r = snapshot.records[i];
        if (r.page !== page + 1) continue;
        const x0 = Math.max(0, Math.floor(r.x * old.scale) - padding);
        const x1 = Math.min(old.width, Math.ceil((r.x + r.width) * old.scale) + padding);
        const y0 = Math.max(0, Math.floor(r.y * old.scale) - padding);
        const y1 = Math.min(old.height, Math.ceil((r.y + r.height) * old.scale) + padding);
        for (let j = 0; j < snapshot.records.length; j++) {
          if (selected.has(j)) continue;
          const n = snapshot.records[j];
          if (n.page === r.page && n.x * old.scale < x1 && (n.x + n.width) * old.scale > x0 &&
            n.y * old.scale < y1 && (n.y + n.height) * old.scale > y0)
            throw new ProofFailure('Target mask overlaps unselected text');
        }
        for (const n of snapshot.graphics ?? []) {
          if (n.page === r.page && n.x * old.scale < x1 && (n.x+n.width)*old.scale > x0 &&
            n.y*old.scale < y1 && (n.y+n.height)*old.scale > y0)
            throw new ProofFailure('Target mask overlaps a non-text object');
        }
        for (let y = y0; y < y1; y++) mask.fill(1, y * old.width + x0, y * old.width + x1);
        }
      }
    }
    for (let y = 0; y < old.height; y++) for (let x = 0; x < old.width; x++) {
      if (mask[y * old.width + x]) continue;
      const offset = y * old.stride + x * old.channels;
      for (let c = 0; c < old.channels; c++) requireProof(a[page][offset+c] === b[page][offset+c],
        'Unselected pixels changed on page '+(page+1)+' at '+x+','+y);
      checked++;
    }
  });
  return {unmaskedPixelsChecked: checked};
}
