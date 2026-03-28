#!/bin/bash

set -e

COMPILATION_TARGET="x86_64-apple-darwin"
LIB_NAME="libcoinswap_ffi.dylib"

echo "Building for target: $COMPILATION_TARGET"

cd ../ffi-commons || exit
rustup target add $COMPILATION_TARGET

cargo build --target $COMPILATION_TARGET

cargo run --bin uniffi-bindgen generate \
  --library ./target/$COMPILATION_TARGET/debug/$LIB_NAME \
  --language ruby \
  --out-dir ../coinswap-ruby \
  --no-format

cp ./target/$COMPILATION_TARGET/debug/$LIB_NAME ../coinswap-ruby/

echo "  Bindings: coinswap-ruby/coinswap.rb"
echo "  Binary: coinswap-ruby/$LIB_NAME"
echo "Build completed for $COMPILATION_TARGET"