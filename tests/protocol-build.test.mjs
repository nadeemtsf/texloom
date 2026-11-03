import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('protocol build checks reject stale runtime and invalid TypeScript without rewriting either file', {timeout: 60_000}, () => {
  const parent = path.join(root, '.tools', 'protocol-build-tests');
  fs.mkdirSync(parent, {recursive: true});
  const fixture = fs.mkdtempSync(path.join(parent, 'owned-'));
  try {
    const files = ['scripts/check-protocol.mjs', 'tsconfig.protocol.json',
      'frontend/protocol/document-client.ts', 'frontend/protocol/document-protocol.ts', 'frontend/protocol/build-protocol.ts',
      'web/generated/document-ipc.d.ts', 'web/core/document-client.js', 'web/core/document-protocol.js', 'web/core/build-protocol.js',
      'tests/types/document-protocol.test.ts'];
    for (const relative of files) {
      const destination = path.join(fixture, relative);
      fs.mkdirSync(path.dirname(destination), {recursive: true});
      fs.copyFileSync(path.join(root, relative), destination);
    }
    const check = () => spawnSync(process.execPath, [path.join(fixture, 'scripts/check-protocol.mjs'), '--check'],
      {cwd: fixture, encoding: 'utf8', windowsHide: true, timeout: 25_000});
    let result = check(); assert.ifError(result.error); assert.equal(result.status, 0, result.stderr);
    const runtime = path.join(fixture, 'web/core/document-client.js');
    fs.writeFileSync(runtime, '// intentionally stale test output\n');
    result = check(); assert.ifError(result.error); assert.notEqual(result.status, 0); assert.match(result.stderr, /is stale/);
    assert.equal(fs.readFileSync(runtime, 'utf8'), '// intentionally stale test output\n');
    const source = path.join(fixture, 'frontend/protocol/document-client.ts');
    fs.appendFileSync(source, '\nconst brokenContract: string = 123;\n');
    result = check(); assert.ifError(result.error); assert.notEqual(result.status, 0); assert.match(result.stderr, /not assignable/);
    assert.equal(fs.readFileSync(runtime, 'utf8'), '// intentionally stale test output\n');
    assert.match(fs.readFileSync(source, 'utf8'), /brokenContract: string = 123/);
  } finally {
    const actual = fs.realpathSync(fixture), workspace = fs.realpathSync(root);
    assert.ok(actual.startsWith(workspace + path.sep) && path.dirname(actual) === fs.realpathSync(parent));
    assert.ok(!fs.lstatSync(fixture).isSymbolicLink());
    fs.rmSync(actual, {recursive: true});
  }
});
