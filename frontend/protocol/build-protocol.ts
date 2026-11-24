/** Validate compiler/mapping wire data. Native verification still owns commit authority. */
import type { BuildRequests, CompileResult, UnverifiedCompilePreview, SourceLocation, SourceRectangle, ResolvedSelection } from '../../web/generated/document-ipc.js';
import type { NativeInvoke, Sources } from './document-protocol.js';
import { editorText } from './document-protocol.js';

export class BuildProtocolError extends Error {
  constructor(message: string) { super(message); this.name = 'BuildProtocolError'; }
}
function require(ok: unknown, message: string): asserts ok {
  if (!ok) throw new BuildProtocolError(message);
}
function record(value: unknown, keys: readonly string[]): asserts value is Record<string, unknown> {
  require(value !== null && typeof value === 'object' && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key)), 'Invalid build reply fields');
}
const integer = (value: unknown, minimum = 0): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value >= minimum;
const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);
const identity = (value: unknown): value is string => typeof value === 'string' && value.length > 0 && value.length <= 256 && !/[\s\x00-\x1f]/u.test(value);
function sourceText(sources: Sources, file: unknown): string {
  require(typeof file === 'string' && Object.hasOwn(sources, file), 'Build reply refers to an unknown source file');
  const source = sources[file];
  require(typeof source === 'string', 'Invalid mapped source');
  return editorText(source);
}
function boundary(text: string, offset: unknown): offset is number {
  if (!integer(offset) || offset > text.length) return false;
  const before = text.charCodeAt(offset - 1), after = text.charCodeAt(offset);
  return !(before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff);
}
function sourceRange(value: Record<string, unknown>, sources: Sources): void {
  const text = sourceText(sources, value.file);
  require(boundary(text, value.start) && boundary(text, value.end) && value.start < value.end &&
    integer(value.line, 1) && value.line <= text.split('\n').length, 'Invalid compiled object source range');
}
export type CompileContext = { revision: number; sources: Sources; engine: string };
/** Decode only the native compile envelope; logical replies remain strictly validated below. */
export function decodeCompilation(value: unknown): unknown {
  // Tauri's postMessage fallback serializes raw replies as number arrays.
  if (Array.isArray(value)) {
    require(value.length <= 8 + 124 * 1024 * 1024, 'Compile envelope is too large');
    for (const byte of value) require(integer(byte) && byte <= 255, 'Invalid compile envelope byte');
    value = Uint8Array.from(value).buffer;
  }
  if (!(value instanceof ArrayBuffer)) return value;
  require(value.byteLength >= 8, 'Truncated compile envelope');
  const bytes = new Uint8Array(value);
  require([84, 76, 80, 49].every((byte, index) => bytes[index] === byte), 'Unknown compile envelope version');
  const length = new DataView(value).getUint32(4, true);
  require(length > 0 && length <= 64 * 1024 * 1024 && length <= value.byteLength - 8, 'Invalid compile metadata length');
  require(value.byteLength - 8 - length <= 60 * 1024 * 1024, 'Invalid compiled PDF bytes');
  let metadata: unknown;
  try { metadata = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes.subarray(8, 8 + length))); }
  catch { throw new BuildProtocolError('Invalid compile metadata encoding'); }
  require(metadata !== null && typeof metadata === 'object' && !Array.isArray(metadata), 'Invalid compile metadata');
  const reply = metadata as Record<string, unknown>;
  require(Array.isArray(reply.pdf) && reply.pdf.length === 0, 'Compile metadata contains PDF bytes');
  reply.pdf = bytes.subarray(8 + length);
  return reply;
}
export function acceptCompilation(value: unknown, context: CompileContext): CompileResult {
  assertCompilation(value, context);
  return value;
}
function assertCompilation(value: unknown, context: CompileContext): asserts value is CompileResult {
  record(value, ['ok', 'build_id', 'revision', 'pdf', 'objects', 'log', 'engine', 'seconds',
    'verification_eligible', 'passes', 'diagnostics', 'job_id']);
  require(typeof value.ok === 'boolean' && typeof value.verification_eligible === 'boolean' &&
    identity(value.build_id) && integer(value.revision) && value.revision === context.revision &&
    integer(value.job_id, 1), 'Invalid build identity or revision');
  require(typeof value.log === 'string' && finite(value.seconds) && value.seconds >= 0 &&
    integer(value.passes) && value.passes <= 6 && typeof value.engine === 'string' &&
    ['pdflatex', 'xelatex', 'lualatex'].includes(value.engine) &&
    (context.engine === 'auto' || context.engine === value.engine), 'Invalid compiler metadata');
  const errors = diagnosticErrors(value.diagnostics);
  pdfBytes(value.pdf);
  require(Array.isArray(value.objects), 'Invalid compiled object inventory');
  if (!value.ok) {
    require(!value.verification_eligible && value.pdf.length === 0 && value.objects.length === 0,
      'A failed compilation cannot supply PDF output or visual objects');
  } else {
    const pdf = value.pdf;
    require(value.passes > 0 && [37, 80, 68, 70, 45].every((byte, index) => pdf[index] === byte), 'Successful compilation has no PDF header');
  }
  require(!value.verification_eligible || value.ok && !errors, 'Invalid build verification eligibility');
  assertInventory(value, context);
}
function diagnosticErrors(value: unknown): boolean {
  require(Array.isArray(value), 'Invalid compiler diagnostics');
  let errors = false;
  for (const diagnostic of value) {
    record(diagnostic, ['code', 'severity', 'message', 'file', 'line']);
    require(typeof diagnostic.code === 'string' && diagnostic.code.length > 0 && typeof diagnostic.message === 'string' &&
      typeof diagnostic.severity === 'string' && ['error', 'warning', 'info'].includes(diagnostic.severity) &&
      (diagnostic.file === null || typeof diagnostic.file === 'string' && diagnostic.file.length > 0) &&
      (diagnostic.line === null || integer(diagnostic.line, 1)), 'Invalid compiler diagnostic');
    if (diagnostic.severity === 'error') errors = true;
  }
  return errors;
}
function pdfBytes(value: unknown): asserts value is number[] | Uint8Array {
  require((Array.isArray(value) || value instanceof Uint8Array) && value.length <= 60 * 1024 * 1024, 'Invalid compiled PDF bytes');
  if (Array.isArray(value)) for (const byte of value) require(integer(byte) && byte <= 255, 'Invalid compiled PDF bytes');
}
function assertInventory(value: Record<string, unknown>, context: CompileContext): void {
  require(Array.isArray(value.objects), 'Invalid compiled object inventory');
  const objectIds = new Set<string>();
  for (const object of value.objects) {
    record(object, ['id', 'file', 'dx', 'dy', 'plain_text', 'content', 'line', 'start', 'end']);
    require(identity(object.id) && !objectIds.has(object.id), 'Invalid or duplicate text object identity');
    objectIds.add(object.id); sourceRange(object, context.sources);
    require(finite(object.dx) && finite(object.dy) && (object.plain_text === null || typeof object.plain_text === 'string') && typeof object.content === 'string', 'Invalid text object metadata');
  }
}

export type PreviewContext = {projectId: string; revision: number; requestId: number; engine: string};
export function acceptUnverifiedPreview(raw: unknown, context: PreviewContext): UnverifiedCompilePreview {
  const value = decodeCompilation(raw);
  record(value, ['schema', 'phase', 'project_id', 'revision', 'request_id', 'engine', 'seconds', 'diagnostics', 'pdf']);
  require(value.schema === 1 && value.phase === 'first_pass' && identity(value.project_id) &&
    value.project_id === context.projectId && integer(value.revision) && value.revision === context.revision &&
    integer(value.request_id, 1) && value.request_id === context.requestId, 'Invalid preliminary preview identity');
  require(typeof value.engine === 'string' && ['pdflatex', 'xelatex', 'lualatex'].includes(value.engine) &&
    (context.engine === 'auto' || value.engine === context.engine) && finite(value.seconds) && value.seconds >= 0,
    'Invalid preliminary preview metadata');
  diagnosticErrors(value.diagnostics); pdfBytes(value.pdf);
  require([37, 80, 68, 70, 45].every((byte, index) => (value.pdf as Uint8Array)[index] === byte), 'Preliminary preview has no PDF header');
  return value as UnverifiedCompilePreview;
}

export function acceptSourceLocation(value: unknown, sources: Sources): SourceLocation {
  assertSourceLocation(value, sources); return value;
}
function assertSourceLocation(value: unknown, sources: Sources): asserts value is SourceLocation {
  record(value, ['file', 'line']);
  const text = sourceText(sources, value.file);
  require(integer(value.line, 1) && value.line <= text.split('\n').length, 'Invalid mapped source line');
}
export function acceptSourceRectangle(value: unknown): SourceRectangle {
  assertSourceRectangle(value); return value;
}
function assertSourceRectangle(value: unknown): asserts value is SourceRectangle {
  record(value, ['page', 'x', 'y', 'width', 'height']);
  require(integer(value.page, 1) && finite(value.x) && finite(value.y) && finite(value.width) && value.width > 0 &&
    finite(value.height) && value.height > 0, 'Invalid mapped PDF rectangle');
}
export function assertSourceLine(sources: Sources, file: string, line: number): void {
  require(integer(line, 1) && line <= sourceText(sources, file).split('\n').length, 'Invalid requested source line');
}
type SelectionContext = { snapshot: CompileContext; buildId: string; page: number };
export function acceptResolvedSelection(value: unknown, context: SelectionContext): ResolvedSelection {
  assertResolvedSelection(value, context); return value;
}
function assertResolvedSelection(value: unknown, {snapshot, buildId, page}: SelectionContext): asserts value is ResolvedSelection {
  record(value, ['schema', 'unit_id', 'build_id', 'revision', 'file', 'expected_source', 'start', 'end',
    'first_line', 'last_line', 'kind', 'content', 'plain_text', 'source_offset_bp', 'rectangles']);
  require(value.schema === 1 && typeof value.unit_id === 'string' && value.unit_id.length > 0 && value.unit_id.length <= 8192 &&
    value.build_id === buildId && value.revision === snapshot.revision, 'Invalid mapped selection identity');
  const text = sourceText(snapshot.sources, value.file);
  require(typeof value.file === 'string' && value.expected_source === snapshot.sources[value.file] &&
    boundary(text, value.start) && boundary(text, value.end) && value.start < value.end && typeof value.content === 'string' &&
    text.slice(value.start, value.end) === value.content.replace(/\r\n?|\n/gu, '\n'), 'Mapped selection does not match the current source');
  require(integer(value.first_line, 1) && integer(value.last_line, value.first_line) && value.last_line <= text.split('\n').length &&
    typeof value.kind === 'string' && ['line', 'paragraph', 'group'].includes(value.kind) &&
    (value.plain_text === null || typeof value.plain_text === 'string') && Array.isArray(value.rectangles) && value.rectangles.length <= 64,
    'Invalid mapped selection metadata');
  const position = value.source_offset_bp;
  require(position === null || (Array.isArray(position) && position.length === 2
    && [position[0], position[1]].every(offset => finite(offset) && Math.abs(offset) < 10_000)), 'Invalid mapped position offset');
  for (const rectangle of value.rectangles) require(acceptSourceRectangle(rectangle).page === page, 'Mapped selection crossed a page boundary');
}
export async function requestBuild<C extends keyof BuildRequests>(invoke: NativeInvoke, name: C, args: BuildRequests[C]): Promise<unknown> {
  const reply = await invoke(name, { ...args });
  return name === 'compile_document' ? decodeCompilation(reply) : reply;
}
type UnitCommand = 'verify_build' | 'pin_displayed_build' | 'cancel_compilation' | 'document_discard' | 'export_document_build';
export async function invokeUnit<C extends UnitCommand>(invoke: NativeInvoke, name: C, args: BuildRequests[C]): Promise<void> {
  require(await requestBuild(invoke, name, args) === null, 'Invalid ' + name + ' acknowledgement');
}
