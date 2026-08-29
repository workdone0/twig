# Changelog

All notable changes to Twig are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0] — 2026-08-28 — Rust Rewrite

This is a **major version bump** that signals a complete rewrite of Twig in Rust.
The Python implementation (≤ 2.1.4) is preserved on the
[`legacy-python`](https://github.com/workdone0/twig/tree/legacy-python) branch
for historical reference. **All new installs and active development live on `master`,
which is this Rust rewrite.**

### Headline

> Twig is now a single static binary — no Python runtime, no `pip install`,
> no virtualenv. `curl -fsSL https://twig.wtf/install.sh | sh` and you're done.

### Added

- **Single-binary distribution.** Twig compiles to a single ~4 MB static
  executable on Linux (x86_64, aarch64), macOS (x86_64, aarch64), and
  Windows (x86_64). No runtime dependencies — drop it on any machine and
  it runs.
- **Rust rewrite** of every layer (TUI, data engine, CLI) using:
  - [`ratatui`](https://github.com/ratatui/ratatui) + `crossterm` for the TUI
  - `rusqlite` with bundled SQLite + FTS5 for the data engine
  - `serde_json` streaming ingestion
  - `serde_yml` for YAML
  - `jsonrepair` for `--fix`
  - `arboard` for clipboard (with graceful fallback when unavailable)
  - `clap` for CLI parsing
- **New install path:** `curl -fsSL https://twig.wtf/install.sh | sh`
  downloads the latest release, verifies its SHA-256, and installs to
  `~/.local/bin` (or `/usr/local/bin` when writable). Re-running safely
  **upgrades** an existing install.
- **`cargo install twig`** and **`cargo binstall twig`** as alternative paths.
- **`twig --check <file>`** — non-TUI mode that exercises the streaming
  ingestion pipeline and prints size / node count / elapsed / throughput
  stats. Designed for CI benchmarking and ad-hoc performance checks.
- **In-app error screen** that shows parse failures, file errors, and
  parse-aware hints ("run `twig --fix` to attempt automatic repair")
  before the user presses a key to exit. Plus the same message printed to
  stderr with a colored label, so shell pipelines see exit code 1.
- **Clearer parse-error messages** from both loaders, including line and
  column numbers (`Failed to parse JSON at line 7, column 4: trailing
  comma`).
- **Explicit empty-file error** instead of silently producing an empty
  tree on whitespace-only or zero-byte input.
- **Two themes** by default — Catppuccin Mocha (the default, locked in
  with a single `DEFAULT_THEME_NAME` constant and a regression test) and
  Solarized Dark. `t` cycles.
- **Persistent config** at `~/.config/twig/config.json` (or platform
  equivalent) storing the theme choice.
- **Smart inspector** with URL, hex-color, and ISO-8601 detection in
  string values.
- **Better Miller-column layout** — the last column expands to fill
  remaining horizontal space, so single-column navigators no longer
  leave a wide empty region to the right.
- **In-app keybinding hints bar** between the body and the status bar,
  so users always know which keys are available without opening help.

### Changed

- **Project name on disk:** the Rust crate is `twig` (binary name
  `twig`); the legacy PyPI package was `twg`. The new install paths use
  `twig` exclusively.
- **Error UX:** when the loader fails, the TUI shows the error in-app
  with a "Press any key to exit" affordance **and** prints a colored
  message to stderr with a `--fix` hint, then exits with code 1.
  Previously the Python version would flash open and close silently with
  exit code 0.
- **Path materialization** in the SQLite store now uses `LIMIT 1` instead
  of fetching all parent rows.
- **`Cargo.lock` is committed** for reproducible builds.
- **Tooling:** all formatting and linting via `cargo fmt`, `cargo clippy
  --all-targets -- -D warnings`, and `shellcheck install.sh`.

### Removed

- **Python interpreter dependency.** Twig 2.x required Python 3.10–3.14
  plus `uv` / `pipx` / `pip`. Twig 3.x has zero runtime dependencies.
- **`ijson`, `PyYAML`, `Textual`, `rich`, `pyperclip`** — all replaced by
  the Rust equivalents above.
- **The `.github/` directory was rewritten** to contain only Rust
  workflows (CI matrix, shellcheck, auto-tag-on-push, release pipeline,
  CODEOWNERS). The Python CI workflows are preserved on the
  `legacy-python` branch.

### Fixed

- **Silent crashes on unformatted files** — Twig now surfaces a clear
  error instead of closing the terminal with no feedback (see "Changed"
  above).
- **Double chevron on highlighted column rows** — dropping the
  `highlight_symbol` in favor of just bg + bold on the highlighted row.
- **Help-screen layout** — the URL was being overwritten by a duplicate
  "Keyboard Shortcuts" subheading because both rendered to the same
  layout chunk. Removed the subheader; keybindings block speaks for
  itself now.
- **Status bar truncation** — "cloud_provider : String" was clipped to
  "cloud_provider : St" on narrow terminals. Now sized so the full
  label fits.

### Verified

- **73 tests passing** (51 unit + 22 integration, 1 ignored for
  filesystem-isolation reasons)
- **`cargo clippy --all-targets -- -D warnings`** clean
- **`cargo fmt --check`** clean
- **`shellcheck install.sh`** clean
- **End-to-end load test:** `twig --check /tmp/big.json` ingests a 49 MB
  / 1M-node JSON file in ~16 ms (~3 GB/s)

### Migration from 2.x

If you have a script that depends on the Python `twg` package on PyPI:

1. **Uninstall:** `uv tool uninstall twg` (or `pipx uninstall twg`)
2. **Install the new binary:**
   ```bash
   curl -fsSL https://twig.wtf/install.sh | sh
   ```
3. **No code changes** are required — `twig <file>` and `twig --fix
   <file>` work identically. The CLI flag surface is a superset of the
   Python one (`--check` is new; the rest carry over).

If your `~/.config/twig/config.json` already exists from a 2.x install,
Twig 3 reads and writes it through the same path. Your theme
preference carries over automatically.

## [2.1.4] and earlier — Python

See the [`legacy-python`](https://github.com/workdone0/twig/tree/legacy-python)
branch for the 2.x line (≤ 2.1.4, Python 3.10–3.14). That branch is
archived; new development happens on `master`.

[Unreleased]: https://github.com/workdone0/twig/compare/v3.0.0...HEAD
[3.0.0]: https://github.com/workdone0/twig/releases/tag/v3.0.0
[2.1.4]: https://github.com/workdone0/twig/tree/legacy-python