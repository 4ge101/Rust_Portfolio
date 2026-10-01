#!/bin/sh
# Downloads the prebuilt `portfolio` binary for this OS/CPU from GitHub
# Releases and puts it on PATH. No git clone, no Rust toolchain required.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/4ge101/Rust_Portfolio/main/install.sh | sh

set -eu

REPO="4ge101/Rust_Portfolio"
INSTALL_DIR="${PORTFOLIO_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
    Linux) platform="unknown-linux-gnu" ;;
    Darwin) platform="apple-darwin" ;;
    *)
        echo "error: unsupported OS: $os" >&2
        echo "Download a release manually: https://github.com/${REPO}/releases" >&2
        exit 1
        ;;
esac

case "$arch" in
    x86_64|amd64) cpu="x86_64" ;;
    arm64|aarch64) cpu="aarch64" ;;
    *)
        echo "error: unsupported CPU architecture: $arch" >&2
        exit 1
        ;;
esac

target="${cpu}-${platform}"
url="https://github.com/${REPO}/releases/latest/download/portfolio-${target}.tar.gz"

echo "Downloading portfolio for ${target} ..."
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "$url" -o "$tmp/portfolio.tar.gz"
tar xzf "$tmp/portfolio.tar.gz" -C "$tmp"

mkdir -p "$INSTALL_DIR"
mv "$tmp/portfolio" "$INSTALL_DIR/portfolio"
chmod +x "$INSTALL_DIR/portfolio"

echo "Installed to $INSTALL_DIR/portfolio"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo
        echo "$INSTALL_DIR is not on your PATH. Add this to your shell profile:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac

echo
echo "Run it with: portfolio"