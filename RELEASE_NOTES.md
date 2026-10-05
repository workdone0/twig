# Twig 3.1.0 — Reliable data, a clearer path

This release hardens the Rust rewrite, improves everyday navigation, and brings
the website and documentation into the same repository as the application.

## Correctness and privacy

- Source changes automatically select a new content-validated cache. Failed or
  cancelled imports never publish partial data; concurrent loads are isolated.
- Mixed arrays preserve order, large unsigned integers stay numeric, and quoted
  bracket paths distinguish punctuation-containing keys from nested keys.
- Unicode previews/highlights no longer slice through UTF-8. Terminal and worker
  guards restore state and cancel unfinished work on exit.
- Private cache permissions, automatic retention, `--no-cache`, and
  `--clear-cache` give local data a defined lifecycle.
- The RustSec-flagged YAML dependencies are replaced with `serde_norway`.

## Navigation and CLI

- JSON node-event parsing uses bounded insertion batches. Navigation uses
  256-row pages and keeps the deepest columns visible; inspector previews have
  explicit limits.
- Mouse selection/scrolling, Vim movement, Enter-to-open, Home/End, and `g/G`.
- Literal substring search in document order. `%` and `_` are literal characters.
- Copy complete selected values up to 10,000 nodes, or show an explicit size
  error; copied data is never silently depth-truncated.
- `--fix --print` composes correctly, JSON indentation works with `-o`, YAML
  detection is case-insensitive, and YAML print supports document streams.
- JSON stdin for noninteractive modes, `-i` indentation, and the legacy `-v`
  version alias. Output-file replacement is atomic; CLI output is plain text.
- Automatic legacy macOS config import when the new config is absent.

## Distribution and documentation

- Fail-closed checksum verification with Linux/macOS tools, working piped help,
  pinned source installs, and destination-aware atomic installation.
- CI on stable and Rust 1.88 across Linux, macOS, and Windows, plus offline
  installer tests, dependency checks, and a cold-ingestion resource budget.
- Releases build the verified tag commit and publish checksums, provenance,
  build metadata, and these notes. Automatic releases wait for successful CI.
- A complete website rewrite with an interactive sample explorer, responsive
  layout, accessible controls, and guides generated from repository Markdown.

## Upgrade notes

Install with `curl -fsSL https://twig.wtf/install.sh | bash`, or download the
archive for your target. Source builds require Rust 1.88+ and a C compiler.

Old caches are ignored; close Twig and use `twig --clear-cache` to remove them.
Punctuation keys now use paths such as `.regions["us-east-1"]`. JSON duplicate
keys and multiple top-level values are rejected. Help is `?`; `h` moves back.
See [the migration guide](https://twig.wtf/guide/migration/).

YAML parser memory, large scalars, full exports, and substring scans still have
input-dependent resource costs. TUI YAML supports JSON-compatible values.
Clipboard persistence after exit depends on the desktop manager. Native binaries
are not OS code-signed/notarized; Linux targets use GNU libc. See the guide for
cache privacy and platform limitations.
