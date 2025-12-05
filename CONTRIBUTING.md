# Contributing to Twig

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to Twig. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Table of Contents

- [Project Overview](#project-overview)
- [Architecture Guide](#architecture-guide)
- [Setting Up the Development Environment](#setting-up-the-development-environment)
- [How to Contribute](#how-to-contribute)
- [Pull Request Policy](#pull-request-policy)
- [Code Standards](#code-standards)

## Project Overview

**Twig** is a terminal-based environment (TUI) for exploring, editing, and understanding structured data files. It aims to bridge the gap between simple text viewers and complex web-based visualizers.

### Core Pillars

1.  **Universal Support**: Open *any* structured file (JSON, YAML, TOML, XML, etc.).
2.  **Miller Column Navigation**: Traverse deep hierarchies effortlessly.
3.  **Safe Editing**: Modify values and keys with confidence.
4.  **Powerful Search**: Find exactly what you need.

## Architecture Guide

This section (formerly the Technical Design Document) outlines the core architecture to help you navigate the codebase.

### 1. Core Capabilities

*   **Universal File Support:** Native ingestion of any structured format into a unified `TwigModel`.
*   **Miller Column Navigation:** The primary interface for deep traversal.
*   **Smart Content Awareness:** Inferred semantic types (e.g., ISO 8601 -> Relative time, Hex -> Color preview).

### 2. User Interface Design

The UI is built on **Textual (Python)**.

#### Layout

*   **Header:** Breadcrumbs with `jq`-compatible path display.
*   **Main Body:** Split view with Navigation Columns (Left) and Inspector Pane (Right).
*   **Footer:** Contextual key hints and Command Bar.

## Setting Up the Development Environment

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
    Using `uv` (recommended):
    ```bash
    uv sync
    ```
    Or using `pip`:
    ```bash
    pip install -e ".[dev]"
    ```

3.  **Run the application**
    ```bash
    uv run twig samples/demo.json
    # or
    uv run twg samples/demo.json
    ```

## How to Contribute

### Reporting Bugs

- Ensure the bug was not already reported.
- Open a new issue with a clear title and description.
- Include a minimal reproduction example.

### Suggesting Enhancements

- Open a new issue describing the enhancement.
- Explain why this enhancement would be useful to most users.

## Pull Request Policy

1.  **No Logic in Views:** UI components should only display data. Business logic belongs in the Controller or Model.
2.  **Adapter Isolation:** Changes to the YAML adapter must not break the JSON adapter.
3.  **Documentation:** Any new keybinding must be added to the in-app help modal.

## Code Standards

*   **Language:** Python 3.10+ (Utilizing strict type hinting).
*   **Linter:** `ruff` for linting and formatting.

## Roadmap

- [x] **Phase 1**: Core Navigation & Visualization (Miller Columns, Inspector, Themes)
- [ ] **Phase 2**: Search & Filtering (Fuzzy search, JMESPath/jq queries)
- [ ] **Phase 3**: Editing (Modify values, rename keys, save changes)
- [ ] **Phase 4**: Multiple File Support & Diffing