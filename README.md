<div align="center">

# Coinswap FFI

Language bindings for the Coinswap protocol

</div>

## Overview

Coinswap FFI packages the Coinswap taker API for JavaScript, Kotlin, Swift, Python, Ruby, and React Native. All bindings are backed by the same Rust implementation, so each language follows the same operational model: initialize a taker, sync wallet state and the offer book, inspect balances and UTXOs, execute swaps, and recover or back up state.

## Repository Layout

| Package | Purpose | Supported platforms | Build entry point |
| --- | --- | --- | --- |
| [coinswap-js](./coinswap-js) | Node.js and TypeScript binding via N-API | Linux x64/arm64, macOS x64/arm64, Windows x64/arm64, FreeBSD x64, Android arm64 | `yarn build` |
| [coinswap-kotlin](./coinswap-kotlin) | Kotlin binding for Android and JVM consumers | Android `arm64-v8a`, `armeabi-v7a`, `x86_64`; JVM/Desktop | `build-scripts/` |
| [coinswap-swift](./coinswap-swift) | Swift Package and XCFramework for Apple platforms | iOS arm64, iOS simulator arm64/x86_64, macOS arm64/x86_64 | `build-xcframework*.sh` |
| [coinswap-python](./coinswap-python) | Python package generated with UniFFI | Linux x86_64/aarch64, macOS x86_64/arm64, Windows amd64 | `build-scripts/` plus `python -m build` |
| [coinswap-ruby](./coinswap-ruby) | Ruby FFI binding generated with UniFFI | Linux x86_64/aarch64, macOS x86_64/arm64 | `build-scripts/` |
| [coinswap-react-native](./coinswap-react-native) | React Native TurboModule wrapper over UniFFI-generated native bindings | Android `arm64-v8a`, `x86_64`; iOS arm64, iOS simulator arm64/x86_64 | `build-scripts/` |
| [ffi-commons](./ffi-commons) | Shared Rust crate and UniFFI generation core | Rust build targets used by the bindings above | Consumed by package-local scripts |

## Build Workflow

Build each package from its own directory. The package-local scripts are now the supported entry points for generating bindings and assembling distributable artifacts.

### JavaScript

```bash
cd coinswap-js
yarn install
yarn build
```

### Kotlin

Use the host-specific scripts in `coinswap-kotlin/build-scripts/`, then package with Gradle.

### Swift

```bash
cd coinswap-swift
bash ./build-xcframework.sh
swift build
```

### Python

Run the appropriate script under `coinswap-python/build-scripts/`, then build the wheel or sdist with `python -m build`.

### Ruby

Run the appropriate script under `coinswap-ruby/build-scripts/` to regenerate `coinswap.rb` and the native library for the target platform.

### React Native

Run the appropriate script under `coinswap-react-native/build-scripts/` to regenerate native bindings and stage Android/iOS artifacts.

See the language-specific READMEs for the exact host and target combinations:
- [coinswap-js](./coinswap-js/README.md)
- [coinswap-kotlin](./coinswap-kotlin/README.md)
- [coinswap-swift](./coinswap-swift/README.md)
- [coinswap-python](./coinswap-python/README.md)
- [coinswap-ruby](./coinswap-ruby/README.md)
- [coinswap-react-native](./coinswap-react-native/README.md)
- [ffi-commons](./ffi-commons/README.md)

## Use Cases

- Desktop wallets built with Node.js, Electron, Tauri, Python, or Ruby.
- Native mobile integrations for Android and Apple platforms.
- Internal tooling and automation around wallet state, balances, and swap execution.

## Reference Implementation

The [taker-app](https://github.com/citadel-tech/taker-app) is the primary desktop reference implementation for the Node.js binding and is a useful integration reference for the unified taker workflow.

## Requirements

### Common

- Rust 1.75.0 or newer.
- Bitcoin Core with RPC access, fully synced, non-pruned, and `-txindex` enabled.
- Tor daemon for maker discovery and privacy-preserving network access.

### Package-specific

- Node.js 18+ for `coinswap-js`.
- Android SDK / NDK and JDK for `coinswap-kotlin`.
- Xcode 14+ and Swift 5.7+ for `coinswap-swift`.
- Python 3.8+ for `coinswap-python`.
- Ruby 2.7+ for `coinswap-ruby`.
- React Native 0.76+ and Xcode/Android SDK toolchains for `coinswap-react-native`.

## Documentation

- [Coinswap Protocol Specification](https://github.com/citadel-tech/Coinswap-Protocol-Specification)
- [Core Coinswap Library](https://github.com/citadel-tech/coinswap)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)

## Development Status

Beta software. These bindings remain under active development and should be treated as experimental. Mainnet deployment is not recommended.

## Contributing

Contributions are welcome. See the [main Coinswap repository](https://github.com/citadel-tech/coinswap) for contribution guidelines and protocol-level discussions.

## Acknowledgments

- [Chris Belcher's Coinswap Design](https://gist.github.com/chris-belcher/9144bd57a91c194e332fb5ca371d0964)
- [NAPI-RS](https://napi.rs)
- [UniFFI](https://mozilla.github.io/uniffi-rs/)
