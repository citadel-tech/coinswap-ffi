<div align="center">

# Coinswap UniFFI

**Multi-language bindings for the Coinswap Bitcoin privacy protocol**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rustc-1.75%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

</div>

## Overview

Coinswap UniFFI provides multi-language bindings for the [Coinswap protocol](https://github.com/citadel-tech/coinswap) using [Mozilla's UniFFI](https://mozilla.github.io/uniffi-rs/). Build native mobile and desktop applications with full coinswap functionality in Kotlin, Swift, Python, and Ruby.

## Supported Languages

| Language | Platform |
|----------|----------|
| **Kotlin** | Android, JVM | ✅ Production Ready |
| **Swift** | iOS, macOS | ✅ Production Ready |
| **Python** | Linux, macOS, Windows | ✅ Production Ready |
| **Ruby** | Linux, macOS | ✅ Production Ready |

## Installation

### Build Core Library

```bash
git clone https://github.com/citadel-tech/coinswap-ffi.git
cd coinswap-ffi/coinswap-uniffi
chmod +x create_bindings.sh
./create_bindings.sh
```

The compiled library will be at:
- Linux: `target/release/libcoinswap_ffi.so`
- macOS: `target/release/libcoinswap_ffi.dylib`
- Windows: `target/release/coinswap_ffi.dll`

## Development

### Prerequisites

- Rust 1.75.0+
- UniFFI CLI: `cargo install uniffi-bindgen`
- Target language toolchain (Android SDK, Xcode, Python, Ruby)

### Cross-compilation

#### Android

```bash
# Add targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Build
cargo build --release --target aarch64-linux-android
```

#### iOS

```bash
# Add targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios  # Simulator
rustup target add aarch64-apple-ios-sim  # M1 Simulator

# Build
cargo build --release --target aarch64-apple-ios
```

## Requirements

See [Coinswap documentation](https://github.com/citadel-tech/coinswap/docs) for setup.

## Platform-Specific Notes

### Android
- Minimum SDK: 24 (Android 7.0)
- Target SDK: 34+
- Native libraries: `arm64-v8a`, `armeabi-v7a`, `x86_64`
- Permissions: Internet, Network State

### iOS
- Minimum iOS: 13.0
- Architectures: arm64, x86_64 (simulator)
- Frameworks: CoinswapFFI.xcframework
- Capabilities: Network access

### Python
- Python 3.8+
- Works with PyPy
- No additional dependencies

### Ruby
- Ruby 2.7+
- FFI gem (auto-installed)

## Performance

UniFFI introduces minimal overhead:
- Function calls: < 1ms
- Data serialization: < 5ms
- Wallet operations: Near-native speed
- Swap execution: Network-bound (30-120s)

## Troubleshooting

### Library not found

Ensure the compiled library is in the correct location:
- Linux: `LD_LIBRARY_PATH`
- macOS: `DYLD_LIBRARY_PATH`
- Android: `jniLibs/`
- iOS: Embedded in framework

## Contributing

Contributions welcome! See [main repository](https://github.com/citadel-tech/coinswap) for guidelines.

## Resources

- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [Coinswap Protocol](https://gist.github.com/chris-belcher/9144bd57a91c194e332fb5ca371d0964)
- [Coinswap Implementation](https://github.com/citadel-tech/coinswap)
- [Android Development](https://developer.android.com/)
- [iOS Development](https://developer.apple.com/)

## Support

- Issues: [GitHub Issues](https://github.com/citadel-tech/coinswap-ffi/issues)
- Discussions: [GitHub Discussions](https://github.com/citadel-tech/coinswap/discussions)