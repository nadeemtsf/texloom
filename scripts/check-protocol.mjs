// Compile strict protocol sources and check their committed runtime output.
// Checks do not rewrite source/fixtures. --write is an explicit developer step.
import ts from 'typescript';
import {readFile, writeFile} from 'node:fs/promises';
import {resolve, dirname, basename} from 'node:path';
import {fileURLToPath} from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const configPath = resolve(root, 'tsconfig.protocol.json');
const config = ts.readConfigFile(configPath, ts.sys.readFile);
const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, root);
const outputs = new Map();
const program = ts.createProgram(parsed.fileNames, parsed.options);
const emitted = program.emit(undefined, (path, text) => outputs.set(basename(path), text));
const contracts = ts.createProgram([resolve(root, 'tests/types/document-protocol.test.ts')], {
  ...parsed.options, noEmit: true, rootDir: root,
});
const diagnostics = [config.error, ...parsed.errors, ...ts.getPreEmitDiagnostics(program), ...emitted.diagnostics,
  ...ts.getPreEmitDiagnostics(contracts)].filter(Boolean);
if (diagnostics.length) {
  process.stderr.write(ts.formatDiagnosticsWithColorAndContext(diagnostics, {
    getCurrentDirectory: () => root, getCanonicalFileName: name => name, getNewLine: () => '\n',
  }));
  process.exit(1);
}
const names = ['document-client.js', 'document-protocol.js', 'build-protocol.js'];
if (outputs.size !== names.length || names.some(name => !outputs.has(name))) throw Error('Unexpected protocol compiler output');
for (const name of names) {
  const output = '// Generated from frontend/protocol/' + name.replace(/\.js$/, '.ts') + '. Run npm run protocol:emit; do not edit.\n' + outputs.get(name);
  const path = resolve(root, 'web/core', name);
  if (process.argv.includes('--write')) await writeFile(path, output);
  else if ((await readFile(path, 'utf8')).replace(/\r\n/g, '\n') !== output) throw Error(name + ' is stale. Run npm run protocol:emit.');
}
console.log(process.argv.includes('--write') ? 'Generated strict document protocol runtime.' : 'Strict document protocol types and generated runtime match.');
