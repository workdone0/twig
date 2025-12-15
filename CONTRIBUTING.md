# Contributing to Twig

Thank you for your interest in contributing to Twig. We welcome contributions from the community to help make this the best terminal-based JSON explorer.

## Project Overview

**Twig** is a terminal-based environment (TUI) for exploring and understanding structured data files. It aims to bridge the gap between simple text viewers (`less`, `cat`) and complex IDEs.

### Architecture Guide
Twig is built on the **Textual** framework.

*   **Core**: `TwigModel` (src/twg/core) handles the in-memory graph of the JSON data.
*   **UI**: `ColumnNavigator` (src/twg/ui/widgets) implements the Miller Column navigation.
*   **Async**: Heavy operations (like loading large files) are non-blocking.

### Project Structure

```text
src/twg
├── core/           # Data models (TwigModel, Node) and utilities (cleaner)
├── adapters/       # File ingestion logic
└── ui/
    ├── app.py      # Main Entry Point
    ├── widgets/    # Reusable Components (Navigator, Inspector)
    └── screens/    # Full-screen views (Help, Loading)
```

## Development Setup

### Prerequisites
- Python 3.10+
- `uv` (recommended) or `pip`

### Installation

1.  **Clone the repository**
    ```bash
    git clone https://github.com/workdone0/twig.git
    cd twig
    ```

2.  **Install dependencies**
    ```bash
    uv sync
    # or
    pip install -e ".[dev]"
    ```

3.  **Run the application**
    ```bash
    uv run twg samples/cloud_infrastructure.json
    ```

## Contribution Guidelines

### Reporting Bugs
- Open a new issue with a clear title.
- Include a minimal reproduction case (e.g., a small JSON snippet).

### Pull Requests
1.  **UI/Logic Separation**: Keep business logic out of UI components where possible.
2.  **Type Safety**: All new code must use Python type hints.
3.  **Linting**: We use `ruff` for code style. Ensure your code passes before submitting.

### Code Standards
*   **Language**: Python 3.10+
*   **Style**: strict `ruff` configuration.

