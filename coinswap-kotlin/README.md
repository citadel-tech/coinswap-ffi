<div align="center">

# Coinswap Kotlin

**Kotlin bindings for the Coinswap Bitcoin privacy protocol**

</div>

## Overview

Kotlin bindings for [Coinswap](https://github.com/citadel-tech/coinswap), enabling native Android and JVM integration with the Bitcoin coinswap privacy protocol. Built using [UniFFI](https://mozilla.github.io/uniffi-rs/).

## Quick Start

### Prerequisites

- Kotlin 1.8+
- JDK 11+ (for JVM) or Android SDK 24+ (for Android)
- Generated bindings (see [Building](#building))

### Building

Generate the Kotlin bindings from the UniFFI core:

```bash
cd ../ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This generates:
- `lib/src/main/kotlin/org/coinswap/coinswap.kt` - Kotlin binding classes
- `libcoinswap_ffi.so` - Native library (Linux)
- `libcoinswap_ffi.dylib` - Native library (macOS)

### Installation

#### Android

1. Copy the generated files to your Android project:
```bash
# Copy Kotlin bindings
cp lib/src/main/kotlin/org/coinswap/coinswap.kt app/src/main/java/org/coinswap/

# Copy native libraries for each architecture
cp target/aarch64-linux-android/release/libcoinswap_ffi.so app/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libcoinswap_ffi.so app/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/libcoinswap_ffi.so app/src/main/jniLibs/x86_64/
```

2. Add to your `build.gradle.kts`:
```kotlin
android {
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}
```

#### JVM/Desktop

Add the library to your project's library path:

```kotlin
System.setProperty("java.library.path", "/path/to/coinswap-kotlin")
```

### Basic Usage

```kotlin
import org.coinswap.*

// Initialize a Taker
val taker = Taker.init(
    dataDir = "/path/to/data",
    walletFileName = "taker_wallet",
    rpcConfig = RpcConfig(
        // regtest
        url = "localhost:18442",
        username = "user",
        password = "password",
        walletName = "taker_wallet"
    ),
    controlPort = 9051u,
    torAuthPassword = "your_tor_password",
    zmqAddr = "tcp://localhost:28332",
    // backup and restore
    password = "your_secure_password"
)

// Setup logging
taker.setupLogging(dataDir = "/path/to/data", logLevel = "info")

// Sync wallet
taker.syncAndSave()

// Get balances
val balances = taker.getBalances()
println("Total Balance: ${balances.spendable} sats")

// Get a new receiving address
val address = taker.getNextExternalAddress(AddressType("P2WPKH"))
println("Receive to: ${address.address}")

// Wait for offerbook to sync
println("Waiting for offerbook synchronization...")
while (taker.isOfferbookSyncing()) {
    println("Offerbook sync in progress...")
    Thread.sleep(2000) // Wait 2 seconds before checking again
}
println("Offerbook synchronized!")

// Perform a coinswap
val swapParams = SwapParams(
    sendAmount = 1000000u, // 0.01 BTC in sats
    makerCount = 2u,
    manuallySelectedOutpoints = null
)

val report = taker.doCoinswap(swapParams)
report?.let {
    println("Swap completed!")
    println("Swap ID: ${it.swapId}")
    println("Target Amount: ${it.targetAmount} sats")
    println("Total Fee: ${it.totalFee} sats")
}
```

## API Reference

### Taker Class

Initialize and manage a coinswap taker:

```kotlin
// Initialize
val taker = Taker.init(
    dataDir: String?,
    walletFileName: String?,
    rpcConfig: RPCConfig?,
    controlPort: UShort?,
    torAuthPassword: String?,
    zmqAddr: String,
    password: String?
)

// Wallet operations
taker.getBalances(): Balances
taker.getNextExternalAddress(addressType: AddressType): Address
taker.getTransactions(count: UInt?, skip: UInt?): List<ListTransactionResult>
taker.listAllUtxoSpendInfo(): List<TotalUtxoInfo>
taker.sendToAddress(address: String, amount: Long, feeRate: Double?, 
                    manuallySelectedOutpoints: List<OutPoint>?): Txid

// Swap operations
taker.doCoinswap(swapParams: SwapParams): SwapReport?
taker.fetchOffers(): OfferBook
taker.isOfferbookSyncing(): Boolean

// Maintenance
taker.syncAndSave()
taker.backup(destinationPath: String, password: String?)
taker.recoverFromSwap()
```

### Data Types

```kotlin
data class SwapParams(
    val sendAmount: ULong,        // Amount to swap in satoshis
    val makerCount: UInt,          // Number of makers (hops)
    val manuallySelectedOutpoints: List<OutPoint>?
)

data class Balances(
    val regular: Long,             // Regular wallet balance in sats
    val swap: Long,                // Swap balance in sats
    val contract: Long,            // Contract balance in sats
    val spendable: Long            // Spendable balance in sats
)

data class SwapReport(
    val swapId: String,            // Unique swap identifier
    val swapDurationSeconds: Double, // Duration of swap in seconds
    val targetAmount: Long,        // Target swap amount in sats
    val totalInputAmount: Long,    // Total input amount in sats
    val totalOutputAmount: Long,   // Total output amount in sats
    val makersCount: UInt,         // Number of makers in swap
    val makerAddresses: List<String>, // List of maker addresses
    val totalFundingTxs: Long,     // Total number of funding transactions
    val fundingTxidsByHop: List<List<String>>, // Funding TXIDs grouped by hop
    val totalFee: Long,            // Total fees paid in sats
    val totalMakerFees: Long,      // Total maker fees in sats
    val miningFee: Long,           // Mining fees in sats
    val feePercentage: Double,     // Fee as percentage of amount
    val makerFeeInfo: List<MakerFeeInfo>, // Detailed fee info per maker
    val inputUtxos: List<Long>,    // Input UTXO amounts
    val outputChangeAmounts: List<Long>,    // Change output amounts
    val outputSwapAmounts: List<Long>,      // Swap output amounts
    val outputChangeUtxos: List<UtxoWithAddress>, // Change UTXOs with addresses
    val outputSwapUtxos: List<UtxoWithAddress>    // Swap UTXOs with addresses
)

enum class AddressType {
    P2WPKH,  // Native SegWit (bech32)
    P2TR     // Taproot (bech32m)
}
```

## Examples

Complete examples are available in the [`test/`](test/) directory:
- [`AndroidExample.kt`](test/AndroidExample.kt) - Android integration with coroutines

## Testing

The project includes comprehensive test suites in Kotlin:

### Running Tests

```bash
# Run all tests
./gradlew test

# Run Taproot/Legacy swap tests
./gradlew :lib:test --tests "coinswap.StandardSwap"
./gradlew :lib:test --tests "coinswap.TaprootSwap"
```

See the [CI workflow](.github/workflows/test-kotlin.yml) for complete test setup.

## Requirements

### Android
- Minimum SDK: 24 (Android 7.0)
- Target SDK: 34+
- Permissions: `INTERNET`, `ACCESS_NETWORK_STATE`

### JVM
- JDK 11 or higher
- Native library in `java.library.path`

### Bitcoin Setup
- Bitcoin Core with RPC enabled
- Synced, non-pruned node with `-txindex`
- Tor daemon running for privacy

## Error Handling

All operations that can fail throw `TakerError`:

```kotlin
try {
    val balances = taker.getBalances()
    println("Balance: ${balances.total}")
} catch (e: TakerError.General) {
    println("General error: ${e.msg}")
} catch (e: TakerError.Wallet) {
    println("Wallet error: ${e.msg}")
} catch (e: TakerError.Network) {
    println("Network error: ${e.msg}")
}
```

## Cross-Compilation

Build for Android architectures:

```bash
# Add Android targets
rustup target add aarch64-linux-android x86_64-linux-android

# Build for all Android architectures
cd ../ffi-commons
cargo build --release --target aarch64-linux-android
cargo build --release --target x86_64-linux-android
```

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
