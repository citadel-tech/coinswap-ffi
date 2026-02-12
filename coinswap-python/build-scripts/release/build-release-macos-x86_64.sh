#!/bin/bash

set -e

COMPILATION_TARGET="x86_64-apple-darwin"
RESOURCE_DIR="darwin-x86_64"
LIB_NAME="libcoinswap_ffi.dylib"

echo "Building for target: $COMPILATION_TARGET"

# Move to ffi-commons directory
cd ../ffi-commons || exit
rustup target add $COMPILATION_TARGET

# Build the library
cargo build --profile release-smaller --target $COMPILATION_TARGET

# Copy the binary to the Python native directory
mkdir -p ../coinswap-python/src/coinswap/native/$RESOURCE_DIR/
cp ./target/$COMPILATION_TARGET/release-smaller/$LIB_NAME ../coinswap-python/src/coinswap/native/$RESOURCE_DIR/
cp ./target/$COMPILATION_TARGET/release-smaller/uniffi-bindgen ../coinswap-python/src/coinswap/native/$RESOURCE_DIR/
cargo run --bin uniffi-bindgen generate --library ./target/$COMPILATION_TARGET/release-smaller/$LIB_NAME --language python --out-dir ../coinswap-python/src/coinswap/native/$RESOURCE_DIR/ --no-format

echo "  Bindings: coinswap-python/src/coinswap/coinswap.py"
echo "✓ Build completed for $COMPILATION_TARGET"
echo "  Binary: coinswap-python/src/coinswap/native/$RESOURCE_DIR/$LIB_NAME"
