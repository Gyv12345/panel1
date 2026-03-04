#!/usr/bin/env bash
set -euo pipefail

REPO="${PANEL1_REPO:-Gyv12345/panel1}"
INSTALL_DIR="${PANEL1_INSTALL_DIR:-/usr/local/bin}"
REQUESTED_VERSION="${PANEL1_VERSION:-latest}"
ALLOW_SOURCE_FALLBACK="${PANEL1_ALLOW_SOURCE_FALLBACK:-1}"

DOWNLOADER=""
TAG=""
VERSION_NO_V=""
ARCH=""
TARGET=""
TMP_DIR=""

usage() {
  cat <<'EOF'
Panel1 installer (Linux)

Usage:
  curl -fsSL https://raw.githubusercontent.com/Gyv12345/panel1/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/Gyv12345/panel1/main/install.sh | bash -s -- --version v0.1.0

Options:
  --version <vX.Y.Z|X.Y.Z|latest>  Release version (default: latest)
  --install-dir <path>             Binary install dir (default: /usr/local/bin)
  --repo <owner/name>              GitHub repository (default: Gyv12345/panel1)
  --no-source-fallback             Disable cargo source fallback when no binary package is found
  -h, --help                       Show this help

Environment variables:
  PANEL1_VERSION
  PANEL1_INSTALL_DIR
  PANEL1_REPO
  PANEL1_ALLOW_SOURCE_FALLBACK=0|1
EOF
}

log() {
  printf '[panel1-install] %s\n' "$*" >&2
}

warn() {
  printf '[panel1-install] WARN: %s\n' "$*" >&2
}

die() {
  printf '[panel1-install] ERROR: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

cleanup() {
  if [[ -n "${TMP_DIR}" && -d "${TMP_DIR}" ]]; then
    rm -rf "${TMP_DIR}"
  fi
}
trap cleanup EXIT

pick_downloader() {
  if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
    return
  fi
  if command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
    return
  fi
  die "curl or wget is required."
}

download_file() {
  local url="$1"
  local output="$2"
  if [[ "${DOWNLOADER}" == "curl" ]]; then
    curl -fsSL "${url}" -o "${output}"
  else
    wget -qO "${output}" "${url}"
  fi
}

download_text() {
  local url="$1"
  if [[ "${DOWNLOADER}" == "curl" ]]; then
    curl -fsSL "${url}"
  else
    wget -qO- "${url}"
  fi
}

resolve_version() {
  if [[ "${REQUESTED_VERSION}" == "latest" ]]; then
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local json
    json="$(download_text "${api_url}")" || die "Failed to query latest release from ${REPO}."
    TAG="$(printf '%s' "${json}" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    [[ -n "${TAG}" ]] || die "Could not parse latest release tag from GitHub API."
  else
    if [[ "${REQUESTED_VERSION}" == v* ]]; then
      TAG="${REQUESTED_VERSION}"
    else
      TAG="v${REQUESTED_VERSION}"
    fi
  fi
  VERSION_NO_V="${TAG#v}"
}

resolve_arch() {
  case "$(uname -m)" in
    x86_64 | amd64)
      ARCH="x86_64"
      ;;
    aarch64 | arm64)
      ARCH="aarch64"
      ;;
    *)
      die "Unsupported CPU architecture: $(uname -m)"
      ;;
  esac
}

download_release_archive() {
  local target
  local asset
  local url
  local archive
  local targets=()

  if [[ "${ARCH}" == "x86_64" ]]; then
    targets=("x86_64-unknown-linux-musl" "x86_64-unknown-linux-gnu")
  else
    targets=("aarch64-unknown-linux-musl" "aarch64-unknown-linux-gnu")
  fi

  TMP_DIR="$(mktemp -d)"
  for target in "${targets[@]}"; do
    asset="panel1-${VERSION_NO_V}-${target}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${TAG}/${asset}"
    archive="${TMP_DIR}/${asset}"
    log "Trying package: ${asset}"
    if download_file "${url}" "${archive}" >/dev/null 2>&1; then
      TARGET="${target}"
      printf '%s' "${archive}"
      return 0
    fi
  done

  return 1
}

install_binary() {
  local source_bin="$1"

  [[ -f "${source_bin}" ]] || die "Binary not found: ${source_bin}"

  if [[ "${EUID}" -eq 0 ]]; then
    mkdir -p "${INSTALL_DIR}"
    install -m 0755 "${source_bin}" "${INSTALL_DIR}/panel1"
    return
  fi

  if mkdir -p "${INSTALL_DIR}" 2>/dev/null && [[ -w "${INSTALL_DIR}" ]]; then
    install -m 0755 "${source_bin}" "${INSTALL_DIR}/panel1"
    return
  fi

  command -v sudo >/dev/null 2>&1 || die "Need write permission to ${INSTALL_DIR} (or install sudo)."
  sudo mkdir -p "${INSTALL_DIR}"
  sudo install -m 0755 "${source_bin}" "${INSTALL_DIR}/panel1"
}

install_from_source() {
  command -v cargo >/dev/null 2>&1 || return 1

  log "No matching prebuilt package found. Falling back to cargo source build."

  local source_repo="https://github.com/${REPO}.git"
  local cargo_args=("install" "--locked" "--git" "${source_repo}" "--bin" "panel1" "--force")

  if [[ "${TAG}" != "latest" && -n "${TAG}" ]]; then
    cargo_args+=("--tag" "${TAG}")
  fi

  cargo_args+=("panel1")

  cargo "${cargo_args[@]}"

  local cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin/panel1"
  [[ -f "${cargo_bin}" ]] || die "cargo install finished but panel1 binary was not found at ${cargo_bin}"
  install_binary "${cargo_bin}"
  return 0
}

verify_linux() {
  local os
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  [[ "${os}" == "linux" ]] || die "This installer supports Linux only. Detected: ${os}"
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)
        [[ $# -ge 2 ]] || die "--version requires a value"
        REQUESTED_VERSION="$2"
        shift 2
        ;;
      --install-dir)
        [[ $# -ge 2 ]] || die "--install-dir requires a value"
        INSTALL_DIR="$2"
        shift 2
        ;;
      --repo)
        [[ $# -ge 2 ]] || die "--repo requires a value"
        REPO="$2"
        shift 2
        ;;
      --no-source-fallback)
        ALLOW_SOURCE_FALLBACK="0"
        shift
        ;;
      -h | --help)
        usage
        exit 0
        ;;
      *)
        die "Unknown option: $1 (use --help)"
        ;;
    esac
  done
}

main() {
  parse_args "$@"
  verify_linux
  pick_downloader
  require_cmd tar
  require_cmd mktemp
  require_cmd find
  resolve_arch
  resolve_version

  log "Installing panel1 ${TAG} for ${ARCH}..."

  local archive
  if archive="$(download_release_archive)"; then
    tar -xzf "${archive}" -C "${TMP_DIR}"
    local extracted_dir="${TMP_DIR}/panel1-${VERSION_NO_V}-${TARGET}"
    local extracted_bin="${extracted_dir}/bin/panel1"

    if [[ ! -f "${extracted_bin}" ]]; then
      extracted_bin="$(find "${TMP_DIR}" -type f -name panel1 | head -n1 || true)"
    fi

    [[ -n "${extracted_bin}" ]] || die "Downloaded package does not contain panel1 binary."
    install_binary "${extracted_bin}"
  else
    if [[ "${ALLOW_SOURCE_FALLBACK}" == "1" ]] && install_from_source; then
      log "Installed from source build."
    else
      die "No compatible release package found for ${ARCH}. Enable cargo fallback or provide a different --repo/--version."
    fi
  fi

  local installed_path="${INSTALL_DIR}/panel1"
  log "Installed: ${installed_path}"

  if "${installed_path}" --version >/dev/null 2>&1; then
    log "Verify OK: $("${installed_path}" --version)"
  else
    warn "Installed binary did not pass --version check. Please run '${installed_path} --version' manually."
  fi

  printf '\n'
  printf 'Quick start:\n'
  printf '  panel1 tui\n'
  printf '  panel1 install <url>\n'
}

main "$@"
