<div align="center">

# Coinswap UniFFI

**Multi-language bindings for the Coinswap Bitcoin privacy protocol**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rustc-1.75%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

</div>

## Overview

Coinswap UniFFI provides multi-language bindings for the [Coinswap protocol](https://github.com/citadel-tech/coinswap) using [Mozilla's UniFFI](https://mozilla.github.io/uniffi-rs/). Build native mobile and desktop applications with full coinswap functionality in Kotlin, Swift, Python, and Ruby.

## Supported Languages

| Language | Platform | Status |
|----------|----------|--------|
| **Kotlin** | Android, JVM | ✅ Production Ready |
| **Swift** | iOS, macOS | ✅ Production Ready |
| **Python** | Linux, macOS, Windows | ✅ Production Ready |
| **Ruby** | Linux, macOS | ✅ Production Ready |

## Installation

### Build Core Library

```bash
git clone https://github.com/citadel-tech/coinswap-ffi.git
cd coinswap-ffi/coinswap-uniffi
cargo build --release
```

The compiled library will be at:
- Linux: `target/release/libcoinswap_ffi.so`
- macOS: `target/release/libcoinswap_ffi.dylib`
- Windows: `target/release/coinswap_ffi.dll`

### Generate Language Bindings

#### Kotlin (Android/JVM)

```bash
cargo run --bin uniffi-bindgen generate \
  --library ./target/release/libcoinswap_ffi.so \
  --language kotlin \
  --out-dir ./bindings/kotlin
```

Add to your Android project:
```kotlin
// build.gradle.kts
dependencies {
    implementation(files("libs/coinswap_ffi.jar"))
}

// Copy libcoinswap_ffi.so to src/main/jniLibs/arm64-v8a/
```

#### Swift (iOS/macOS)

```bash
cargo run --bin uniffi-bindgen generate \
  --library ./target/release/libcoinswap_ffi.dylib \
  --language swift \
  --out-dir ./bindings/swift
```

Create XCFramework:
```bash
# Build for multiple targets
cargo build --release --target aarch64-apple-ios
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create XCFramework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libcoinswap_ffi.a \
  -library target/aarch64-apple-darwin/release/libcoinswap_ffi.dylib \
  -output CoinswapFFI.xcframework
```

#### Python

```bash
cargo run --bin uniffi-bindgen generate \
  --library ./target/release/libcoinswap_ffi.so \
  --language python \
  --out-dir ./bindings/python

# Install
cd bindings/python
pip install -e .
```

#### Ruby

```bash
cargo run --bin uniffi-bindgen generate \
  --library ./target/release/libcoinswap_ffi.so \
  --language ruby \
  --out-dir ./bindings/ruby
```

## Quick Start

### Kotlin (Android)

```kotlin
import uniffi.coinswap.*

// Configure RPC
val rpcConfig = RpcConfig(
    url = "http://127.0.0.1:38332",
    auth = RpcAuth.UserPass("user", "password"),
    network = Network.SIGNET,
    walletName = "my_wallet"
)

// Create wallet
val wallet = createWallet(
    dataDir = null,  // Uses default
    walletName = "my_wallet",
    rpcConfig = rpcConfig,
    walletBirthday = null,
    password = "secure_password"
)

// Query balance
val balances = getBalances(wallet)
println("Seed: ${balances.seedBalance} sats")
println("Swap: ${balances.swapBalance} sats")

// Fetch maker offers
val offers = fetchOffers(wallet)
println("Found ${offers.size} makers")
println("Total liquidity: ${offers.sumOf { it.maxSize }} sats")

// Execute swap
val swapParams = SwapParams(
    sendAmount = 1_000_000u,  // 0.01 BTC
    makerCount = 3u,
    manuallySelectedOutpoints = null
)

val report = doSwap(wallet, swapParams)
println("Swap completed in ${report.swapDurationSeconds}s")
println("Total fee: ${report.totalFee} sats")
```

### Swift (iOS)

```swift
import CoinswapFFI

// Configure RPC
let rpcConfig = RpcConfig(
    url: "http://127.0.0.1:38332",
    auth: .userPass(username: "user", password: "password"),
    network: .signet,
    walletName: "my_wallet"
)

// Create wallet
let wallet = try createWallet(
    dataDir: nil,
    walletName: "my_wallet",
    rpcConfig: rpcConfig,
    walletBirthday: nil,
    password: "secure_password"
)

// Query balance
let balances = try getBalances(wallet: wallet)
print("Seed: \(balances.seedBalance) sats")
print("Swap: \(balances.swapBalance) sats")

// Execute swap
let swapParams = SwapParams(
    sendAmount: 1_000_000,
    makerCount: 3,
    manuallySelectedOutpoints: nil
)

let report = try doSwap(wallet: wallet, params: swapParams)
print("Swap completed in \(report.swapDurationSeconds)s")
print("Total fee: \(report.totalFee) sats")
```

### Python

```python
from coinswap_ffi import *

# Configure RPC
rpc_config = RpcConfig(
    url="http://127.0.0.1:38332",
    auth=RpcAuth.user_pass("user", "password"),
    network=Network.SIGNET,
    wallet_name="my_wallet"
)

# Create wallet
wallet = create_wallet(
    data_dir=None,
    wallet_name="my_wallet",
    rpc_config=rpc_config,
    wallet_birthday=None,
    password="secure_password"
)

# Query balance
balances = get_balances(wallet)
print(f"Seed: {balances.seed_balance} sats")
print(f"Swap: {balances.swap_balance} sats")

# Fetch offers
offers = fetch_offers(wallet)
print(f"Found {len(offers)} makers")

# Execute swap
swap_params = SwapParams(
    send_amount=1_000_000,  # 0.01 BTC
    maker_count=3,
    manually_selected_outpoints=None
)

report = do_swap(wallet, swap_params)
print(f"Swap completed in {report.swap_duration_seconds}s")
print(f"Total fee: {report.total_fee} sats")
```

### Ruby

```ruby
require 'coinswap_ffi'

# Configure RPC
rpc_config = CoinswapFfi::RpcConfig.new(
  url: "http://127.0.0.1:38332",
  auth: CoinswapFfi::RpcAuth.user_pass("user", "password"),
  network: CoinswapFfi::Network::SIGNET,
  wallet_name: "my_wallet"
)

# Create wallet
wallet = CoinswapFfi.create_wallet(
  data_dir: nil,
  wallet_name: "my_wallet",
  rpc_config: rpc_config,
  wallet_birthday: nil,
  password: "secure_password"
)

# Query balance
balances = CoinswapFfi.get_balances(wallet)
puts "Seed: #{balances.seed_balance} sats"
puts "Swap: #{balances.swap_balance} sats"

# Execute swap
swap_params = CoinswapFfi::SwapParams.new(
  send_amount: 1_000_000,
  maker_count: 3,
  manually_selected_outpoints: nil
)

report = CoinswapFfi.do_swap(wallet, swap_params)
puts "Swap completed in #{report.swap_duration_seconds}s"
puts "Total fee: #{report.total_fee} sats"
```

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

See [Coinswap documentation](https://github.com/citadel-tech/coinswap/blob/master/docs) for setup.

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