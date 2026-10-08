import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, { Document } from '../src/wasm/twig_wasm.js';
await init({ module_or_path: await readFile(new URL('../src/wasm/twig_wasm_bg.wasm', import.meta.url)) });
const encoder = new TextEncoder();
for (const format of ['json', 'yaml']) {
  const prefix = format === 'json' ? '{"payload":"' : 'payload: "';
  const suffix = format === 'json' ? '"}' : '"';
  const source = prefix + 'a'.repeat(20 * 1024 * 1024 - prefix.length - suffix.length) + suffix;
  const start = performance.now();
  const doc = new Document(encoder.encode(source), format);
  const info = JSON.parse(doc.info());
  assert.ok(info.nodes >= 2);
  const hit = JSON.parse(doc.search('payload', undefined, 1));
  assert.ok(hit);
  assert.ok(doc.preview(hit.id).length < 70_000);
  console.log(`${format}: 20 MiB ingested and queried in ${Math.round(performance.now() - start)}ms; estimated retained ${info.estimatedBytes} bytes`);
  assert.throws(() => doc.export(hit.id), /4 MiB/);
  doc.free();
}
const doc = new Document(encoder.encode('{"n":18446744073709551615}'), 'json');
const root = JSON.parse(doc.info()).root;
assert.equal(JSON.parse(doc.children(root.id, 0)).rows[0].value, '18446744073709551615');
assert.match(doc.export(root.id), /18446744073709551615/);
doc.free();
assert.throws(() => new Document(encoder.encode('{"a":1,"a":2}'), 'json'), /Duplicate/);
assert.throws(() => new Document(encoder.encode('[' + '0,'.repeat(250_000) + '0]'), 'json'), /resource limit/);
console.log('WebAssembly parser, precision, duplicate-key, and resource-limit gates passed.');

const many = new Document(encoder.encode('[' + '0,'.repeat(10_000) + '0]'), 'json');
assert.throws(() => many.export(JSON.parse(many.info()).root.id), /10,000/);
many.free();
const wideKey = 'k'.repeat(100_000);
assert.throws(() => new Document(encoder.encode(('{' + JSON.stringify(wideKey) + ':').repeat(100) + '0' + '}'.repeat(100)), 'json'), /resource limit/);
console.log('Clipboard and estimated-allocation budgets passed.');
