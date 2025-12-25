<p align="center">
  <img src="https://raw.githubusercontent.com/workdone0/twig/master/asset/logo.png" alt="Twig Logo" width="200"/>
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

![Twig Demo](https://raw.githubusercontent.com/workdone0/twig/master/asset/demo.gif)

## What is Twig?

**Twig** is a high-performance **terminal UI** for exploring **JSON** and **YAML** files interactively. It turns deeply nested data into a navigable tree, letting you search, jump, and inspect complex structures without piping commands together or scrolling endlessly.

Twig is designed for **understanding data**, not editing it. It fills the gap between `cat`/`less` (no structure) and heavy IDEs (too slow/GUI-based), making it perfect for **production logs, Kubernetes manifests, and large API responses**.

---

## Installation

### Using uv (Recommended)
The modern, fast, and reliable way to install Python tools.

```bash
# 1. Install uv (if needed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# 2. Install Twig
uv tool install twg
```

### Other Methods
<details>
<summary>Click to show pipx or pip instructions</summary>

#### Using pipx
```bash
pipx install twg
```

#### Using pip
> **Note:** Recommended only for virtual environments.
```bash
pip install twg
```
</details>

### Uninstalling
```bash
uv tool uninstall twg
```

---

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

### Controls & Cheat Sheet

| Key | Action | Key | Action |
| :--- | :--- | :--- | :--- |
| **Navigation** | | **Actions** | |
| `Arrow Keys` | **Traverse** Tree | `c` | **Copy Path** |
| `/` | **Search** (Global) | `y` | **Copy Source** |
| `n` / `N` | **Next / Prev** Match | `t` | **Toggle Theme** |
| `:` | **Jump** to path | `?` | **Help** |
| | | `q` | **Quit** |

---

## Key Features

- **📂 Multi-Format**: Native support for **JSON** and **YAML**.
- **👀 Read-Only by Design**: Safely explore production data, logs, and configs without accidental edits.
- **🔍 Deep Search**: Fast fuzzy search across keys and values (e.g. `Pull` matches `imagePullPolicy`).
- **🧭 Tree-Based Navigation**: Navigate large, deeply nested files without losing context.
- **🎨 Themes**: Includes **Catppuccin Mocha** (default) and **Solarized Dark**.
- **⚡ Performance-Focused**: Designed to handle large files efficiently with a low memory footprint.

---

## Why Twig Exists

Many real-world files — API responses, K8s manifests, Terraform state — contain **sensitive information**. Pasting them into web-based viewers is a security risk.

Existing CLI tools like `jq` are powerful for **transformation** but can be unintuitive for **interactive exploration**. Twig focuses purely on the latter:

- Runs **entirely locally**
- Works well over **SSH and headless environments**
- Optimized for **reading**, not mutation

### Comparison
| Tool | Strength | Limitation |
| --- | --- | --- |
| `jq` | Powerful transformations | Steep learning curve for exploration |
| `less` / `cat` | Simple and universal | No structure awareness |
| Web viewers | Visual and easy | Privacy, size, and trust issues |
| **Twig** | Interactive understanding | Read-only, exploration-focused |

### Non-Goals
Twig is **not**:
- An editor.
- A replacement for `jq`.
- A streaming log viewer.

---

## Performance & Architecture

Twig is built using **[Textual](https://github.com/Textualize/textual)** and uses **SQLite** with **FTS5** for indexing. This allows instant search and navigation even for large files.

**Benchmarks:**
| File Size | Load Time (Cold Start) | Experience |
| :--- | :--- | :--- |
| **< 10MB** | < 1s | ⚡ Instant |
| **50MB** | ~8s | 🚀 Fast |
| **90MB** | ~17s | ✅ Usable |
| **> 100MB** | 20s+ | 🐢 Slower |

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details and setup instructions.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
