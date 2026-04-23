#!/bin/bash

set -euo pipefail

if [ -z "${ANDROID_NDK_ROOT:-}" ]; then
  echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
  exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
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
