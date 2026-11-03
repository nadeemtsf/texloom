/** Validate compiler/mapping wire data. Native verification still owns commit authority. */
import type { BuildRequests, CompileResult, SourceLocation, SourceRectangle, ResolvedSelection } from '../../web/generated/document-ipc.js';
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
export function acceptCompilation(value: unknown, context: CompileContext): CompileResult {
  assertCompilation(value, context);
  return value;
}
function assertCompilation(value: unknown, context: CompileContext): asserts value is CompileResult {
  record(value, ['ok', 'build_id', 'revision', 'pdf', 'blocks', 'objects', 'log', 'engine', 'seconds',
    'verification_eligible', 'passes', 'diagnostics', 'job_id']);
  require(typeof value.ok === 'boolean' && typeof value.verification_eligible === 'boolean' &&
    identity(value.build_id) && integer(value.revision) && value.revision === context.revision &&
    integer(value.job_id, 1), 'Invalid build identity or revision');
  require(typeof value.log === 'string' && finite(value.seconds) && value.seconds >= 0 &&
    integer(value.passes) && value.passes <= 6 && typeof value.engine === 'string' &&
    ['pdflatex', 'xelatex', 'lualatex'].includes(value.engine) &&
    (context.engine === 'auto' || context.engine === value.engine), 'Invalid compiler metadata');
  require(Array.isArray(value.diagnostics), 'Invalid compiler diagnostics');
  let errors = false;
  for (const diagnostic of value.diagnostics) {
    record(diagnostic, ['code', 'severity', 'message', 'file', 'line']);
    require(typeof diagnostic.code === 'string' && diagnostic.code.length > 0 && typeof diagnostic.message === 'string' &&
      typeof diagnostic.severity === 'string' && ['error', 'warning', 'info'].includes(diagnostic.severity) &&
      (diagnostic.file === null || typeof diagnostic.file === 'string' && diagnostic.file.length > 0) &&
      (diagnostic.line === null || integer(diagnostic.line, 1)), 'Invalid compiler diagnostic');
    if (diagnostic.severity === 'error') errors = true;
  }
  require(Array.isArray(value.pdf) && value.pdf.length <= 60 * 1024 * 1024, 'Invalid compiled PDF bytes');
  for (const byte of value.pdf) require(integer(byte) && byte <= 255, 'Invalid compiled PDF bytes');
  require(Array.isArray(value.blocks) && Array.isArray(value.objects), 'Invalid compiled object inventory');
  if (!value.ok) {
    require(!value.verification_eligible && value.pdf.length === 0 && value.blocks.length === 0 && value.objects.length === 0,
      'A failed compilation cannot supply PDF output or visual objects');
  } else {
    const pdf = value.pdf;
    require(value.passes > 0 && [37, 80, 68, 70, 45].every((byte, index) => pdf[index] === byte), 'Successful compilation has no PDF header');
  }
  require(!value.verification_eligible || value.ok && !errors, 'Invalid build verification eligibility');
  const blockIds = new Set<string>(), objectIds = new Set<string>();
  for (const block of value.blocks) {
    record(block, ['id', 'file', 'page', 'x', 'y', 'width', 'size', 'content', 'height', 'start', 'end', 'line']);
    require(identity(block.id) && !blockIds.has(block.id), 'Invalid or duplicate positioned block identity');
    blockIds.add(block.id); sourceRange(block, context.sources);
    require(integer(block.page, 1) && finite(block.x) && finite(block.y) && finite(block.width) && block.width > 0 &&
      finite(block.height) && block.height >= 0 && finite(block.size) && block.size > 0 && typeof block.content === 'string', 'Invalid positioned block geometry');
  }
  for (const object of value.objects) {
    record(object, ['id', 'file', 'dx', 'dy', 'plain_text', 'content', 'line', 'start', 'end']);
    require(identity(object.id) && !objectIds.has(object.id), 'Invalid or duplicate text object identity');
    objectIds.add(object.id); sourceRange(object, context.sources);
    require(finite(object.dx) && finite(object.dy) && (object.plain_text === null || typeof object.plain_text === 'string') && typeof object.content === 'string', 'Invalid text object metadata');
  }
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
    'first_line', 'last_line', 'kind', 'content', 'plain_text', 'rectangles']);
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
  for (const rectangle of value.rectangles) require(acceptSourceRectangle(rectangle).page === page, 'Mapped selection crossed a page boundary');
}
export function requestBuild<C extends keyof BuildRequests>(invoke: NativeInvoke, name: C, args: BuildRequests[C]): Promise<unknown> {
  return invoke(name, { ...args });
}
type UnitCommand = 'verify_build' | 'pin_displayed_build' | 'cancel_compilation' | 'document_discard' | 'export_document_build';
export async function invokeUnit<C extends UnitCommand>(invoke: NativeInvoke, name: C, args: BuildRequests[C]): Promise<void> {
  require(await requestBuild(invoke, name, args) === null, 'Invalid ' + name + ' acknowledgement');
}
