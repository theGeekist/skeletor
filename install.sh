#!/usr/bin/env bash
set -euo pipefail

# Ensure required commands are available
command -v curl >/dev/null 2>&1 || { echo "❌ 'curl' is required but not installed."; exit 1; }

REPO="jasonnathan/skeletor"
TMP_DIR="/tmp"  # Consider using mktemp for uniqueness if needed

# Detect OS using OSTYPE
case "$OSTYPE" in
  darwin*) OS="macos" ;;
  linux*) OS="linux" ;;
  msys*|cygwin*|win32*|win64*) OS="windows" ;;
  *) echo "❌ Unsupported OS: $OSTYPE" && exit 1 ;;
esac

# Set OS string for asset naming (note: Linux assets are tagged as "ubuntu")
if [[ "$OS" == "linux" ]]; then
  OS_STR="ubuntu"
else
  OS_STR="$OS"
fi

# Detect Architecture
if [[ "$OS" == "windows" ]]; then
  if command -v wmic >/dev/null 2>&1; then
    ARCH=$(wmic os get osarchitecture | grep -Eo '64-bit|32-bit' || echo "unknown")
  else
    ARCH=$(uname -m 2>/dev/null || echo "unknown")
  fi
  if [[ "$ARCH" == "64-bit" ]]; then
    ARCH="x86_64"
  elif [[ "$ARCH" == "32-bit" ]]; then
    echo "❌ 32-bit Windows is not supported." && exit 1
  else
    echo "❌ Unsupported architecture on Windows: $ARCH" && exit 1
  fi
else
  ARCH=$(uname -m)
fi

# Map architecture to release naming conventions
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH" && exit 1 ;;
esac

# For macOS and Linux, force architecture to x86_64 as that is the only supported build.
if [[ "$OS" == "macos" || "$OS" == "linux" ]]; then
  ARCH="x86_64"
fi

# Determine platform target based on OS and architecture
EXT="tar.gz"
if [[ "$OS" == "macos" ]]; then
    TARGET="${ARCH}-apple-darwin"
elif [[ "$OS" == "linux" ]]; then
    TARGET="${ARCH}-unknown-linux-gnu"
elif [[ "$OS" == "windows" ]]; then
    TARGET="${ARCH}-pc-windows-msvc"
    EXT="zip"
else
    echo "❌ Unsupported OS detected." && exit 1
fi

# Set binary name for asset naming (note: .exe for Windows)
if [[ "$OS" == "windows" ]]; then
  BINARY_NAME="skeletor.exe"
else
  BINARY_NAME="skeletor"
fi

# Fetch latest version using sed for portability
VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
          sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
if [[ -z "$VERSION" ]]; then
    echo "❌ Failed to determine latest version. Check GitHub Releases."
    exit 1
fi

# Define asset URL
ASSET="${BINARY_NAME}-${OS_STR}-${TARGET}.${EXT}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "🔽 Downloading ${BINARY_NAME} ${VERSION} for ${OS}/${ARCH}..."
curl -L --fail "$URL" -o "${TMP_DIR}/${ASSET}"

# Extract & Install
if [[ "$OS" == "windows" ]]; then
    echo "📦 Extracting Windows binary..."
    command -v unzip >/dev/null 2>&1 || { echo "❌ 'unzip' is required but not installed."; exit 1; }
    INSTALL_DIR="/c/Program Files/Skeletor"
    mkdir -p "$INSTALL_DIR"
    unzip -o "${TMP_DIR}/${ASSET}" -d "$INSTALL_DIR"
    echo "✅ Installed to $INSTALL_DIR"
    echo "🔧 Please add '$INSTALL_DIR' to your PATH if necessary."
else
    echo "📦 Extracting binary..."
    command -v tar >/dev/null 2>&1 || { echo "❌ 'tar' is required but not installed."; exit 1; }
    tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
    chmod +x "${TMP_DIR}/skeletor"

    # Install to /usr/local/bin
    INSTALL_DIR="/usr/local/bin"
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "${TMP_DIR}/skeletor" "${INSTALL_DIR}/skeletor"
    else
        sudo mv "${TMP_DIR}/skeletor" "${INSTALL_DIR}/skeletor"
    fi

    echo "✅ Installed to $INSTALL_DIR"
fi

# Cleanup
rm -f "${TMP_DIR}/${ASSET}"

echo "🎉 Installation complete! Run 'skeletor --help' to get started."
