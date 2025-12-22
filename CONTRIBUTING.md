# Contributing to Twig

Thank you for your interest in contributing to Twig! We are building the best terminal-based data explorer, and we'd love your help.

## Technical Architecture

Twig uses a unique architecture to handle large files efficiently while providing a rich TUI experience. It is built on **Textual** (UI) and **SQLite** (Data Engine).

### Core Concepts

1.  **Streaming Ingestion**: Twig does *not* load the entire file into RAM. It uses **streaming parsers** to populate a local SQLite cache.
    *   **JSON**: Uses `ijson` (C-backend) for streaming parsing.
    *   **YAML**: Uses `PyYAML` (C-loader preferred) to stream events.
2.  **Defer Indexing Strategy**: To achieve <20s load times for 100MB+ files, we use a "Defer Indexing" pattern (`src/twg/adapters/sqlite_loader.py`).
    *   We drop all Indices and Triggers before ingestion.
    *   We bulk insert raw data into the Main Table.
    *   We rebuild Indices and populates the FTS5 Search Table in a single batch operation at the end.
3.  **Virtual Windowing**: The UI (`src/twg/ui/widgets/navigator.py`) only renders the visible slice of the tree. This allows it to scroll smoothly over datasets with millions of nodes.

### Project Structure

```text
src/twg
├── core/           # Data models, DB schema, and FTS logic
├── adapters/       # Ingestion logic (BaseLoader, SQLiteLoader, YamlLoader)
├── ui/
    ├── app.py      # Application entry point & layout
    ├── widgets/    # Reusable UI components (Navigator, Inspector, LoadingScreen)
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

### 3. Verify Changes
Since this is a TUI, manual verification is critical.
*   **Load Test**: Ensure a 50MB file loads in <10s.
*   **Search Test**: Verify fuzzy search finds deep keys.
*   **UI Check**: Ensure valid rendering (no overlapping widgets).

## Submission Guidelines

*   **Logic Separation**: Keep business logic (search, parsing) in `core/` or `adapters/`, and visual logic in `ui/`.
*   **Type Safety**: We enforce strict type hints.
*   **Linting**: Run `ruff check .` before submitting.
*   **PR Title**: Use conventional commits (e.g., `feat: add graph view`, `fix: search crash`).

