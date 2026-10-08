# Architecture: one Rust core, two interfaces

Twig has two interactive front ends: a native terminal app and a browser explorer.
Parsing and exploration live in Rust; each interface owns its input, storage,
rendering, and platform integration. CLI formatting and repair remain native.

## Shared core and browser explorer

`crates/twig-core` contains the JSON/YAML node-event parsers, model, storage
contracts, path resolution, lineage, preview and export algorithms. The native
crate re-exports its model and parser entry points for compatibility. SQL search
and indexed child/path lookup remain native optimizations behind `NodeStore`.
`NodeSink` accepts document-order events; IDs are local to a document. Native
loading retains cancellation, 1,024-node SQL batches, snapshots and cache publication.

`MemoryStore` is the browser storage adapter, written in Rust. It checks every
node before retaining it, with limits of 250,000 nodes and 128 MiB of estimated
retained allocations. The estimate includes index/path duplication and per-node
overhead; it is not a hard process-memory ceiling. Input buffers, parser state,
temporary strings, exports and the WebAssembly allocator consume additional
memory. YAML buffering remains a parser limitation. Limits can reject a file
smaller than the 20 MiB input cap, and a constrained device may fail earlier.

`crates/twig-wasm` exposes a small `wasm-bindgen` document API. The web worker owns
the Rust document. Messages carry commands and bounded results, never the full
tree. Scalar values cross the bridge as formatted strings so unsigned 64-bit
numbers retain their precision. Lists contain at most 256 nodes, with clipped
row labels. Previews retain native depth/node/child limits and additionally cap
browser display at 64K characters. Complete clipboard exports retain the
10,000-node limit and add a 4 MiB browser limit; copying a path uses its full
stored value, not the clipped display label.

`web/` contains the TypeScript/React interface. Each open document gets its own
worker. Replacement, closing and cancellation terminate that worker; a generation
counter discards stale file reads and replies. Queries are serialized. Only the
theme preference is saved; there is no server processing, document persistence, telemetry, service worker or third-party
runtime request. Bundled examples need no data fetch. Browser files are never
included in URLs, logs or request bodies. The page downloads its own scripts and
WASM, so the installed TUI remains useful for fully offline workflows.

Build the WASM bridge, run its engine gate, build the Vite application, then run
`build_site.py`. The assembler places the explorer at `/`, installation at
`/install/`, and preserves `/guide/`, `/install.sh` and `/install.ps1`. A missing
web bundle fails the build. Deployment remains on GitHub Pages with the existing
domain; no API hosting or DNS migration is needed.

## Shared presentation contract

`crates/twig-core/themes.json` defines exactly two palettes, Dark and Light.
Ratatui converts these colors to RGB and the web interface consumes the same
file as CSS variables. Dark is the default on both surfaces. Native legacy
theme names resolve to Dark, and theme toggles save `dark`/`light`. The browser
persists only `twig.theme`, handling unavailable local storage without blocking
the application. Documents and selected paths are never persisted or synced.

`crates/twig-core/bindings.json` defines normal-mode key bindings for both
interfaces. Each adapter translates the shared actions into interface events.
Search and jump accept Enter then return focus to navigation; Esc dismisses
entry/help before navigating back. Browser modifier shortcuts remain native.
The browser maps `q` to closing its document; the TUI exits. The web layout fills
the viewport with independently scrolling columns and inspector, shows the
deepest complete columns that fit like the TUI, and keeps shortcuts/status
pinned at the bottom. Installation and guides are compact header links.

## Native entry points

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
bounded. The shared explorer model supports JSON-compatible values, not all
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

`web/` builds the static browser explorer with Vite and WebAssembly. Python
build tooling assembles it with guides generated from repository Markdown,
installation templates in `website/`, and both canonical installers
(`install.sh` and `install.ps1`). Website deployment
follows a successful release or a maintainer's manual dispatch, publishing to
`twig-web` for the existing `twig.wtf` GitHub Pages domain. Old Astro source on
`gh-pages` is no longer the maintained source.
