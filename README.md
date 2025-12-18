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
> The modern, terminal-based data explorer for developers who value speed and privacy.

![Twig Demo](asset/demo.gif)

## Why Twig?

Data files are getting bigger, but our tools haven't kept up. `cat` floods your screen. `less` feels ancient. `jq` requires learning a new language.

**Twig** brings the fluid, intuitive navigation of a modern IDE directly to your terminal. It transforms raw JSON into a navigable, searchable, and interactive tree.

### The Twig Difference

*   **⚡️ High Performance**: Built on a streaming SQLite backend, Twig handles large files with ease.
*   **🔒 Privacy by Default**: Your data never leaves your machine. Twig runs 100% locally—safe for production logs, PII, and sensitive configurations.
*   **🎹 Fluid Navigation**: Navigate deep hierarchies naturally using standard **Arrow Keys**. Smart expansion keeps your context clear.
*   **🧠 Developer Workflow**:
    *   **Deep Search**: Instantly find any key or value with `/`.
    *   **Smart Jump**: Jump directly to a path (e.g., `.users[0].address`).
    *   **Clipboard Ready**: One-key copy for paths (`c`) or raw JSON (`y`).
*   **🎨 Premium UI**: A polished TUI with syntax highlighting, multiple themes (Catppuccin, Dracula), and a distraction-free design.

## Installation

### Using pipx (Recommended)
Install in an isolated environment to keep your system clean:
```bash
pipx install twg
```

### Using pip
```bash
pip install twg
```

## Quick Start

**Explore a file:**
```bash
twg data.json
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
| `Arrow Keys` / `h,j,k,l` | **Traverse** Tree |
| `/` | **Search** (Global) |
| `n` / `N` | **Next / Prev** Match |
| `:` | **Jump** to path |
| **Actions** | |
| `c` | **Copy Path** |
| `y` | **Copy Value** (JSON) |
| `t` | **Toggle Theme** |
| `?` | **Help** |
| `q` | **Quit** |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details and setup instructions.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
