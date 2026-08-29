<p align="center">
  <img src="https://raw.githubusercontent.com/workdone0/twig/master/asset/logo.png" alt="Twig Logo" width="200"/>
</p>

<h1 align="center">Twig 🌿</h1>

<p align="center">
  <a href="https://github.com/workdone0/twig/releases/latest"><img src="https://img.shields.io/github/v/release/workdone0/twig?style=flat-square&color=2ecc71" alt="Latest release"/></a>
  <a href="https://crates.io/crates/twig"><img src="https://img.shields.io/crates/v/twig.svg?style=flat-square&color=2ecc71" alt="Crates.io"/></a>
  <a href="https://github.com/workdone0/twig/blob/master/LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"/></a>
  <a href="https://github.com/workdone0/twig/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/workdone0/twig/ci.yml?style=flat-square&branch=master" alt="CI"/></a>
  <a href="https://buymeacoffee.com/workdone0"><img src="https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee"/></a>
</p>

<p align="center">
  <strong>Inspect. Navigate. Understand.</strong>
  <br/>
  A modern, terminal-based explorer for <strong>JSON</strong> and <strong>YAML</strong> files,
  written in Rust as a single static binary — no Python runtime required.
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/workdone0/twig/master/asset/demo.gif" alt="Twig demo"/>
</p>

---

## What is Twig?

**Twig** is a high-performance **terminal UI** for exploring **JSON** and **YAML** files interactively. It turns deeply nested data into a navigable tree, letting you search, jump, and inspect complex structures without piping commands together or scrolling endlessly.

Twig is designed for **understanding data**, not editing it. It fills the gap between `cat`/`less` (no structure) and heavy IDEs (too slow, GUI-only), making it perfect for production logs, Kubernetes manifests, Terraform state, and large API responses.

Since v3.0.0, Twig is a single ~4 MB static binary with zero runtime dependencies — drop it on any Linux, macOS, or Windows machine and it runs.

---

## Installation

### One-line installer (recommended)

Downloads a prebuilt binary for your platform, verifies its SHA-256 against the GitHub-published `*.sha256` file, and installs it to `~/.local/bin` (or `/usr/local/bin` when writable):

```bash
curl -fsSL https://twig.wtf/install.sh | sh
```

Re-running the command safely **upgrades** an existing install. Useful flags:

```bash
# Pin a specific version (e.g. v3.0.0)
curl -fsSL https://twig.wtf/install.sh | sh -s -- --version v3.0.0

# Install to a different directory
curl -fsSL https://twig.wtf/install.sh | sh -s -- --to /usr/local/bin

# Build from source instead of downloading a binary
curl -fsSL https://twig.wtf/install.sh | sh -s -- --method build

# Skip the install confirmation prompt
curl -fsSL https://twig.wtf/install.sh | sh -s -- --yes
```

The script supports Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), and reports unsupported platforms with a clear error instead of failing silently. Run `curl -fsSL https://twig.wtf/install.sh | sh -s -- --help` for the full flag list.

### Cargo

```bash
cargo install twig
```

### cargo-binstall

```bash
cargo binstall twig
```

### Manual download

Grab a prebuilt `.tar.gz` from the [Releases page](https://github.com/workdone0/twig/releases/latest). Each archive contains a single `twig` (or `twig.exe`) binary and a matching `*.sha256` checksum file.

Supported targets:

| OS      | Architectures          |
| ------- | ---------------------- |
| Linux   | x86_64, aarch64        |
| macOS   | x86_64 (Intel), aarch64 (Apple Silicon) |
| Windows | x86_64                 |

### Build from source

Requires Rust 1.75 or later:

```bash
git clone https://github.com/workdone0/twig.git
cd twig
cargo build --release
./target/release/twig --help
```

### Legacy Python version

Twig ≤ v2.1.4 was a Python project distributed on PyPI as `twg`. That implementation is preserved on the [`legacy-python`](https://github.com/workdone0/twig/tree/legacy-python) branch for users who specifically need it. **All new installs and active development happen on `master`, which is the Rust rewrite shipped in v3.0.0.**

If you're upgrading from `twg`:

```bash
uv tool uninstall twg                       # remove the old install
curl -fsSL https://twig.wtf/install.sh | sh # install the new binary
```

No code changes required — `twig <file>`, `twig --fix <file>`, and `twig --print <file>` all work identically, and the persistent config at `~/.config/twig/config.json` is shared across both versions.

---

## Usage

```bash
# Open a file in the interactive TUI
twig data.json
twig config.yaml

# Pretty-print to stdout (non-TUI, with syntax highlighting)
twig -p large.json

# Repair common JSON malformations (trailing commas, unquoted keys,
# single quotes, NaN/Infinity) and write a clean round-trippable copy
twig --fix broken.json -o clean.json

# Benchmark the streaming ingestion pipeline (non-TUI)
twig --check huge.json
```

`twig --check` is non-interactive and prints size, node count, elapsed time, and throughput — useful in CI and ad-hoc perf checks.

---

## Key features

- **📂 Multi-format native**: JSON and YAML with the same UI; format is auto-detected from file extension.
- **👀 Read-only by design**: Safely explore production secrets without risk of accidental edits.
- **🔍 Deep search**: Substring search across keys and values, with `n` / `N` to jump between matches.
- **🧭 Miller-column navigation**: Traverse deep trees with the keyboard, breadcrumbs keep you oriented.
- **🎨 Themes**: **Catppuccin Mocha** (default) and **Solarized Dark** ship in the box; `t` cycles. Persistent config at `~/.config/twig/config.json`.
- **⚡ Streaming ingestion**: Constant memory relative to file size — load 10 MB or 10 GB without the process growing.
- **🩺 Smart inspector**: String values get auto-detection for URLs, hex colors, and ISO-8601 timestamps; each gets a side panel.
- **📋 Clipboard actions**: `c` copies the JSONPath, `y` copies the source slice.

---

## Keyboard shortcuts

| Context    | Action                  | Key                              |
| ---------- | ----------------------- | -------------------------------- |
| General    | Quit                     | `q`                              |
|            | Help / cheatsheet        | `?`                              |
|            | Toggle theme             | `t`                              |
| Navigation | Move selection           | `↑` `↓` `←` `→`                  |
|            | Drill in / expand        | `→` `Enter` `l`                  |
|            | Step back / collapse     | `←` `Esc` `h`                    |
|            | Jump to top / bottom     | `g` `G`                          |
|            | Jump to path             | `:`                              |
| Search     | Global search            | `/`                              |
|            | Next / prev match        | `n` `N`                          |
| Actions    | Copy path                | `c`                              |
|            | Copy source slice        | `y`                              |

---

## Why Twig exists

Many real-world files (API responses, K8s manifests, Terraform state, log exports) contain **sensitive information**. Pasting them into web-based viewers is a security risk. Existing CLI tools like `jq` are powerful for **transformation** but unintuitive for **interactive exploration**. IDE plugins are heavy and require GUI.

Twig is the missing middle ground:

- **Runs entirely locally** — no network calls, no telemetry.
- **Works over SSH** and on headless servers.
- **Optimized for reading**, not mutation.
- **Single binary** — copy it onto a fresh container and it just works.

### Compared to alternatives

| Tool           | Strength                        | Limitation                                |
| -------------- | ------------------------------- | ----------------------------------------- |
| `jq`           | Powerful transformation         | Steep learning curve for exploration      |
| `less` / `cat` | Simple and universal            | No structure awareness                    |
| Web viewers    | Visual and easy                 | Privacy, size limits, trust issues        |
| IDE plugins    | Integrated with the editor      | Heavy, GUI-only, can't run over SSH       |
| **Twig**       | Interactive, structured, local  | Read-only, exploration-focused            |

### Non-goals

Twig is **not**:

- An editor (use `vi` / your IDE).
- A replacement for `jq` (use `jq` for transforms).
- A streaming log viewer (use `lnav` / `less +F`).
- A web service (no server, no API).

---

## Architecture

Built with:

- **[ratatui](https://github.com/ratatui/ratatui)** + **crossterm** — TUI rendering and terminal I/O.
- **rusqlite** with **bundled SQLite + FTS5** — persistent data store with full-text search.
- **serde_json** streaming ingestion — constant memory regardless of file size.
- **serde_yml** — YAML parsing.
- **jsonrepair** — automatic JSON repair for `--fix`.
- **arboard** — system clipboard with graceful fallback.
- **clap** — CLI argument parsing.
- **owo-colors** — colored terminal output for `--print`.

The on-disk cache is a SQLite file with an FTS5 virtual table; old (Python-era) flat-file caches are ignored and rebuilt on first run. There's no migration script because there's nothing to preserve.

### Performance

Benchmark on a 50 MB / ~1 M-node synthetic JSON file (cold start, M-class CPU):

| Build          | Load time   | Throughput     |
| -------------- | ----------- | -------------- |
| Debug          | ~6 s        | ~8 MB/s        |
| Release (LTO)  | ~16 ms      | ~3 GB/s        |

Release builds use `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` to get the binary down to ~4 MB.

---

## CLI reference

```
twig [OPTIONS] <FILE>

Arguments:
  <FILE>   Path to a JSON or YAML file

Options:
  -p, --print            Print pretty-printed output to stdout (non-TUI)
  -o, --output <FILE>    Write output to FILE instead of stdout (with --print or --fix)
  -i, --indent <N>       Indent width for pretty-printed / fixed output [default: 2]
      --fix              Attempt to repair malformed JSON and write a clean copy
      --check            Load the file and print size/node count/throughput stats
      --rebuild-db       Drop the on-disk SQLite cache before loading
      --version          Print version and exit
      --help             Print help and exit
```

---

## Project layout

```
.
├── Cargo.toml          Rust crate manifest
├── Cargo.lock          Locked dependency graph (committed for reproducible builds)
├── schema.sql          SQLite schema (nodes + FTS5 virtual table)
├── .cargo/config.toml  Cross-compile linker config for aarch64-linux
├── src/                Library + binary sources
│   ├── main.rs         CLI entry point
│   ├── cli/            clap parser, --check, --fix, --print modes
│   ├── core/           Node, DataType, paths, store, config, repair
│   ├── adapters/       Streaming JSON and YAML loaders
│   └── tui/            ratatui app, theme, widgets
├── tests/              Integration test suite
├── samples/            Test fixtures (k8s manifest, HAR, etc.)
├── install.sh          POSIX bash installer (curl | sh)
├── asset/              Logo and demo.gif used by README + site
└── .github/workflows/  CI matrix, release pipeline, auto-tag-on-push
```

---

## Releasing a new version

The release pipeline is fully automated:

1. Bump the `version` field in `Cargo.toml` and push to `master`.
2. `.github/workflows/auto-release.yml` detects the version change, creates a `v<version>` tag (idempotent — skips if the tag already exists), and dispatches the release workflow via `workflow_dispatch`.
3. `.github/workflows/release.yml` runs in response: builds Linux x86_64, Linux aarch64 (via `cargo-cross`), macOS x86_64, macOS aarch64, and Windows x86_64 in parallel; packages each as `twig-<triple>.tar.gz` with matching `*.sha256` files; and publishes them to a GitHub release via `softprops/action-gh-release@v2`.
4. `curl -fsSL https://twig.wtf/install.sh | sh` always picks up the highest-versioned release from the GitHub Releases API.

For a hotfix, push the tag manually (with a PAT, since `GITHUB_TOKEN` can't fire tag-push events):

```bash
git tag v3.0.1
git push origin v3.0.1
```

---

## Contributing

Contributions welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup, architecture notes, and submission guidelines.

---

## License

MIT — see [`LICENSE`](LICENSE).
