#!/usr/bin/env bash
#
# Build the coinswap_ffi native library for macOS and stage it under
# Coinswap/runtimes/<rid>/native. Builds the host arch by default; pass a target
# to cross-build (e.g. x86_64-apple-darwin on an Apple-silicon machine).
#
# Usage:
#   build-scripts/build-macos.sh [rust-target]
#     rust-target: aarch64-apple-darwin (default on arm64) | x86_64-apple-darwin
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CSHARP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_COMMONS_DIR="$(cd "$CSHARP_DIR/../ffi-commons" && pwd)"

PROFILE="${PROFILE:-release-smaller}"
TARGET="${1:-$([[ "$(uname -m)" == "arm64" ]] && echo aarch64-apple-darwin || echo x86_64-apple-darwin)}"

case "$TARGET" in
  aarch64-apple-darwin) RID="osx-arm64" ;;
  x86_64-apple-darwin)  RID="osx-x64" ;;
  *) echo "error: unsupported macOS target '$TARGET'" >&2; exit 1 ;;
esac

LIB_NAME="libcoinswap_ffi.dylib"
DEST="$CSHARP_DIR/Coinswap/runtimes/$RID/native"

(
  cd "$FFI_COMMONS_DIR"
  rustup target add "$TARGET"
  cargo build --package coinswap-ffi --profile "$PROFILE" --target "$TARGET"
)

mkdir -p "$DEST"
cp "$FFI_COMMONS_DIR/target/$TARGET/$PROFILE/$LIB_NAME" "$DEST/"
echo "Staged: $DEST/$LIB_NAME"
