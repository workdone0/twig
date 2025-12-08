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

## The Story

Every developer knows the nightmare of dealing with JSON files in the terminal. You try to `cat` a file, and you get a wall of unreadable text. You try `less`, but it's passive and hard to search.

**The alternatives aren't ideal either:**
*   **Editors**: Opening a heavy editor just to check a config file is overkill, and often not an option when you are SSH'd into a remote machine.
*   **Online Formatters**: Pasting your data into "Pretty Print Online" websites works, but it forces you to send potentially sensitive or private data to an untrusted server.

I decided to solve this problem. I wanted a tool that was fast, local, and ran everywhere.

**Why "Twig"?**
The name represents a small, thin branch on a tree or bush—which is exactly how we visualize your JSON data. Just as a twig is part of a larger structure, this tool helps you navigate the branches of your data trees.

**The Inspiration**
The UI is heavily inspired by **macOS Finder's column view**. It's the most natural way to traverse deep hierarchies, and I wanted to bring that fluid experience to the terminal.

## Core Pillars

1.  **JSON Optimized**: Built specifically to make JSON files readable and navigable.
2.  **Privacy First**: 100% local execution. Your data never leaves your machine.
3.  **SSH Friendly**: Runs entirely in the terminal. No GUI required.
4.  **Finder-like Navigation**: Traverse deep nested structures effortlessly using Miller columns.

## Features

### Seamless Navigation
- **Miller Columns**: Intuitively move right to drill down, left to go back.
- **Smart Truncation**: Large lists (1000+ items) are automatically bucketed (e.g., `[0 ... 999]`) so the UI never freezes.
- **Breadcrumbs**: Always know your location with a `jq`-compatible path display at the top.

### Powerful Search
- **Deep Search**: Press `/` to search for keys or values. Twig transparently drills down into nested structures to find matches.
- **Jump-to-path**: Press `:` to instantly jump to any specific path (e.g., `.users[0].name`).

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

## Project Stats

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=Date)](https://star-history.com/#workdone0/twig&Date)

## Current Status

**Version**: v1.0.0 (Stable)

Twig is currently optimized for **JSON** files and runs in **Read-Only** mode. I am actively working on making it even more stable and feature-rich.
