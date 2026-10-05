const data = { project: 'twig', regions: [{ name: 'eu-west-1', status: 'healthy', services: { api: { replicas: 3, port: 8080 }, worker: { replicas: 2, queue: 'events' } } }, { name: 'us-east-1', status: 'healthy', services: { api: { replicas: 4, port: 8080 } } }], environment: 'production', telemetry: false };
const columns = document.querySelector('#demo-columns');
let selections = ['regions'];
const kind = v => Array.isArray(v) ? 'array' : v === null ? 'null' : typeof v;
const container = v => v !== null && typeof v === 'object';
function renderDemo() {
  columns.replaceChildren(); let current = data; let path = '.'; let selectedValue = data; let selectedKey = 'root';
  for (let depth = 0; container(current) && depth < 5; depth++) {
    const column = document.createElement('div'); column.className = 'demo-column';
    const level = depth; const array = Array.isArray(current);
    for (const [key, value] of Object.entries(current)) {
      const row = document.createElement('button'); row.type = 'button'; row.className = 'demo-row'; row.setAttribute('aria-pressed', String(selections[depth] === key));
      const label = document.createElement('span'); label.textContent = key;
      const hint = document.createElement('small'); hint.textContent = container(value) ? '›' : String(value); row.append(label, hint);
      row.addEventListener('click', () => { selections = selections.slice(0, level); selections[level] = key; renderDemo(); }); column.append(row);
    }
    columns.append(column);
    const key = selections[depth]; if (key === undefined || !Object.hasOwn(current,key)) break;
    path += array ? `[${key}]` : `${path === '.' ? '' : '.'}${key}`; selectedKey = key; selectedValue = current[key]; current = selectedValue;
  }
  document.querySelector('#demo-path').textContent = path;
  document.querySelector('#inspector-key').textContent = selectedKey;
  document.querySelector('#inspector-type').textContent = kind(selectedValue);
  document.querySelector('#inspector-value').textContent = JSON.stringify(selectedValue, null, 2);
  columns.scrollLeft = columns.scrollWidth;
}
if (columns) {
  renderDemo(); document.querySelector('#reset-demo').addEventListener('click', () => {selections=['regions'];renderDemo();});
}
const methods = {
  unix: ['curl -fsSL https://twig.wtf/install.sh | bash', 'Downloads the latest release and verifies its SHA-256 checksum. Requires Bash.'],
  windows: ['tar -xzf twig-x86_64-pc-windows-msvc.tar.gz', 'Download the Windows archive and checksum below, verify its SHA-256 with Get-FileHash, then extract it and add twig.exe to PATH.'],
  source: ['cargo install --locked --git https://github.com/workdone0/twig --tag v3.1.0 twig', 'Builds version 3.1.0. Requires Rust 1.88+ and a C compiler/linker for bundled SQLite.']
};
document.querySelectorAll('[data-platform]').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('[data-platform]').forEach(b => b.setAttribute('aria-pressed',String(b===button)));
  const [command,note]=methods[button.dataset.platform]; document.querySelector('#install-command').textContent=command;document.querySelector('#install-note').textContent=note;document.querySelector('#copy-status').textContent='';
}));
document.querySelector('#copy-install')?.addEventListener('click',async () => {
  const status=document.querySelector('#copy-status');
  try { await navigator.clipboard.writeText(document.querySelector('#install-command').textContent);status.textContent='Command copied.'; }
  catch {status.textContent='Copy unavailable. Select and copy the command above.';}
});
