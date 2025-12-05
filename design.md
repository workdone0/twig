This is a comprehensive **Technical Design Document (TDD)** for `twig`.

-----

# Project Design Document: Twig

| **Project** | Twig |
| :--- | :--- |
| **Version** | 0.2.0 (Beta) |
| **Status** | **Active Development** |
| **Scope** | Core Architecture, UI Framework |
| **Tagline** | *Inspect. Navigate. Understand.* |

-----

## 1\. Executive Summary

**Twig** is a terminal-based environment (TUI) for exploring, editing, and understanding structured data files.

While CLI tools like `cat`, `less`, and `jq` exist, they treat data as either flat text or query output. **Twig** treats data as a **structure**. It creates an immersive environment where a date string is rendered as a calendar, a color code is rendered as a visual swatch, and a JWT token can be decoded instantly—all without leaving the terminal.

### 1.1 The "Why"

As modern infrastructure grows complex, developers deal with massive JSON/YAML/TOML configurations. We lack a tool that bridges the gap between:

1.  **Read-only Viewers** (fast but passive).
2.  **Text Editors** (powerful but risk syntax errors and lack context).
3.  **Web Visualizers** (rich context but require leaving the terminal/uploading sensitive data).

Twig is the bridge. It brings rich, context-aware visualization to your local terminal.

-----

## 2\. Product Requirements

### 2.1 Core Capabilities

  * **Universal File Support:** Native ingestion of any structured format (JSON, YAML, TOML, XML, etc.) into a unified `TwigModel`.
  * **Miller Column Navigation:** The primary interface must be a column-based layout for deep traversal.
  * **Safe Editing:** Users can edit values and rename keys.
  * **Search & Discovery:** Robust fuzzy search and query capabilities to locate nodes instantly.
  * **High Performance:** Zero-latency navigation for large files.

### 2.2 The "Smart" Requirement (Content Awareness)

The system must not treat values merely as strings. It must attempt to infer the *semantic type* of the value:

  * **Temporal:** ISO 8601 strings $\rightarrow$ Relative time display (e.g., "2 hours ago").
  * **Visual:** Hex/RGB codes $\rightarrow$ Color block preview.
  * **Encoded:** Base64/JWT $\rightarrow$ Auto-decoded view.
  * **Media:** Image URLs $\rightarrow$ Resolution/Size metadata (or render via terminal graphics protocols like kitty/sixel where available).

-----

## 3\. User Interface Design

The UI is built on **Textual (Python)**.

### 3.1 Layout

The screen is divided into three areas:

1.  **Header:** Breadcrumbs (e.g., `config > database > connections > 0`) with `jq`-compatible path display.
2.  **Main Body (Split 75/25):**
      * **Left (The Root):** The navigation columns (Miller Columns).
      * **Right (The Leaf):** The Inspector/Preview pane.
 
   * **Miller Columns:**
      * Data is displayed in a series of columns.
      * Selecting an item in Column N opens its children in Column N+1.
      * This allows for deep traversal without horizontal scrolling issues common in trees.
3.  **Footer:** Contextual key hints and Command Bar.

### 3.2 Keybindings

   * **Navigation:** `Arrow Keys` (Up, Down, Left, Right).
  * **Action:**
      * `c`: Copy current path (jq format) to clipboard.
      * `t`: Cycle through themes.
      * `q`: Quit the application.
  * **Global:**
      * `/`: Search (Fuzzy find keys) [Planned].
      * `:`: Command Mode (e.g., `:save`, `:quit`, `:theme dark`) [Planned].

-----

## 4\. Implementation Phases

### Phase 1: Visualization (MVP) - **COMPLETED**

  * **Goal:** High-fidelity visualization in the terminal.
  * **Features:**
      * Smart resolvers (Date, URL).
      * Split-view navigation (Miller Columns + Inspector).
      * Fast file loading.
      * Theme support (Catppuccin, Solarized, etc.).
      * `jq` path generation.
  * **Success Metric:** Users can explore complex files with rich context.

### Phase 2: Search & Filtering - **NEXT**

  * **Goal:** Find data quickly.
  * **Features:**
      * Fuzzy search for keys.
      * `jq` or JMESPath query support.

### Phase 3: Safe Mutations

  * **Goal:** Allow safe modifications.
  * **Features:**
      * Edit values (with type validation).
      * Rename keys.
      * Save back to disk.
      * **Constraint:** No deletion allowed initially.
  * **Focus:** Robustness and preserving file comments/structure.

-----

## 5\. Contribution Guidelines

### 5.1 Code Standards

  * **Language:** Python 3.10+ (Utilizing strict type hinting).
  * **Linter:** `ruff` for linting and formatting.
  * **Testing:** `pytest` is required for all logic in the Model and Adapters.

### 5.2 Pull Request Policy

1.  **No Logic in Views:** UI components should only display data. Business logic belongs in the Controller or Model.
2.  **Adapter Isolation:** Changes to the YAML adapter must not break the JSON adapter.
3.  **Documentation:** Any new keybinding must be added to the in-app help modal.

-----

## 6\. Open Questions (RFC)

  * *Performance:* Should we move the parser engine to Rust (via PyO3) for very large files (\>100MB)?
  * *Validation:* Should we support JSON Schema validation for editing?
  * *Layout:* Should the Right Pane be collapsible for a "Distraction Free" mode?

-----

*End of Design Document*