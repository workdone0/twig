<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg)](https://pypi.org/project/twg/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/twg.svg)](https://pypi.org/project/twg/)
[![Downloads](https://static.pepy.tech/badge/twg)](https://pepy.tech/project/twg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Inspect. Navigate. Understand.**
>
> A modern, terminal-based data explorer.

![Twig Demo](asset/demo.gif)

## Why Twig?

Reading raw data files in the terminal is painful. `cat` gives you a wall of text. `less` is passive. `jq` requires memorizing syntax.

**Twig** solves this by bringing the fluid, hierarchical navigation of **macOS Finder** to your data files. It is a TUI (Terminal User Interface) built for speed, privacy, and usability.

### Key Features

*   **⚡️ Zero-Lag Performance**: Handles gigabyte-sized files effortlessly using a streaming SQLite backend.
*   **🔒 Privacy First**: Your data never leaves your machine. Everything runs locally.
*   **🎹 Vim-like Navigation**: Traverse deep structures naturally with arrow keys or Vim bindings.
*   **🛠️ Developer Essentials**:
    *   **Copy Source**: Press `y` to copy the raw JSON of any selected node.
    *   **Copy Path**: Press `c` to copy the `jq`-compatible path (e.g., `.users[0].address`).
    *   **Persistent Config**: Themes and settings are saved across sessions.
*   **🎨 Beautiful UI**: Clean, modern interface with multiple themes (Catppuccin, Dracula, etc.).
*   **🔍 Deep Search**: Instantly find keys or values anywhere in the tree with `/`.

## Installation

### Using pipx (Recommended)
`pipx` installs Twig in an isolated environment, keeping your system clean.
```bash
pipx install twg
```

### Using pip
```bash
pip install twg
```

## Usage

### Interactive Mode (TUI)
Simply pass a file to start exploring:
```bash
twig data.json
```

Or use the short alias:
```bash
twg data.json
```

### CLI Utils
Twig also includes powerful CLI tools for quick fixes and formatting.

**Repair Broken JSON**
Automatically fix common errors like trailing commas or unquoted keys:
```bash
# Print fixed JSON to stdout
twig --fix bad.json

# Save to a new file
twig --fix bad.json clean.json
```

**Pretty Print**
Output formatted, syntax-highlighted JSON:
```bash
twig -p data.json

# Custom indentation
twig -p --indent 4 data.json
```

## Cheat Sheet

| Key | Action |
| :--- | :--- |
| **Navigation** | |
| `Arrow Keys` / `h,j,k,l` | Navigate Tree |
| `:` | **Smart Jump** to path |
| `/` | **Search** |
| `n` / `N` | Next / Prev Search Match |
| **Actions** | |
| `c` | **Copy Path** (`.key.path`) |
| `y` | **Copy Source** (Raw JSON) |
| `t` | Toggle Theme |
| `?` | Show Help Dashboard |
| `q` | Quit |

## FAQ

**Why is the package named `twg`?**
The name `twig` was taken on PyPI. Install as `twg`, but the command is `twig` (or `twg`).

**Can it handle large files?**
Yes. Twig streams data into a local SQLite cache. It opens gigabyte-sized files instantly after the initial index.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
