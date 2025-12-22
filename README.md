<p align="center">
  <img src="asset/logo.png" alt="Twig Logo" width="200"/>
</p>

# Twig 🌿

[![PyPI version](https://img.shields.io/pypi/v/twg.svg?style=flat-square&color=2ecc71)](https://pypi.org/project/twg/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/twg.svg?style=flat-square)](https://pypi.org/project/twg/)
[![Downloads](https://static.pepy.tech/badge/twg)](https://pepy.tech/project/twg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/workdone0)

> **Inspect. Navigate. Understand.**
>
> The modern, terminal-based data explorer for **JSON** & **YAML**.
> Designed for developers who value speed and privacy.

![Twig Demo](asset/demo.gif)

## What is Twig?

**Twig** is a high-performance **Terminal JSON Viewer** and **YAML Viewer**, designed to replace `cat`, `less`, and `jq` for interactive data exploration.

It turns raw data into a navigable tree, allowing you to filter, search, and explore deep hierarchies without leaving your terminal.

### Key Features

*   **📂 Multi-Format**: Native support for **JSON** and **YAML**.
*   **👀 Read Only**: Safely explore production logs or configs without risk of accidental edits.
*   **🔍 Deep Search**: Instantly find any key or value with fuzzy matching (e.g., matching `Pull` in `imagePullPolicy`).
*   **🎨 Themes**: Includes beautiful themes like **Catppuccin Mocha** (Default) and **Solarized Dark**.
*   **⚡️ Performance Optimized**: Built on a streaming SQLite backend to handle files efficiently.

### Comparison

| Feature | **Twig** | `jq` | `less` | `cat` |
| :--- | :---: | :---: | :---: | :---: |
| **Interactive Navigation** | ✅ | ❌ | ❌ | ❌ |
| **Tree View** | ✅ | ❌ | ❌ | ❌ |
| **Fuzzy Search** | ✅ | ❌ | ✅ | ❌ |
| **JSON/YAML Support** | ✅ | ✅ | ⚠️ | ❌ |
| **Learning Curve** | **Low** | High | Low | Low |

## Common Use Cases

*   **Kubernetes Manifests**: Navigate deep hierarchies in K8s YAML files without scrolling for miles.
*   **Cloud Logs**: Quickly inspect massive JSON log dumps from AWS CloudWatch or GCP.
*   **API Responses**: Visualize complex nested JSON responses from REST or GraphQL APIs.
*   **Configuration Management**: Audit large Terraform states or Ansible playbooks safely.

## Performance Benchmarks

Twig is highly optimized for files **under 100MB**, with a "sweet spot" for files under **50MB**.

| File Size | Load Time (Cold Start) | Experience |
| :--- | :--- | :--- |
| **< 10MB** | < 1s | ⚡️ Instant |
| **50MB** | ~8s | 🚀 Fast |
| **90MB** | ~17s | ✅ Usable |
| **> 100MB** | 20s+ | 🐢 Slower |

> **Note:** For files larger than 100MB, initial loading will take longer as Twig indexes the entire structure for instant search and navigation.

## Installation

### Using pipx (Recommended)
Install in an isolated environment to keep your system clean:
```bash
pipx install twg
```

### Using pip
```bash
pip install twg
```

## Usage

**Explore a file:**
```bash
twg data.json
# or
twg config.yaml
```

**Fix broken JSON:**
Automatically repair common errors (trailing commas, unquoted keys) or sanitize `NaN`/`Infinity` values:
```bash
twg --fix bad.json -o clean.json
```

**Pretty Print:**
```bash
twg -p large.json
```

## Cheat Sheet

| Key | Action |
| :--- | :--- |
| **Navigation** | |
| `Arrow Keys` | **Traverse** Tree |
| `/` | **Search** (Global) |
| `n` / `N` | **Next / Prev** Match |
| `:` | **Jump** to path |
| **Actions** | |
| `c` | **Copy Path** |
| `y` | **Copy Source** (JSON/YAML) |
| `t` | **Toggle Theme** |
| `?` | **Help** |
| `q` | **Quit** |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details and setup instructions.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=workdone0/twig&type=timeline&legend=bottom-right)](https://www.star-history.com/#workdone0/twig&type=timeline&legend=bottom-right)
