# Twig 3.0.0 — Rust Rewrite

The first Rust release of Twig is out. Twig is now a single static binary — no Python runtime, no `pip install`, no virtualenv. Drop it on any machine and it runs.

```bash
curl -fsSL https://twig.wtf/install.sh | sh
```

This is a **major version bump** (2 → 3) that signals a complete rewrite of every layer in Rust. The Python implementation (≤ 2.1.4) is preserved on the [`legacy-python`](https://github.com/workdone0/twig/tree/legacy-python) branch for historical reference. New installs and active development live on `master`, which is this Rust rewrite.

---

## What you get

- **One binary, anywhere.** Linux (x86_64, aarch64), macOS (x86_64, aarch64), and Windows (x86_64). About 4 MB per target. Zero runtime dependencies.
- **Same workflow.** `twig <file>` opens the TUI. `twig --fix <file>` repairs unformatted JSON. `twig --print <file>` dumps a colored, pretty-printed copy. Nothing in your muscle memory changes.
- **Cold start is gone.** A 50 MB / 1 M-node JSON file loads in roughly 16 ms — a ~3 GB/s throughput on the streaming ingestion path. The Python implementation needed about two seconds for the same input.

## What's new in 3.0.0

### Engine

- **Bundled SQLite with FTS5.** No more system `sqlite3` version dance; every install has the same full-text search index behavior.
- **Streaming ingestion via `serde_json`.** Constant memory relative to file size — load 10 MB or 10 GB without the process growing.
- **`jsonrepair`-based `--fix`.** Repairs common JSON malformations (trailing commas, unquoted keys, single quotes, missing braces) and writes a clean round-trippable file.

### TUI

- **`ratatui` + `crossterm` renderer.** Lighter and faster than the Python `Textual` / `rich` stack; no GIL, no event-loop contention.
- **In-app error screen.** When the loader fails, the TUI shows the error with a "Press any key to exit" affordance instead of closing the terminal silently. The same error is also printed to stderr with a colored label and a `--fix` hint, so CI scripts see exit code 1.
- **Smart inspector.** String values get auto-detection for URLs, hex colors, and ISO-8601 timestamps — each gets a side panel.
- **Miller columns that fill the terminal.** The right-most column expands to fill remaining horizontal space, so single-column navigators no longer leave a wide empty region.
- **In-app keybinding hints bar** between the body and the status bar, so the user can always see what's available without opening help.
- **Two themes**, both shipped by default: **Catppuccin Mocha** (the locked-in default) and **Solarized Dark**. `t` cycles.

### CLI

- **`twig --check <file>`** is new. Loads the file through the same streaming pipeline the TUI uses and prints size / node count / elapsed / throughput. Designed for CI benchmarking and ad-hoc perf checks. Non-TUI; no terminal takeover.
- All other flags (`-p`/`--print`, `-o`, `-i`, `--rebuild-db`, `--fix`) carry over from the Python line.

### Distribution

- **`install.sh`** — a POSIX `bash` script that downloads the latest release, verifies its SHA-256 against the GitHub-published `*.sha256` file, and installs to `~/.local/bin` (or `/usr/local/bin` when writable). Re-running upgrades safely. Supports `--to`, `--version`, `--method fetch|build`, `--yes`, `--help`.
- **Cross-platform release artifacts** for the five target triples, produced from `release.yml` on every `v*` tag push.

### Internals

- `Cargo.lock` is committed for reproducible builds.
- `[profile.release]` uses `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` — that's what gets the binary down to ~4 MB.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are both green on every commit; `shellcheck install.sh` is clean.

## Migration from 2.x

If you have a script pinned to the PyPI `twg` package:

```bash
uv tool uninstall twg                       # 1. uninstall the old
curl -fsSL https://twig.wtf/install.sh | sh # 2. install the new
```

That's it. No code changes required.

- The CLI flag surface is a **superset** of the Python one. `--check` is new; everything else (`--fix`, `--print`, `-o`, `-i`, `--rebuild-db`, `--file`, positional `<file>`) carries over.
- The persistent config at `~/.config/twig/config.json` (or the platform equivalent) is read and written by both versions through the same path, so your theme preference carries over automatically.
- The on-disk cache layout changed: 3.x uses a SQLite file with an FTS5 virtual table rather than the 2.x flat-file index. Old cache files are ignored (they'll be re-built on first run); there's no migration script because there's nothing to preserve.

## Verified

- **73 tests passing** (51 unit + 22 integration; one is `#[ignore]` because it mutates the real user `config.json`).
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --check` clean.
- `shellcheck install.sh` clean.
- End-to-end load test: `twig --check /tmp/big.json` ingests a 49 MB / 1 M-node synthetic JSON file in ~16 ms (~3 GB/s).

## What's not in 3.0.0

- **Mouse support.** The Python line had it; the Rust rewrite intentionally ships without it. Keyboard-only navigation is the primary interface and we want to keep the surface tight. If you need mouse support, file an issue and we'll add it.
- **Custom themes via config.** You can switch between Catppuccin Mocha and Solarized Dark. Defining a new theme in `~/.config/twig/config.json` is on the roadmap for 3.1.
- **`twig --watch`.** Live-reload of the file as it's edited is on the roadmap; not in this release.

## Acknowledgements

Thanks to everyone who filed issues, tested releases, and reported load-times on the Python line. The Rust rewrite exists because the Python architecture had a fundamental ceiling on cold-start time that no amount of optimization could move. The Python branch will stay around for reference but no new development happens there.

---

**Install:** `curl -fsSL https://twig.wtf/install.sh | sh`
**Source:** [github.com/workdone0/twig](https://github.com/workdone0/twig)
**Python line:** [legacy-python](https://github.com/workdone0/twig/tree/legacy-python)
**Full diff:** [8c1a9cf…](https://github.com/workdone0/twig/compare/0fe15b8...8c1a9cf)
