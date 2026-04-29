#!/bin/bash

set -euo pipefail

if [ -z "${ANDROID_NDK_ROOT:-}" ]; then
  if [ -n "${ANDROID_HOME:-}" ] && [ -d "$ANDROID_HOME/ndk" ]; then
    ANDROID_NDK_ROOT=$(ls -d "$ANDROID_HOME/ndk/"*/ 2>/dev/null | sort -V | tail -1)
    ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT%/}"
  elif [ -d "$HOME/Library/Android/sdk/ndk" ]; then
    ANDROID_NDK_ROOT=$(ls -d "$HOME/Library/Android/sdk/ndk/"*/ 2>/dev/null | sort -V | tail -1)
    ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT%/}"
  fi
fi

if [ -z "${ANDROID_NDK_ROOT:-}" ]; then
  echo "Error: ANDROID_NDK_ROOT is not defined and could not be auto-detected."
  echo "Set ANDROID_NDK_ROOT to your NDK path, e.g.: export ANDROID_NDK_ROOT=\$HOME/Library/Android/sdk/ndk/<version>"
  exit 1
fi

if [ "$(uname -s)" = "Darwin" ]; then NDK_HOST="darwin-x86_64"; else NDK_HOST="linux-x86_64"; fi
PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$NDK_HOST/bin:$PATH"
LIB_NAME="libcoinswap_ffi.so"
COMPILATION_TARGET="x86_64-linux-android"
RESOURCE_DIR="x86_64"

cd ../ffi-commons || exit

rustup target add "$COMPILATION_TARGET"

CC="x86_64-linux-android24-clang" \
CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android24-clang" \
cargo build --target "$COMPILATION_TARGET"

mkdir -p ../coinswap-react-native/android/src/main/jniLibs/$RESOURCE_DIR/
cp ./target/$COMPILATION_TARGET/debug/$LIB_NAME \
  ../coinswap-react-native/android/src/main/jniLibs/$RESOURCE_DIR/

cargo run --bin uniffi-bindgen generate \
  --library ./target/$COMPILATION_TARGET/debug/$LIB_NAME \
  --language kotlin \
  --out-dir ../coinswap-react-native/android/src/main/java/ \
  --no-format

echo "✓ React Native Android dev build completed for $COMPILATION_TARGET"
