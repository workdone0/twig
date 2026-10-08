# Migrating to Twig 3.1

The current application is Rust. Python 2.1.4 source remains on
[legacy-python](https://github.com/workdone0/twig/tree/legacy-python).
Use the [installation guide](../README.md#installation) for the latest release.

## From Python 2.x

The Python package `twg` installed both `twig` and `twg`; Rust installs `twig`
only. Uninstall the old package using its original manager (`uv tool uninstall
twg`, `pipx uninstall twg`, or your environment's pip). Check which executable
your shell resolves if both installations coexist.

| Workflow | Twig 3.1 |
| --- | --- |
| `twig <file>` | Same interactive entry point |
| `twg <file>` | Use `twig`; the alias is not installed |
| `-v` / `--version` | Supported, alongside `-V` |
| `--fix --print` | Repair, then format the repaired value |
| `--indent N` | JSON stdout and output-file formatting; `-i` is now an alias |
| `-o` / `--output` | Atomic output-file replacement |
| `--rebuild-db` | Force parsing; source changes now invalidate caches automatically |
| `--check` | Noninteractive validation and timing |
| `--no-cache` | Temporary SQLite storage without persistent cache publication |
| `--clear-cache` | Remove cached database files, including older versions |

Neither the inspected Python 2.1.4 parser nor Rust defines `--file`; use the
positional argument. Stdin JSON is supported with `-` in print/fix/check modes.
The TUI requires a terminal. Mouse and keyboard navigation are available; consult
the [current controls](../README.md#keyboard-controls).

## From Rust 3.0

- Cached data is content-validated. Old cache files are ignored; remove them
  with `--clear-cache` after closing other Twig processes.
- JSON is parsed as a single document with bounded node batches. Trailing roots
  and duplicate object keys are rejected instead of being partly hidden.
- Arrays preserve source order, and unsigned integers retain numeric type.
- Punctuation/space/empty keys now use quoted bracket paths. For example,
  `.regions.us-east-1` becomes `.regions["us-east-1"]`.
- Search `%` and `_` now mean literal characters. Match traversal follows source
  order rather than lexicographic path order. Unicode matching outside ASCII is
  exact; ASCII search remains case-insensitive.
- JSON indentation applies consistently to stdout, `-o`, and repaired output.
  Print/fix may be combined; check conflicts with them instead of overriding.
- YAML extension detection is case-insensitive in every mode. Print supports
  multiple YAML documents.
- Formatted CLI output is plain text; the old minimal ANSI colorizer is removed.
- `y` copies the full selected value up to 10,000 nodes, or shows an explicit
  error. The inspector remains a limited preview. Copy no longer truncates
  descendants silently at five levels.
- Help opens with `?`; `h` moves to the parent. Enter opens a container, and
  `j/k/l`, `g/G`, Home/End, and mouse navigation are supported.

## Configuration

Themes are now `dark` and `light`, shared with the browser explorer. Both old
`catppuccin-mocha` and `solarized-dark` preferences resolve to `dark`; toggling
with `t` saves a canonical name. Unknown JSON
keys are retained; unknown theme names fall back to Catppuccin.

Linux and Windows keep their normal platform config locations. Rust uses
`~/Library/Application Support/twig/config.json` on macOS. Python used
`$XDG_CONFIG_HOME/twig/config.json` or `~/.config/twig/config.json`. Version 3.1
imports the old config when the new file is absent; an existing new config takes
precedence. A save failure is reported rather than silently treated as success.

## Data and resource contracts

Both implementations use SQLite, not a Python flat-file cache. New caches are
private but unencrypted, with retention and cleanup controls. `--no-cache` still
uses temporary disk storage. See [local data](../README.md#configuration-and-local-data).

TUI YAML is represented as an array of JSON-compatible documents. Original
comments, anchors, whitespace, and quoting are not preserved. Complex mapping
keys, non-finite numbers, and tagged values are outside the TUI data model.
JSON has bounded node batches; YAML parser memory can still scale with input.
Clipboard/export and large scalar values can also allocate significant memory.

Live watch mode and custom theme definitions are not implemented. These are
explicit non-goals for this release, not promised compatibility features.
