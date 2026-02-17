#!/bin/bash

set -euo pipefail

HEADER_BASENAME="CoinswapFFI"
TARGETDIR="../ffi-commons/target"
NAME="coinswap_ffi"
STATIC_LIB_NAME="lib${NAME}.a"
NEW_HEADER_DIR="../coinswap-swift/Sources/CoinswapFFI/include"
PROFILE_DIR="debug"
SWIFT_OUT_DIR="../coinswap-swift/Sources/Coinswap"

HOST_ARCH=$(uname -m)
if [ "$HOST_ARCH" = "arm64" ]; then
    MAC_TARGET="aarch64-apple-darwin"
    IOS_SIM_TARGET="aarch64-apple-ios-sim"
else
    MAC_TARGET="x86_64-apple-darwin"
    IOS_SIM_TARGET="x86_64-apple-ios"
fi
IOS_DEVICE_TARGET="aarch64-apple-ios"

cd ../ffi-commons/ || exit

rustup component add rust-src
rustup target add "$MAC_TARGET" "$IOS_SIM_TARGET" "$IOS_DEVICE_TARGET"

cargo build --package coinswap-ffi --target "$MAC_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --target "$IOS_SIM_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --target "$IOS_DEVICE_TARGET"

# Copy dylib to Sources/CoinswapFFI
mkdir -p ../coinswap-swift/Sources/CoinswapFFI
cp ./target/$MAC_TARGET/$PROFILE_DIR/lib${NAME}.dylib ../coinswap-swift/Sources/CoinswapFFI/

UNIFFI_LIBRARY_PATH="./target/$MAC_TARGET/$PROFILE_DIR/lib${NAME}.dylib"
cargo run --bin uniffi-bindgen generate \
    --library "${UNIFFI_LIBRARY_PATH}" \
    --language swift \
    --out-dir "${SWIFT_OUT_DIR}" \
    --no-format

# Final xcframework structure (per-arch):
#   Headers/
#     <ModuleName>/
#       <ModuleName>.h
#       module.modulemap
rm -rf "${NEW_HEADER_DIR:?}"/*
mkdir -p "${NEW_HEADER_DIR}"
cargo run --bin uniffi-bindgen generate \
    --library "${UNIFFI_LIBRARY_PATH}" \
    --language swift \
    --out-dir "${NEW_HEADER_DIR}" \
    --no-format

# Keep the header output directory clean: xcframework headers should only contain .h + module.modulemap
find "${NEW_HEADER_DIR}" -maxdepth 1 -name '*.swift' -delete

# Uniffi emits <basename>.modulemap; rename it to module.modulemap (expected by Apple toolchains)
if [ -f "${NEW_HEADER_DIR}/${HEADER_BASENAME}.modulemap" ]; then
    mv "${NEW_HEADER_DIR}/${HEADER_BASENAME}.modulemap" "${NEW_HEADER_DIR}/module.modulemap"
fi

echo -e "\n" >> "${NEW_HEADER_DIR}/module.modulemap"

# Keep Swift sources clean: only .swift files should stay in the package Sources dir
rm -f "${SWIFT_OUT_DIR}/${HEADER_BASENAME}.h"
rm -f "${SWIFT_OUT_DIR}/${HEADER_BASENAME}.modulemap"

cd ../coinswap-swift/ || exit

rm -rf "./${NAME}.xcframework"

xcodebuild -create-xcframework \
    -library "${TARGETDIR}/${MAC_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/${IOS_DEVICE_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/${IOS_SIM_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -output "./${NAME}.xcframework"