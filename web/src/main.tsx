import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { createRoot } from 'react-dom/client';
import { Engine } from './engine';
import { examples } from './examples';
import { MAX_BYTES, type Format, type Row, type Page, type Info } from './protocol';
import { actionFor, bindings, keyLabel, savedTheme, applyTheme, persistTheme } from './presentation';
import './style.css';

applyTheme(savedTheme());

type Column = Page & { parent: Row; selected: number };
const formatFor = (name: string): Format => /\.ya?ml$/i.test(name) ? 'yaml' : /\.har$/i.test(name) ? 'har' : 'json';
const friendly = (e: unknown) => e instanceof Error ? e.message : String(e);
function App() {
  const [columns, setColumns] = useState<Column[]>([]);
  const [focused, setFocused] = useState<Row | null>(null);
  const [info, setInfo] = useState<Info | null>(null);
  const [preview, setPreview] = useState('');
  const [name, setName] = useState('');
  const [format, setFormat] = useState<Format>('json');
  const [sample, setSample] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState('');
  const [query, setQuery] = useState('');
  const [path, setPath] = useState('');
  const [pasteOpen, setPasteOpen] = useState(false);
  const [pasted, setPasted] = useState('');
  const [pasteFormat, setPasteFormat] = useState<Format>('json');
  const [dragging, setDragging] = useState(false);
  const [help, setHelp] = useState(false);
  const [theme, setTheme] = useState(savedTheme);
  const [visibleColumns, setVisibleColumns] = useState(3);
  function toggleTheme() {
    const next = theme === 'dark' ? 'light' : 'dark';
    setTheme(next); applyTheme(next);
    setStatus(`Theme: ${next}${persistTheme(next) ? '' : ' (this session only)'}`);
  }
  function focusExplorer() { explorer.current?.focus({ preventScroll: true }); }
  const engine = useRef<Engine | null>(null);
  const generation = useRef(0);
  const working = useRef(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const searchInput = useRef<HTMLInputElement>(null);
  const pathInput = useRef<HTMLInputElement>(null);
  const explorer = useRef<HTMLDivElement>(null);
  const scrollArea = useRef<HTMLDivElement>(null);

  function clear(message = '') {
    generation.current++;
    engine.current?.close(); engine.current = null;
    working.current = false; setBusy(false); setInfo(null); setColumns([]); setFocused(null); setPreview('');
    setName(''); setQuery(''); setPath(''); setError(''); setStatus(message);
  }
  async function load(source: File | string, filename: string, kind: Format, isSample = false) {
    const size = typeof source === 'string' ? new Blob([source]).size : source.size;
    if (size > MAX_BYTES) { setError('File exceeds 20 MiB. Install Twig for larger files.'); return; }
    clear(); const gen = generation.current;
    working.current = true; setBusy(true); setName(filename); setFormat(kind); setSample(isSample);
    setStatus('Reading and indexing your file locally…');
    try {
      const bytes = typeof source === 'string' ? new TextEncoder().encode(source).buffer : await source.arrayBuffer();
      if (gen !== generation.current) return;
      const worker = new Engine(); engine.current = worker;
      const result = await worker.request('load', { bytes, format: kind });
      if (gen !== generation.current) return;
      if (!result.root) throw new Error('The document has no root.');
      const page = await worker.request('children', { parent: result.root.id, offset: 0 });
      const initial: Column[] = [{ ...page, parent: result.root, selected: 0 }];
      let selected = page.rows[0] ?? result.root;
      // Show a real nested branch in the bundled JSON example, so the interface
      // explains itself immediately. User files always begin at their root.
      if (isSample && kind === 'json') {
        const index = page.rows.findIndex(row => row.key === 'environments');
        if (index >= 0) {
          initial[0].selected = index;
          let parent = page.rows[index];
          for (let depth = 0; depth < 2 && parent.container; depth++) {
            const childPage = await worker.request('children', { parent: parent.id, offset: 0 });
            initial.push({ ...childPage, parent, selected: 0 });
            selected = childPage.rows[0] ?? parent;
            parent = selected;
          }
        }
      }
      const text = await worker.request('preview', { target: selected.id });
      if (gen !== generation.current) return;
      setInfo(result); setColumns(initial); setFocused(selected); setPreview(text);
      setStatus(isSample ? 'Example loaded. Open your own file whenever you’re ready.' : 'Ready. Your file stays on this device.');
      setPasteOpen(false); setPasted('');
      if (pasteOpen) requestAnimationFrame(focusExplorer);
      else focusExplorer();
    } catch (e) {
      if (gen === generation.current) { engine.current?.close(); engine.current = null; setError(friendly(e)); setStatus('Could not open this document.'); }
    } finally { if (gen === generation.current) { working.current = false; setBusy(false); } }
  }
  // The initial example is bundled: loading it makes no document network request.
  useEffect(() => { void load(examples.json, 'workspace.json', 'json', true); return () => { generation.current++; engine.current?.close(); }; }, []);
  useEffect(() => {
    const area = scrollArea.current;
    if (!area) return;
    const observer = new ResizeObserver(() => setVisibleColumns(Math.max(1, Math.floor(area.clientWidth / 230))));
    observer.observe(area);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    const row = scrollArea.current?.querySelector<HTMLElement>('.column:last-child [aria-selected="true"]');
    const list = row?.parentElement;
    if (row && list) {
      const top = row.getBoundingClientRect().top - list.getBoundingClientRect().top;
      if (top < 0) list.scrollTop += top;
      else if (top + row.offsetHeight > list.clientHeight) list.scrollTop += top + row.offsetHeight - list.clientHeight;
    }
  }, [focused?.id]);

  async function run(action: (worker: Engine, valid: () => boolean) => Promise<void>) {
    if (!engine.current || working.current) return;
    const gen = generation.current; const valid = () => gen === generation.current;
    working.current = true; setBusy(true); setError('');
    try { await action(engine.current, valid); }
    catch (e) { if (valid()) setError(friendly(e)); }
    finally { if (valid()) { working.current = false; setBusy(false); } }
  }
  async function selection(worker: Engine, next: Column[], row: Row, valid: () => boolean) {
    const text = await worker.request('preview', { target: row.id });
    if (valid()) { setColumns(next); setFocused(row); setPreview(text); }
  }
  function select(column: number, index: number) {
    const col = columns[column]; const row = col.rows[index];
    if (!row) return;
    if (column === columns.length - 1 && focused?.id === row.id && row.container) { drill(); return; }
    void run((worker, valid) => selection(worker, [...columns.slice(0, column), { ...col, selected: index }], row, valid));
  }
  function move(position: number) {
    const col = columns.at(-1); if (!col || !col.total) return;
    position = Math.max(0, Math.min(position, col.total - 1));
    void run(async (worker, valid) => {
      const offset = Math.floor(position / 256) * 256;
      const page = offset === col.offset ? col : await worker.request('children', { parent: col.parent.id, offset });
      const next = { ...col, ...page, selected: position - offset };
      await selection(worker, [...columns.slice(0, -1), next], next.rows[next.selected], valid);
    });
  }
  function drill() {
    if (!focused?.container || columns.at(-1)?.parent.id === focused.id) return;
    const parent = focused;
    void run(async (worker, valid) => {
      const page = await worker.request('children', { parent: parent.id, offset: 0 });
      await selection(worker, [...columns, { ...page, parent, selected: 0 }], page.rows[0] ?? parent, valid);
    });
  }
  function back(index = columns.length - 2) {
    if (index < 0) return;
    const next = columns.slice(0, index + 1); const col = next.at(-1)!;
    void run((worker, valid) => selection(worker, next, col.rows[col.selected] ?? col.parent, valid));
  }
  async function reveal(worker: Engine, row: Row, valid: () => boolean) {
    const lineage = await worker.request('lineage', { target: row.id });
    const parents = lineage.length > 1 ? lineage.slice(0, -1) : lineage;
    const next: Column[] = [];
    for (let i = 0; i < parents.length; i++) {
      const target = lineage[i + 1] ?? row;
      const offset = Math.floor(target.rank / 256) * 256;
      const page = await worker.request('children', { parent: parents[i].id, offset });
      next.push({ ...page, parent: parents[i], selected: Math.max(0, page.rows.findIndex(n => n.id === target.id)) });
    }
    await selection(worker, next, row, valid);
  }
  function search(direction = 1) {
    if (!query.trim()) return;
    void run(async (worker, valid) => {
      const row = await worker.request('search', { query, start: focused?.id, direction });
      if (!row) { if (valid()) { setStatus('No matches found.'); focusExplorer(); } return; }
      await reveal(worker, row, valid);
      if (valid()) { setStatus(`Match at ${row.path}. Search wraps in document order.`); focusExplorer(); }
    });
  }
  function jump() {
    void run(async (worker, valid) => {
      const row = await worker.request('resolve', { path });
      if (!row) throw new Error('That path was not found. Try .environments.production');
      await reveal(worker, row, valid);
      if (valid()) focusExplorer();
    });
  }
  function copy(kind: 'path' | 'export') {
    if (!focused) return;
    // ClipboardItem preserves Safari's user activation while the worker responds.
    if (working.current || !engine.current) return;
    const worker = engine.current; const target = focused.id;
    void run(async (_worker, valid) => {
      const text = worker.request(kind, { target });
      void text.catch(() => {});
      try {
      if (typeof ClipboardItem !== 'undefined' && navigator.clipboard?.write) {
        await navigator.clipboard.write([new ClipboardItem({ 'text/plain': text.then(value => new Blob([value], { type: 'text/plain' })) })]);
      } else {
        const value = await text;
        if (!navigator.clipboard?.writeText) throw new Error('Clipboard is unavailable. Select and copy the inspector text, or use the terminal app.');
        await navigator.clipboard.writeText(value);
      }
      } catch (clipboardError) {
        // Preserve the engine's explicit export-limit error when a browser
        // wraps a rejected ClipboardItem promise in a generic clipboard error.
        await text;
        throw clipboardError;
      }
      if (valid()) setStatus(kind === 'path' ? 'Full path copied.' : 'Complete selected value copied.');
    });
  }
  function onKey(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey || event.altKey || event.nativeEvent.isComposing) return;
    if (help) {
      if (['Escape', '?', 'h', 'Enter'].includes(event.key)) { event.preventDefault(); setHelp(false); focusExplorer(); }
      return;
    }
    if (pasteOpen) return;
    if ((event.target as HTMLElement).closest('input, textarea, select')) {
      if (event.key === 'Escape') { event.preventDefault(); focusExplorer(); }
      return;
    }
    const col = columns.at(-1); const position = col ? col.offset + col.selected : 0;
    const actions: Record<string, () => void> = {
      move_down: () => move(position + 1), move_up: () => move(position - 1),
      open: drill, back: () => back(), first: () => move(0), last: () => move((col?.total ?? 1) - 1),
      search: () => { setQuery(''); searchInput.current?.focus(); },
      jump: () => { setPath(''); pathInput.current?.focus(); },
      next_match: () => search(), previous_match: () => search(-1),
      copy_path: () => copy('path'), copy_value: () => copy('export'),
      toggle_theme: toggleTheme, help: () => setHelp(true), quit: () => clear('Document closed.'),
    };
    // Enter activates focused controls normally, but opens data rows like the TUI.
    if (event.key === 'Enter' && (event.target as HTMLElement).closest('button:not([role="option"]), a, summary')) return;
    const action = actionFor(event.key);
    if (action && actions[action]) { event.preventDefault(); focusExplorer(); actions[action](); }
  }
  function openFile(file?: File) { if (file) void load(file, file.name, formatFor(file.name)); }

  return <div className="app" onKeyDown={onKey} onDragOver={e => { e.preventDefault(); setDragging(true); }} onDragLeave={e => { if (!e.currentTarget.contains(e.relatedTarget as Node)) setDragging(false); }} onDrop={e => { e.preventDefault(); setDragging(false); openFile(e.dataTransfer.files[0]); }}>
    <a className="skip" href="#explorer">Skip to explorer</a>
    <header className="site-header">
      <a className="brand" href="/" aria-label="Twig home"><svg width="21" height="25" viewBox="0 0 29 34" aria-hidden="true"><path d="M14 31V3m0 18L3 10m11 4L25 3m-11 24L25 16" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/></svg>twig<span>.</span></a>
      <span className="workspace-name">Data explorer</span>
      <nav aria-label="Application controls">
        <button className="theme-toggle" onClick={toggleTheme} aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`} title="Toggle theme (t)">{theme === 'dark' ? '◐' : '◑'} <span>{theme === 'dark' ? 'Dark' : 'Light'}</span><kbd>t</kbd></button>
        <button onClick={() => setHelp(true)} aria-label="Keyboard shortcuts" title="Keyboard shortcuts (?)">?<span className="control-label"> Shortcuts</span></button>
        <a href="/guide/">Guide</a><a className="install-link" href="/install/">&gt;_ <span>Install the terminal app</span></a>
      </nav>
    </header>
    <main>
      <section className={`workspace ${dragging ? 'dragging' : ''}`} id="explorer" ref={explorer} tabIndex={0} aria-label="Data explorer">
        <div className="filebar"><div className="file-label"><span className="file-icon">{format === 'yaml' ? '≡' : '{ }'}</span><strong>{name || 'No file open'}</strong>{sample && name && <span className="tag">EXAMPLE</span>}{info && <span className="node-count">{info.nodes.toLocaleString()} nodes</span>}</div><div className="examples"><span>Examples</span>{(['json', 'yaml', 'har'] as Format[]).map(kind => <button key={kind} aria-label={`${kind.toUpperCase()} example`} onClick={() => void load(examples[kind], `example.${kind}`, kind, true)}>{kind.toUpperCase()}</button>)}</div><div className="file-actions"><button onClick={() => fileInput.current?.click()}>↑ Open file</button><button onClick={() => setPasteOpen(v => !v)}>Paste data</button>{(info || busy) && <button className="quiet" onClick={() => clear(busy ? 'Operation cancelled. Your document was cleared.' : 'Document closed.')}>{busy ? 'Cancel' : 'Close'}</button>}</div><input ref={fileInput} type="file" aria-label="Choose JSON, YAML or HAR file" accept=".json,.yaml,.yml,.har,application/json" onChange={e => { openFile(e.target.files?.[0]); e.target.value = ''; }} hidden/></div>
        {pasteOpen && <WorkspaceDialog title="Paste data" onClose={() => { setPasteOpen(false); focusExplorer(); }}><form className="paste-panel" onSubmit={e => { e.preventDefault(); void load(pasted, `pasted.${pasteFormat}`, pasteFormat); }}><label htmlFor="paste">Paste your data</label><textarea id="paste" value={pasted} onChange={e => setPasted(e.target.value)} placeholder={'{"hello": "world"}'} autoFocus/><div><select aria-label="Pasted data format" value={pasteFormat} onChange={e => setPasteFormat(e.target.value as Format)}><option value="json">JSON</option><option value="yaml">YAML</option><option value="har">HAR</option></select><button className="primary" type="submit">Explore data</button><button type="button" onClick={() => setPasteOpen(false)}>Dismiss</button></div></form></WorkspaceDialog>}
        <div className="toolbar"><form onSubmit={e => { e.preventDefault(); search(); }}><span aria-hidden="true">⌕</span><input ref={searchInput} aria-label="Search keys and values" value={query} onChange={e => setQuery(e.target.value)} placeholder="Search keys and values"/><button disabled={!info || busy || !query.trim()} aria-label="Previous match" type="button" onClick={() => search(-1)}>↑</button><button disabled={!info || busy || !query.trim()} aria-label="Next match">↓</button></form><form onSubmit={e => { e.preventDefault(); jump(); }}><span className="path-symbol">.</span><input ref={pathInput} aria-label="Jump to path" value={path} onChange={e => setPath(e.target.value)} placeholder="Jump to a path"/><button disabled={!info || busy} aria-label="Go to path">↵</button></form></div>
        {help && <WorkspaceDialog title="Keyboard shortcuts" onClose={() => { setHelp(false); focusExplorer(); }}>
          <p className="dialog-note">The same navigation in your browser and terminal.</p>
          <dl className="shortcut-list">{bindings.map(binding => <div key={binding.action}><dt>{binding.keys.map(key => <kbd key={key}>{keyLabel(key)}</kbd>)}</dt><dd>{binding.label}</dd></div>)}</dl>
          <p className="dialog-note">Esc returns from search and path entry. In the browser, q closes the document; your tab stays open.</p>
        </WorkspaceDialog>}
        <div className="breadcrumbs"><button disabled={busy || !columns.length} onClick={() => back(0)}>root</button>{columns.slice(1).map((col, i) => <span key={col.parent.id}><span className="separator">/</span><button disabled={busy} onClick={() => back(i + 1)}>{col.parent.key}</button></span>)}<span className="crumb-hint">Select to inspect · select again to open</span></div>
        {error && <div role="alert" className="error"><span>{error}</span><a href="/install/">Install Twig ↗</a><button aria-label="Dismiss error" onClick={() => setError('')}>×</button></div>}
        <div className="explorer-body" aria-busy={busy}>
          <div className="columns" ref={scrollArea} onContextMenu={e => { e.preventDefault(); back(); }}>
            {!columns.length && <div className="empty"><span className="empty-symbol">{busy ? '◌' : '{ }'}</span><h2>{busy ? 'Loading document…' : 'Open a document'}</h2><p>{busy ? 'Your file stays on this device.' : 'Drop a file here, open a file, or paste your data.'}</p><p className="muted">JSON · YAML · HAR / up to 20 MiB</p>{!busy && <button onClick={() => fileInput.current?.click()}>Open a file</button>}</div>}
            {columns.map((col, ci) => <section className="column" data-hidden={ci < columns.length - visibleColumns} key={`${ci}-${col.parent.id}`} aria-label={`Children of ${col.parent.key}`}><div className="column-title"><span>{ci === 0 ? 'ROOT' : col.parent.key}</span><span>{col.total}</span></div><div className="rows" role="listbox" aria-label={`${col.parent.key} entries`}>
              {col.rows.map((row, ri) => <button key={row.id} role="option" tabIndex={ri === col.selected ? 0 : -1} aria-selected={ri === col.selected} className={`row ${focused?.id === row.id ? 'focused' : ''}`} aria-disabled={busy} onClick={() => select(ci, ri)}><span className={`type-icon ${row.kind}`}>{row.container ? (row.kind === 'array' ? '[ ]' : '{ }') : row.kind === 'string' ? 'Aa' : row.kind === 'null' ? '∅' : row.kind === 'boolean' ? '◐' : '#'}</span><span className="row-content"><span className="row-key">{row.key || '""'}</span>{!row.container && <span className="row-value">{row.value}</span>}</span>{row.container && <span className="chevron">›</span>}</button>)}
              {!col.total && <p className="empty-container">Empty {col.parent.kind}</p>}
            </div>{col.total > 256 && <div className="pagination"><button disabled={busy || ci !== columns.length - 1 || !col.offset} aria-label="Previous page" onClick={() => move(col.offset - 256)}>←</button><span>{col.offset + 1}–{Math.min(col.offset + 256, col.total)} / {col.total}</span><button disabled={busy || ci !== columns.length - 1 || col.offset + 256 >= col.total} aria-label="Next page" onClick={() => move(col.offset + 256)}>→</button></div>}</section>)}
          </div>
          <aside className="inspector"><div className="inspector-top"><span className="eyebrow">Inspector</span><span className="tag">{focused?.kind ?? '—'}</span></div>{focused ? <><h2>{focused.key || '""'}</h2><p className="inspector-path">{focused.path}</p><div className="copy-actions"><button disabled={busy} onClick={() => copy('path')}>Copy path</button><button disabled={busy} onClick={() => copy('export')}>Copy value</button></div><div className="preview-label">Source preview <span>4 levels · 200 nodes · 64K chars</span></div><pre tabIndex={0} aria-label="Value preview">{preview}</pre><p className="inspector-note">Copy value exports the complete selection, up to 10,000 nodes and 4 MiB.</p></> : <p className="inspector-note">Select a value to take a closer look.</p>}</aside>
        </div>
        <div className="keybar" aria-label="Navigation shortcuts">
          <span><kbd>↑↓</kbd><kbd>j k</kbd> select</span><span><kbd>→</kbd><kbd>Enter</kbd> open</span><span><kbd>←</kbd><kbd>Esc</kbd> back</span>
          <button onClick={() => searchInput.current?.focus()}><kbd>/</kbd> search</button><button onClick={() => pathInput.current?.focus()}><kbd>:</kbd> jump</button>
          <span><kbd>n N</kbd> matches</span><button onClick={() => copy('path')} disabled={!focused}><kbd>c</kbd> path</button><button onClick={() => copy('export')} disabled={!focused}><kbd>y</kbd> value</button>
          <button onClick={toggleTheme}><kbd>t</kbd> theme</button><button onClick={() => setHelp(true)}><kbd>?</kbd> help</button>
        </div>
        <div className="statusbar"><span role="status"><span className={`dot ${busy ? 'busy' : ''}`}/>{busy ? 'Working locally…' : status}</span><span className="session-info">{info ? `${info.nodes.toLocaleString()} nodes · ${format.toUpperCase()} · ` : ''}{theme === 'dark' ? 'Dark' : 'Light'}</span><span className="privacy-note" title="Files are processed in your browser and are not uploaded.">Local only · No uploads</span></div>
      </section>
    </main>
    {dragging && <div className="drop-overlay">Drop your file to explore it locally.</div>}
  </div>;
}
function WorkspaceDialog({ title, onClose, children }: { title: string; onClose: () => void; children: React.ReactNode }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { const element = dialog.current; element?.showModal(); return () => {
    element?.close();
    // The explorer is inert until the modal has closed.
    document.getElementById('explorer')?.focus({ preventScroll: true });
  }; }, []);
  return <dialog ref={dialog} aria-label={title} onCancel={e => { e.preventDefault(); onClose(); }}>
    <div className="dialog-heading"><h2>{title}</h2><button onClick={onClose} aria-label="Close dialog">×</button></div>{children}
  </dialog>;
}
createRoot(document.getElementById('root')!).render(<App/>);
