#!/usr/bin/env bash
# filepath: /Users/jasonnathan/Repos/skeletor/install.sh
set -e

VERSION="v2.2.0"  # You could modify this to be dynamic via GitHub API
REPO="jasonnathan/skeletor"

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH" ; exit 1 ;;
esac

# Build the binary name. Adjust if needed by your release artifacts pattern.
BINARY="skeletor"
ASSET="${BINARY}-${OS}-${ARCH}.tar.gz"

# Download URL for the asset, assuming GitHub releases are used.
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "Downloading ${BINARY} ${VERSION} for ${OS}/${ARCH}..."
curl -L --fail "$URL" -o /tmp/"${ASSET}"

echo "Extracting binary..."
tar -xzf /tmp/"${ASSET}" -C /tmp
chmod +x /tmp/"${BINARY}"

# Move binary to /usr/local/bin (adjust as necessary for your system)
echo "Installing ${BINARY} to /usr/local/bin (you might need sudo)..."
if [ "$(id -u)" -ne 0 ]; then
    sudo mv /tmp/"${BINARY}" /usr/local/bin/"${BINARY}"
else
    mv /tmp/"${BINARY}" /usr/local/bin/"${BINARY}"
fi

rm /tmp/"${ASSET}"
echo "Installation complete! Run 'skeletor --help' to get started."