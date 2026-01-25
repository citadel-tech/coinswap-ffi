<div align="center">

# Coinswap FFI Commons

**Core UniFFI binding generator for multi-language Coinswap bindings**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rustc-1.75%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

</div>

## Overview

This is the core UniFFI binding generator for the [Coinswap protocol](https://github.com/citadel-tech/coinswap). It uses [Mozilla's UniFFI](https://mozilla.github.io/uniffi-rs/) to generate Foreign Function Interface (FFI) bindings for multiple programming languages from a single Rust codebase.

**Generated Language Bindings:**
- **[coinswap-kotlin](../coinswap-kotlin)** - Android & JVM
- **[coinswap-swift](../coinswap-swift)** - iOS & macOS
- **[coinswap-python](../coinswap-python)** - Cross-platform Python
- **[coinswap-ruby](../coinswap-ruby)** - Ruby applications

## Supported Languages

Language bindings are generated and placed in their respective directories:

| Language | Directory | Platform | Status |
|----------|-----------|----------|--------|
| **Kotlin** | [coinswap-kotlin](../coinswap-kotlin) | Android, JVM | ✅ Production Ready |
| **Swift** | [coinswap-swift](../coinswap-swift) | iOS, macOS | ✅ Production Ready |
| **Python** | [coinswap-python](../coinswap-python) | Linux, macOS, Windows | ✅ Production Ready |
| **Ruby** | [coinswap-ruby](../coinswap-ruby) | Linux, macOS | ✅ Production Ready |

## Building Bindings

### Generate All Language Bindings

```bash
git clone https://github.com/citadel-tech/coinswap-ffi.git
cd coinswap-ffi/ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This will:
1. Build the core Rust library (`libcoinswap_ffi`)
2. Generate bindings for all supported languages
3. Place generated files in their respective language directories:
   - Kotlin: `../coinswap-kotlin/lib/src/main/kotlin/org/coinswap/`
   - Swift: `../coinswap-swift/`
   - Python: `../coinswap-python/`
   - Ruby: `../coinswap-ruby/`

### Using Generated Bindings

After generation, refer to each language's README for usage instructions:
- [Kotlin Quick Start](../coinswap-kotlin/README.md)
- [Swift Quick Start](../coinswap-swift/README.md)
- [Python Quick Start](../coinswap-python/README.md)
- [Ruby Quick Start](../coinswap-ruby/README.md)

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