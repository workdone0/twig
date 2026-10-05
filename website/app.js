// Illustrative data only. No files, telemetry, or network requests in this demo.
const data = {
  project: 'twig',
  regions: [
    { name: 'eu-west-1', status: 'healthy', services: { api: { replicas: 3, port: 8080 }, worker: { replicas: 2, queue: 'events' } } },
    { name: 'us-east-1', status: 'healthy', services: { api: { replicas: 4, port: 8080 } } }
  ],
  environment: 'production',
  telemetry: false
};
const columns = document.querySelector('#demo-columns');
let selections = ['regions'];
const kind = value => Array.isArray(value) ? 'array' : value === null ? 'null' : typeof value;
const container = value => value !== null && typeof value === 'object';

function selectRow(level, key) {
  selections = selections.slice(0, level);
  selections[level] = key;
  renderDemo(level, key);
}
function renderDemo(focusLevel, focusKey) {
  columns.replaceChildren();
  let current = data;
  let path = '.';
  let selectedValue = data;
  let selectedKey = 'root';
  let focusTarget;
  for (let depth = 0; container(current) && depth < 5; depth++) {
    const column = document.createElement('div');
    column.className = 'demo-column';
    column.setAttribute('role', 'group');
    column.setAttribute('aria-label', path === '.' ? 'Root properties' : `Children of ${path}`);
    const level = depth;
    const array = Array.isArray(current);
    const entries = Object.entries(current);
    entries.forEach(([key, value], index) => {
      const row = document.createElement('button');
      row.type = 'button'; row.className = 'demo-row';
      row.setAttribute('aria-pressed', String(selections[depth] === key));
      row.tabIndex = key === (selections[depth] ?? entries[0]?.[0]) ? 0 : -1;
      const label = document.createElement('span'); label.textContent = key;
      const hint = document.createElement('small'); hint.textContent = container(value) ? '›' : String(value);
      row.append(label, hint);
      row.addEventListener('click', () => selectRow(level, key));
      row.addEventListener('keydown', event => {
        if (!['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
        event.preventDefault();
        if (event.key === 'ArrowLeft' && level > 0) {
          selections = selections.slice(0, level);
          renderDemo(level - 1, selections[level - 1]);
        } else if (event.key === 'ArrowRight' && container(value) && Object.keys(value).length) {
          selections = selections.slice(0, level);
          selections[level] = key;
          selections[level + 1] = Object.keys(value)[0];
          renderDemo(level + 1, selections[level + 1]);
        } else if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
          const next = event.key === 'Home' ? 0 : event.key === 'End' ? entries.length - 1 : Math.max(0, Math.min(entries.length - 1, index + (event.key === 'ArrowDown' ? 1 : -1)));
          selectRow(level, entries[next][0]);
        }
      });
      column.append(row);
      if (level === focusLevel && key === focusKey) focusTarget = row;
    });
    columns.append(column);
    const key = selections[depth];
    if (key === undefined || !Object.hasOwn(current, key)) break;
    path += array ? `[${key}]` : `${path === '.' ? '' : '.'}${key}`;
    selectedKey = key; selectedValue = current[key]; current = selectedValue;
  }
  document.querySelector('#demo-path').textContent = path;
  document.querySelector('#inspector-key').textContent = selectedKey;
  document.querySelector('#inspector-type').textContent = kind(selectedValue);
  document.querySelector('#inspector-value').textContent = JSON.stringify(selectedValue, null, 2);
  if (focusTarget) {
    focusTarget.focus({ preventScroll: true });
    // Reveal the focused column without scrolling the whole page.
    const column = focusTarget.parentElement;
    columns.scrollLeft = Math.max(0, column.offsetLeft - columns.offsetLeft + column.offsetWidth - columns.clientWidth);
  }
}
if (columns) {
  renderDemo();
  document.querySelector('#reset-demo').addEventListener('click', () => {
    selections = ['regions']; renderDemo(); columns.scrollLeft = 0;
  });
}

const methods = {
  unix: {
    shell: 'BASH / ZSH',
    command: 'curl -fsSL https://twig.wtf/install.sh | bash',
    note: 'For macOS and Linux with GNU libc. Installs to ~/.local/bin. Requires Bash, curl, and tar.',
    steps: ['Run the command in your terminal.', 'Add ~/.local/bin to PATH if the installer asks.', 'Run twig --version to check the installation.'],
    guide: '/guide/#linux-and-macos', label: 'macOS / Linux instructions ↗'
  },
  windows: {
    shell: 'POWERSHELL · WINDOWS x64',
    command: 'Invoke-WebRequest https://twig.wtf/install.ps1 -OutFile install.ps1\npowershell -NoProfile -ExecutionPolicy Bypass -File .\\install.ps1',
    note: 'PowerShell 5.1+ and tar.exe. Installs for your account and adds Twig to user PATH. No administrator access needed. The execution-policy option applies only to this installer process.',
    steps: ['Download the script with the first line; inspect it before running the second.', 'Close and reopen your terminal to load the updated PATH.', 'Run twig --version. For restricted scripts, use the manual guide below.'],
    guide: '/guide/#windows', label: 'Windows instructions & troubleshooting ↗'
  },
  source: {
    shell: 'CARGO · RUST 1.88+',
    command: 'cargo install --locked --git https://github.com/workdone0/twig --tag v3.1.0 twig',
    note: 'Builds v3.1.0. Requires Rust 1.88+ and a C compiler/linker. On Windows, use the MSVC toolchain and Visual Studio C++ Build Tools.',
    steps: ['Run the command in a terminal with Cargo available.', 'Ensure the Cargo bin directory is on PATH.', 'Run twig --version to check the installation.'],
    guide: '/guide/#build-from-source', label: 'Source build requirements ↗'
  }
};
document.querySelectorAll('[data-platform]').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('[data-platform]').forEach(other => other.setAttribute('aria-pressed', String(other === button)));
  const method = methods[button.dataset.platform];
  document.querySelector('#install-shell').textContent = method.shell;
  document.querySelector('#install-command').textContent = method.command;
  document.querySelector('#install-note').textContent = method.note;
  const steps = document.querySelector('#install-steps'); steps.replaceChildren();
  method.steps.forEach(text => { const item = document.createElement('li'); item.textContent = text; steps.append(item); });
  const guide = document.querySelector('#platform-guide'); guide.href = method.guide; guide.textContent = method.label;
  document.querySelector('#copy-status').textContent = '';
}));
document.querySelector('#copy-install')?.addEventListener('click', async () => {
  const status = document.querySelector('#copy-status');
  try {
    await navigator.clipboard.writeText(document.querySelector('#install-command').textContent);
    status.textContent = 'Copied. Paste into your terminal when ready.';
  } catch { status.textContent = 'Select the commands above and copy them manually.'; }
});
