import type { Operations, Request, Response } from './protocol';
export class Engine {
  private worker = new Worker(new URL('./engine.worker.ts', import.meta.url), { type: 'module' });
  private next = 0;
  private pending = new Map<number, { resolve: (result: unknown) => void; reject: (error: Error) => void }>();
  private closed = false;
  constructor() {
    this.worker.onmessage = ({ data }: MessageEvent<Response>) => {
      const pending = this.pending.get(data.id);
      if (!pending) return;
      this.pending.delete(data.id);
      if (data.ok) pending.resolve(data.result); else pending.reject(new Error(data.error));
    };
    this.worker.onerror = () => this.close('The browser engine stopped. Reopen the file or install Twig for larger files.');
    this.worker.onmessageerror = () => this.close('The browser could not read the engine response. Please reopen the file.');
  }
  request<K extends keyof Operations>(op: K, args: Operations[K]['args']): Promise<Operations[K]['result']> {
    if (this.closed) return Promise.reject(new Error('Document closed.'));
    const id = ++this.next;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve: value => resolve(value as Operations[K]['result']), reject });
      const request = { id, op, args } as Request;
      this.worker.postMessage(request, request.op === 'load' ? [request.args.bytes] : []);
    });
  }
  close(message = 'Document closed.') {
    this.closed = true;
    this.worker.terminate();
    this.pending.forEach(p => p.reject(new Error(message)));
    this.pending.clear();
  }
}
