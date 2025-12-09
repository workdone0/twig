<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg)](https://pypi.org/project/twg/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/twg.svg)](https://pypi.org/project/twg/)
[![Downloads](https://static.pepy.tech/badge/twg)](https://pepy.tech/project/twg)
[![Code style: ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)

> **Inspect. Navigate. Understand.**
>
> A Calm Way to Explore JSON in Your Terminal

![Twig Demo](asset/demo.gif)

**The Demo Shows:**
1.  **Navigation**: Traversing regions (`us-east-1`) → VPCs → Subnets → Instances.
2.  **Global Search**: Finding "api-gateway" instantly.
3.  **Smart Jump**: Entering a path (`.regions...instances[0].name`) to jump directly to a deeply nested key.
4.  **Match Cycling**: Searching for "available" and using `n`/`N` to cycle through results.
5.  **Themes & Copy**: Toggling themes and copying paths.

## The Story

Every developer knows the nightmare of dealing with JSON files in the terminal. You try to `cat` a file, and you get a wall of unreadable text. You try `less`, but it's passive and hard to search.

**The alternatives aren't ideal either:**
*   **Editors**: Opening a heavy editor just to check a config file is overkill, and often not an option when you are SSH'd into a remote machine.
*   **Online Formatters**: Pasting your data into "Pretty Print Online" websites works, but it forces you to send potentially sensitive or private data to an untrusted server.

Twig was created to solve this problem providing a tool that is fast, local, and runs everywhere.

**Why "Twig"?**
The name represents a small, thin branch on a tree or bush—which is exactly how Twig visualizes your JSON data. Just as a twig is part of a larger structure, this tool helps you navigate the branches of your data trees.

**The Inspiration**
The UI is heavily inspired by **macOS Finder's column view**. It's the most natural way to traverse deep hierarchies, bringing that fluid experience to the terminal.

## Core Pillars

1.  **JSON Optimized**: Built specifically to make JSON files readable and navigable.
2.  **Privacy First**: 100% local execution. Your data never leaves your machine.
3.  **SSH Friendly**: Runs entirely in the terminal. No GUI required.
4.  **Finder-like Navigation**: Traverse deep nested structures effortlessly using Miller columns.

## Features

### Seamless Navigation
- **Miller Columns**: Intuitively move right to drill down, left to go back.
- **Breadcrumbs**: Always know your location with a `jq`-compatible path display at the top.

### Powerful Search
- **Deep Search**: Press `/` to search for keys or values.
    - **Smart Path Support**: If your query starts with `.` (e.g., `.users[0]`), Twig automatically jumps to that path.
- **Direct Jump**: Press `:` to instantly jump to a specific path (legacy/alternative).

### Developer Essentials
- **Instant Copy**: Press `c` to copy the current path to your clipboard (jq syntax ready).
- **Themeable**: Toggle between themes (like `catppuccin-mocha`, `dracula`, `monokai`) with `t`.
- **Inline Help**: Press `?` anytime to see a cheat sheet of keyboard shortcuts.

## Installation

### Using pipx (Recommended)
`pipx` installs the application in an isolated environment, keeping your system clean.

```bash
pipx install twg
```

### Using uv
```bash
uv pip install twg
```

### Using pip
```bash
pip install twg
```

## Usage

```bash
# Open a JSON file
twig data.json

# You can also use the shorter alias
twg data.json
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `Arrow Keys` | Navigate the tree (Up/Down/Left/Right) |
| `/` | Search for keys or values |
| `n` / `N` | Jump to next / previous search match |
| `:` | Jump to a specific `jq` path |
| `c` | Copy the current path to clipboard |
| `t` | Cycle through themes |
| `?` | Show Help / Shortcuts |
| `q` | Quit |

## FAQ

**Why is the package named `twg` instead of `twig`?**
The name `twig` was already taken on PyPI. You can install it via `pipx install twg`, but the CLI command is still `twig` (or the alias `twg`).

**What file sizes can it handle?**
*   **8GB RAM Machine**: comfortably handles files up to **~100MB**.
*   **Larger Files**: May cause the application to slow down or crash due to memory limits.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)

## Current Status

**Version**: v1.0.0 (Stable)

Twig is currently optimized for **JSON** files and runs in **Read-Only** mode. Active development is focused on making it even more stable and feature-rich.
