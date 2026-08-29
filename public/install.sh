#!/usr/bin/env bash
#
# Twig one-line installer.
#
# Usage:
#   curl -fsSL https://twig.wtf/install.sh | sh
#   curl -fsSL https://twig.wtf/install.sh | sh -s -- --version v3.0.0
#   curl -fsSL https://twig.wtf/install.sh | sh -s -- --to /usr/local/bin
#   curl -fsSL https://twig.wtf/install.sh | sh -s -- --method fetch
#
# Options:
#   -v, --version <TAG>   Release tag to install. Default: latest.
#   -t, --to <DIR>         Install directory. Default: ~/.local/bin
#                          (or /usr/local/bin when writable).
#   -m, --method <METHOD>  Install method: "fetch" (download from GitHub,
#                          default) or "build" (cargo install --locked).
#   -y, --yes              Skip the confirmation prompt.
#   -h, --help             Show this message.
#
# Environment overrides (mostly useful for testing):
#   TWIG_REPO   GitHub owner/repo slug. Default: workdone0/twig.
#   TWIG_BIN    Override the binary name (default: twig).
#
# The script is idempotent: re-running overwrites the existing install.

set -euo pipefail

# ---------------------------------------------------------------------------
# Pretty output
# ---------------------------------------------------------------------------

if [[ -t 1 ]] && command -v tput >/dev/null 2>&1 && [[ "$(tput colors 2>/dev/null || echo 0)" -ge 8 ]]; then
    C_RESET=$'\033[0m'
    C_BOLD=$'\033[1m'
    C_DIM=$'\033[2m'
    C_BLUE=$'\033[34m'
    C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'
    C_RED=$'\033[31m'
    C_CYAN=$'\033[36m'
else
    C_RESET="" C_BOLD="" C_DIM="" C_BLUE="" C_GREEN="" C_YELLOW="" C_RED="" C_CYAN=""
fi

info()    { printf "%s==>%s %s\n" "${C_BLUE}" "${C_RESET}" "$*"; }
success() { printf "%s ✓%s %s\n" "${C_GREEN}" "${C_RESET}" "$*"; }
warn()    { printf "%s !%s %s\n" "${C_YELLOW}" "${C_RESET}" "$*" >&2; }
err()     { printf "%s ✗%s %s\n" "${C_RED}" "${C_RESET}" "$*" >&2; }

banner() {
    printf "%s%s   __         __     %s\n" "${C_BOLD}" "${C_CYAN}" "${C_RESET}" >&2
    printf "%s%s   \ \       / /_    %s\n" "${C_BOLD}" "${C_CYAN}" "${C_RESET}" >&2
    printf "%s%s    \ \  _  / / _ \   %s  Twig installer\n" "${C_BOLD}" "${C_CYAN}" "${C_RESET}" >&2
    printf "%s%s    / / / / /  __/   %s\n" "${C_BOLD}" "${C_CYAN}" "${C_RESET}" >&2
    printf "%s%s   /_/ /_/ /_/ |_|     %s  https://github.com/workdone0/twig\n\n" "${C_BOLD}" "${C_CYAN}" "${C_RESET}" >&2
}

# ---------------------------------------------------------------------------
# Defaults & argument parsing
# ---------------------------------------------------------------------------

TWIG_REPO="${TWIG_REPO:-workdone0/twig}"
TWIG_BIN="${TWIG_BIN:-twig}"
VERSION="latest"
INSTALL_DIR=""
METHOD="fetch"
ASSUME_YES=0

usage() {
    sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--version) VERSION="$2"; shift 2 ;;
        -t|--to)      INSTALL_DIR="$2"; shift 2 ;;
        -m|--method)  METHOD="$2"; shift 2 ;;
        -y|--yes)     ASSUME_YES=1; shift ;;
        -h|--help)    usage; exit 0 ;;
        --) shift; break ;;
        -*) err "unknown option: $1"; usage >&2; exit 64 ;;
        *)  err "unexpected positional argument: $1"; exit 64 ;;
    esac
done

if [[ -z "$INSTALL_DIR" ]]; then
    if [[ -w "/usr/local/bin" ]]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
    fi
fi

# ---------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------

banner
info "Installing ${TWIG_BIN} (version: ${VERSION}, method: ${METHOD}, into: ${INSTALL_DIR})"

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        err "required command not found: $1"
        exit 69
    fi
}

# ---------------------------------------------------------------------------
# Method: cargo install (build from source)
# ---------------------------------------------------------------------------

install_via_cargo() {
    need_cmd cargo
    local cargo_args=(--git "https://github.com/${TWIG_REPO}.git")
    if [[ "$VERSION" != "latest" ]]; then
        cargo_args+=(--tag "$VERSION" --locked)
    else
        cargo_args+=(--locked)
    fi
    info "running: cargo install ${cargo_args[*]} ${TWIG_BIN}"
    cargo install "${cargo_args[@]}" "${TWIG_BIN}"
}

# ---------------------------------------------------------------------------
# Method: download prebuilt binary from GitHub Releases
# ---------------------------------------------------------------------------

detect_target() {
    local os arch
    case "$(uname -s)" in
        Linux)   os="unknown-linux-gnu" ;;
        Darwin)  os="apple-darwin" ;;
        FreeBSD) os="unknown-freebsd" ;;
        *)       err "unsupported OS: $(uname -s)"; return 1 ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             err "unsupported architecture: $(uname -m)"; return 1 ;;
    esac
    TARGET_TRIPLE="${arch}-${os}"
}

# Print the user a friendly summary of what we're about to do and let
# them bail out unless --yes was passed.
confirm() {
    if [[ "$ASSUME_YES" -eq 1 ]] || [[ ! -t 0 ]]; then
        return 0
    fi
    printf "Proceed? [y/N] "
    local ans
    read -r ans || true
    [[ "${ans:-}" =~ ^[Yy]$ ]]
}

# Resolve "latest" to the actual tag via the GitHub API.
resolve_latest_version() {
    local url="https://api.github.com/repos/${TWIG_REPO}/releases/latest"
    local tag
    tag="$(curl -fsSL "$url" \
        | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' \
        | head -n1 || true)"
    if [[ -z "$tag" ]]; then
        err "could not resolve latest release tag from $url"
        err "(is the repo accessible? rate-limited?)"
        return 1
    fi
    printf "%s" "$tag"
}

install_via_fetch() {
    need_cmd curl
    need_cmd uname
    need_cmd install
    need_cmd tar

    if ! detect_target; then
        return 1
    fi
    info "detected platform: ${TARGET_TRIPLE}"

    if [[ "$VERSION" == "latest" ]]; then
        VERSION="$(resolve_latest_version)"
        info "resolved latest version: ${VERSION}"
    fi

    # The release archive name is <twig>-<triple>.tar.gz and the
    # extracted contents are a single file named `twig` (or
    # `twig.exe` on Windows).
    local asset="${TWIG_BIN}-${TARGET_TRIPLE}"
    local inner_name="${TWIG_BIN}"
    if [[ "$TARGET_TRIPLE" == *windows* ]]; then
        inner_name="${TWIG_BIN}.exe"
    fi

    local base_url="https://github.com/${TWIG_REPO}/releases/download/${VERSION}"
    local tarball="${asset}.tar.gz"
    local checksum="${asset}.tar.gz.sha256"

    local tmp
    tmp="$(mktemp -d -t twig-install.XXXXXX)"
    trap 'rm -rf "${tmp:-}"' EXIT

    info "downloading ${base_url}/${tarball}"
    if ! curl -fL --retry 3 --connect-timeout 10 -o "${tmp}/${tarball}" "${base_url}/${tarball}"; then
        err "download failed — is ${VERSION} published for ${TARGET_TRIPLE}?"
        err "if you just tagged a release, the CI workflow may still be running."
        return 1
    fi

    # Verify checksum if the release ships one. The .sha256 file is
    # expected in sha256sum format ("<hash>  <filename>", two spaces),
    # which is what GitHub Actions produces with `sha256sum "$f"`.
    if curl -fsSL -o "${tmp}/${checksum}" "${base_url}/${checksum}" 2>/dev/null; then
        info "verifying sha256"
        local actual
        actual="$(sha256sum "${tmp}/${tarball}" 2>/dev/null \
            | awk '{print $1}')"
        local expected
        expected="$(awk '{print $1}' "${tmp}/${checksum}")"
        if [[ "${actual}" != "${expected}" ]]; then
            err "checksum mismatch: expected ${expected}, got ${actual}"
            return 1
        fi
        success "checksum ok"
    else
        warn "no .sha256 published alongside ${tarball}; skipping integrity check"
    fi

    info "extracting"
    tar -xzf "${tmp}/${tarball}" -C "${tmp}"

    if [[ ! -x "${tmp}/${inner_name}" ]]; then
        err "extracted archive did not contain an executable named '${inner_name}'"
        err "  (extracted: $(find "${tmp}" -mindepth 1 -maxdepth 1 | tr '\n' ' '))"
        return 1
    fi

    mkdir -p "${INSTALL_DIR}"
    info "installing to ${INSTALL_DIR}/${TWIG_BIN}"
    install -m 0755 "${tmp}/${inner_name}" "${INSTALL_DIR}/${TWIG_BIN}"

    if ! command -v "${TWIG_BIN}" >/dev/null 2>&1 \
        || [[ "$(command -v "${TWIG_BIN}")" != "${INSTALL_DIR}/${TWIG_BIN}"* ]]; then
        warn "${INSTALL_DIR} is not on your PATH"
        warn "add it to your shell rc, e.g.:"
        warn "  ${C_BOLD}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${C_RESET}"
    fi
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------

case "$METHOD" in
    fetch) install_via_fetch ;;
    build|cargo) install_via_cargo ;;
    *) err "unknown install method: ${METHOD}"; exit 64 ;;
esac

# ---------------------------------------------------------------------------
# Smoke test (best-effort)
# ---------------------------------------------------------------------------

if command -v "${TWIG_BIN}" >/dev/null 2>&1; then
    if version_output="$("${TWIG_BIN}" --version 2>/dev/null)"; then
        success "installed: ${version_output}"
    else
        success "installed: ${TWIG_BIN} -> $(command -v "${TWIG_BIN}")"
    fi
    printf "%sTry:%s %s%s --help%s\n" "${C_DIM}" "${C_RESET}" "${C_BOLD}" "${TWIG_BIN}" "${C_RESET}"
else
    success "installed ${TWIG_BIN} to ${INSTALL_DIR}/${TWIG_BIN}"
    warn "${TWIG_BIN} is not yet on PATH; open a new shell or run ${INSTALL_DIR}/${TWIG_BIN} directly"
fi

printf "\n%sDocs:%s  https://github.com/%s\n" "${C_DIM}" "${C_RESET}" "${TWIG_REPO}"
printf "%sSponsor:%s https://buymeacoffee.com/workdone0\n" "${C_DIM}" "${C_RESET}"