# Publishing Twig

This guide describes how to build and publish the Twig package to PyPI using `uv`.

## Prerequisites

Ensure you have `uv` installed:

```bash
pip install uv
# or
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Building the Package

To build the package (source distribution and wheel), run:

```bash
uv build
```

This will create the distribution files in the `dist/` directory. `uv` will automatically use the `hatchling` build backend defined in `pyproject.toml`.

## Publishing to PyPI

To publish the package to PyPI, use `uv publish`:

```bash
uv publish
```

You will be prompted for your PyPI API token.

## Installation for Users

Once published to PyPI, users can install Twig using `pipx` (recommended for CLI tools) or `pip`.

### Using pipx (Recommended)

`pipx` installs the application in an isolated environment and makes the `twg` and `twig` commands available globally.

```bash
pipx install twg
```

### Using pip

```bash
pip install twg
```

## Local Development

To install the package in editable mode for development:

```bash
uv pip install -e .
```

This allows you to run `twig` and `twg` commands directly from your terminal while reflecting changes in the source code immediately.

## Cross-Platform Compatibility

Twig is built with `textual`, which runs on macOS, Linux, and Windows. The entry points defined in `pyproject.toml` will automatically create the appropriate executable scripts (`twig.exe` on Windows, `twig` script on Unix) when installed via pip/uv/pipx.
