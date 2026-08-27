# Contributing to Twig

Thank you for your interest in contributing to Twig! We are building the best terminal-based data explorer, and we'd love your help.

## Technical Architecture

Twig uses a unique architecture to handle large files efficiently while providing a rich TUI experience. It is built on **ratatui** (UI) and **SQLite** (Data Engine), all in Rust.

### Core Concepts

1.  **Streaming Ingestion**: Twig does *not* load the entire file into RAM. It uses **streaming parsers** to populate a local SQLite cache.
    *   **JSON**: Uses `serde_json`'s `Deserializer::from_reader` (streaming `Value` iterator) for memory-efficient parsing.
    *   **YAML**: Uses `serde_yml`'s streaming `Deserializer` to feed the same stack-based emitter.
2.  **Defer Indexing Strategy**: To achieve <20s load times for 100MB+ files, we use a "Defer Indexing" pattern (`src/adapters/json_loader.rs`, `src/adapters/yaml_loader.rs`).
    *   We drop all Indices and Triggers before ingestion.
    *   We bulk insert raw data into the Main Table.
    *   We rebuild Indices and populate the FTS5 Search Table in a single batch operation at the end.
3.  **Virtual Windowing**: The UI (`src/tui/widgets/navigator.rs`) only renders the visible slice of the tree. This allows it to scroll smoothly over datasets with millions of nodes.

### Project Structure

```text
src/
├── main.rs           # CLI entrypoint (clap)
├── core/             # Data models, DB schema, FTS, config, repair
├── adapters/         # Ingestion logic (Loader trait, JsonLoader, YamlLoader)
├── tui/              # ratatui app, theme, widgets (Navigator, Inspector, modals)
└── cli/              # --fix and --print non-TUI modes
schema.sql            # SQLite schema with FTS5 triggers
```

## Development Setup

We recommend using the standard Rust toolchain.

### 1. Clone & Setup
```bash
git clone https://github.com/workdone0/twig.git
cd twig

# Build & run
cargo run -- samples/cloud_infrastructure.json
```

### 2. Verify Changes
Since this is a TUI, manual verification is critical.
*   **Load Test**: Ensure a 50MB file loads in <10s.
*   **Search Test**: Verify fuzzy search finds deep keys.
*   **UI Check**: Ensure valid rendering (no overlapping widgets).

## Submission Guidelines

*   **Logic Separation**: Keep business logic (search, parsing) in `core/` or `adapters/`, and visual logic in `tui/`.
*   **Type Safety**: We rely on the Rust compiler.
*   **Linting**: Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` before submitting.
*   **PR Title**: Use conventional commits (e.g., `feat: add graph view`, `fix: search crash`).