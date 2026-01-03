<div align="center">

# Coinswap FFI

**Language bindings for the Coinswap protocol**

</div>

## Overview

Coinswap FFI provides Foreign Function Interface (FFI) bindings for the [Coinswap](https://github.com/citadel-tech/coinswap) Bitcoin privacy protocol, enabling integration with multiple programming languages and platforms. This repository contains binding implementations for:

### JavaScript/TypeScript
- **[coinswap-js](./coinswap-js)** - Node.js bindings via NAPI-RS for JavaScript/TypeScript applications

### Multi-Language Bindings (via UniFFI)
Generated from **[ffi-commons](./ffi-commons)** - the core UniFFI binding generator:

- **[coinswap-kotlin](./coinswap-kotlin)** - Kotlin bindings for Android and JVM applications
- **[coinswap-swift](./coinswap-swift)** - Swift bindings for iOS and macOS applications
- **[coinswap-python](./coinswap-python)** - Python bindings for cross-platform applications
- **[coinswap-ruby](./coinswap-ruby)** - Ruby bindings for Ruby applications

## Quick Start

### Node.js (NAPI)

```bash
cd coinswap-napi
yarn install
yarn build
```

See [coinswap-napi/README.md](./coinswap-napi/README.md) for detailed usage.

### Multi-Language Bindings (Kotlin/Swift/Python/Ruby)

```bash
cd coinswap-ffi/ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This generates bindings for all supported languages. See individual language README files for usage:
- [Kotlin README](./coinswap-kotlin/README.md)
- [Swift README](./coinswap-swift/README.md)
- [Python README](./coinswap-python/README.md)
- [Ruby README](./coinswap-ruby/README.md)
- [UniFFI Core README](./ffi-commons/README.md) - Build and binding generation

## Use Cases

- **Desktop Wallets** - Build privacy-focused Bitcoin wallets with Node.js (Electron/Tauri)
- **Mobile Applications** - Native iOS and Android apps with coinswap support
- **Web Applications** - Browser-based wallets via WebAssembly

### Reference Implementation

The [taker-app](https://github.com/citadel-tech/taker-app) demonstrates a production-ready desktop GUI built with the NAPI bindings, showcasing wallet management, swap execution, market analytics, and UTXO control. Use it as a reference for your own applications.

## Requirementss

### Common Dependencies
- Rust 1.75.0 or higher
- Bitcoin Core with RPC access (synced, non-pruned, `-txindex`)
- Tor daemon (for privacy and maker discovery)

### Platform-Specific

#### NAPI (Node.js)
- Node.js 18.0.0 or higher
- Build tools: `build-essential`, `automake`, `libtool`

#### UniFFI (Multi-language)
- Target language SDK (Android SDK, Xcode, Python 3.8+)
- Platform-specific build tools

## Documentation

- [Coinswap Protocol Specification](https://github.com/citadel-tech/Coinswap-Protocol-Specification)
- [Core Coinswap Library](https://github.com/citadel-tech/coinswap)
- [API Documentation](./docs)

## Development Status

**⚠️ Beta Software - Experimental**

These bindings are under active development and in an experimental stage. There are known and unknown bugs. **Mainnet use is strictly NOT recommended.** Use on Custom Signet or Testnet only.

## Contributing

Contributions are welcome! Please see the [main Coinswap repository](https://github.com/citadel-tech/coinswap) for contribution guidelines.

## Acknowledgments

Built on the excellent work of:
- [Chris Belcher's Coinswap Design](https://gist.github.com/chris-belcher/9144bd57a91c194e332fb5ca371d0964)
- [NAPI-RS](https://napi.rs) - Rust bindings for Node.js
- [UniFFI](https://mozilla.github.io/uniffi-rs/) - Multi-language bindings generator
