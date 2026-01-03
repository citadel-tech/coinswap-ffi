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
- `uniffi/coinswap/coinswap.kt` - Kotlin binding classes
- `libcoinswap_ffi.so` - Native library (Linux)
- `libcoinswap_ffi.dylib` - Native library (macOS)

### Installation

#### Android

1. Copy the generated files to your Android project:
```bash
# Copy Kotlin bindings
cp uniffi/coinswap/coinswap.kt app/src/main/java/coinswap/

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
import coinswap.*

// Initialize a Taker
val taker = Taker.init(
    dataDir = "/path/to/data",
    walletFileName = "taker_wallet",
    rpcConfig = RPCConfig(
        url = "http://localhost:18443",
        user = "bitcoin",
        password = "bitcoin",
        walletName = "taker_wallet"
    ),
    controlPort = 9051,
    torAuthPassword = null,
    zmqAddr = "tcp://localhost:28332",
    password = "your_secure_password"
)

// Setup logging
taker.setupLogging(dataDir = "/path/to/data")

// Sync wallet
taker.syncAndSave()

// Get balances
val balances = taker.getBalances()
println("Total Balance: ${balances.total} sats")

// Get a new receiving address
val address = taker.getNextExternalAddress(AddressType.P2WPKH)
println("Receive to: ${address.value}")

// Perform a coinswap
val swapParams = SwapParams(
    sendAmount = 1000000uL, // 0.01 BTC in sats
    makerCount = 2u,
    manuallySelectedOutpoints = null
)

val report = taker.doCoinswap(swapParams)
report?.let {
    println("Swap completed!")
    println("Amount swapped: ${it.amountSwapped} sats")
    println("Routing fee paid: ${it.routingFeesPaid} sats")
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
    val total: ULong,              // Total balance in sats
    val confirmed: ULong,          // Confirmed balance
    val unconfirmed: ULong         // Unconfirmed balance
)

data class SwapReport(
    val amountSwapped: ULong,      // Amount successfully swapped
    val routingFeesPaid: ULong,    // Total routing fees
    val numSuccessfulSwaps: UInt,  // Number of successful hops
    val totalSwapTime: ULong       // Time taken in seconds
)

enum class AddressType {
    P2WPKH,  // Native SegWit (bech32)
    P2TR     // Taproot (bech32m)
}
```

## Android Example

```kotlin
class WalletActivity : AppCompatActivity() {
    private lateinit var taker: Taker
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize in background thread
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                taker = Taker.init(
                    dataDir = filesDir.absolutePath,
                    walletFileName = "wallet",
                    rpcConfig = getRpcConfig(),
                    controlPort = 9051u,
                    torAuthPassword = null,
                    zmqAddr = "tcp://localhost:28332",
                    password = getUserPassword()
                )
                
                taker.setupLogging(filesDir.absolutePath)
                taker.syncAndSave()
                
                withContext(Dispatchers.Main) {
                    updateUI()
                }
            } catch (e: TakerError) {
                Log.e("Wallet", "Error: ${e.message}")
            }
        }
    }
    
    private fun performSwap(amount: Long) {
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val params = SwapParams(
                    sendAmount = amount.toULong(),
                    makerCount = 2u,
                    manuallySelectedOutpoints = null
                )
                
                val report = taker.doCoinswap(params)
                withContext(Dispatchers.Main) {
                    showSwapResult(report)
                }
            } catch (e: TakerError) {
                Log.e("Swap", "Swap failed: ${e.message}")
            }
        }
    }
}
```

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
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Build for all Android architectures
cd ../ffi-commons
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android
```

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
