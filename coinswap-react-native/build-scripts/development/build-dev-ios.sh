#!/bin/bash

set -euo pipefail

HEADER_BASENAME="CoinswapFFI"
TARGETDIR="../ffi-commons/target"
NAME="coinswap_ffi"
STATIC_LIB_NAME="lib${NAME}.a"
PROFILE_DIR="debug"
SWIFT_OUT_DIR="../coinswap-react-native/ios/generated"
HEADER_OUT_DIR="../coinswap-react-native/ios/generated/include"

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

rustup target add "$MAC_TARGET" "$IOS_SIM_TARGET" "$IOS_DEVICE_TARGET"

cargo build --package coinswap-ffi --target "$MAC_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --target "$IOS_SIM_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --target "$IOS_DEVICE_TARGET"

UNIFFI_LIBRARY_PATH="./target/$MAC_TARGET/$PROFILE_DIR/lib${NAME}.dylib"
mkdir -p "$SWIFT_OUT_DIR" "$HEADER_OUT_DIR"

cargo run --bin uniffi-bindgen generate \
  --library "$UNIFFI_LIBRARY_PATH" \
  --language swift \
  --out-dir "$SWIFT_OUT_DIR" \
  --no-format

cargo run --bin uniffi-bindgen generate \
  --library "$UNIFFI_LIBRARY_PATH" \
  --language swift \
  --out-dir "$HEADER_OUT_DIR" \
  --no-format

find "$HEADER_OUT_DIR" -maxdepth 1 -name '*.swift' -delete
if [ -f "$HEADER_OUT_DIR/${HEADER_BASENAME}.modulemap" ]; then
  mv "$HEADER_OUT_DIR/${HEADER_BASENAME}.modulemap" "$HEADER_OUT_DIR/module.modulemap"
fi

cd ../coinswap-react-native/ || exit
rm -rf ./ios/${NAME}.xcframework

xcodebuild -create-xcframework \
  -library "${TARGETDIR}/${IOS_DEVICE_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
  -headers "$HEADER_OUT_DIR" \
  -library "${TARGETDIR}/${IOS_SIM_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
  -headers "$HEADER_OUT_DIR" \
  -output "./ios/${NAME}.xcframework"

echo "✓ React Native iOS dev build completed"
