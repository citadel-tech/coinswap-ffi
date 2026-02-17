#!/bin/bash

set -euo pipefail

HEADER_BASENAME="CoinswapFFI"
TARGETDIR="../ffi-commons/target"
OUTDIR="."
RELDIR="release-smaller"
NAME="coinswap_ffi"
STATIC_LIB_NAME="lib${NAME}.a"
NEW_HEADER_DIR="../ffi-commons/target/include"
SWIFT_OUT_DIR="../coinswap-swift/Sources/Coinswap"
HEADER_OUT_DIR="${NEW_HEADER_DIR}/${HEADER_BASENAME}"

cd ../ffi-commons/ || exit

# install component and targets
rustup component add rust-src
rustup target add aarch64-apple-ios      # iOS arm64
rustup target add x86_64-apple-ios       # iOS x86_64
rustup target add aarch64-apple-ios-sim  # simulator mac M1
rustup target add aarch64-apple-darwin   # mac M1
rustup target add x86_64-apple-darwin    # mac x86_64

# build coinswap-ffi rust lib for apple targets
cargo build --package coinswap-ffi --profile release-smaller --target x86_64-apple-darwin
cargo build --package coinswap-ffi --profile release-smaller --target aarch64-apple-darwin
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile release-smaller --target x86_64-apple-ios
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile release-smaller --target aarch64-apple-ios
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile release-smaller --target aarch64-apple-ios-sim

# build coinswap-ffi Swift bindings and put in coinswap-swift Sources
UNIFFI_LIBRARY_PATH="./target/aarch64-apple-ios/${RELDIR}/lib${NAME}.dylib"
cargo run --bin uniffi-bindgen generate --library "${UNIFFI_LIBRARY_PATH}" --language swift --out-dir "${SWIFT_OUT_DIR}" --no-format

# Final xcframework structure (per-arch):
#   Headers/
#     <ModuleName>/
#       <ModuleName>.h
#       module.modulemap
rm -rf "${NEW_HEADER_DIR:?}"/*
rm -rf "${HEADER_OUT_DIR:?}"
mkdir -p "${HEADER_OUT_DIR}"
cargo run --bin uniffi-bindgen generate --library "${UNIFFI_LIBRARY_PATH}" --language swift --out-dir "${HEADER_OUT_DIR}" --no-format

# Keep the header output directory clean: xcframework headers should only contain .h + module.modulemap
find "${HEADER_OUT_DIR}" -maxdepth 1 -name '*.swift' -delete

# Uniffi emits <basename>.modulemap; rename it to module.modulemap (expected by Apple toolchains)
if [ -f "${HEADER_OUT_DIR}/${HEADER_BASENAME}.modulemap" ]; then
    mv "${HEADER_OUT_DIR}/${HEADER_BASENAME}.modulemap" "${HEADER_OUT_DIR}/module.modulemap"
fi

echo -e "\n" >> "${HEADER_OUT_DIR}/module.modulemap"

# Keep Swift sources clean: only .swift files should stay in the package Sources dir
rm -f "${SWIFT_OUT_DIR}/${HEADER_BASENAME}.h"
rm -f "${SWIFT_OUT_DIR}/${HEADER_BASENAME}.modulemap"

# combine coinswap-ffi static libs for aarch64 and x86_64 targets via lipo tool
mkdir -p target/lipo-macos/${RELDIR}
lipo target/aarch64-apple-darwin/${RELDIR}/${STATIC_LIB_NAME} target/x86_64-apple-darwin/${RELDIR}/${STATIC_LIB_NAME} -create -output target/lipo-macos/${RELDIR}/${STATIC_LIB_NAME}
mkdir -p target/lipo-ios-sim/${RELDIR}
lipo target/aarch64-apple-ios-sim/${RELDIR}/${STATIC_LIB_NAME} target/x86_64-apple-ios/${RELDIR}/${STATIC_LIB_NAME} -create -output target/lipo-ios-sim/${RELDIR}/${STATIC_LIB_NAME}

cd ../coinswap-swift/ || exit

# remove old xcframework directory
rm -rf "${OUTDIR}/${NAME}.xcframework"

# create new xcframework directory from coinswap-ffi static libs and headers
xcodebuild -create-xcframework \
    -library "${TARGETDIR}/lipo-macos/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/lipo-ios-sim/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -output "${OUTDIR}/${NAME}.xcframework"
#   1. macOS (both Intel + Apple Silicon)
#   2. iOS device (arm64)
#   3. iOS simulator (both Intel + Apple Silicon)