<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg)](https://pypi.org/project/twg/)
[![PyPI downloads](https://img.shields.io/pypi/dw/twg.svg)](https://pypi.org/project/twg/)
> **Inspect. Navigate. Understand.**
>
> A terminal-based environment (TUI) for exploring and understanding structured data files.
> Fast, secure, and local.

![Twig Demo](asset/demo.gif)

## Why Twig?

Modern development involves wrestling with massive JSON files. `cat` is too raw, `less` is too passive, and `jq` requires learning a query language.

**Twig** bridges the gap. It provides rich, context-aware visualization with the speed and security of a local terminal application. No data leaves your machine.

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=Date)](https://star-history.com/#workdone0/twig&Date)

## Core Pillars
1.  **JSON Support**: Optimized for JSON files.
2.  **Miller Column Navigation**: Traverse deep hierarchies effortlessly.
3.  **Powerful Search**: Recursive deep search and jump-to-path.
4.  **Privacy First**: 100% local execution.

## Features

### 🧭 Seamless Navigation
- **Miller Column Navigation**: Navigate deep structures effortlessly using columns (like Finder or JSON Hero).
- **Smart Truncation**: Large lists and objects (1000+ items) are automatically grouped into navigable buckets (e.g., `[0 ... 999]`) to keep performance instant.
- **Intuitive Controls**: Use **Arrow Keys** to dive in (Right) or go back (Left).
- **Breadcrumbs**: Always know where you are with a `jq`-compatible path display at the top.

### 🔍 Smart Inspection
- **Inspector Pane**: View detailed information about the selected node in a dedicated pane.
- **Smart Types**: Automatically detects and formats:
    - **Dates**: ISO 8601 timestamps are parsed and displayed in a readable format.
    - **URLs**: Links are identified and labeled.
- **jq Integration**: Copy the path to any node in `jq` syntax (e.g., `.users[0].name`) with a single keystroke.

### ⚡ Powerful Navigation
- **Deep Search**: Press `/` to search for keys or values. Twig transparently expands deep structures to find matches.
- **Direct Jump**: Press `:` to instantly jump to any `jq` path (e.g., `.data[0].id`). Validates paths and provides helpful errors.
- **Async Engine**: Navigation is built on a non-blocking, async engine, ensuring the UI remains responsive even when traversing massive datasets.

### 🎨 Beautiful & Customizable
- **Themes**: Comes with `catppuccin-mocha` by default.
- **Theme Cycling**: Cycle through all available Textual themes (Solarized, Monokai, Dracula, etc.) with the `t` key.
- **Visual Polish**: Clean headers, clear dividers, and theme-aware highlighting.

## Installation



### Using pipx (Recommended)
`pipx` is the recommended way to install Python CLI applications as it isolates dependencies. If you don't have `pipx` installed, check out the [official installation guide](https://pipx.pypa.io/stable/installation/).

```bash
pipx install twg
```

### Using pip
```bash
pip install twg
```

### Using uv
```bash
uv pip install twg
```

## Usage

```bash
# Open a JSON file
twg data.json
# or use the alias
twig data.json
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `Arrow Keys` | Navigate the tree (Up/Down/Left/Right) |
| `/` | Search for keys or values |
| `n` / `N` | Jump to next / previous search match |
| `:` | Jump to a specific `jq` path |
| `c` | Copy the current path (jq format) to clipboard |
| `t` | Cycle through themes |
| `q` | Quit the application |

## Current Status & Limitations

**Version**: v1.0.0 (Stable)

Twig is a robust **read-only** viewer optimized for **JSON** files.

- **Read-Only**: Twig safely opens files in read-only mode. You cannot modify data yet.
- **Format Support**: Optimized for JSON.
- **Large Files**: Capable of handling massive files (100MB+) via smart bucketization. The UI remains responsive.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architectural details and contribution guidelines.

## Contributors

<a href="https://github.com/workdone0/twig/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=workdone0/twig" />
</a>
