# Twig 🌿

**Inspect. Navigate. Understand.** A local terminal explorer for JSON, YAML, and
JSON-based HAR files, written in Rust.

Explore nested data with Miller columns, search keys and values, jump to paths,
and inspect values without a browser. The TUI never edits your input. Separate
CLI modes format data and repair JSON.

[Website](https://twig.wtf) · [Guide](https://twig.wtf/guide/) ·
[Releases](https://github.com/workdone0/twig/releases) ·
[Contributing](CONTRIBUTING.md)

## Installation

Choose the instructions for the terminal you are using. Native Windows uses
PowerShell; WSL uses the Linux instructions. No Rust or Python installation is
needed for a prebuilt release.

### Windows

Use **Windows PowerShell 5.1 or PowerShell 7** on Windows x64. Open a regular
PowerShell tab in Windows Terminal; administrator access is not needed.
The installer needs `tar.exe` (included in Windows 10 1803+ and Windows 11).
Windows ARM64 and 32-bit binaries are not provided.

Download the installer, inspect it if desired, then run it:

```powershell
Invoke-WebRequest https://twig.wtf/install.ps1 -OutFile install.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

`-ExecutionPolicy Bypass` applies only to that installer process; it does not
change your saved execution policy. Organization policies can still block
scripts. If that happens, use the manual download instructions below or ask
your administrator; do not change an organization policy to install Twig.

The script downloads the latest release, verifies SHA-256 and the executable's
version, installs to `%LOCALAPPDATA%\Programs\Twig\bin`, and adds that folder to
your **user PATH**. It does not edit the system PATH. Close and reopen your
terminal, then check the installation:

```powershell
twig --version
twig --help
twig 'C:\path\to\data.json'
```

Choose a version or directory, or skip PATH changes:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Version v3.1.0 -InstallDir "$env:LOCALAPPDATA\Programs\Twig\bin"
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -NoPath
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Help
```

Close any running Twig process before upgrading, then rerun the installer.
Downloads and checksum verification happen before the installed executable is
replaced. Failed downloads or mismatched versions leave the old executable
intact. Temporary downloads are cleaned up on success and failure.

#### Windows manual download

If scripts are restricted, download and verify the release in PowerShell:

```powershell
$version = 'v3.1.0'
$asset = 'twig-x86_64-pc-windows-msvc.tar.gz'
$base = "https://github.com/workdone0/twig/releases/download/$version"
Invoke-WebRequest "$base/$asset" -OutFile $asset
Invoke-WebRequest "$base/$asset.sha256" -OutFile "$asset.sha256"
$expected = ((Get-Content "$asset.sha256" -Raw).Trim() -split '\s+')[0]
$actual = (Get-FileHash $asset -Algorithm SHA256).Hash
if ($expected -notmatch '^[a-fA-F0-9]{64}$' -or $actual -ine $expected) {
    throw 'Checksum mismatch. Do not extract this archive.'
}
$dest = "$env:LOCALAPPDATA\Programs\Twig\bin"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
tar -xzf $asset -C $dest twig.exe LICENSE
if ($LASTEXITCODE -ne 0) { throw 'Extraction failed.' }
& "$dest\twig.exe" --version
```

To run `twig` by name, open **Edit environment variables for your account** from
the Start menu. Under **User variables**, select **Path → Edit → New**, add
`%LOCALAPPDATA%\Programs\Twig\bin`, and save. Keep the existing entries. Close
and reopen your terminal. The manual method overwrites the extracted files;
close Twig and verify the checksum before doing so.

#### Windows troubleshooting

- **“twig is not recognized”**: reopen Windows Terminal completely. Try
  `& "$env:LOCALAPPDATA\Programs\Twig\bin\twig.exe" --version` to check the
  installation independently of PATH. Use your chosen directory if customized.
- **An older version runs**: `Get-Command twig -All` lists copies on PATH. Remove
  the obsolete executable or adjust your user PATH order, then open a new terminal.
- **Access denied while upgrading**: close Twig in every terminal and retry.
  Choose a directory owned by your account; do not install into Program Files
  with the per-user installer.
- **Missing `tar.exe`**: run `Get-Command tar.exe`. Use a Windows installation
  with the built-in tar tool available, or extract the verified archive with
  an archive utility and follow the manual PATH steps.
- **Using Git Bash or WSL**: run the PowerShell installer for native Windows.
  WSL is a separate Linux environment and uses `install.sh` inside WSL.

To uninstall, close Twig, delete its installation directory (the default
contains `twig.exe` and LICENSE), and remove that directory from your user PATH.
If you used a shared custom directory, remove only Twig's installed files.
`twig --clear-cache` removes cached data before uninstalling; config is separate.

### Linux and macOS

```bash
curl -fsSL https://twig.wtf/install.sh | bash
```

The Bash installer downloads the latest release, verifies SHA-256 and the
executable's version, and installs to `~/.local/bin`. It requires Bash, curl,
tar and its gzip support, `install`, and either `sha256sum` or macOS's `shasum`.
Linux uses GNU libc; Alpine/musl binaries are not provided. macOS builds target
11.0 or newer on Intel and Apple Silicon.

To inspect the script first or choose a version/directory:

```bash
curl -fsSL https://twig.wtf/install.sh -o install.sh
bash install.sh --help
bash install.sh --version v3.1.0 --to "$HOME/.local/bin" --yes
```

If `twig` is not found, add `export PATH="$HOME/.local/bin:$PATH"` to your shell
profile (`~/.zshrc` for zsh or `~/.bashrc` for Bash), then open a new terminal.
Run `twig --version` and `twig --help`. For a custom directory, use that path.
`command -v twig` shows which executable your shell resolves.

Use `--method build` to compile the selected release with Cargo; it honors the
same destination. Interactive invocations ask before installing unless `--yes`
is supplied. Piped invocations run noninteractively. Rerun to upgrade through
an atomic executable replacement. To uninstall, remove the installed `twig`
file; configuration and caches are separate.

### Release downloads and verification

Download `twig-<target>.tar.gz` and its separate `.tar.gz.sha256` file from
[Releases](https://github.com/workdone0/twig/releases/latest).

| OS | Targets |
| --- | --- |
| Linux (GNU libc) | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` |
| macOS | `x86_64-apple-darwin`, `aarch64-apple-darwin` |
| Windows x64 | `x86_64-pc-windows-msvc` |

Verify with `sha256sum -c <checksum-file>` on Linux or
`shasum -a 256 -c <checksum-file>` on macOS, then use `tar -xzf <archive>`.
Windows commands are shown above. Archives contain the executable and LICENSE.

These are native executables, not universal static binaries. Release `.build.txt`
assets record the commit, compiler, size, and available linkage diagnostics.
Clipboard support depends on the host desktop; SSH sessions may not provide it.
Binaries and the PowerShell script are not OS code-signed/notarized. Archives
have SHA-256 checksums and GitHub build provenance attestations.

### Build from source

Requires Rust 1.88+, Cargo, and a C compiler/linker for bundled SQLite. On Windows,
use the MSVC Rust toolchain with Visual Studio Build Tools' C++ workload and
Windows SDK; on macOS, install Xcode Command Line Tools; on Linux, use your
distribution's C build toolchain.

```bash
cargo install --locked --git https://github.com/workdone0/twig --tag v3.1.0 twig
```

Cargo installs into its own bin directory, usually `~/.cargo/bin` on Unix or
`%USERPROFILE%\.cargo\bin` on Windows. Ensure that directory is on PATH.
Use the Git source explicitly: the `twig` package on crates.io is unrelated.
Python is not needed to run the application.

## Usage

```bash
twig samples/cloud_infrastructure.json
twig samples/k8s_manifest.yaml
twig samples/browser_navigation.har

# Format JSON to stdout or an output file
twig --print data.json --indent 4
twig --print data.json --indent 4 -o formatted.json

# Repair JSON, then format the repaired value
twig --fix --print broken.json -o repaired.json

# Validate/load without a terminal; force parsing for a cold benchmark
twig --check --rebuild-db data.json

# Keep no persistent cache
twig --no-cache secrets.json

# JSON stdin is available for non-interactive modes
cat response.json | twig --print -
```

Output is plain JSON/YAML, suitable for piping. `-o` writes atomically after
successful parsing and can replace an existing file, including the input.
Prefer a separate output when reviewing repairs. Never redirect stdout to the
input path: the shell truncates that file before Twig reads it.

### CLI reference

| Option | Behavior |
| --- | --- |
| `<FILE>` | Positional JSON, YAML, or HAR path; `-` means JSON stdin in CLI modes |
| `-p`, `--print` | Format JSON or a YAML document stream |
| `--fix` | Repair JSON and format it; may combine with `--print` |
| `-o`, `--output <PATH>` | Write formatted output atomically; requires print/fix |
| `-i`, `--indent <N>` | JSON indentation, 0–16 spaces, default 2; YAML uses its formatter |
| `--check` | Report validation/load time, size, nodes, throughput; conflicts with print/fix |
| `--rebuild-db` | Parse again even if a matching completed cache exists |
| `--no-cache` | Use a private temporary SQLite database |
| `--clear-cache` | Remove stored cache files and exit; takes no input file |
| `-h`, `--help` | Show help |
| `-V`, `-v`, `--version` | Show version; `-v` preserves the Python alias |

Format detection is case-insensitive: `.yaml`/`.yml` use YAML, other extensions
use JSON. HAR is ordinary JSON. There is no `--file` flag or live watch mode.
The TUI requires stdin and stdout attached to a terminal.

### Keyboard controls

| Action | Controls |
| --- | --- |
| Move up / down | `↑` / `↓`, `k` / `j` |
| Open selected container | `→`, `l`, `Enter` |
| Return to parent | `←`, `h`, `Esc` |
| First / last sibling | `g` / `G`, `Home` / `End` |
| Search keys and values | `/`, query, `Enter` |
| Next / previous match | `n` / `N` |
| Jump to a path | `:`, path, `Enter` |
| Copy path / entire selected value | `c` / `y` |
| Cycle theme | `t` |
| Open help | `?` |
| Dismiss search/jump | `Esc` |
| Dismiss help | `Esc`, `?`, `h`, or `Enter` |
| Quit | `q` in normal/loading mode; `Ctrl+C` in any mode |

Mouse wheel moves selection. Click a row to select it; click the selected row to
open its container. Right-click returns to the parent. Clipboard ownership is
kept while Twig runs; persistence after exit depends on the desktop clipboard
manager. Failures are shown in the status message.

Paths look like `.users[0].name`; root arrays use `.[0]`. Keys containing
punctuation, spaces, or empty strings use JSON-quoted brackets, such as
`.regions["us-east-1"]` or `.["a.b"]`. This is a path lookup syntax, not a full jq
or JSONPath query language. YAML documents are wrapped in an array:
`.[0].kind` addresses the first document, and `.kind` is a shorthand fallback.

Search is a literal substring match, with ASCII case-insensitivity; `%` and `_`
are ordinary characters. Matches cycle in source traversal order, including
numeric array order. Non-ASCII characters match exactly.

### Copying and preview limits

The inspector shows a limited preview. Clipboard `y` exports the **complete
selected value**, preserving scalar types and order, up to 10,000 nodes; larger
selections give an explicit error directing you to `--print`. Serialization does
not preserve original whitespace, comments, anchors, or quoting.

## Configuration and local data

Themes: `catppuccin-mocha` (default) and `solarized-dark`. Press `t` to save.
Unknown config keys are preserved. Unknown theme names fall back to the default;
custom theme definitions are not supported.

```json
{"theme": "catppuccin-mocha"}
```

| Platform | Config directory (`config.json`) | Cache directory |
| --- | --- | --- |
| Linux | `$XDG_CONFIG_HOME/twig` or `~/.config/twig` | `$XDG_CACHE_HOME/twig` or `~/.cache/twig` |
| macOS | `~/Library/Application Support/twig` | `~/Library/Caches/twig` |
| Windows | `%APPDATA%\twig` | `%LOCALAPPDATA%\twig` |

On macOS, when the new config is missing, Twig imports the legacy Python config
from `$XDG_CONFIG_HOME/twig/config.json` or `~/.config/twig/config.json`. An
existing new config is never overwritten by migration.

The explorer makes no network requests or telemetry calls. Parsed values are
stored **unencrypted** in private SQLite cache files. Unix cache directories are
restricted to mode 0700 and published files to 0600; Windows uses profile ACLs.

Every load snapshots and hashes the input. Only completed caches matching the
source contents and schema version can be reused. Failed/cancelled builds are
never published. Concurrent loads use independent staging databases and publish
immutable generations. A source change automatically selects a new generation.

New-format caches are pruned on load after seven days, or oldest-first when the
existing cache exceeds 512 MiB. A newly built cache may exceed that budget until
a subsequent load. Close other Twig processes and run `twig --clear-cache` to
remove caches, including older versions. Configuration is preserved.

`--no-cache` uses temporary disk storage, not an in-memory database. Normal exit
cleans up temporary data; abrupt OS/process termination may leave temporary
files. Sensitive data is not encrypted in either mode.

## Performance and supported data

JSON is parsed into node events and inserted in batches of 1,024. It does not
materialize the whole document tree. Memory still depends on the largest
scalar, nesting/path lengths, SQLite working space, and requested exports.
YAML parsing can buffer document state; bounded parser memory is not guaranteed.

Navigation fetches pages of at most 256 siblings, with a horizontal viewport for
deep trees. Inspector previews have depth/node limits. Search is a SQLite
substring scan; broad searches on very large files can take time.

JSON input must contain a single document. Duplicate object keys and ambiguous
stored paths are rejected. Maximum supported nesting is 128 levels (the JSON
parser may reject at its own recursion boundary). YAML's TUI model supports
JSON-compatible scalar types, sequences, and string-keyed mappings; non-finite
numbers, tagged values, and complex mapping keys are not supported. `--print`
uses the YAML value serializer and supports a wider YAML value model.

Measure cold ingestion explicitly:

```bash
cargo build --release --locked
python3 scripts/benchmark.py --binary target/release/twig
```

The benchmark generates a deterministic file and reports wall time and peak RSS.
`--check` uses MiB units. Cached loads still read/hash the source to verify it;
they are not evidence of parsing throughput. No universal latency or fixed
binary-size claim is made.

## Development

- [Contributing](CONTRIBUTING.md): setup, tests, review, release workflow.
- [Architecture](docs/ARCHITECTURE.md): data flow and implementation boundaries.
- [Migration](docs/MIGRATION.md): Python 2.x and Rust 3.0 upgrade differences.
- [Release evaluation](docs/EVALUATION.md): audit findings and their resolution.
- [Changelog](CHANGELOG.md) and [release notes](RELEASE_NOTES.md).

Website source lives in `website/`. Its guides are generated from these Markdown
files, and both deployed installers are copied from this repository's `install.sh` and
`install.ps1`.
The Python source remains on
[legacy-python](https://github.com/workdone0/twig/tree/legacy-python).

## License

MIT — see [LICENSE](LICENSE).
