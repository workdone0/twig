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
    command: "Invoke-WebRequest -UseBasicParsing `\n  https://twig.wtf/install.ps1 `\n  -OutFile install.ps1\npowershell -NoProfile `\n  -ExecutionPolicy Bypass `\n  -File .\\install.ps1",
    note: 'PowerShell 5.1+ and tar.exe. Installs for your account and adds Twig to user PATH. No administrator access needed. The execution-policy option applies only to this installer process.',
    steps: ['Download the script with the first command; inspect it before running the second.', 'Close and reopen your terminal to load the updated PATH.', 'Run twig --version. For restricted scripts, use the manual guide below.'],
    guide: '/guide/#windows', label: 'Windows instructions & troubleshooting ↗'
  },
  source: {
    shell: 'CARGO · RUST 1.88+',
    command: 'cargo install --locked --git https://github.com/workdone0/twig --tag v3.2.0 twig',
    note: 'Builds v3.2.0. Requires Rust 1.88+ and a C compiler/linker. On Windows, use the MSVC toolchain and Visual Studio C++ Build Tools.',
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

// Guides use the same copy interaction as the installation panel.
document.querySelectorAll('.copy-code').forEach(button => button.addEventListener('click', async () => {
  const block = button.closest('.code-block');
  const feedback = block.querySelector('.copy-feedback');
  try {
    await navigator.clipboard.writeText(block.querySelector('code').textContent);
    feedback.textContent = 'Copied.';
  } catch { feedback.textContent = 'Select and copy the example manually.'; }
}));
