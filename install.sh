#!/usr/bin/env bash
set -euo pipefail

# -----------------------------------------
# Skeletor installer (macOS arm/x64, Linux x64)
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
      x86_64)         ARCH_TRIPLE="x86_64-apple-darwin" ;;
      *) echo "❌ Unsupported macOS arch: ${UNAME_M}"; exit 1 ;;
    esac
    ;;
  linux)
    OS_STR="ubuntu"  # matches release asset naming
    case "${UNAME_M}" in
      x86_64) ARCH_TRIPLE="x86_64-unknown-linux-gnu" ;;
      *) echo "❌ Only x86_64 Linux build is published presently."; exit 1 ;;
    esac
    ;;
  msys*|cygwin*|*mingw*)
    echo "❌ Windows installer is not published yet. Use WSL or build from source." ; exit 1 ;;
  *)
    echo "❌ Unsupported OS: ${UNAME_S}" ; exit 1 ;;
esac

ASSET="skeletor-${OS_STR}-${ARCH_TRIPLE}.tar.gz"
BASE="https://github.com/${REPO}/releases/download/${VERSION}"
URL="${BASE}/${ASSET}"

# HEAD check to fail fast if asset doesn’t exist
if ! curl -fsI "${URL}" >/dev/null 2>&1; then
  echo "❌ Asset not found: ${URL}"
  echo "   Ensure ${REPO} has a release ${VERSION} with asset ${ASSET}"
  exit 1
fi

TMP_DIR="$(mktemp -d)"; trap 'rm -rf "$TMP_DIR"' EXIT

echo "🔽 Downloading skeletor ${VERSION} for ${OS_STR} (${ARCH_TRIPLE})"
curl -fSL "${URL}" -o "${TMP_DIR}/${ASSET}"

echo "📦 Extracting…"
tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"

# Expect the binary to be named 'skeletor' inside the tarball
BIN_PATH="${TMP_DIR}/skeletor"
[[ -f "${BIN_PATH}" ]] || { echo "❌ Binary 'skeletor' not found in archive."; exit 1; }
chmod +x "${BIN_PATH}"

INSTALL_DIR="/usr/local/bin"
if [[ -w "${INSTALL_DIR}" ]]; then
  mv "${BIN_PATH}" "${INSTALL_DIR}/skeletor"
else
  sudo mv "${BIN_PATH}" "${INSTALL_DIR}/skeletor"
fi

echo "✅ Installed: ${INSTALL_DIR}/skeletor"
echo "👉 skeletor --help"