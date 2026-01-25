#!/bin/bash

if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
    exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=24"
AR="llvm-ar"
LIB_NAME="libcoinswap_ffi.so"
COMPILATION_TARGET_ARM64_V8A="aarch64-linux-android"
COMPILATION_TARGET_X86_64="x86_64-linux-android"
RESOURCE_DIR_ARM64_V8A="arm64-v8a"
RESOURCE_DIR_X86_64="x86_64"

# Move to the Rust library directory
cd ../ffi-commons/ || exit

# Install perl and make (required for building vendored OpenSSL)
# Uncomment if not installed: sudo apt-get install -y perl make

rustup target add $COMPILATION_TARGET_ARM64_V8A $COMPILATION_TARGET_X86_64 

# Build the binaries
# The CC and CARGO_TARGET_<TARGET>_LINUX_ANDROID_LINKER environment variables must be declared on the same line as the cargo build command
CC="aarch64-linux-android24-clang" CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_ARM64_V8A
CC="x86_64-linux-android24-clang" CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_X86_64

mkdir -p ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARM64_V8A/
mkdir -p ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/
cp ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARM64_V8A/
cp ./target/$COMPILATION_TARGET_X86_64/release-smaller/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/

cargo run --bin uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME --language kotlin --out-dir ../coinswap-kotlin/lib/src/main/kotlin/ --no-format
cp ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARM64_V8A/
cp ./target/$COMPILATION_TARGET_X86_64/release-smaller/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/