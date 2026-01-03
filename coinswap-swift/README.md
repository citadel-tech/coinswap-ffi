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
        url: "http://localhost:18443",
        user: "bitcoin",
        password: "bitcoin",
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
    let total: UInt64              // Total balance in sats
    let confirmed: UInt64          // Confirmed balance
    let unconfirmed: UInt64        // Unconfirmed balance
}

struct SwapReport {
    let amountSwapped: UInt64      // Amount successfully swapped
    let routingFeesPaid: UInt64    // Total routing fees
    let numSuccessfulSwaps: UInt32 // Number of successful hops
    let totalSwapTime: UInt64      // Time taken in seconds
}

enum AddressType {
    case p2wpkh  // Native SegWit (bech32)
    case p2tr    // Taproot (bech32m)
}
```

## iOS Example

```swift
import UIKit
import Combine

class WalletViewController: UIViewController {
    private var taker: Taker?
    private var cancellables = Set<AnyCancellable>()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // Initialize in background
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let documentsPath = FileManager.default.urls(
                    for: .documentDirectory, 
                    in: .userDomainMask
                )[0].path
                
                self?.taker = try Taker(
                    dataDir: documentsPath,
                    walletFileName: "wallet",
                    rpcConfig: self?.getRpcConfig(),
                    controlPort: 9051,
                    torAuthPassword: nil,
                    zmqAddr: "tcp://localhost:28332",
                    password: self?.getUserPassword()
                )
                
                try self?.taker?.setupLogging(dataDir: documentsPath)
                try self?.taker?.syncAndSave()
                
                DispatchQueue.main.async {
                    self?.updateUI()
                }
            } catch {
                print("Error initializing: \(error)")
            }
        }
    }
    
    func performSwap(amount: UInt64) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let params = SwapParams(
                    sendAmount: amount,
                    makerCount: 2,
                    manuallySelectedOutpoints: nil
                )
                
                let report = try self?.taker?.doCoinswap(swapParams: params)
                
                DispatchQueue.main.async {
                    self?.showSwapResult(report)
                }
            } catch {
                print("Swap failed: \(error)")
            }
        }
    }
    
    private func getRpcConfig() -> RPCConfig {
        RPCConfig(
            url: "http://localhost:18443",
            user: "bitcoin",
            password: "bitcoin",
            walletName: "taker_wallet"
        )
    }
}
```

## SwiftUI Example

```swift
import SwiftUI

class WalletViewModel: ObservableObject {
    @Published var balance: UInt64 = 0
    @Published var isLoading = false
    @Published var errorMessage: String?
    
    private var taker: Taker?
    
    func initialize() async {
        isLoading = true
        do {
            let documentsPath = FileManager.default.urls(
                for: .documentDirectory,
                in: .userDomainMask
            )[0].path
            
            taker = try Taker(
                dataDir: documentsPath,
                walletFileName: "wallet",
                rpcConfig: getRpcConfig(),
                controlPort: 9051,
                torAuthPassword: nil,
                zmqAddr: "tcp://localhost:28332",
                password: getUserPassword()
            )
            
            try taker?.setupLogging(dataDir: documentsPath)
            try taker?.syncAndSave()
            
            let balances = try taker?.getBalances()
            await MainActor.run {
                self.balance = balances?.total ?? 0
                self.isLoading = false
            }
        } catch {
            await MainActor.run {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
    
    func performSwap(amount: UInt64) async {
        isLoading = true
        do {
            let params = SwapParams(
                sendAmount: amount,
                makerCount: 2,
                manuallySelectedOutpoints: nil
            )
            
            let report = try taker?.doCoinswap(swapParams: params)
            await MainActor.run {
                self.isLoading = false
                // Handle report
            }
        } catch {
            await MainActor.run {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
}

struct WalletView: View {
    @StateObject private var viewModel = WalletViewModel()
    
    var body: some View {
        VStack {
            if viewModel.isLoading {
                ProgressView()
            } else {
                Text("Balance: \(viewModel.balance) sats")
                Button("Perform Swap") {
                    Task {
                        await viewModel.performSwap(amount: 1_000_000)
                    }
                }
            }
        }
        .task {
            await viewModel.initialize()
        }
    }
}
```

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
