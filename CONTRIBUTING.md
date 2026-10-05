# Contributing to Twig

Thanks for helping make a focused, reliable terminal tool. Bugs, documentation,
accessibility improvements, and well-tested code changes are welcome. Please
keep discussions constructive and explain the user problem before proposing a
large feature or new dependency.

## Setup

The application is Rust; the website and tooling use Python's standard library.
Use Rust 1.88+ with Cargo, rustfmt, Clippy, a C compiler/linker for bundled SQLite,
and Python 3.10+. ShellCheck checks the Bash installer. No Python application
runtime, Node installation, or website package install is required.

```bash
git clone https://github.com/workdone0/twig.git
cd twig
cargo build --locked --all-targets
cargo run --locked -- samples/cloud_infrastructure.json
```

## Project structure

| Location | Responsibility |
| --- | --- |
| `src/main.rs`, `src/cli/` | CLI arguments, output, terminal lifecycle |
| `src/adapters/` | Event parsing, immutable cache publication, cancellation |
| `src/core/` | Node types, SQLite queries, paths, configuration, repair |
| `src/tui/` | Event loop, themes, paginated navigation, inspector and modals |
| `schema.sql` | Node table and navigation indexes |
| `tests/` | Integration, process-level CLI, data and lifecycle regressions |
| `scripts/` | Offline installer tests, benchmarks, static website build/checks |
| `website/` | HTML/CSS/JS source; generated `dist/` is ignored |
| `docs/` | Architecture, migration, audit resolution |
| `.github/workflows/` | CI, release, website deployment |

See [architecture](docs/ARCHITECTURE.md) before changing ingestion or storage.
Keep widgets independent from parsing. Preserve scalar types, sibling order,
source-path uniqueness, atomic publication, and cancellation checkpoints.

## Required checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
shellcheck install.sh
bash -n install.sh
python3 scripts/test_installer.py
python3 scripts/build_site.py
python3 scripts/check_site.py
```

CI runs stable and minimum-supported Rust on Linux, macOS, and Windows. Test
caches and config must use `tempfile`, loader options/`with_cache_dir`, and
`App::with_config`; never mutate a contributor's profile. No test should require
a real clipboard, GUI, network service, or existing configuration.

For CLI changes, spawn the executable and assert stdout, stderr, exit status,
and output-file contents. For storage changes, compare reconstructed values,
including mixed containers/scalars, unsigned integers, duplicate/special keys,
Unicode, cancellation, corrupted caches, and concurrent loads. Keep process and
widget coverage: testing a loader is not a substitute for testing CLI dispatch.

Manually verify JSON/HAR and multi-document YAML, scalar/empty roots, narrow and
resized terminals, mouse/keyboard navigation, search/jump, copy failures, quitting
during loading, and terminal restoration. Clipboard ownership after exit depends
on the platform manager; test Linux desktop and SSH separately when changing it.

## Performance

```bash
cargo build --release --locked
python3 scripts/benchmark.py --binary target/release/twig
```

The deterministic benchmark reports cold wall time and peak RSS. CI checks a
120-second / 512-MiB budget on Ubuntu for 90,000 generated items. These are
regression budgets, not product speed guarantees. Report hardware, Rust version,
commit, bytes, nodes, cache state, and memory when comparing results.

Ingestion batches and navigation pages must remain bounded. YAML parser state,
large scalar strings, search scans, and complete exports have different resource
profiles; do not claim constant memory for all operations.

## Website and documentation

Edit `website/index.html`, `style.css`, and `app.js` for the landing page.
User guides are generated directly from README, CONTRIBUTING, release notes,
and `docs/`; edit those Markdown files rather than generated pages.

```bash
python3 scripts/build_site.py
python3 scripts/check_site.py
python3 -m http.server 4321 --directory website/dist --bind 127.0.0.1
```

Check desktop/mobile widths, keyboard navigation, readable contrast, sample data
exploration, install tabs, copy feedback, and links. The site uses no tracking,
external fonts, or framework runtime. `install.sh` is copied from the root at
build time so it cannot silently diverge from application releases.

`master` is the source of truth for both app and website. The old `gh-pages`
branch held Astro source; it is historical. Do not deploy from that branch.
`twig-web` contains generated public files for the existing `twig.wtf` domain.

## Security and dependencies

Use [GitHub private vulnerability reporting](https://github.com/workdone0/twig/security/advisories/new)
for sensitive issues where available; do not attach real secrets or private
input files to public issues. See [SECURITY.md](SECURITY.md).

Run `cargo audit --deny warnings` when changing dependencies. Audit results are
point-in-time, not a security guarantee. The 3.1 work replaces the RustSec-flagged
`serde_yml`/`libyml` stack with `serde_norway`. Avoid dependencies that add unused
image/desktop functionality. Keep `Cargo.lock` updated and preserve license
notices. CI checks dependencies with cargo-deny.

## Pull requests

Create a focused branch and describe the concrete problem, final behavior,
compatibility changes, and checks performed. Use conventional titles such as
`fix: preserve mixed array order`. Include a regression test for bugs. Update
README/help/migration docs when flags, keys, formats, cache/config paths, or
serialization change. Screenshots help for TUI and website changes.

## Releasing

The default branch is **master**, not main. A merge alone does not create a new
version: update `Cargo.toml`, `Cargo.lock`, CHANGELOG, and RELEASE_NOTES together.

1. Open a PR and complete all CI jobs and relevant manual checks.
2. Merge the release commit into `master`.
3. Successful **CI** on that exact push triggers **Auto release after CI**.
4. Automation creates the missing `vX.Y.Z` tag and dispatches **Release** with an
   authenticated token. If a tag exists without a release at the same verified
   commit, dispatch is retried. Existing releases are not republished.
5. Release resolves the tag once, verifies its package version, re-runs CI at
   that commit, builds all five targets, and publishes checksums, provenance,
   build metadata, and the checked-in release notes.
6. Successful release completion triggers **Website**, which builds and validates
   the site from `master`, deploys generated files to `twig-web`, and explicitly
   requests a Pages build (token-authored pushes do not trigger it).

For recovery, run Release manually with an existing `vX.Y.Z` tag. Builds always
check out the resolved tag commit, independent of the dispatch ref. A tag/manifest
mismatch fails before building. A conflicting tag must be investigated, not
silently moved. For docs-only website changes, run Website manually after CI.

Check that the published version, archive checksums, installer, website, and all
target downloads match. OS signing/notarization requires maintainer credentials
and is not configured; do not claim signed native binaries. Repository-level
branch protection and private security reporting are maintainer settings, not
properties enforced by these source files.
