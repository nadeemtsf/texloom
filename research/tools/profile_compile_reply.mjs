import {readFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {acceptCompilation, decodeCompilation} from '../../web/core/build-protocol.js';

const [file, baselineModule] = process.argv.slice(2);
if (!file || !baselineModule) throw new Error('Provide a disposable synthetic PDF and the preserved baseline validator module');
const {acceptCompilation: acceptBaseline} = await import(pathToFileURL(resolve(baselineModule)).href);
const sample = await readFile(file);
if (sample.length === 0 || sample.length > 1024 * 1024 || !sample.subarray(0, 5).equals(Buffer.from('%PDF-'))) {
  throw new Error('Expected a small synthetic PDF');
}
const context = {revision: 1, sources: {'main.tex': 'Synthetic'}, engine: 'pdflatex'};

for (const mib of [1, 8]) {
  const bytes = Buffer.alloc(mib * 1024 * 1024);
  for (let position = 0; position < bytes.length; position += sample.length) {
    sample.copy(bytes, position);
  }
  const logical = {
    ok: true, build_id: 'synthetic-build', revision: 1, pdf: Array.from(bytes),
    blocks: [], objects: [], log: '', engine: 'pdflatex', seconds: 0,
    verification_eligible: true, passes: 2, diagnostics: [], job_id: 1,
  };
  const wire = JSON.stringify(logical);
  const metadata = new TextEncoder().encode(JSON.stringify({...logical, pdf: []}));
  const envelope = new Uint8Array(8 + metadata.length + bytes.length);
  envelope.set([84, 76, 80, 49]);
  new DataView(envelope.buffer).setUint32(4, metadata.length, true);
  envelope.set(metadata, 8); envelope.set(bytes, 8 + metadata.length);
  const trials = [];
  for (let trial = 0; trial < 6; trial++) {
    const pair = {warmup: trial === 0};
    for (const mode of trial % 2 === 0 ? ['legacy', 'binary'] : ['binary', 'legacy']) {
      const begun = performance.now();
      const reply = mode === 'binary' ? acceptCompilation(decodeCompilation(envelope.buffer), context) : acceptBaseline(JSON.parse(wire), context);
      const data = new Uint8Array(reply.pdf);
      const milliseconds = performance.now() - begun;
      if (!bytes.equals(data)) throw new Error('PDF payload bytes changed');
      pair[mode + '_ms'] = milliseconds;
    }
    pair.saved_ms = pair.legacy_ms - pair.binary_ms;
    trials.push(pair);
  }
  const median = key => trials.filter(x => !x.warmup).map(x => x[key]).sort((a, b) => a - b)[2];
  console.log(JSON.stringify({
    node: process.version, baseline_module: baselineModule, payload_bytes: bytes.length, json_bytes: Buffer.byteLength(wire),
    binary_bytes: envelope.byteLength, metadata_bytes: metadata.length,
    legacy_median_ms: median('legacy_ms'), binary_median_ms: median('binary_ms'),
    paired_saving_median_ms: median('saved_ms'), trials,
  }));
}
