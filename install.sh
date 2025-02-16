#!/usr/bin/env bash
set -euo pipefail

# Ensure required commands are available
command -v curl >/dev/null 2>&1 || { echo "❌ 'curl' is required but not installed."; exit 1; }

REPO="jasonnathan/skeletor"
# Create a unique temporary directory and ensure cleanup on exit
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Detect OS using OSTYPE
case "$OSTYPE" in
  darwin*) OS="macos" ;;
  linux*) OS="linux" ;;
  msys*|cygwin*|win32*|win64*) OS="windows" ;;
  *) echo "❌ Unsupported OS: $OSTYPE" && exit 1 ;;
esac

# For asset naming, Linux assets are tagged as "ubuntu"
if [[ "$OS" == "linux" ]]; then
  OS_STR="ubuntu"
else
  OS_STR="$OS"
fi

# Determine architecture based on OS
if [[ "$OS" == "windows" ]]; then
    # For Windows, use wmic if available
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
elif [[ "$OS" == "macos" ]]; then
    ARCH=$(uname -m)
    if [[ "$ARCH" == "arm64" ]]; then
        ARCH="aarch64"
    else
        ARCH="x86_64"
    fi
elif [[ "$OS" == "linux" ]]; then
    # Currently only the x86_64 Linux build is provided
    ARCH="x86_64"
fi

# Determine target and file extension based on OS and architecture
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

# Determine binary name for asset naming (append .exe for Windows)
if [[ "$OS" == "windows" ]]; then
  BINARY_NAME="skeletor.exe"
else
  BINARY_NAME="skeletor"
fi

# Fetch latest version using sed (avoids non-portable grep -P)
VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
          sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
if [[ -z "$VERSION" ]]; then
    echo "❌ Failed to determine latest version. Check GitHub Releases."
    exit 1
fi

# Construct asset name and URL
ASSET="${BINARY_NAME}-${OS_STR}-${TARGET}.${EXT}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "🔽 Downloading ${BINARY_NAME} ${VERSION} for ${OS} ($ARCH)..."
curl -L --fail "$URL" -o "${TMP_DIR}/${ASSET}"

# Extract & install
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

echo "🎉 Installation complete! Run 'skeletor --help' to get started."
