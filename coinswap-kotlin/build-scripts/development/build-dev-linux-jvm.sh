#!/bin/bash

# JVM Test build script - builds for native Linux x86_64 (for running JVM tests)
# This builds for the HOST system, not Android
# Use build-dev-linux-x86_64.sh for Android emulator builds

set -e  # Exit on error

LIB_NAME="libcoinswap_ffi.so"
COMPILATION_TARGET="x86_64-unknown-linux-gnu"
RESOURCE_DIR="x86_64"

echo "========================================="
echo "Building for Linux JVM Testing"
echo "Target: $COMPILATION_TARGET"
echo "========================================="
echo ""

cd ../ffi-commons || exit
echo "📦 Adding Rust target: $COMPILATION_TARGET"
rustup target add $COMPILATION_TARGET
echo "🔨 Building Rust library for native Linux..."
cargo build --target $COMPILATION_TARGET
echo "📁 Creating jniLibs directory..."
mkdir -p ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR/
echo "🔧 Generating Kotlin bindings..."
cargo run --bin uniffi-bindgen generate \
    --library ./target/$COMPILATION_TARGET/debug/$LIB_NAME \
    --language kotlin \
    --out-dir ../coinswap-kotlin/lib/src/main/kotlin/ \
    --no-format

# Copy the native library to jniLibs
echo "📋 Copying native library..."
cp ./target/$COMPILATION_TARGET/debug/$LIB_NAME \
   ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR/
cp ./target/$COMPILATION_TARGET/debug/libcoinswap_ffi.d \
   ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR/
cp ./target/debug/uniffi-bindgen \
   ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR/
cp ./target/debug/uniffi-bindgen.d \
   ../coinswap-kotlin/lib/src/main/jniLibs/$RESOURCE_DIR/

echo ""
echo "========================================="
echo "✅ Build Complete!"
echo "========================================="
echo "Target: $COMPILATION_TARGET (Native Linux JVM)"
echo "Binary: lib/src/main/jniLibs/$RESOURCE_DIR/$LIB_NAME"
echo ""
echo "You can now run JVM tests with:"
echo "  cd ../coinswap-kotlin"
echo "  ./gradlew :lib:test"
echo "========================================="

