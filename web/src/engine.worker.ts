/// <reference lib="webworker" />
import init, { Document } from './wasm/twig_wasm';
import wasmUrl from './wasm/twig_wasm_bg.wasm?url';
import type { Request, Response } from './protocol';
const ready = init({ module_or_path: wasmUrl });
let document: Document | null = null;
// Serialize requests even while the WASM module is being initialized.
let queue = Promise.resolve();
self.onmessage = ({ data }: MessageEvent<Request>) => {
  queue = queue.then(async () => {
    try {
      await ready;
      let result: unknown;
      if (data.op === 'dispose') { document?.free(); document = null; result = null; }
      else if (data.op === 'load') {
        document?.free(); document = null;
        document = new Document(new Uint8Array(data.args.bytes), data.args.format);
        result = JSON.parse(document.info());
      } else {
        if (!document) throw new Error('Open a document first.');
        switch (data.op) {
          case 'children': result = JSON.parse(document.children(data.args.parent, data.args.offset)); break;
          case 'search': result = JSON.parse(document.search(data.args.query, data.args.start, data.args.direction)); break;
          case 'resolve': result = JSON.parse(document.resolve(data.args.path)); break;
          case 'lineage': result = JSON.parse(document.lineage(data.args.target)); break;
          case 'preview': result = document.preview(data.args.target); break;
          case 'export': result = document.export(data.args.target); break;
          case 'path': result = document.path(data.args.target); break;
        }
      }
      self.postMessage({ id: data.id, ok: true, result } satisfies Response);
    } catch (e) {
      self.postMessage({ id: data.id, ok: false, error: e instanceof Error ? e.message : String(e) } satisfies Response);
    }
  });
};
