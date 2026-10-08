export type Format = 'json' | 'yaml' | 'har';
export interface Row { id: string; parent: string | null; key: string; path: string; kind: string; rank: number; value: string; container: boolean }
export interface Page { rows: Row[]; total: number; offset: number }
export interface Info { root: Row | null; nodes: number; estimatedBytes: number }
export interface Operations {
  load: { args: { bytes: ArrayBuffer; format: Format }; result: Info };
  children: { args: { parent: string; offset: number }; result: Page };
  search: { args: { query: string; start?: string; direction: number }; result: Row | null };
  resolve: { args: { path: string }; result: Row | null };
  lineage: { args: { target: string }; result: Row[] };
  preview: { args: { target: string }; result: string };
  export: { args: { target: string }; result: string };
  path: { args: { target: string }; result: string };
  dispose: { args: Record<string, never>; result: null };
}
export type Request = { [K in keyof Operations]: { id: number; op: K; args: Operations[K]['args'] } }[keyof Operations];
export type Response = { id: number; ok: true; result: unknown } | { id: number; ok: false; error: string };
export const MAX_BYTES = 20 * 1024 * 1024;
