#!/usr/bin/env bash
#
# Build the coinswap_ffi native library for Windows and stage it under
# Coinswap/runtimes/<rid>/native. Intended to run on a Windows runner (Git Bash /
# MSYS) with the MSVC toolchain, or with a configured cross toolchain.
#
# Usage:
#   build-scripts/build-windows.sh [rust-target]
#     rust-target: x86_64-pc-windows-msvc (default) | aarch64-pc-windows-msvc
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CSHARP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_COMMONS_DIR="$(cd "$CSHARP_DIR/../ffi-commons" && pwd)"

PROFILE="${PROFILE:-release-smaller}"
TARGET="${1:-x86_64-pc-windows-msvc}"

case "$TARGET" in
  x86_64-pc-windows-msvc)  RID="win-x64" ;;
  aarch64-pc-windows-msvc) RID="win-arm64" ;;
  *) echo "error: unsupported Windows target '$TARGET'" >&2; exit 1 ;;
esac

# On Windows the cdylib is emitted without the "lib" prefix.
LIB_NAME="coinswap_ffi.dll"
DEST="$CSHARP_DIR/Coinswap/runtimes/$RID/native"

(
  cd "$FFI_COMMONS_DIR"
  rustup target add "$TARGET"
  cargo build --package coinswap-ffi --profile "$PROFILE" --target "$TARGET"
)

mkdir -p "$DEST"
cp "$FFI_COMMONS_DIR/target/$TARGET/$PROFILE/$LIB_NAME" "$DEST/"
echo "Staged: $DEST/$LIB_NAME"
