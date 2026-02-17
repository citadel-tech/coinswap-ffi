#!/bin/bash

set -euo pipefail

HEADER_BASENAME="CoinswapFFI"
TARGETDIR="../ffi-commons/target"
NAME="coinswap_ffi"
PROFILE_DIR="debug"
SWIFT_OUT_DIR="../coinswap-swift/Sources/Coinswap"

MAC_TARGET="x86_64-apple-darwin"
# MAC_TARGET="aarch64-apple-darwin"

cd ../ffi-commons/ || exit

rustup component add rust-src
rustup target add "$MAC_TARGET"

cargo build --package coinswap-ffi --target "$MAC_TARGET"

# # Copy dylib to Sources/CoinswapFFI
# mkdir -p ../coinswap-swift/Sources/CoinswapFFI
# cp ./target/$MAC_TARGET/$PROFILE_DIR/lib${NAME}.dylib ../coinswap-swift/Sources/CoinswapFFI/

UNIFFI_LIBRARY_PATH="./target/$MAC_TARGET/$PROFILE_DIR/lib${NAME}.dylib"
cargo run --bin uniffi-bindgen generate \
    --library "${UNIFFI_LIBRARY_PATH}" \
    --language swift \
    --out-dir "${SWIFT_OUT_DIR}" \
    --no-format

mkdir -p "$SWIFT_OUT_DIR/${HEADER_BASENAME}"
mv "$SWIFT_OUT_DIR/${HEADER_BASENAME}.h" "$SWIFT_OUT_DIR/${HEADER_BASENAME}/${HEADER_BASENAME}.h"
mv "$SWIFT_OUT_DIR/${HEADER_BASENAME}.modulemap" "$SWIFT_OUT_DIR/${HEADER_BASENAME}/module.modulemap"

cd ../coinswap-swift/ || exit

rm -rf "./coinswap_ffi.xcframework"

xcodebuild -create-xcframework \
    -library "${TARGETDIR}/${MAC_TARGET}/${PROFILE_DIR}/libcoinswap_ffi.a" \
    -headers "${SWIFT_OUT_DIR}/${HEADER_BASENAME}" \
    -output "./coinswap_ffi.xcframework"

# Keep Swift sources clean: only .swift files should stay in the package Sources dir
rm -rf "${SWIFT_OUT_DIR}/${HEADER_BASENAME}"
