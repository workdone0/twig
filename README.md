<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg)](https://pypi.org/project/twg/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/twg.svg)](https://pypi.org/project/twg/)
[![Downloads](https://static.pepy.tech/badge/twg)](https://pepy.tech/project/twg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Code style: ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)

> **Inspect. Navigate. Understand.**
>
> A modern, terminal-based JSON explorer.

![Twig Demo](asset/demo.gif)

**The Demo Shows:**
1.  **Navigation**: Traversing regions (`us-east-1`) → VPCs → Subnets → Instances.
2.  **Global Search**: Finding "api-gateway" instantly.
3.  **Smart Jump**: Entering a path (`.regions...instances[0].name`) to jump directly to a deeply nested key.
4.  **Match Cycling**: Searching for "available" and using `n`/`N` to cycle through results.
5.  **Themes & Copy**: Toggling themes and copying paths.

## Why Twig?

Every developer knows the pain of `cat data.json`. You get a wall of unreadable text. `less` helps, but it’s passive. Editors are heavy. Online formatters leak privacy.

Twig solves this. It brings the fluid, hierarchical navigation of **macOS Finder** directly to your terminal.

### The Twig Advantage
*   **Privacy First**: Your data never leaves your machine. 100% local.
*   **Zero-Lag**: Built for speed. Traverse deep structures instantly without creating temporary files.
*   **No GUI Required**: Perfect for SSH sessions and remote debugging.
*   **Native Feel**: Miller Columns allow you to visualize the tree structure naturally—left to go back, right to drill down.

## Features

### Deep Search
*   **Instant Access**: Type `/` to search keys and values across the entire file.
*   **Unlimited Depth**: Finds matches (like "created_at") even if nested 50 levels deep.

### Smart Path Jump
Know exactly where you want to go?
*   **Direct Navigation**: Type `/` and start with a dot: `.users[0].address`.
*   **Auto-Expansion**: Twig instantly jumps to that location, expanding the tree as needed.

### Developer Essentials
*   **Clipboard Ready**: Press `c` to copy the current `jq`-compatible path (e.g., `.orders[5].id`).
*   **Themeable**: Cycle through themes (Catppuccin, Dracula, Monokai) with `t`.
*   **Zero Learning Curve**: Use Arrow keys or Vim keys. Press `?` for help.

## Installation

### ⚡ Using pipx (Recommended)
`pipx` installs Twig in an isolated environment, keeping your system clean.
```bash
pipx install twg
```

### 🐢 Using pip
```bash
pip install twg
```

## Usage

```bash
# Open any JSON file
twig data.json

# Or use the short alias
twg data.json
```

### Cheat Sheet

| Key | Action |
| :--- | :--- |
| `Arrow Keys` | Navigate (Up/Down/Left/Right) |
| `/` | **Search** (Keys, Values, or Paths) |
| `n` / `N` | Cycle Search Matches |
| `c` | **Copy Path** to Clipboard |
| `t` | Toggle Theme |
| `?` | Show Shortcuts |
| `q` | Quit |

## FAQ

*   **Why is the package named `twg`?**

    The name `twig` was taken on PyPI. You install it as `twg`, but the CLI command and behavior are all `twig`.

*   **Can it handle large files?**

    Yes. On a standard 8GB machine, Twig comfortably handles files up to **~100MB**. Larger files are possible but may impact performance since the graph is loaded into memory.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
