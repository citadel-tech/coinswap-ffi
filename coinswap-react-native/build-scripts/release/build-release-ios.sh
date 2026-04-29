#!/bin/bash

set -euo pipefail

HEADER_BASENAME="CoinswapFFI"
TARGETDIR="../ffi-commons/target"
NAME="coinswap_ffi"
STATIC_LIB_NAME="lib${NAME}.a"
PROFILE_DIR="release-smaller"
SWIFT_OUT_DIR="../coinswap-react-native/ios/generated"
HEADER_OUT_DIR="../coinswap-react-native/ios/generated/include"

IOS_DEVICE_TARGET="aarch64-apple-ios"
IOS_SIM_ARM64="aarch64-apple-ios-sim"
IOS_SIM_X86_64="x86_64-apple-ios"

HOST_ARCH=$(uname -m)
if [ "$HOST_ARCH" = "arm64" ]; then
  MAC_TARGET="aarch64-apple-darwin"
else
  MAC_TARGET="x86_64-apple-darwin"
fi

cd ../ffi-commons/ || exit

rustup target add "$MAC_TARGET" "$IOS_DEVICE_TARGET" "$IOS_SIM_ARM64" "$IOS_SIM_X86_64"

cargo build --package coinswap-ffi --profile "$PROFILE_DIR" --target "$MAC_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile "$PROFILE_DIR" --target "$IOS_DEVICE_TARGET"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile "$PROFILE_DIR" --target "$IOS_SIM_ARM64"
IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --package coinswap-ffi --profile "$PROFILE_DIR" --target "$IOS_SIM_X86_64"

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

mkdir -p "$TARGETDIR/lipo-ios-sim/$PROFILE_DIR"
lipo \
  "$TARGETDIR/$IOS_SIM_ARM64/$PROFILE_DIR/$STATIC_LIB_NAME" \
  "$TARGETDIR/$IOS_SIM_X86_64/$PROFILE_DIR/$STATIC_LIB_NAME" \
  -create -output "$TARGETDIR/lipo-ios-sim/$PROFILE_DIR/$STATIC_LIB_NAME"

cd ../coinswap-react-native/ || exit
rm -rf ./ios/${NAME}.xcframework

xcodebuild -create-xcframework \
  -library "${TARGETDIR}/${IOS_DEVICE_TARGET}/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
  -headers "$HEADER_OUT_DIR" \
  -library "${TARGETDIR}/lipo-ios-sim/${PROFILE_DIR}/${STATIC_LIB_NAME}" \
  -headers "$HEADER_OUT_DIR" \
  -output "./ios/${NAME}.xcframework"

echo "✓ React Native iOS release build completed"
