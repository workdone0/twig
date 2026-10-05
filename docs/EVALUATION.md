# Release evaluation — Twig 3.1

The initial review covered Rust `f6718af` (3.0.0) and Python reference `cce3235`.
It found data-integrity, resource-use, compatibility, installer, and release gaps
despite the existing suite passing. This page tracks the implementation work
for 3.1.0 on 2026-10-05, rather than leaving resolved issues labeled as open.

## Audit resolution

| Finding | Resolution | Regression evidence |
| --- | --- | --- |
| Stale source caches | Snapshot + SHA-256 content key; validate completed schema and SQLite integrity | Changed/deleted input and corrupt-cache tests |
| Partial caches accepted after failure | Private staging; publish only completed immutable databases | Failed trailing JSON, cancellation, concurrent builders |
| Mixed arrays reordered | Rank every scalar/container in source order | Exact reconstructed mixed-array equality |
| Large unsigned integers become strings | Restore signed or unsigned integer values | `u64::MAX` and `i64::MIN` round trips |
| Unicode rendering panics | Grapheme/display-width truncation; byte-stable ASCII folding for highlights | Emoji, CJK, dotted-I and tiny-terminal rendering |
| Path collisions / hidden JSON roots | JSON-quoted bracket paths, unique path index, single-root validation | Special keys, duplicate keys, trailing documents |
| Whole-tree JSON allocation | Serde node visitor with 1,024-node insertion batches | Large-file benchmark with time/RSS budgets |
| All siblings allocated/rendered | 256-row pages; deepest-column viewport; bounded inspector preview | 1,000-item traversal, page-crossing jumps |
| Unused FTS index and wildcard surprises | Remove unused FTS; literal substring search in source order | `%_` literal and numeric array-order tests |
| CLI flag and output inconsistencies | Composable fix/print, conflicting check rejected, shared output path, consistent JSON indentation | Real executable stdout/file/status tests |
| YAML case/multi-document differences | Shared extension detection; stream-aware print mode | Uppercase extension and document-stream tests |
| Silent truncated copying | Complete subtree export or explicit size error | Export depth/size contract; preview/export separation |
| UI lifecycle gaps | Panic-safe terminal guard, worker cancellation/join, disconnect errors, synchronized focus and propagated draw errors | Injected-config tests and render/interaction regressions |
| Scalar roots and missing controls | Selectable root, mouse controls, Vim movement and first/last keys | Scalar, navigation and key-event coverage |
| Clipboard handle lifetime | Retain handle while application runs | Host-dependent persistence remains a manual check |
| Real-profile test side effects | Temporary loader/config injection; remove ignored theme test and leaked test directories | Entire suite runs inside workspace sandbox |
| Plaintext cache lifecycle undocumented | Private Unix permissions, retention, clear/no-cache modes and explicit docs | Ephemeral and cache contract tests |
| macOS config migration missing | Import legacy config only when new config is absent | Explicit source contract; existing-config precedence |
| Broken auto-release dispatch | Authenticated token, required permission, retry existing unpublished tag after successful CI | Workflow review and hosted CI/release run |
| Wrong-ref release binaries | Resolve and validate tag/commit before verification/build jobs | Tag input validation and immutable checkout |
| Installer failures | Bash help, checksum fallback/fail-closed policy, release pinning, destination handling, cleanup and atomic install | Offline installer contract suite |
| Unsupported declared Rust minimum | Set 1.88 and test it alongside stable on three OSes | Local minimum-version tests and CI matrix |
| Unsound/unmaintained YAML dependencies | Replace `serde_yml`/`libyml` with `serde_norway` | RustSec audit and YAML regression suite |
| Website/source drift | Rewrite website in main repo; generate guides and installer from canonical files | Static link/anchor checks and browser QA |

The YAML dependency decision follows [RustSec's serde_yml advisory](https://rustsec.org/advisories/RUSTSEC-2025-0068)
and the [serde_norway project](https://github.com/cafkafk/serde-norway). Clipboard
image support is disabled because Twig only copies text, reducing unnecessary
dependencies. CI also checks licenses, advisory status, and dependency sources.

## Verification commands

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo +1.88.0 test --locked --all-targets
cargo audit --deny warnings
shellcheck install.sh
python3 scripts/test_installer.py
python3 scripts/build_site.py
python3 scripts/check_site.py
cargo build --release --locked
python3 scripts/benchmark.py --binary target/release/twig
```

The release workflow is the authoritative record of platform results. Tests run
without real user caches/config or a live clipboard. The benchmark's Linux CI
budget is 120 seconds and 512 MiB for 90,000 deterministic items; this is a
regression ceiling, not a speed guarantee or a comparison against Python.

## Local verification results

On macOS, Rust 1.88.0 and the current installed toolchain passed 90 unit,
integration, and regression tests with none ignored; the documentation test
also passed. Formatting, Clippy, seven installer contract tests, site link
checks, and the dependency audit passed. The audit reported no advisories or
warnings. Browser checks covered desktop/mobile layouts, sample navigation,
installation tabs, copying, and guide links.

A release build ingested the deterministic 42,378,891-byte JSON fixture
(900,001 nodes) in 12.24 seconds with 179.92 MiB peak RSS on the development
Mac. These are local observations, not cross-platform performance guarantees.
Hosted CI and release logs provide the platform-specific verification record.

## Deliberate limits and external requirements

- YAML parser memory is not bounded like the JSON node stream. Very large scalar
  strings and complete exports can allocate significant memory.
- Search remains a SQLite substring scan. Pagination bounds navigation data,
  not the cost of every possible search query.
- The TUI models JSON-compatible YAML. Formatting cannot preserve original
  comments, whitespace, aliases, or quoting. Non-finite values, tagged values,
  complex mapping keys, and JSON duplicate keys are not silently normalized.
- Clipboard export caps selections at 10,000 nodes and reports an error above
  the cap. Inspector previews are explicitly limited.
- Clipboard behavior after process exit, SSH integration, and native terminal
  restoration need manual checks on each supported desktop/terminal. Automated
  widget tests do not establish every platform integration.
- Caches and temporary snapshots are unencrypted. Normal cleanup and retention
  are not secure erasure; abrupt termination can leave temporary files.
- Native OS signing/notarization requires platform credentials and is not
  configured. Checksum/provenance verification does not replace OS signing.
- GitHub branch protection and private vulnerability reporting are repository
  settings. Maintainers should configure them separately from source changes.
- Watch mode and user-defined themes remain out of scope; documentation does not
  advertise them as existing features or promise a delivery version.

These limits are documented in the user guide instead of being hidden behind
claims of universal static binaries, constant memory, or complete Python parity.
