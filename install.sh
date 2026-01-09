#!/usr/bin/env bash
set -euo pipefail

# -----------------------------------------
# Skeletor installer (macOS arm/x64, Linux x64/arm64, Windows x64/arm64)
# -----------------------------------------

command -v curl >/dev/null 2>&1 || { echo "❌ 'curl' is required."; exit 1; }
command -v tar  >/dev/null 2>&1 || { echo "❌ 'tar' is required.";  exit 1; }

REPO="theGeekist/skeletor"

# Optional pin: SKELETOR_VERSION=vX.Y.Z or pass as first arg
VERSION="${1:-${SKELETOR_VERSION:-}}"
if [[ -z "${VERSION}" ]]; then
  # Robust 'latest' resolution: try API, fall back to Location of releases/latest
  VERSION="$(curl -fsSL -H 'Accept: application/vnd.github+json' \
    "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' || true)"
  if [[ -z "${VERSION}" ]]; then
    VERSION="$(curl -sI -L "https://github.com/${REPO}/releases/latest" \
      | grep -i '^location:' | sed -n 's#.*/tag/\(v[^[:space:]]*\).*#\1#p' | tr -d '\r')"
  fi
fi
[[ -n "${VERSION}" ]] || { echo "❌ Failed to determine version."; exit 1; }

# OS / ARCH matrix -> asset names used in releases
UNAME_S="$(uname -s | tr '[:upper:]' '[:lower:]')"
UNAME_M="$(uname -m)"

case "${UNAME_S}" in
  darwin)
    OS_STR="macos"
    case "${UNAME_M}" in
      arm64|aarch64) ARCH_TRIPLE="aarch64-apple-darwin" ;;
      x86_64)        ARCH_TRIPLE="x86_64-apple-darwin" ;;
      *) echo "❌ Unsupported macOS arch: ${UNAME_M}"; exit 1 ;;
    esac
    ;;
  linux)
    OS_STR="ubuntu"  # matches release asset naming
    case "${UNAME_M}" in
      x86_64)        ARCH_TRIPLE="x86_64-unknown-linux-gnu" ;;
      arm64|aarch64) ARCH_TRIPLE="aarch64-unknown-linux-gnu" ;;
      *) echo "❌ Unsupported Linux arch: ${UNAME_M}"; exit 1 ;;
    esac
    ;;
  msys*|cygwin*|mingw*)
    OS_STR="windows"
    case "${UNAME_M}" in
      x86_64)        ARCH_TRIPLE="x86_64-pc-windows-msvc" ;;
      arm64|aarch64) ARCH_TRIPLE="aarch64-pc-windows-msvc" ;;
      *) echo "❌ Unsupported Windows arch: ${UNAME_M}"; exit 1 ;;
    esac
    ;;
  *)
    echo "❌ Unsupported OS: ${UNAME_S}" ; exit 1 ;;
esac

ASSET="skeletor-${OS_STR}-${ARCH_TRIPLE}.tar.gz"
CHECKSUM_ASSET="${ASSET}.sha256"
BASE="https://github.com/${REPO}/releases/download/${VERSION}"
URL="${BASE}/${ASSET}"
CHECKSUM_URL="${BASE}/${CHECKSUM_ASSET}"

# HEAD check to fail fast if asset doesn’t exist
if ! curl -fsI "${URL}" >/dev/null 2>&1; then
  echo "❌ Asset not found: ${URL}"
  echo "   Ensure ${REPO} has a release ${VERSION} with asset ${ASSET}"
  exit 1
fi

TMP_DIR="$(mktemp -d)"; trap 'rm -rf "$TMP_DIR"' EXIT

echo "🔽 Downloading skeletor ${VERSION} for ${OS_STR} (${ARCH_TRIPLE})"
curl -fSL "${URL}" -o "${TMP_DIR}/${ASSET}"
curl -fSL "${URL}" -o "${TMP_DIR}/${ASSET}"

# Check if checksum file exists before downloading
if curl -fsI "${CHECKSUM_URL}" >/dev/null 2>&1; then
  curl -fSL "${CHECKSUM_URL}" -o "${TMP_DIR}/${CHECKSUM_ASSET}"
  HAS_CHECKSUM=true
else
  echo "⚠️  Checksum file not found. Skipping verification."
  HAS_CHECKSUM=false
fi

# Checksum verification
if [[ "${HAS_CHECKSUM}" == "true" ]]; then
  echo "🔐 Verifying checksum..."
  EXPECTED_SUM="$(cat "${TMP_DIR}/${CHECKSUM_ASSET}")"
  if command -v shasum >/dev/null 2>&1; then
    ACTUAL_SUM="$(shasum -a 256 "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    ACTUAL_SUM="$(sha256sum "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  else
    echo "⚠️  Missing 'shasum' or 'sha256sum', skipping checksum verification."
    ACTUAL_SUM=""
  fi

  if [[ -n "${ACTUAL_SUM}" ]] && [[ "${ACTUAL_SUM}" != "${EXPECTED_SUM}" ]]; then
    echo "❌ Checksum mismatch!"
    echo "   Expected: ${EXPECTED_SUM}"
    echo "   Actual:   ${ACTUAL_SUM}"
    exit 1
  fi
fi

echo "📦 Extracting…"
tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"

# Expect the binary to be named 'skeletor' (or 'skeletor.exe') inside the tarball
BIN_NAME="skeletor"
if [[ "${OS_STR}" == "windows" ]]; then
  BIN_NAME="skeletor.exe"
fi

# The tarball structure from release.yml is dist/<out>/skeletor
# But `tar -C dist -czf "${{ matrix.out }}.tar.gz" "${{ matrix.out }}"`
# means the tarball contains a top-level directory "${{ matrix.out }}"
# So it extracts to: $TMP_DIR/skeletor-<os>-<arch>/skeletor(.exe)

EXTRACTED_DIR="skeletor-${OS_STR}-${ARCH_TRIPLE}"
BIN_PATH="${TMP_DIR}/${EXTRACTED_DIR}/${BIN_NAME}"

# Fallback: find it if the directory structure is different
if [[ ! -f "${BIN_PATH}" ]]; then
  BIN_PATH="$(find "${TMP_DIR}" -name "${BIN_NAME}" -type f | head -n 1)"
fi

if [[ -z "${BIN_PATH}" ]] || [[ ! -f "${BIN_PATH}" ]]; then
  echo "❌ Binary '${BIN_NAME}' not found in archive."
  find "${TMP_DIR}"
  exit 1
fi

chmod +x "${BIN_PATH}"

# Determine install directory
INSTALL_DIR="/usr/local/bin"
USE_SUDO=false

if [[ "${EUID}" -eq 0 ]]; then
  # We are root, so we should install to /usr/local/bin
  if [[ ! -d "${INSTALL_DIR}" ]]; then
    mkdir -p "${INSTALL_DIR}"
  fi
  # Root is always writable to /usr/local/bin if it exists/was created
elif [[ ! -w "${INSTALL_DIR}" ]]; then
  # Check if we can write to /usr/local/bin, if not switch to local bin
  # Note: if /usr/local/bin doesn't exist, -w fails, so we fall here
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "${INSTALL_DIR}"
else
  # /usr/local/bin exists and is writable by current non-root user (rare but possible)
  # We already checked ! -w above, so if we are here, it IS writable.
  :
fi

echo "🚀 Installing to ${INSTALL_DIR}..."

if [[ "${USE_SUDO}" == "true" ]]; then
  sudo mv "${BIN_PATH}" "${INSTALL_DIR}/${BIN_NAME}"
else
  mv "${BIN_PATH}" "${INSTALL_DIR}/${BIN_NAME}"
fi

echo "✅ Installed: ${INSTALL_DIR}/${BIN_NAME}"

# PATH warning
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  echo "⚠️  Warning: ${INSTALL_DIR} is not in your PATH."
fi

echo "👉 skeletor --help"