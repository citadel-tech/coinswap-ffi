<div align="center">

# Coinswap Swift

**Swift bindings for the Coinswap Bitcoin privacy protocol**

</div>

## Overview

Swift bindings for [Coinswap](https://github.com/citadel-tech/coinswap), enabling native iOS and macOS integration with the Bitcoin coinswap privacy protocol. Built using [UniFFI](https://mozilla.github.io/uniffi-rs/).

## Quick Start

### Prerequisites

- Swift 5.7+
- Xcode 14+ (for iOS/macOS development)
- Generated bindings (see [Building](#building))

### Building

Generate the Swift bindings from the UniFFI core:

```bash
cd ../ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This generates:
- `coinswap.swift` - Swift binding classes
- `coinswapFFI.h` - C header file
- `coinswapFFI.modulemap` - Module map
- `libcoinswap_ffi.dylib` - Native library (macOS)
- `libcoinswap_ffi.a` - Static library (iOS)

### Installation

#### iOS

1. Build for iOS targets:
```bash
cd ../ffi-commons
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo build --release --target aarch64-apple-ios
```

2. Create an XCFramework:
```bash
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libcoinswap_ffi.a \
  -headers coinswap-swift \
  -output CoinswapFFI.xcframework
```

3. Add to your Xcode project:
   - Drag `CoinswapFFI.xcframework` into your project
   - Add to "Frameworks, Libraries, and Embedded Content"
   - Copy `coinswap.swift` to your project

#### macOS

1. Copy files to your project:
```bash
cp coinswap-swift/coinswap.swift YourApp/
cp coinswap-swift/libcoinswap_ffi.dylib YourApp/
```

2. Add the library to your Xcode project:
   - Add `libcoinswap_ffi.dylib` to "Frameworks and Libraries"
   - Set "Embed" to "Embed & Sign"

### Basic Usage

```swift
import Foundation

// Initialize a Taker
let taker = try Taker(
    dataDir: "/path/to/data",
    walletFileName: "taker_wallet",
    rpcConfig: RPCConfig(
        url: "http://localhost:18442",
        user: "user",
        password: "password",
        walletName: "taker_wallet"
    ),
    controlPort: 9051,
    torAuthPassword: nil,
    zmqAddr: "tcp://localhost:28332",
    password: "your_secure_password"
)

// Setup logging
try taker.setupLogging(dataDir: "/path/to/data")

// Sync wallet
try taker.syncAndSave()

// Get balances
let balances = try taker.getBalances()
print("Total Balance: \(balances.total) sats")

// Get a new receiving address
let address = try taker.getNextExternalAddress(addressType: .p2wpkh)
print("Receive to: \(address.value)")

// Wait for offerbook to sync
print("Waiting for offerbook synchronization...")
while try taker.isOfferbookSyncing() {
    print("Offerbook sync in progress...")
    Thread.sleep(forTimeInterval: 2.0) 
}
print("Offerbook synchronized!")

// Perform a coinswap
let swapParams = SwapParams(
    sendAmount: 1_000_000, // 0.01 BTC in sats
    makerCount: 2,
    manuallySelectedOutpoints: nil
)

if let report = try taker.doCoinswap(swapParams: swapParams) {
    print("Swap completed!")
    print("Amount swapped: \(report.amountSwapped) sats")
    print("Routing fee paid: \(report.routingFeesPaid) sats")
}
```

## API Reference

### Taker Class

Initialize and manage a coinswap taker:

```swift
// Initialize
let taker = try Taker(
    dataDir: String?,
    walletFileName: String?,
    rpcConfig: RPCConfig?,
    controlPort: UInt16?,
    torAuthPassword: String?,
    zmqAddr: String,
    password: String?
)

// Wallet operations
let balances = try taker.getBalances()
let address = try taker.getNextExternalAddress(addressType: AddressType)
let txs = try taker.getTransactions(count: UInt32?, skip: UInt32?)
let utxos = try taker.listAllUtxoSpendInfo()
let txid = try taker.sendToAddress(
    address: String,
    amount: Int64,
    feeRate: Double?,
    manuallySelectedOutpoints: [OutPoint]?
)

// Swap operations
let report = try taker.doCoinswap(swapParams: SwapParams)
let offers = try taker.fetchOffers()
let syncing = try taker.isOfferbookSyncing()

// Maintenance
try taker.syncAndSave()
try taker.backup(destinationPath: String, password: String?)
try taker.recoverFromSwap()
```

### Data Types

```swift
struct SwapParams {
    let sendAmount: UInt64        // Amount to swap in satoshis
    let makerCount: UInt32         // Number of makers (hops)
    let manuallySelectedOutpoints: [OutPoint]?
}

struct Balances {
    let regular: Int64             // Regular wallet balance in sats
    let swap: Int64                // Swap balance in sats
    let contract: Int64            // Contract balance in sats
    let spendable: Int64           // Spendable balance in sats
}

struct SwapReport {
    let swapId: String             // Unique swap identifier
    let swapDurationSeconds: Double // Duration of swap in seconds
    let targetAmount: Int64        // Target swap amount in sats
    let totalInputAmount: Int64    // Total input amount in sats
    let totalOutputAmount: Int64   // Total output amount in sats
    let makersCount: UInt32        // Number of makers in swap
    let makerAddresses: [String]   // List of maker addresses
    let totalFundingTxs: Int64     // Total number of funding transactions
    let fundingTxidsByHop: [[String]] // Funding TXIDs grouped by hop
    let totalFee: Int64            // Total fees paid in sats
    let totalMakerFees: Int64      // Total maker fees in sats
    let miningFee: Int64           // Mining fees in sats
    let feePercentage: Double      // Fee as percentage of amount
    let makerFeeInfo: [MakerFeeInfo] // Detailed fee info per maker
    let inputUtxos: [Int64]        // Input UTXO amounts
    let outputChangeAmounts: [Int64]    // Change output amounts
    let outputSwapAmounts: [Int64]      // Swap output amounts
    let outputChangeUtxos: [UtxoWithAddress] // Change UTXOs with addresses
    let outputSwapUtxos: [UtxoWithAddress]   // Swap UTXOs with addresses
}

enum AddressType {
    case p2wpkh  // Native SegWit (bech32)
    case p2tr    // Taproot (bech32m)
}
```

## Examples

Complete examples are available in the [`test/`](test/) directory:
- [`iOSExample.swift`](test/iOSExample.swift) - iOS integration with UIKit
- [`SwiftUIExample.swift`](test/SwiftUIExample.swift) - SwiftUI integration with async/await

## Requirements

### iOS
- iOS 13.0 or later
- Architectures: arm64 (device), x86_64/arm64 (simulator)
- Capabilities: Network access

### macOS
- macOS 10.15 or later
- Architectures: x86_64, arm64 (Apple Silicon)

### Bitcoin Setup
- Bitcoin Core with RPC enabled
- Synced, non-pruned node with `-txindex`
- Tor daemon running for privacy

## Error Handling

All operations that can fail throw `TakerError`:

```swift
do {
    let balances = try taker.getBalances()
    print("Balance: \(balances.total)")
} catch TakerError.General(let msg) {
    print("General error: \(msg)")
} catch TakerError.Wallet(let msg) {
    print("Wallet error: \(msg)")
} catch TakerError.Network(let msg) {
    print("Network error: \(msg)")
} catch {
    print("Unknown error: \(error)")
}
```

## Cross-Compilation

Build for iOS and macOS:

```bash
cd ../ffi-commons

# iOS Device (arm64)
rustup target add aarch64-apple-ios
cargo build --release --target aarch64-apple-ios

# iOS Simulator (arm64 for M1+, x86_64 for Intel)
rustup target add aarch64-apple-ios-sim x86_64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

# macOS
cargo build --release --target aarch64-apple-darwin  # Apple Silicon
cargo build --release --target x86_64-apple-darwin   # Intel
```

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
