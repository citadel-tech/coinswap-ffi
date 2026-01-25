#!/bin/bash

# Test/Development build script - builds only x86_64 for Android Studio Emulator
# Use this for fast local testing and development

if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
    exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=24"
AR="llvm-ar"
LIB_NAME="coinswap_ffi.dylib"

COMPILATION_TARGET_X86_64="x86_64-linux-android"
RESOURCE_DIR_X86_64="x86_64"

# Move to the Rust library directory
cd ../ffi-commons || exit

# Install perl and make (required for building vendored OpenSSL)
# Uncomment if not installed: sudo apt-get install -y perl make

rustup target add $COMPILATION_TARGET_X86_64
CC="x86_64-linux-android24-clang" CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android24-clang" cargo build --target $COMPILATION_TARGET_X86_64
mkdir -p ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/
cargo run --bin uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_X86_64/debug/$LIB_NAME --language kotlin --out-dir ../coinswap-kotlin/lib/src/main/kotlin/ --no-format
cp ./target/$COMPILATION_TARGET_X86_64/debug/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64
cp ./target/$COMPILATION_TARGET_X86_64/debug/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/
cp ./target/debug/uniffi-bindgen ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64
cp ./target/debug/uniffi-bindgen.d ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64

echo ""
echo "====================================="
echo "Built for: x86_64"
echo "Binary location: lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/$LIB_NAME"
echo "====================================="
