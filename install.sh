#!/usr/bin/env bash
# Twig installer. Requires Bash 3.2+; curl .../install.sh | bash is supported.
set -euo pipefail
TWIG_REPO="${TWIG_REPO:-workdone0/twig}"
TWIG_BIN="twig"
VERSION=latest
METHOD=fetch
INSTALL_DIR="${HOME}/.local/bin"
ASSUME_YES=0
TWIG_INSTALL_TMP=""
cleanup() { if [[ -n "$TWIG_INSTALL_TMP" ]]; then rm -rf "$TWIG_INSTALL_TMP"; fi; }
trap cleanup EXIT
usage() {
    cat <<'HELP'
Twig installer
Usage: bash install.sh [options]
  -v, --version TAG   Release tag (default: latest published release)
  -t, --to DIR        Destination (default: ~/.local/bin)
  -m, --method MODE   fetch prebuilt binary, or build from the release source
  -y, --yes          Skip confirmation when running interactively
  -h, --help         Show this help
Requires curl, tar, install, and sha256sum or shasum for fetch mode.
Build mode additionally requires Cargo and a C compiler/linker.
HELP
}
fail() { printf 'twig installer: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"; }
value() { [[ $# -ge 2 && -n "$2" && "$2" != -* ]] || fail "missing value for $1"; }
while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--version) value "$@"; VERSION="$2"; shift 2 ;;
        -t|--to) value "$@"; INSTALL_DIR="$2"; shift 2 ;;
        -m|--method) value "$@"; METHOD="$2"; shift 2 ;;
        -y|--yes) ASSUME_YES=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) fail "unknown argument: $1" ;;
    esac
done
[[ "$METHOD" == fetch || "$METHOD" == build ]] || fail "method must be fetch or build"
need curl
if [[ "$VERSION" == latest ]]; then
    VERSION="$(curl --proto '=https' --tlsv1.2 -fsSL "https://api.github.com/repos/${TWIG_REPO}/releases/latest" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
fi
[[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9.-]+)?$ ]] || fail "invalid release tag: $VERSION"
printf 'Install Twig %s via %s into %s\n' "$VERSION" "$METHOD" "$INSTALL_DIR"
if [[ "$ASSUME_YES" -eq 0 && -t 0 ]]; then
    read -r -p 'Proceed? [y/N] ' answer
    [[ "$answer" =~ ^[Yy]$ ]] || exit 0
fi
need install
TWIG_INSTALL_TMP="$(mktemp -d "${TMPDIR:-/tmp}/twig-install.XXXXXX")"
if [[ "$METHOD" == build ]]; then
    need cargo
    cargo install --locked --git "https://github.com/${TWIG_REPO}.git" --tag "$VERSION" --root "$TWIG_INSTALL_TMP/build" twig
    BINARY="$TWIG_INSTALL_TMP/build/bin/twig"
else
    need tar
    case "$(uname -s)" in
        Linux) os=unknown-linux-gnu ;;
        Darwin) os=apple-darwin ;;
        *) fail 'prebuilt installer supports Linux and macOS; use a Windows release archive on Windows' ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch=x86_64 ;;
        arm64|aarch64) arch=aarch64 ;;
        *) fail 'unsupported architecture' ;;
    esac
    if command -v sha256sum >/dev/null 2>&1; then
        hash_command=(sha256sum)
    elif command -v shasum >/dev/null 2>&1; then
        hash_command=(shasum -a 256)
    else
        fail 'checksum verification requires sha256sum or shasum'
    fi
    asset="twig-${arch}-${os}.tar.gz"
    url="https://github.com/${TWIG_REPO}/releases/download/${VERSION}/${asset}"
    curl --proto '=https' --tlsv1.2 -fL --retry 3 --connect-timeout 15 -o "$TWIG_INSTALL_TMP/$asset" "$url"
    curl --proto '=https' --tlsv1.2 -fL --retry 3 --connect-timeout 15 -o "$TWIG_INSTALL_TMP/checksum" "$url.sha256" || fail 'checksum download failed; refusing to install'
    expected="$(awk 'NR==1 {print $1}' "$TWIG_INSTALL_TMP/checksum")"
    [[ "$expected" =~ ^[a-fA-F0-9]{64}$ ]] || fail 'invalid checksum file'
    actual="$("${hash_command[@]}" "$TWIG_INSTALL_TMP/$asset" | awk '{print $1}')"
    [[ "$actual" == "$expected" ]] || fail 'checksum mismatch; refusing to install'
    tar -xzf "$TWIG_INSTALL_TMP/$asset" -C "$TWIG_INSTALL_TMP" twig
    BINARY="$TWIG_INSTALL_TMP/twig"
fi
[[ -x "$BINARY" ]] || fail 'archive/build did not contain an executable twig'
"$BINARY" --version
mkdir -p "$INSTALL_DIR"
# Stage next to the destination so replacement is atomic on the filesystem.
STAGED="$(mktemp "$INSTALL_DIR/.twig.XXXXXX")"
if install -m 0755 "$BINARY" "$STAGED" && mv -f "$STAGED" "$INSTALL_DIR/$TWIG_BIN"; then
    printf 'Installed %s\n' "$INSTALL_DIR/$TWIG_BIN"
else
    rm -f "$STAGED"
    fail 'could not install binary'
fi
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) printf 'Add this directory to your PATH: %s\n' "$INSTALL_DIR" ;;
esac
