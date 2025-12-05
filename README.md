<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://badge.fury.io/py/twg.svg)](https://badge.fury.io/py/twg)
> **Inspect. Navigate. Understand.**
>
> A terminal-based environment (TUI) for exploring and understanding structured data files.
> Planned: safe inline editing and diffs.

![Twig Demo](asset/demo.gif)

## Why Twig?

Modern development involves wrestling with massive JSON, YAML, and TOML files. `cat` is too raw, `less` is too passive, and `jq` requires learning a query language.

**Twig** bridges the gap. It gives you rich, context-aware visualization with the speed and security of a local terminal application.

## Goals
We are building the ultimate terminal-based data explorer with four core pillars:

1.  **Universal Support**: Open *any* structured file (JSON today; YAML, TOML, XML planned) and treat it as a unified data structure.
2.  **Miller Column Navigation**: Traverse deep hierarchies effortlessly using a Finder-like column layout.
3.  **Safe Editing** (Planned): Modify values and keys with confidence.
4.  **Powerful Search**: Find exactly what you need with fuzzy search and query capabilities.

## Features

### 🧭 Seamless Navigation
- **Miller Column Navigation**: Navigate deep structures effortlessly using columns (like Finder or JSON Hero).
- **Intuitive Controls**: Use **Arrow Keys** to dive in (Right) or go back (Left).
- **Breadcrumbs**: Always know where you are with a `jq`-compatible path display at the top.

### 🔍 Smart Inspection
- **Inspector Pane**: View detailed information about the selected node in a dedicated pane.
- **Smart Types**: Automatically detects and formats:
    - **Dates**: ISO 8601 timestamps are parsed and displayed in a readable format.
    - **URLs**: Links are identified and labeled.
- **jq Integration**: Copy the path to any node in `jq` syntax (e.g., `.users[0].name`) with a single keystroke.

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
| `c` | Copy the current path (jq format) to clipboard |
| `t` | Cycle through themes |
| `q` | Quit the application |

## Roadmap

- [x] **Phase 1**: Core Navigation & Visualization (Miller Columns, Inspector, Themes)
- [ ] **Phase 2**: Search & Filtering (Fuzzy search, JMESPath/jq queries)
- [ ] **Phase 3**: Editing (Modify values, rename keys, save changes)
- [ ] **Phase 4**: Multiple File Support & Diffing

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architectural details and contribution guidelines.

## Contributors

<a href="https://github.com/workdone0/twig/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=workdone0/twig" />
</a>
