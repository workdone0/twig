# Contributing to Twig

Thank you for your interest in contributing to Twig! We are building the best terminal-based data explorer, and we'd love your help.

## Technical Architecture

Twig is built on top of the **Textual** framework (for TUI primitives) and uses a **Streaming SQLite** backend for performance.

### Core Concepts

1.  **Streaming Ingestion**: Twig does *not* load the entire JSON into RAM. Instead, it uses `ijson` to stream the file and populate a local SQLite cache (`src/twg/adapters/sqlite_loader.py`).
2.  **Miller Columns**: The visualization is a hierarchical column view (`src/twg/ui/widgets/navigator.py`). Logic for expanding/collapsing nodes is handled here.
3.  **FTS Search**: Global search uses SQLite's **FTS5** full-text search engine for instant results (`src/twg/core/model.py`), decoupling search complexity from Python.

### Project Structure

```text
src/twg
├── core/           # Data models, DB schema, and FTS logic
├── adapters/       # Ingestion (JSON -> SQLite)
└── ui/
    ├── app.py      # Application entry point & layout
    ├── widgets/    # Reusable UI components (Navigator, Inspector)
    └── screens/    # Modals (Help, Search, Jump)
```

## Development Setup

We recommend using `uv` for a fast, modern workflow, but standard `pip` works too.

### 1. Clone & Setup
```bash
git clone https://github.com/workdone0/twig.git
cd twig

# Install in editable mode with dev dependencies
uv sync
# or
pip install -e ".[dev]"
```

### 2. Run Locally
```bash
# Run against a sample file
uv run twg samples/large_data.json
```

### 3. Run Tests
(Coming Soon)

## Submission Guidelines

*   **Logic Separation**: Keep business logic (search, parsing) in `core/`, and visual logic in `ui/`.
*   **Type Safety**: We enforce strict type hints.
*   **Linting**: Run `ruff check .` before submitting.
*   **PR Title**: Use conventional commits (e.g., `feat: add graph view`, `fix: search crash`).

