<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg?style=flat-square&color=2ecc71)](https://pypi.org/project/twg/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/twg.svg?style=flat-square)](https://pypi.org/project/twg/)
[![Downloads](https://static.pepy.tech/badge/twg)](https://pepy.tech/project/twg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/workdone0)

> **Inspect. Navigate. Understand.**
>
> A modern, terminal-based explorer for **JSON** and **YAML** files.
> Built for developers who work with real data in real environments.

![Twig Demo](asset/demo.gif)

## What is Twig?

**Twig** is a high-performance **terminal UI** for exploring **JSON** and **YAML** files interactively.

It turns deeply nested data into a navigable tree, letting you search, jump, and inspect complex structures without piping commands together or scrolling endlessly.

Twig is designed for **understanding data**, not editing it. If you want to modify files, tools like `vim` or `jq` are better suited. Twig focuses on helping you *make sense* of large and unfamiliar data safely.

## Installation

### Using uv (Recommended)
The modern, fast, and reliable way to install Python tools.

1. **Install uv** (if you don't have it):
   
   **macOS / Linux:**
   ```bash
   curl -LsSf https://astral.sh/uv/install.sh | sh
   ```
   **Windows:**
   ```powershell
   powershell -c "irm https://astral.sh/uv/install.ps1 | iex"
   ```

2. **Install Twig**:
   ```bash
   uv tool install twg
   ```


### Other Methods
> **Note:** Only use these if you are experienced with Python environments and know what you are doing.

#### Using pipx
```bash
pipx install twg
```

#### Using pip
```bash
pip install twg
```

## Uninstalling
```bash
uv tool uninstall twg
# or if you used pipx
pipx uninstall twg
```

## Why Twig Exists

Twig was built out of a very practical problem:

Many real-world JSON and YAML files — API responses, Kubernetes manifests, Terraform state, logs — contain **sensitive information**. Pasting them into browser-based viewers is often not an option.

There are excellent CLI tools that run locally, but many are optimized for **transformation**, not **exploration**, and can feel unintuitive when your goal is simply to *understand* a large, complex structure.

Twig fills that gap:
- Runs **entirely locally**
- Works well over **SSH and headless environments**
- Optimized for **reading and exploration**, not mutation

## Key Features

- **📂 Multi-Format**  
  Native support for **JSON** and **YAML**.

- **👀 Read-Only by Design**  
  Safely explore production data, logs, and configs without accidental edits.

- **🔍 Deep Search**  
  Fast fuzzy search across keys and values  
  (e.g. `Pull` matches `imagePullPolicy`).

- **🧭 Tree-Based Navigation**  
  Navigate large, deeply nested files without losing context.

- **🎨 Themes**  
  Includes **Catppuccin Mocha** (default) and **Solarized Dark**.

- **⚡ Performance-Focused**  
  Designed to handle large files efficiently with a low memory footprint.

## How Twig Works (High Level)

Twig is built using **[Textual](https://github.com/Textualize/textual)**, a Python framework for building modern terminal UIs.

Under the hood:
- Files are **parsed and indexed into SQLite**
- **FTS5 (Full-Text Search)** powers fast global search

This architecture allows Twig to stay responsive even with large files, while enabling instant search and navigation once indexing is complete.

## Performance Benchmarks

Twig is optimized for files **under 100MB**, with a sweet spot below **50MB**.

| File Size | Load Time (Cold Start) | Experience |
| :--- | :--- | :--- |
| **< 10MB** | < 1s | ⚡ Instant |
| **50MB** | ~8s | 🚀 Fast |
| **90MB** | ~17s | ✅ Usable |
| **> 100MB** | 20s+ | 🐢 Slower |

> **Note:** Large files take longer on first load because Twig indexes the entire structure to enable instant search and navigation.

## Common Use Cases

- **Kubernetes & Infrastructure**  
  Navigate large YAML manifests without scrolling endlessly.

- **Logs & API Dumps**  
  Inspect massive JSON outputs from cloud services or APIs.

- **Configuration Audits**  
  Explore Terraform state, Ansible playbooks, or generated configs safely.

- **Remote Systems**  
  Explore data directly over SSH without copying files locally.

## Comparison (Conceptual)

Twig is not a replacement for everything — it’s a complement.

| Tool | Strength | Limitation |
| --- | --- | --- |
| `jq` | Powerful transformations | Steep learning curve for exploration |
| `less` / `cat` | Simple and universal | No structure awareness |
| Web viewers | Visual and easy | Privacy, size, and trust issues |
| **Twig** | Interactive understanding | Read-only, exploration-focused |

## Usage

**Explore a file:**
```bash
twg data.json
# or
twg config.yaml
```

**Fix broken JSON:**
Automatically repair common errors (trailing commas, unquoted keys) or sanitize `NaN`/`Infinity` values:
```bash
twg --fix bad.json -o clean.json
```

**Pretty Print:**
```bash
twg -p large.json
```

## Cheat Sheet

| Key | Action |
| :--- | :--- |
| **Navigation** | |
| `Arrow Keys` | **Traverse** Tree |
| `/` | **Search** (Global) |
| `n` / `N` | **Next / Prev** Match |
| `:` | **Jump** to path |
| **Actions** | |
| `c` | **Copy Path** |
| `y` | **Copy Source** (JSON/YAML) |
| `t` | **Toggle Theme** |
| `?` | **Help** |
| `q` | **Quit** |

## Non-Goals

Twig is intentionally **not**:

- A JSON or YAML editor
- A replacement for `jq` or other transformation tools
- A streaming log viewer
- A web-based or SaaS tool

Twig is focused on **reading and understanding structured data well**, especially in local and remote (SSH) environments.


## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details and setup instructions.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
