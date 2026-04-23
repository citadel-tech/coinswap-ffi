#!/bin/bash

set -euo pipefail

if [ -z "${ANDROID_NDK_ROOT:-}" ]; then
  echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
  exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
LIB_NAME="libcoinswap_ffi.so"
COMPILATION_TARGET="aarch64-linux-android"
RESOURCE_DIR="arm64-v8a"

cd ../ffi-commons || exit

rustup target add "$COMPILATION_TARGET"

CC="aarch64-linux-android24-clang" \
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android24-clang" \
cargo build --profile release-smaller --target "$COMPILATION_TARGET"

mkdir -p ../coinswap-react-native/android/src/main/jniLibs/$RESOURCE_DIR/
cp ./target/$COMPILATION_TARGET/release-smaller/$LIB_NAME \
  ../coinswap-react-native/android/src/main/jniLibs/$RESOURCE_DIR/

cargo run --bin uniffi-bindgen generate \
  --library ./target/$COMPILATION_TARGET/release-smaller/$LIB_NAME \
  --language kotlin \
  --out-dir ../coinswap-react-native/android/src/main/java/ \
  --no-format

echo "✓ React Native Android release build completed for $COMPILATION_TARGET"
