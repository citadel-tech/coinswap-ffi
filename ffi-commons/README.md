<div align="center">

# Coinswap FFI Commons

Shared Rust and UniFFI core for the Coinswap language bindings

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rustc-1.75%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

</div>

## Overview

`ffi-commons` contains the Rust crate, UniFFI configuration, and helper tooling shared by the Kotlin, Swift, Python, Ruby, and React Native bindings. It is the source of truth for the exported taker API, data types, and generated foreign-language surfaces.

In normal use, you should build from the language package you are shipping. Each package owns the supported build scripts for staging native artifacts and regenerating bindings.

## Downstream Bindings

| Binding | Output directory | Runtime targets |
| --- | --- | --- |
| [coinswap-kotlin](../coinswap-kotlin) | `../coinswap-kotlin/lib/src/main/` | Android `arm64-v8a`, `armeabi-v7a`, `x86_64`, JVM/Desktop |
| [coinswap-swift](../coinswap-swift) | `../coinswap-swift/Sources/` and `../coinswap-swift/coinswap_ffi.xcframework` | iOS arm64, iOS simulator arm64/x86_64, macOS arm64/x86_64 |
| [coinswap-python](../coinswap-python) | `../coinswap-python/src/coinswap/` | Linux x86_64/aarch64, macOS x86_64/arm64, Windows amd64 |
| [coinswap-ruby](../coinswap-ruby) | `../coinswap-ruby/` | Linux x86_64/aarch64, macOS x86_64/arm64 |
| [coinswap-react-native](../coinswap-react-native) | `../coinswap-react-native/android/src/main/` and `../coinswap-react-native/ios/` | Android `arm64-v8a`, `x86_64`; iOS arm64, iOS simulator arm64/x86_64 |

## Supported Build Model

The supported workflow is package-local:

- Kotlin builds are driven from `coinswap-kotlin/build-scripts/` and then packaged with Gradle.
- Swift builds are driven from `coinswap-swift/build-xcframework-dev.sh`, `build-xcframework-ci.sh`, or `build-xcframework.sh`.
- Python builds are driven from `coinswap-python/build-scripts/` and then packaged with `python -m build`.
- Ruby builds are driven from `coinswap-ruby/build-scripts/`.
- React Native TurboModule builds are driven from `coinswap-react-native/build-scripts/`.

This keeps target selection, output layout, and packaging concerns next to the language consumer instead of centralizing them in a single monolithic script.

## Direct Core Development

Work directly in `ffi-commons` when you are changing the exported Rust API, UniFFI schema, or shared build logic.

### Prerequisites

- Rust 1.75.0 or newer.
- `cargo run --bin uniffi-bindgen` available from this workspace.
- Platform toolchains for the targets you intend to build.

### Example: Build a Shared Library Directly

```bash
cd ffi-commons
rustup target add x86_64-unknown-linux-gnu
cargo build --package coinswap-ffi --profile release-smaller --target x86_64-unknown-linux-gnu
```

### Example: Generate Bindings Manually

```bash
cd ffi-commons
cargo run --bin uniffi-bindgen generate \
   --library ./target/x86_64-unknown-linux-gnu/release-smaller/libcoinswap_ffi.so \
   --language python \
   --out-dir ../coinswap-python/src/coinswap/native/linux-x86_64 \
   --no-format
```

The package-local scripts wrap these steps and place outputs in the paths expected by each binding.

## Target Notes

### Android

- Minimum SDK: 24.
- Primary ABIs: `arm64-v8a`, `armeabi-v7a`, `x86_64`.
- Requires Android NDK for native builds.

### Apple Platforms

- Swift packaging targets iOS 13+ and macOS 10.15+.
- XCFramework builds combine device and simulator slices for the Apple consumers.

### Python

- Packaged native resources are staged under `src/coinswap/native/<platform>/`.
- The Python package metadata declares Linux, macOS, and Windows native resources.

### Ruby

- Generated Ruby bindings live at the package root as `coinswap.rb`.
- Native libraries are staged next to the binding for direct FFI loading.

### React Native (TurboModule)

- JavaScript surface and TurboModule spec live under `coinswap-react-native/src/`.
- Android native bridge and JNI libraries are staged under `coinswap-react-native/android/src/main/`.
- iOS native bridge and `coinswap_ffi.xcframework` are staged under `coinswap-react-native/ios/`.
- Live Legacy and Taproot swap tests are provided in `coinswap-react-native/__tests__/` and use the shared docker regtest stack.

## Docker Test Environment

`ffi-docker-setup` provisions the local regtest environment used by the live integration flows:

```bash
cd ffi-commons
./ffi-docker-setup setup
./ffi-docker-setup start 4
./ffi-docker-setup stop
```

`start 4` brings up Bitcoin Core, Tor, and four maker services for end-to-end taker testing.

## Resources

- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [Coinswap Protocol](https://gist.github.com/chris-belcher/9144bd57a91c194e332fb5ca371d0964)
- [Coinswap Implementation](https://github.com/citadel-tech/coinswap)

## Support

- Issues: [GitHub Issues](https://github.com/citadel-tech/coinswap-ffi/issues)
- Discussions: [GitHub Discussions](https://github.com/citadel-tech/coinswap/discussions)