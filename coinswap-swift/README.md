<div align="center">

# Coinswap Swift

**Swift bindings for the Coinswap Bitcoin privacy protocol**

</div>

## Overview

Swift bindings for [Coinswap](https://github.com/citadel-tech/coinswap), enabling native iOS and macOS integration with the Bitcoin coinswap protocol. Built using [UniFFI](https://mozilla.github.io/uniffi-rs/).

## Quick Start

### Prerequisites

- Swift 5.7+
- Xcode 14+ (for iOS/macOS development)
- Generated bindings (see [Building](#building))

### Building

Use the xcframework scripts in this folder:

```bash
# Dev build (fast, debug; builds host arch + iOS device + iOS simulator)
bash ./build-xcframework-dev.sh

# Workflow build (configured for github CI only; builds x86_64 Mac-Intel)
bash ./build-xcframework-ci.sh

# Release build (release-smaller profile; builds all Apple targets)
bash ./build-xcframework.sh
```

Outputs for production build:
- `Sources/Coinswap/Coinswap.swift` - Swift bindings
- `CoinswapFFI.h` and `module.modulemap` - C headers
- `libcoinswap_ffi.a` - native static lib
- `coinswap_ffi.xcframework` - packaged slices (macOS, iOS device, iOS simulator). 
- Each platform slice has the aforementioned C header and static lib files.

### Installation

4. Add the package to your app:

Xcode: File > Add Packages... and select the `coinswap-swift` folder.

Package.swift:
```swift
.package(path: "../coinswap-swift")
```

Then depend on `Coinswap` and `import Coinswap` in your app.

#### iOS and macOS

Use the generated `coinswap_ffi.xcframework` and the Swift package in this repo.

### Basic Usage

```swift
import Foundation

// Initialize a Taker
let taker = try Taker.`init`(
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


// Manual offerbook sync in case the syncing doesn't initialize
try taker.runOfferSyncNow()

// Perform a coinswap
let swapParams = SwapParams(
    sendAmount: 1_000_000, // 0.01 BTC in sats
    makerCount: 2,
    manuallySelectedOutpoints: nil
)

if let report = try taker.doCoinswap(swapParams: swapParams) {
    print("Swap completed!")
}
```

## API Reference

### Taker Class

Initialize and manage a coinswap taker:

```swift
// Initialize
let taker = try Taker.`init`(
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

Live integration tests are in [Tests/CoinswapTests](Tests/CoinswapTests):
- [LiveStandardSwapTests.swift](Tests/CoinswapTests/LiveStandardSwapTests.swift)
- [LiveTaprootSwapTests.swift](Tests/CoinswapTests/LiveTaprootSwapTests.swift)
- [LiveTestSupport.swift](Tests/CoinswapTests/LiveTestSupport.swift)

To test them, fire up the docker setup in ```../ffi-commons``` folder:
```bash
./ffi-docker-setup help 
./ffi-docker-setup start  (Starts tor, 2 legacy makers, regtest bitcoind)
./ffi-docker-setup start --run-all  (Starts tor, 2 legacy makers and 2 taproot makers, regtest bitcoind)
./ffi-docker-setup stop
```
Alternatively, you can set them up locally and toggle the ports in the [LiveTaprootSwapTests.swift](Tests/CoinswapTests/LiveTaprootSwapTests.swift)

While testing, you may encounter warnings like "ld: warning: object file (...) was built for newer 'macOS' version (X.X) than being linked (14.0)". This occurs because the library is built with a minimum deployment target of macOS 14.0 for broad compatibility, but the build environment may be running a newer macOS version. These warnings are harmless and do not affect the functionality or performance of the library.

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

Prefer the xcframework script, which handles targets and combines slices:

```bash
bash ./build-xcframework.sh
```

It builds all Apple targets, then uses `lipo` to merge:
- macOS arm64 + x86_64 -> a universal macOS static library
- iOS simulator arm64 + x86_64 -> a universal simulator static library

Those combined slices are packaged into `coinswap_ffi.xcframework` alongside the iOS device (arm64) slice.

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
