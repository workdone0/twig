# Twig 3.2.0 — One core, terminal and browser

Twig now opens directly into a full-page web explorer at [twig.wtf](https://twig.wtf).
The browser and terminal share Rust parsers, node and path handling, search,
preview and export behavior. The native app keeps its SQLite cache and queries;
the browser uses an in-memory store inside a WebAssembly worker.

## Browser explorer

- Open or drop JSON, YAML and HAR files, paste input, or explore bundled examples.
- Navigate Miller columns, breadcrumbs and an inspector with mouse or keyboard.
- Search literal keys and values, jump to paths, and copy paths or complete values
  without losing large-integer precision.
- Files are processed in your browser and are not uploaded. Documents are session-only,
  with no accounts, analytics or persistent document cache.
- Inputs are capped at 20 MiB, with additional node/allocation limits. Cancel or
  replace a document by terminating its worker. Use the TUI for larger files.
- Responsive layouts keep the focused column accessible on narrow screens.

## A consistent terminal and web experience

- Exactly two themes, Dark and Light, with palettes and normal navigation bindings
  shared across both interfaces. Press `t` to switch; each interface remembers its
  own preference. Legacy theme names resolve to Dark.
- Arrows and `hjkl`, Enter, `g/G`, `/`, `:`, `n/N`, `c`, `y`, and `?` work across
  both interfaces. `q` closes the browser document or exits the terminal app.
- Existing CLI commands, native caching and the 10,000-node export limit remain.
- Windows PowerShell installation includes checksum/version verification, PATH
  setup and staged upgrades; platform installation guidance is expanded.

## Install or upgrade

See [installation instructions](https://twig.wtf/install/) or download a binary
archive below. Releases include checksums, provenance and per-platform build metadata.
Source builds require Rust 1.88+ and a C compiler. Existing configuration is supported;
the old named themes now select Dark. Web and terminal documents/preferences do not sync.

Browser export also has a 4 MiB limit. Parsing and export remain subject to resource
limits; formatting and repair are available through the CLI, not the browser UI.
