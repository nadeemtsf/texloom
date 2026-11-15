import {readFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import {acceptCompilation} from '../../web/core/build-protocol.js';

const [file] = process.argv.slice(2);
if (!file) throw new Error('Provide a disposable synthetic PDF from compile_profile');
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
  const wire = JSON.stringify({
    ok: true, build_id: 'synthetic-build', revision: 1, pdf: Array.from(bytes),
    blocks: [], objects: [], log: '', engine: 'pdflatex', seconds: 0,
    verification_eligible: true, passes: 2, diagnostics: [], job_id: 1,
  });
  const trials = [];
  for (let trial = 0; trial < 6; trial++) {
    const begun = performance.now();
    const reply = acceptCompilation(JSON.parse(wire), context);
    const data = new Uint8Array(reply.pdf);
    const milliseconds = performance.now() - begun;
    if (!bytes.equals(data)) throw new Error('PDF payload bytes changed');
    trials.push({warmup: trial === 0, milliseconds});
  }
  const values = trials.filter(x => !x.warmup).map(x => x.milliseconds).sort((a, b) => a - b);
  console.log(JSON.stringify({
    node: process.version, payload_bytes: bytes.length, json_bytes: Buffer.byteLength(wire),
    median_ms: values[2], trials,
  }));
}
