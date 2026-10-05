# Changelog

Notable changes to Twig. Version 3 is the Rust implementation; version 2 and
earlier are the Python implementation.

## [Unreleased]

- Add a per-user Windows PowerShell installer with checksum/version verification,
  PATH setup, staged upgrades, cleanup, and native PowerShell 5.1/7 tests.
- Clarify platform installation, manual verification, troubleshooting, upgrades,
  and uninstall steps; harden Bash platform/version checks and cleanup.
- Polish the website with muted rust-red colors, clearer copy, consistent guide
  navigation, complete installation commands, and keyboard/mobile demo support.

## [3.1.0] — 2026-10-05

### Fixed

- Cache freshness, failed-load recovery, concurrent publication, mixed-array
  order, unsigned-number fidelity, path collisions, and Unicode rendering.
- CLI mode conflicts, fix/print composition, indentation and YAML consistency,
  atomic output, scalar roots, initial focus, and terminal/worker cleanup.
- Installer integrity/portability, release authentication and retry behavior,
  exact-tag builds, and test isolation from user profiles.

### Added

- Bounded JSON node ingestion and paginated columns; limited inspector previews
  separate from complete clipboard export with an explicit size cap.
- Mouse/Vim navigation, first/last keys, JSON stdin, no-cache/clear-cache modes,
  private cache permissions/retention, and macOS legacy config import.
- Rust 1.88 CI, data/CLI regressions, offline installer tests, dependency checks,
  resource-budget benchmarks, and release provenance/build metadata.
- Rewritten website, generated guides, updated contribution/security/migration
  docs, and a release evaluation tracking the audit findings.

### Changed

- Literal substring search follows source order; quoted bracket paths disambiguate
  special keys. Reject duplicate keys/multiple JSON roots. Remove unused FTS.
- Replace unsound YAML dependencies with serde_norway; disable clipboard images.
- Formatted CLI output is plain text. Preserve the Python `-v` alias and add `-i`.
- Automatic releases wait for CI; website source now lives on master.

## [3.0.0] — 2026-08-28 — Rust rewrite

The release date above is retained from the original changelog. The description
below has been corrected against the implementation; it is not a new release
or a revalidation of published artifacts.

### Added

- Native Rust `twig` executable with ratatui/crossterm UI, bundled SQLite,
  serde_json/serde_yml loaders, jsonrepair, clap, and arboard clipboard support.
- `--check` mode reporting load timing, size, and node count. Use `--rebuild-db`
  when measuring ingestion or validating changed input.
- In-app load errors and parse-error hints, with nonzero exit on load failure.
- Catppuccin Mocha and Solarized Dark themes with persistent selection.
- Inspector detection of URLs, hex colors, and timestamps; keyboard hints.
- Rust CI, cross-platform release packaging, SHA-256 sidecar assets, and a Bash
  installer. Distribution limitations are tracked in the project evaluation.

### Changed

- Python runtime dependencies replaced by Rust dependencies. Python is no longer
  needed to run Twig; OS libraries and services can still be required.
- Only the `twig` binary is built; the Python `twg` alias is not provided.
- Version shorthand changed from `-v` to `-V`.
- On macOS, config now uses `~/Library/Application Support/twig/config.json`.
- Cache filename hashing changed from Python MD5 to Rust `DefaultHasher`.
  Both versions use SQLite; existing Python caches are not migrated.
- `Cargo.lock` is committed; release builds enable thin LTO and symbol stripping.
- Single-column layout fills the available navigator width; help, status,
  selection styling, and inspector presentation have regression coverage.

### Compatibility notes

The Rust port is not a drop-in replacement for every Python workflow. Combined
fix/print behavior, formatting options, memory use, and interaction features
need attention. See [migration](docs/MIGRATION.md) and [evaluation](docs/EVALUATION.md).
Earlier test-count and ~16 ms / ~3 GB/s ingestion claims are superseded by the
scoped verification record in the evaluation; no cold-load guarantee is made.

## [2.1.4] and earlier — Python

Source is retained on the
[`legacy-python`](https://github.com/workdone0/twig/tree/legacy-python) branch.
The Python package name was `twg`; it installed both `twig` and `twg` commands.

[Unreleased]: https://github.com/workdone0/twig/compare/v3.1.0...HEAD
[3.0.0]: https://github.com/workdone0/twig/releases/tag/v3.0.0
[2.1.4]: https://github.com/workdone0/twig/tree/legacy-python

[3.1.0]: https://github.com/workdone0/twig/releases/tag/v3.1.0
