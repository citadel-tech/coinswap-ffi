#!/bin/bash

if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
    exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"
export CC_armv7_linux_androideabi="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi24-clang"
export CXX_armv7_linux_androideabi="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi24-clang++"
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=24"
AR="llvm-ar"
LIB_NAME="libcoinswap_ffi.so"
COMPILATION_TARGET_ARMEABI_V7A="armv7-linux-androideabi"
RESOURCE_DIR_ARMEABI_V7A="armeabi-v7a"

# Move to the ffi creator directory
cd ../ffi-commons/ || exit

# Install perl and make (required for building vendored OpenSSL)
# Uncomment if not installed: sudo apt-get install -y perl make

rustup target add $COMPILATION_TARGET_ARMEABI_V7A
# Build the binaries
# The CC and CARGO_TARGET_<TARGET>_LINUX_ANDROID_LINKER environment variables must be declared on the same line as the cargo build command
CC="armv7a-linux-androideabi24-clang" CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="armv7a-linux-androideabi24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_ARMEABI_V7A
# Copy the binaries to their respective resource directories
mkdir -p ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARMEABI_V7A/
cp ./target/$COMPILATION_TARGET_ARMEABI_V7A/release-smaller/$LIB_NAME ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARMEABI_V7A/

cargo run --bin uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_ARMEABI_V7A/release-smaller/$LIB_NAME --language kotlin --out-dir ../coinswap-kotlin/lib/src/main/kotlin/ --no-format
cp ./target/$COMPILATION_TARGET_ARMEABI_V7A/release-smaller/libcoinswap_ffi.d ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR_ARMEABI_V7A/
