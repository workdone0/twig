# Rust architecture

## Entry points

`main` parses `cli::Cli` and selects cache management, formatting/repair,
noninteractive validation, or the TUI. Clap rejects incompatible modes.
Formatting reads a whole value/document stream and bypasses the store. JSON
stdin is accepted by CLI modes; the TUI requires a terminal.

Output is plain text with consistent JSON indentation. File output is written
to a same-directory temporary file, synced, and atomically persisted only after
successful parsing. A broken stdout pipe exits quietly. A terminal guard and
panic hook restore raw mode, alternate screen, and mouse capture.

## Input snapshots and cache publication

`LoadOptions` carries cache location, ephemeral mode, and shared cancellation.
`load_cached` reads the source into a private temporary snapshot while computing
SHA-256. Parsing and the cache digest therefore refer to identical bytes even
if another process changes the original file. Opening/snapshotting errors are
reported before cache reuse.

Caches include a path-derived identifier, full content digest, and schema version.
Only a nonempty database with the completion version and passing SQLite
`quick_check` can be reused. An invalid cache is ignored. Hashing/verification
cost is included in warm-load timing; “warm” does not mean no input I/O.

Each builder owns a private staging directory/database. Indexes are deferred,
node batches are committed there, and completion metadata is written after a
successful parse/index build. A hard link publishes the finished immutable
SQLite file without overwriting another builder's result. Existing readers keep
their own generation. If publication is unsupported or another builder wins,
the completed private database remains usable and is cleaned up with the Store.
No mutable database is shared between builders.

Published files are opened read-only. The completion version is independent of
the package version and must change when stored semantics change. Legacy caches
are not reused. Unix directory/file modes are 0700/0600. New-format generations
are pruned on load by age (seven days) and existing total size (512 MiB).
`--clear-cache` removes database/sidecar files; `--no-cache` avoids publication.
Temporary storage is unencrypted; abrupt process termination can leave files.

## Node-event parsing

JSON uses a custom Serde `DeserializeSeed`/`Visitor`, emitting object/array/scalar
nodes directly into a 1,024-node insertion buffer. It never creates a full JSON
`Value` for the document. Sibling rank is assigned independently of child type.
Every scalar preserves its JSON type, including the unsigned 64-bit range.

A single JSON root is required; trailing values are rejected. Unique path indexes
reject duplicate keys. Ordinary identifier keys use dot syntax and other keys
use JSON-quoted brackets, preventing collisions between `a.b` and nested `a/b`.
Maximum nesting is 128; the parser's own recursion boundary also applies.

YAML uses `serde_norway` document deserializers with the same visitor and bounded
node buffer. Documents become children of a virtual array, in source order.
The YAML parser may buffer document state, so its memory is not guaranteed to be
bounded. The TUI model intentionally supports JSON-compatible values, not all
YAML tags or complex mapping keys. YAML print mode uses the YAML value model.

## Store and search

`schema.sql` contains a node table and parent/rank and path indexes. Containers
store child relationships rather than embedded JSON. Batched insertion reuses
a prepared statement. Immutable caches have no FTS table: the earlier index was
unused by the actual search implementation.

Search uses `instr(lower(...), lower(query))` against keys/values, so `%` and `_`
are literal. SQLite's ASCII case folding is matched by the UI highlighter;
non-ASCII matching is exact. Matches traverse row IDs in insertion/source order,
with wraparound. Search is a scan, and broad queries may be slow on large inputs.
Paths are exact lookups with the historical first-YAML-document fallback.

## TUI

Each column owns a page of at most 256 siblings. Movement/jumps load the relevant
page; rendering constructs only that page's items. Horizontal layout displays
the deepest columns that fit the current width. Scalar roots get a selectable
row. Mouse selection and scrolling use the most recently rendered column areas.
Keyboard handling includes arrows, Vim movement, first/last, search/jump, themes,
and clipboard actions.

Text truncation uses grapheme boundaries and terminal display widths. Search
highlighting never applies offsets from Unicode case conversion to original text.
Inspector previews have a four-level/200-node budget, with at most 30 children
per container. They are explicitly labeled as limited previews. Clipboard
export uses complete reconstruction and refuses subtrees over 10,000 nodes,
rather than silently replacing descendants with placeholder strings.

A background worker loads data. The UI initializes focused state on completion,
converts worker disconnects into errors, and propagates draw failures. A worker
guard requests cancellation and joins the worker on exit; cancellation is checked
during snapshot reads, parser reads, and node emission. Clipboard handles are
retained on the UI thread until process exit. Config save errors appear in status.

## Configuration and test boundaries

Config is JSON with unknown keys preserved. macOS imports an existing Python
XDG config only when the new Application Support config does not exist.
`App::with_config` and loader options permit tests to inject isolated temporary
state; tests do not require a user's theme/cache or real clipboard.

## Distribution and website

CI checks Rust stable and the 1.88 minimum across three OSes, plus installer,
website, dependencies, and cold-ingestion budgets. Auto-release runs after CI
succeeds on master. Release resolves one existing tag/commit, verifies the
manifest version, runs CI, builds five targets, and publishes archives, checksums,
build metadata, and provenance. Recovery does not require moving a tag.

`website/` is a dependency-free static site. Python build tooling generates guides
from repository Markdown and copies both canonical installers (`install.sh` and `install.ps1`). Website deployment
follows a successful release or a maintainer's manual dispatch, publishing to
`twig-web` for the existing `twig.wtf` GitHub Pages domain. Old Astro source on
`gh-pages` is no longer the maintained source.
