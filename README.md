<p align="center">
  <img src="https://raw.githubusercontent.com/workdone0/twig/master/asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![Crates.io](https://img.shields.io/crates/v/twig.svg?style=flat-square&color=2ecc71)](https://crates.io/crates/twig)
[![Supported Rust versions](https://img.shields.io/badge/rust-1.75%2B-blue.svg?style=flat-square)](https://www.rust-lang.org)
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

### Install with Cargo (Recommended)

The easiest way to install Twig — a single static binary, no runtime dependencies:

```bash
# Stable Rust (1.75+)
cargo install twig
```

### Install with cargo-binstall

```bash
cargo binstall twig
```

### Download a release binary

Grab a prebuilt binary from the [Releases page](https://github.com/workdone0/twig/releases):

- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

### Build from source

```bash
git clone https://github.com/workdone0/twig.git
cd twig
cargo build --release
./target/release/twig --help
```

---

## Usage

**Explore a file:**
```bash
twig data.json
# or
twig config.yaml
```

**Fix broken JSON:**
Automatically repair common errors (trailing commas, unquoted keys) or sanitize `NaN/Infinity` values:
```bash
twig --fix bad.json -o clean.json
```

**Pretty Print:**
```bash
twig -p large.json
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
- **🔍 Deep Search**: Fast substring search across keys and values (e.g. `Pull` matches `imagePullPolicy`).
- **🧭 Tree-Based Navigation**: Navigate large, deeply nested files without losing context.
- **🎨 Themes**: Includes **Catppuccin Mocha** (default) and **Solarized Dark**.
- **⚡ Performance-Focused**: Streaming ingestion, SQLite FTS5 indexing, and `--release` LTO keep cold-start load times low on huge files.

---

## Why Twig Exists

Many real-world files — API responses, K8s manifests, Terraform state — contain **sensitive information**. Pasting them into web-based viewers is a security risk.

Existing CLI tools like `jq` are powerful for **transformation** but can be unintuitive for **interactive exploration**. Twig focuses purely on the latter:

- Runs **entirely locally**, no network calls
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

Twig is built using **[ratatui](https://github.com/ratatui/ratatui)** for the TUI and an embedded **SQLite** + **FTS5** engine for the data layer. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full architecture notes.

**Benchmarks (50 MB JSON, cold start):**
| Build | Load Time |
| :--- | :--- |
| Debug | ~6s |
| Release (LTO) | ~2s |

---

## Contributing

We welcome contributions! See [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup, architecture, and submission guidelines.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)