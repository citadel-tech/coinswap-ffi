#!/usr/bin/env bash
#
# Build the coinswap_ffi native library for Linux and stage it under
# Coinswap/runtimes/<rid>/native.
#
# Usage:
#   build-scripts/build-linux.sh [rust-target]
#     rust-target: x86_64-unknown-linux-gnu (default) | aarch64-unknown-linux-gnu
#
# Cross-compiling to aarch64 needs an appropriate linker/toolchain
# (e.g. `cross`, or gcc-aarch64-linux-gnu with the CARGO_TARGET_* linker set).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CSHARP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_COMMONS_DIR="$(cd "$CSHARP_DIR/../ffi-commons" && pwd)"

PROFILE="${PROFILE:-release-smaller}"
TARGET="${1:-x86_64-unknown-linux-gnu}"

case "$TARGET" in
  x86_64-unknown-linux-gnu)  RID="linux-x64" ;;
  aarch64-unknown-linux-gnu) RID="linux-arm64" ;;
  *) echo "error: unsupported Linux target '$TARGET'" >&2; exit 1 ;;
esac

LIB_NAME="libcoinswap_ffi.so"
DEST="$CSHARP_DIR/Coinswap/runtimes/$RID/native"

(
  cd "$FFI_COMMONS_DIR"
  rustup target add "$TARGET"
  cargo build --package coinswap-ffi --profile "$PROFILE" --target "$TARGET"
)

mkdir -p "$DEST"
cp "$FFI_COMMONS_DIR/target/$TARGET/$PROFILE/$LIB_NAME" "$DEST/"
echo "Staged: $DEST/$LIB_NAME"
