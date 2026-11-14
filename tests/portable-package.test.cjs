const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {spawnSync} = require('node:child_process');

const root = path.resolve(__dirname, '..');
const parent = path.join(root, '.tools', 'portable-test-runs');
// Prefer the CI runtime, but retain the system PowerShell on Windows machines
// that have not installed version 7. Never change execution policy for tests.
const hasPwsh = (process.env.PATH || '').split(path.delimiter)
  .some(folder => fs.existsSync(path.join(folder.replace(/^"|"$/g, ''), 'pwsh.exe')));
const powershell = hasPwsh ? 'pwsh.exe' :
  path.join(process.env.SystemRoot || 'C:\\Windows', 'System32', 'WindowsPowerShell', 'v1.0', 'powershell.exe');
const cases = [
  ['isolation', 'portable packaging excludes old projects, preserves old archives and verifies exact Unicode bytes'],
  ['unknown', 'portable packaging refuses an unreceipted private file before creating a ZIP'],
  ['tampered', 'portable packaging refuses staged contents changed after their receipt'],
  ['existing', 'portable packaging cannot overwrite an archive that appears after name selection'],
  ['paths', 'portable packaging rejects escaping, duplicate and missing file receipts'],
  ['link', 'portable packaging refuses staging junctions without reading or changing their targets'],
];

for (const [name, description] of cases) {
  test(description, {skip: process.platform !== 'win32', timeout: 60_000}, t => {
    fs.mkdirSync(parent, {recursive: true});
    const fixture = fs.mkdtempSync(path.join(parent, name + '-'));
    try {
      const result = spawnSync(powershell, ['-NoProfile', '-NonInteractive', '-File',
        path.join(root, 'tests', 'fixtures', 'portable-archive-harness.ps1'),
        '-FixtureRoot', fixture, '-ArchiveHelper', path.join(root, 'scripts', 'portable-archive.ps1'), '-Case', name],
      {cwd: root, encoding: 'utf8', timeout: 55_000, windowsHide: true});
      assert.ifError(result.error);
      assert.equal(result.status, 0, result.stderr || result.stdout);
      const outcome = JSON.parse(result.stdout.trim());
      if (name === 'link' && outcome.passed === false && outcome.skip) {
        t.skip(outcome.skip);
        return;
      }
      assert.deepEqual(outcome, {case: name, passed: true});
    } finally {
      const resolved = path.resolve(fixture);
      assert.ok(resolved.startsWith(parent + path.sep), 'Cleanup must stay in the owned test directory');
      assert.ok(!fs.lstatSync(resolved).isSymbolicLink(), 'Cleanup root must be a real directory');
      fs.rmSync(resolved, {recursive: true});
    }
  });
}
