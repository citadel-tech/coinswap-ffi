# coinswap-ffi — agent orientation

This repo is the FFI layer for the Coinswap Bitcoin UTXO privacy-swap protocol and the integration map for agents building apps on top of these bindings. The shared Rust Taker client initializes and persists a taker wallet, syncs wallet state, discovers maker offers through Tor, prepares a swap route, executes the swap, and exposes recovery. Confirmed binding packages are `coinswap-js` through NAPI-RS, `coinswap-react-native` through a generated React Native Turbo Module / JSI layer from `uniffi-bindgen-react-native`, and `coinswap-kotlin`, `coinswap-swift`, `coinswap-python`, and `coinswap-ruby` through UniFFI. Beta software; mainnet deployment is not recommended.

## repository map

| Package | Mechanism | Published name | Min runtime | Build entry point |
| --- | --- | --- | --- | --- |
| `ffi-commons/` | Rust `cdylib` / `staticlib` plus UniFFI scaffolding | `coinswap-ffi` | Rust 1.75.0 or newer | `cargo build --package coinswap-ffi --profile release-smaller` |
| `coinswap-js/` | NAPI-RS Node addon | `coinswap-napi` in `package.json`; README install uses `coinswap-js` | Node.js `>= 18.0.0`; Rust 1.75.0 or newer | `yarn build` |
| `coinswap-react-native/` | `uniffi-bindgen-react-native` Turbo Module / JSI | `coinswap-react-native` | React Native `>=0.76`; React `>=18`; Android min SDK 24; iOS `min_ios_version_supported` | `npm run ubrn:android`; `npm run ubrn:ios` |
| `coinswap-kotlin/` | UniFFI Kotlin | Maven `org.coinswap:coinswap-kotlin:1.0.0` | Kotlin 1.8+; JDK 11+; Android SDK 24+ | `./gradlew :lib:assembleRelease` |
| `coinswap-swift/` | UniFFI Swift Package plus `coinswap_ffi.xcframework` | Swift package product `Coinswap` | Swift 5.7+; iOS 13+; macOS 10.15+ | `bash ./build-xcframework.sh`; `swift build` |
| `coinswap-python/` | UniFFI Python | `coinswap` | Python `>=3.8`; CPython or PyPy | `python -m build` |
| `coinswap-ruby/` | UniFFI Ruby plus generated `coinswap.rb` | `coinswap` require path; no gemspec found | Ruby 2.7 or newer; `ffi` gem | `bash ./build-scripts/release/build-release-linux-x86_64.sh` |

## shared runtime requirements

### bitcoin core

Regtest from `ffi-commons/ffi-docker-setup`:

```bash
bitcoind -datadir=/home/bitcoin/.bitcoin -server=1 -rest=1 -rpcuser=user -rpcpassword=password -rpcallowip=0.0.0.0/0 -txindex=1 -blockfilterindex=1 -regtest=1 -fallbackfee=0.00001000 -rpcbind=0.0.0.0 -rpcport=18442 -zmqpubrawblock=tcp://0.0.0.0:28332 -zmqpubrawtx=tcp://0.0.0.0:28332
```

`rpcuser=user`, `rpcpassword=password`, and `tor_auth_password = coinswap` are local Docker defaults from this repo. Replace them for any non-local environment.

Mutinynet / custom Signet from `ffi-commons/signet-docker-script`; treat the script values as environment-specific examples, not constants to paste into new projects:

```bash
bitcoind -datadir=/home/bitcoin/.bitcoin -server=1 -rest=1 -rpcuser=user -rpcpassword=password -rpcallowip=0.0.0.0/0 -txindex=1 -blockfilterindex=1 -signet=1 -signetchallenge=<custom-signet-challenge-from-script-or-your-network> -addnode=<custom-signet-peer-from-script-or-your-network> -dnsseed=<custom-dnsseed-setting> -signetblocktime=<custom-block-time> -rpcbind=0.0.0.0 -rpcport=38332 -zmqpubrawblock=tcp://127.0.0.1:28332 -zmqpubrawtx=tcp://127.0.0.1:28332
```

⚠️ Plain `signet=1` without the matching custom `signetchallenge`, `addnode`, `dnsseed`, and `signetblocktime` for the target network does not connect to the project's Mutinynet environment. Read `ffi-commons/signet-docker-script` for the currently used example values.

### tor

```torrc
SocksPort 0.0.0.0:9050
ControlPort 0.0.0.0:9051
HashedControlPassword 16:16E946B27E1A526E60A2B64061EC3F140ACA991F1B705ACD35374D6730
DataDirectory /var/lib/tor
```

Maker config uses:

```toml
socks_port = 9050
control_port = 9051
tor_auth_password = coinswap
connection_type = TOR
```

### rust toolchain

`ffi-commons/README.md` states Rust 1.75.0 or newer. `ffi-commons/Cargo.toml` uses `edition = "2024"`, `uniffi = "0.30.0"`, `bitcoin = "0.32.7"`, `bitcoind = "0.36.1"`, and `panic = "abort"` in `profile.release-smaller`.

## binding quick-reference

Canonical app flow: `setupLogging` / `setup_logging` / `setupLogging` → `Taker` constructor or `init` → `syncAndSave` / `sync_and_save` → `syncOfferbookAndWait` / `sync_offerbook_and_wait` → `fetchOffers` / `fetch_offers` → filter usable makers → `prepareCoinswap` / `prepare_coinswap` → `startCoinswap` / `start_coinswap` → final wallet sync.

### JavaScript (coinswap-js)

Package-name mismatch: `coinswap-js/package.json` says `"name": "coinswap-napi"`, while `coinswap-js/README.md` installs and imports `coinswap-js`. Verify the registry package before publishing or adding dependency automation.

**1. Install / build**

```bash
npm install coinswap-js
yarn add coinswap-js
pnpm add coinswap-js
cd coinswap-js
yarn install
yarn build
```

**2. Canonical import**

```typescript
import { AddressType, Taker, type RpcConfig, type SwapParams } from 'coinswap-js'
```

**3. Constructor / initialiser**

```typescript
new Taker(dataDir: string | undefined | null, walletFileName: string | undefined | null, rpcConfig: RpcConfig | undefined | null, controlPort: number | undefined | null, torAuthPassword: string | undefined | null, zmqAddr: string, password?: string | undefined | null)
```

**4. Minimal working call sequence**

1. `Taker.setupLogging(dataDir, "info")` configures taker logging.
2. `const taker = new Taker(dataDir, walletFileName, rpcConfig, controlPort, torAuthPassword, zmqAddr, password)` creates or loads the wallet.
3. `taker.syncAndSave()` syncs wallet state and persists it.
4. `taker.syncOfferbookAndWait()` blocks until the offer book is synchronized.
5. `const offerBook = taker.fetchOffers()` reads all makers.
6. `const swapId = taker.prepareCoinswap(swapParams)` prepares a route and returns a swap id.
7. `const report = taker.startCoinswap(swapId)` executes the prepared swap.
8. `taker.syncAndSave()` persists post-swap wallet state.

**5. Threading / concurrency constraint**

All Taker methods block; run inside a `worker_threads.Worker`.

### React Native (coinswap-react-native)

<!-- `coinswap-react-native/src/generated/`, Android `.kt` / `.java` bridge files, and iOS `.swift` / `.m` / `.mm` bridge files are absent in this checkout; run `npm run ubrn:generate`, `npm run ubrn:android`, or `npm run ubrn:ios` to generate them. -->

**1. Install / build**

```bash
npm install coinswap-react-native
yarn add coinswap-react-native
cd coinswap-react-native
npm install
npm run ubrn:android
npm run ubrn:ios
cd ios
pod install
```

**2. Canonical import**

```typescript
import { AddressType, CoinswapTaker, isNativeCoinswapAvailable } from 'coinswap-react-native'

if (!isNativeCoinswapAvailable()) {
  throw new Error('Coinswap TurboModule is unavailable in this runtime')
}
```

**3. Constructor / initialiser**

```typescript
CoinswapTaker.init(config: {
  dataDir?: string | null
  walletFileName?: string | null
  rpcConfig?: RpcConfig | null
  controlPort?: number | null
  torAuthPassword?: string | null
  zmqAddr: string
  password?: string | null
}): Promise<CoinswapTaker>
```

**4. Minimal working call sequence**

1. `await CoinswapTaker.setupLogging(dataDir, "info")` calls the generated native logger method through a Promise.
2. `const taker = await CoinswapTaker.init(config)` calls generated `Taker.init(dataDir, walletFileName, rpcConfig, controlPort, torAuthPassword, zmqAddr, password)` through a Promise.
3. `await taker.syncAndSave()` syncs wallet state through a Promise.
4. `await taker.syncOfferbookAndWait()` waits for maker discovery through a Promise.
5. `const swapId = await taker.prepareCoinswap(swapParams)` prepares the route through a Promise.
6. `const report = await taker.startCoinswap(swapId)` executes the prepared swap through a Promise.
7. `await taker.syncAndSave()` persists post-swap wallet state through a Promise.
8. `await taker.dispose()` releases the native object through a Promise.

**5. Threading / concurrency constraint**

All calls enter the RN Turbo Module / JSI native layer; the checked-in JS wrapper exposes Promises.

### Kotlin (coinswap-kotlin)

<!-- `coinswap-kotlin/lib/src/main/kotlin/` contains no checked-in generated Kotlin files in this checkout; class and method names below are verified from `coinswap-kotlin/README.md`, tests, and the Rust UniFFI exports. -->

**1. Install / build**

```bash
bash ./build-scripts/development/build-dev-linux-jvm.sh
bash ./build-scripts/development/build-dev-linux-x86_64.sh
bash ./build-scripts/release/build-release-linux-arm64_v8a.sh
bash ./build-scripts/release/build-release-linux-armeabi-v7a.sh
./gradlew :lib:assembleRelease
./gradlew :lib:publishToMavenLocal -PlocalBuild=true
```

**2. Canonical import**

```kotlin
import org.coinswap.*
```

**3. Constructor / initialiser**

```kotlin
Taker.init(dataDir: String?, walletFileName: String?, rpcConfig: RpcConfig?, controlPort: UShort?, torAuthPassword: String?, zmqAddr: String, password: String?): Taker
```

**4. Minimal working call sequence**

1. `val taker = Taker.init(dataDir, walletFileName, rpcConfig, controlPort, torAuthPassword, zmqAddr, password)` creates or loads the wallet.
2. `taker.setupLogging(dataDir = dataDir, logLevel = "info")` configures logging.
3. `taker.syncAndSave()` syncs wallet state.
4. `taker.syncOfferbookAndWait()` waits for maker discovery.
5. `val offerBook = taker.fetchOffers()` reads makers.
6. `val swapId = taker.prepareCoinswap(swapParams)` prepares the route.
7. `val report = taker.startCoinswap(swapId)` executes the prepared swap.
8. `taker.syncAndSave()` persists post-swap wallet state.

**5. Threading / concurrency constraint**

Methods are blocking UniFFI calls; use a coroutine dispatcher or background thread for wallet sync, offer discovery, and swaps.

### Swift (coinswap-swift)

<!-- `coinswap-swift/Sources/` is absent in this checkout; generated Swift files are not checked in. Class and method names below are verified from `coinswap-swift/README.md`, tests, and the Rust UniFFI exports. -->

**1. Install / build**

```bash
bash ./build-xcframework-dev.sh
bash ./build-xcframework-ci.sh
bash ./build-xcframework.sh
swift build
```

**2. Canonical import**

```swift
import Coinswap
```

**3. Constructor / initialiser**

```swift
try Taker.`init`(dataDir: String?, walletFileName: String?, rpcConfig: RpcConfig?, controlPort: UInt16?, torAuthPassword: String?, zmqAddr: String, password: String?)
```

**4. Minimal working call sequence**

1. `let taker = try Taker.\`init\`(dataDir: dataDir, walletFileName: walletFileName, rpcConfig: rpcConfig, controlPort: controlPort, torAuthPassword: torAuthPassword, zmqAddr: zmqAddr, password: password)` creates or loads the wallet.
2. `try taker.setupLogging(dataDir: dataDir, logLevel: "info")` configures logging.
3. `try taker.syncAndSave()` syncs wallet state.
4. `try taker.syncOfferbookAndWait()` waits for maker discovery.
5. `let offerBook = try taker.fetchOffers()` reads makers.
6. `let swapId = try taker.prepareCoinswap(swapParams: swapParams)` prepares the route.
7. `let report = try taker.startCoinswap(swapId: swapId)` executes the prepared swap.
8. `try taker.syncAndSave()` persists post-swap wallet state.

**5. Threading / concurrency constraint**

Can be called from any thread; internal dispatch is managed by the UniFFI runtime [unverified — check generated Swift source].

### Python (coinswap-python)

<!-- `coinswap-python/src/coinswap/coinswap.py` is absent in this checkout; generated Python API files are not checked in. Class and method names below are verified from `coinswap-python/README.md`, tests, and the Rust UniFFI exports. -->

**1. Install / build**

```bash
bash ./build-scripts/development/build-dev-linux-x86_64.sh
bash ./build-scripts/development/build-dev-macos-x86_64.sh
bash ./build-scripts/release/build-release-linux-x86_64.sh
bash ./build-scripts/release/build-release-linux-aarch64.sh
bash ./build-scripts/release/build-release-macos-x86_64.sh
bash ./build-scripts/release/build-release-macos-aarch64.sh
python -m pip install --upgrade build
python -m build
```

**2. Canonical import**

```python
from coinswap import AddressType, RpcConfig, SwapParams, Taker
```

**3. Constructor / initialiser**

```python
Taker.init(data_dir: str | None, wallet_file_name: str | None, rpc_config: RpcConfig | None, control_port: int | None, tor_auth_password: str | None, zmq_addr: str, password: str | None) -> Taker
```

**4. Minimal working call sequence**

1. `taker = Taker.init(data_dir, wallet_file_name, rpc_config, control_port, tor_auth_password, zmq_addr, password)` creates or loads the wallet.
2. `taker.setup_logging(data_dir=data_dir, log_level="Info")` configures logging.
3. `taker.sync_and_save()` syncs wallet state.
4. `taker.sync_offerbook_and_wait()` waits for maker discovery.
5. `offerbook = taker.fetch_offers()` reads makers.
6. `swap_id = taker.prepare_coinswap(swap_params=swap_params)` prepares the route.
7. `report = taker.start_coinswap(swap_id=swap_id)` executes the prepared swap.
8. `taker.sync_and_save()` persists post-swap wallet state.

**5. Threading / concurrency constraint**

GIL must be held; run blocking calls in a thread pool executor.

### Ruby (coinswap-ruby)

<!-- `coinswap-ruby/lib/coinswap.rb` and `coinswap-ruby/coinswap.rb` are absent in this checkout; generated Ruby API files are not checked in. The checked-in `test/standard_swap.rb` still calls `do_coinswap`, but Rust exports and README use `prepare_coinswap` plus `start_coinswap`. -->

**1. Install / build**

```bash
bash ./build-scripts/development/build-dev-linux-x86_64.sh
bash ./build-scripts/release/build-release-linux-x86_64.sh
bash ./build-scripts/release/build-release-linux-aarch64.sh
bash ./build-scripts/development/build-dev-macos-x86_64.sh
bash ./build-scripts/release/build-release-macos-x86_64.sh
bash ./build-scripts/release/build-release-macos-aarch64.sh
```

**2. Canonical import**

```ruby
$LOAD_PATH.unshift('/path/to/coinswap-ffi/coinswap-ruby')
require 'coinswap'
```

**3. Constructor / initialiser**

```ruby
Coinswap::Taker.init(data_dir, wallet_file_name, rpc_config, control_port, tor_auth_password, zmq_addr, password)
```

**4. Minimal working call sequence**

1. `taker = Coinswap::Taker.init(data_dir, wallet_file_name, rpc_config, control_port, tor_auth_password, zmq_addr, password)` creates or loads the wallet.
2. `taker.setup_logging(data_dir, "info")` configures logging.
3. `taker.sync_and_save` syncs wallet state.
4. `taker.sync_offerbook_and_wait` waits for maker discovery.
5. `offerbook = taker.fetch_offers` reads makers.
6. `swap_id = taker.prepare_coinswap(swap_params)` prepares the route.
7. `report = taker.start_coinswap(swap_id)` executes the prepared swap.
8. `taker.sync_and_save` persists post-swap wallet state.

**5. Threading / concurrency constraint**

Methods are blocking FFI calls; run wallet sync, offer discovery, and swaps outside the UI/request thread.

## react native specifics

1. **Module type**: Turbo Module / JSI. `coinswap-react-native/package.json` declares `codegenConfig`, `coinswap-react-native/README.md` calls it a JSI TurboModule, `coinswap-react-native/android/build.gradle` applies `com.facebook.react` only when `newArchEnabled=true`, and `coinswap-react-native/CoinswapReactNative.podspec` depends on `ReactCommon/turbomodule/core` when `RCT_NEW_ARCH_ENABLED=1`. Turbo Module / JSI allows synchronous native calls in generated code; the checked-in wrapper exposes async Promise methods.

Generated bridge paths are absent until build. Edit `coinswap-react-native/src/index.ts`, `coinswap-react-native/ubrn.config.yaml`, build scripts, plugin files, or `ffi-commons`; do not patch generated bridge files before they exist.

2. **Native dependency chain**: It compiles its own Rust from `../ffi-commons`; it does not depend on the Kotlin `.aar` or Swift package. Exact source:

```yaml
rust:
  directory: ../ffi-commons
  manifestPath: Cargo.toml
```

```ruby
s.vendored_frameworks = "CoinswapReactNativeFramework.xcframework"
s.dependency    "uniffi-bindgen-react-native", "0.31.0-2"
```

3. **Android setup**: Exact Gradle dependency and ProGuard/R8 rule:

```gradle
implementation "com.facebook.react:react-native:+"
```

```proguard
-keep class org.coinswap.** { *; }
```

4. **iOS setup**: Package consumer runs:

```bash
cd ios
pod install
```

Pod dependency comes from:

```ruby
s.dependency    "uniffi-bindgen-react-native", "0.31.0-2"
```

5. **Initialisation**:

```typescript
const taker = await CoinswapTaker.init({
  dataDir: null,
  walletFileName: 'my_wallet',
  rpcConfig: {
    url: 'localhost:18442',
    username: 'user',
    password: 'password',
    walletName: 'my_wallet',
  },
  controlPort: 9051,
  torAuthPassword: 'coinswap',
  zmqAddr: 'tcp://127.0.0.1:28332',
  password: '',
})
```

6. **Event / callback model**: No event emitters, callback registrations, or event name strings exist in checked-in RN source. API is Promise-returning methods only.

7. **Metro bundler gotcha**: No `metro.config.js` exists and no Metro resolver changes appear in `coinswap-react-native/README.md`, `package.json`, `react-native.config.js`, or plugin files.

## type mapping

Generated Kotlin, Swift, Python, Ruby, and RN type files are absent in this checkout. Names marked `[unverified — check source]` come from Rust UniFFI exports plus package READMEs/tests and must be checked against generated output after a binding build.

| Rust type | JS/Node | React Native (JS side) | Kotlin | Swift | Python | Ruby | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Taker` | `Taker` | `CoinswapTaker` wrapper over generated `Taker` | `Taker` | `Taker` | `Taker` | `Coinswap::Taker` | RN generated `Taker` source absent. |
| `RPCConfig` | `RpcConfig` | `RpcConfig` | `RpcConfig` | `RpcConfig` | `RpcConfig` | `Coinswap::RpcConfig` | JS uses camel-case `walletName`; UniFFI READMEs use language casing. |
| `SwapParams` | `SwapParams` | `SwapParams` | `SwapParams` | `SwapParams` | `SwapParams` | `Coinswap::SwapParams` | Rust `send_amount: u64`; JS NAPI `sendAmount: number`; RN README says `bigint` but tests use `number`. |
| `TakerError` | `TakerError` enum plus thrown `Error` | rejected Promise shape [unverified — check source] | `TakerError` [unverified — check source] | `TakerError` [unverified — check source] | `TakerError` [unverified — check source] | `Coinswap::TakerError` [unverified — check source] | Variants: `Wallet`, `Protocol`, `Network`, `General`, `IO`. |
| `TakerBehavior` | `TakerBehavior` | not exported by wrapper | `TakerBehavior` [unverified — check source] | `TakerBehavior` [unverified — check source] | `TakerBehavior` [unverified — check source] | `Coinswap::TakerBehavior` [unverified — check source] | Not wired into `Taker.init`. |
| `Balances` | `Balances` | `Balances` | `Balances` | `Balances` | `Balances` | `Balances` | Fields: `regular`, `swap`, `contract`, `fidelity`, `spendable`. |
| `OutPoint` | `OutPoint` | `OutPoint` [unverified — check source] | `OutPoint` [unverified — check source] | `OutPoint` | `OutPoint` [unverified — check source] | `OutPoint` [unverified — check source] | JS `txid: string`; UniFFI Rust `txid: Txid`. |
| `Address` | `Address` | `Address` | `Address` | `Address` | `Address` | `Address` | Field: `address`. |
| `ListTransactionResult` | `ListTransactionResult` | `ListTransactionResult` [unverified — check source] | `ListTransactionResult` [unverified — check source] | `ListTransactionResult` | `ListTransactionResult` | `ListTransactionResult` | Contains `WalletTxInfo` and `GetTransactionResultDetail`. |
| `WalletTxInfo` | `WalletTxInfo` | `WalletTxInfo` [unverified — check source] | `WalletTxInfo` [unverified — check source] | `WalletTxInfo` | `WalletTxInfo` | `WalletTxInfo` | JS casing: `bip125Replaceable`, `walletConflicts`. |
| `GetTransactionResultDetail` | `GetTransactionResultDetail` | `GetTransactionResultDetail` [unverified — check source] | `GetTransactionResultDetail` [unverified — check source] | `GetTransactionResultDetail` | `GetTransactionResultDetail` | `GetTransactionResultDetail` | Amount fields use `SignedAmountSats`. |
| `Amount` | `Amount` | `Amount` [unverified — check source] | `Amount` [unverified — check source] | `Amount` | `Amount` | `Amount` | Field: `sats`. |
| `Txid` | `Txid` | `Txid` [unverified — check source] | `Txid` [unverified — check source] | `Txid` | `Txid` | `Txid` | Field: `value`. |
| `ScriptBuf` | `ScriptBuf` | `ScriptBuf` [unverified — check source] | `ScriptBuf` [unverified — check source] | `ScriptBuf` | `ScriptBuf` | `ScriptBuf` | Field: `hex`. |
| `SignedAmountSats` | `SignedAmountSats` | `SignedAmountSats` [unverified — check source] | `SignedAmountSats` [unverified — check source] | `SignedAmountSats` | `SignedAmountSats` | `SignedAmountSats` | Field: `sats`. |
| `ListUnspentResultEntry` | `ListUnspentResultEntry` | `ListUnspentResultEntry` [unverified — check source] | `ListUnspentResultEntry` [unverified — check source] | `ListUnspentResultEntry` | `ListUnspentResultEntry` | `ListUnspentResultEntry` | Wallet UTXO record. |
| `UtxoSpendInfo` | `UtxoSpendInfo` | `UtxoSpendInfo` [unverified — check source] | `UtxoSpendInfo` [unverified — check source] | `UtxoSpendInfo` | `UtxoSpendInfo` | `UtxoSpendInfo` | `spend_type` values include `SeedCoin`, `IncomingSwapCoin`, `OutgoingSwapCoin`, `TimelockContract`, `HashlockContract`, `FidelityBondCoin`, `SweptCoin`. |
| `TotalUtxoInfo` | returned as tuple `[ListUnspentResultEntry, UtxoSpendInfo]` | `TotalUtxoInfo` [unverified — check source] | `TotalUtxoInfo` [unverified — check source] | `TotalUtxoInfo` | `TotalUtxoInfo` | `TotalUtxoInfo` | Rust UniFFI has record; JS NAPI returns tuple. |
| `FeeRates` | `FeeRates` | `FeeRates` [unverified — check source] | `FeeRates` [unverified — check source] | `FeeRates` | `FeeRates` | `FeeRates` | Fields: `fastest`, `standard`, `economy`. |
| `LockTime` | `LockTime` | `LockTime` [unverified — check source] | `LockTime` [unverified — check source] | `LockTime` | `LockTime` | `LockTime` | `lock_type` / `lockType`, `value`. |
| `PublicKey` | `PublicKey` | `PublicKey` [unverified — check source] | `PublicKey` [unverified — check source] | `PublicKey` | `PublicKey` | `PublicKey` | Fields: `compressed`, `inner`. |
| `FidelityProof` | `FidelityProof` | `FidelityProof` [unverified — check source] | `FidelityProof` [unverified — check source] | `FidelityProof` | `FidelityProof` | `FidelityProof` | Fields: `bond`, `cert_hash`, `cert_sig`. |
| `FidelityBond` | `FidelityBond` | `FidelityBond` [unverified — check source] | `FidelityBond` [unverified — check source] | `FidelityBond` | `FidelityBond` | `FidelityBond` | Fields include `outpoint`, `amount`, `lock_time`, `pubkey`. |
| `Offer` | `Offer` | `Offer` [unverified — check source] | `Offer` [unverified — check source] | `Offer` | `Offer` | `Offer` | Contains fee, size, proof fields. |
| `MakerAddress` | `MakerAddress` | `MakerAddress` [unverified — check source] | `MakerAddress` [unverified — check source] | `MakerAddress` | `MakerAddress` | `MakerAddress` | Field: `address`. |
| `MakerState` | `MakerState` | `MakerState` [unverified — check source] | `MakerState` [unverified — check source] | `MakerState` | `MakerState` | `MakerState` | `state_type` / `stateType`: `Good`, `Unresponsive`, `Bad`. |
| `MakerProtocol` | `MakerProtocol` | `MakerProtocol` [unverified — check source] | `MakerProtocol` [unverified — check source] | `MakerProtocol` | `MakerProtocol` | `MakerProtocol` | Values: `Legacy`, `Taproot`, `Unified`. |
| `AddressType` | enum `AddressType.P2WPKH`, `AddressType.P2TR` | string constants `AddressType.P2WPKH`, `AddressType.P2TR`; wrapper creates `GeneratedAddressType` | `AddressType` record | `AddressType` record | `AddressType` record | `Coinswap::AddressType` record | Rust UniFFI record field: `addr_type`; JS NAPI enum differs. |
| `MakerOfferCandidate` | `MakerOfferCandidate` | `MakerOfferCandidate` [unverified — check source] | `MakerOfferCandidate` [unverified — check source] | `MakerOfferCandidate` | `MakerOfferCandidate` | `MakerOfferCandidate` | Fields: `address`, `offer`, `state`, `protocol`. |
| `OfferBook` | `OfferBook` | `OfferBook` [unverified — check source] | `OfferBook` [unverified — check source] | `OfferBook` | `OfferBook` | `OfferBook` | Field: `makers`. |
| `MakerFeeInfo` | `MakerFeeInfo` | `MakerFeeInfo` [unverified — check source] | `MakerFeeInfo` [unverified — check source] | `MakerFeeInfo` | `MakerFeeInfo` | `MakerFeeInfo` | Per-maker fees. |
| `SwapReport` | `SwapReport` | `SwapReport` | `SwapReport` | `SwapReport` | `SwapReport` | `SwapReport` | Swap execution report. |
| `UtxoWithAddress` | `UtxoWithAddress` | `UtxoWithAddress` [unverified — check source] | `UtxoWithAddress` [unverified — check source] | `UtxoWithAddress` | `UtxoWithAddress` | `UtxoWithAddress` | Fields: `amount`, `address`. |
| `WalletBackup` | `WalletBackup` | not exported by wrapper | `WalletBackup` [unverified — check source] | `WalletBackup` [unverified — check source] | `WalletBackup` [unverified — check source] | `WalletBackup` [unverified — check source] | JS placeholder only. |

## error handling

Rust UniFFI exports `TakerError` as an error enum with `Wallet { msg }`, `Protocol { msg }`, `Network { msg }`, `General { msg }`, and `IO { msg }`. JS/NAPI methods convert failures with `napi::Error::from_reason(reason)`, so JavaScript receives thrown `Error` objects with message text prefixed by `Init error:`, `Prepare coinswap error:`, or `Failed to acquire taker lock:`. React Native checked-in wrapper methods return Promises; generated native rejection structure is absent [unverified — check source]. Kotlin, Swift, Python, and Ruby generated error class shapes are absent [unverified — check source].

| Error pattern | Meaning | Recoverable? | Action |
| --- | --- | --- | --- |
| `Wallet error: {msg}` | Wallet sync, balance, address, backup, lock, or send operation failed. | Sometimes | Run `syncAndSave` / language equivalent, inspect Bitcoin Core RPC, then retry idempotent wallet reads. |
| `Protocol error: {msg}` | Coinswap protocol state or message flow failed. | Sometimes | Call `recoverActiveSwap` before declaring funds lost. |
| `Network error: {msg}` | Offerbook sync, maker fetch, Tor, or fee API failed. | Yes | Verify Tor `9050` / `9051`, maker containers, and Bitcoin ZMQ. |
| `General error: {msg}` | Invalid protocol, txid parse, lock acquisition, or uncategorized taker failure. | Depends | Fix the input or stop concurrent access to the same wallet. |
| `IO error: {msg}` | Filesystem or process I/O failed. | Yes | Verify absolute `dataDir`, writable wallet directories, and native artifact paths. |

Rust panics terminate the host process because `profile.release-smaller` sets `panic = "abort"`. JS Worker: detect via `worker.on('exit', (code) => handleExit(code))`, not `try/catch`. React Native: the native module process crashes or the bridge/runtime resets [unverified — check source]. Swift/Kotlin: the app/JVM process aborts [unverified — check source]. Python/Ruby: the interpreter process aborts [unverified — check source].

## critical rules

- Use one `Taker` instance per wallet `dataDir`; two concurrent instances can corrupt wallet state.
- [JS] Blocking methods must not run on the main thread.
- [RN] Blocking native work must not run on the UI path; the checked-in wrapper returns Promises.
- [Python] Blocking methods must not run on the main event-loop thread.
- Execute the call sequence: logging setup → constructor/init → `syncAndSave` → `fetchOffers` / `syncOfferbookAndWait` → `prepareCoinswap` → `startCoinswap` → `syncAndSave`.
- Never read balances without `syncAndSave` first.
- Filter `offerBook.makers` for `stateType === 'Good'` and swap-amount range before `prepareCoinswap`.
- Pass the `swapId` returned by `prepareCoinswap` to `startCoinswap`; do not pass `swapParams` again.
- Call `syncAndSave` after every `startCoinswap` before reading post-swap balances.
- On `startCoinswap` failure, call `recoverActiveSwap` before concluding funds are lost.
- [JS] Use an absolute `dataDir` when not passing `null`.
- [RN] Use an absolute `dataDir` when not passing `null`.
- [Python] Use an absolute `data_dir` when not passing `None`.
- [Ruby] Use an absolute `data_dir` when not passing `nil`.
- [JS] `rpcConfig.url` must include `http://` or `https://`; JS README uses `http://127.0.0.1:18442`.
- [Kotlin] Prefer `rpcConfig.url` with `http://` or `https://`; README uses `http://127.0.0.1:18442`, tests use `localhost:18442`.
- [Python] `rpc_config.url` must include `http://` or `https://`; README uses `http://127.0.0.1:18442`.
- [RN] Prefer `rpcConfig.url` with `http://` or `https://`; README and tests use `localhost:18442`.
- Do not use mainnet or `Network.Mainnet` in production apps unless upstream explicitly documents production support.
- [JS] Detect Rust panics through Worker `exit`, not `try/catch`.
- [Python] Hold the GIL when calling binding methods.
- [RN] Enable `newArchEnabled=true`; the Expo plugin adds it when absent.
- [RN] Do not expect events; no event emitters or event names exist in checked-in source.
- [Ruby] Do not use `do_coinswap` from `test/standard_swap.rb`; Rust exports and README use `prepare_coinswap` plus `start_coinswap`.

## common mistakes

### ❌ Reusing one wallet directory from two Taker instances

```typescript
const a = new Taker('/abs/wallets/taker', 'w', rpcConfig, 9051, 'coinswap', 'tcp://127.0.0.1:28332', '')
const b = new Taker('/abs/wallets/taker', 'w', rpcConfig, 9051, 'coinswap', 'tcp://127.0.0.1:28332', '')
```

```typescript
const taker = new Taker('/abs/wallets/taker', 'w', rpcConfig, 9051, 'coinswap', 'tcp://127.0.0.1:28332', '')
```

### ❌ Reading stale balances

```python
balances = taker.get_balances()
```

```python
taker.sync_and_save()
balances = taker.get_balances()
```

### ❌ Passing swap params into start

```typescript
const report = taker.startCoinswap(swapParams as unknown as string)
```

```typescript
const swapId = taker.prepareCoinswap(swapParams)
const report = taker.startCoinswap(swapId)
```

### ❌ Swapping against bad or out-of-range makers

```typescript
const swapId = taker.prepareCoinswap({ sendAmount: 500000, makerCount: 2 })
```

```typescript
const good = taker.fetchOffers().makers.filter((maker) => maker.state.stateType === 'Good')
const swapId = taker.prepareCoinswap({ sendAmount: 500000, makerCount: 2, preferredMakers: good.map((maker) => maker.address.address) })
```

### ❌ Assuming plain Signet is Mutinynet

```bash
bitcoind -signet=1 -rpcport=38332
```

```bash
bitcoind -signet=1 -signetchallenge=<custom-signet-challenge> -addnode=<custom-signet-peer> -dnsseed=<custom-dnsseed-setting> -signetblocktime=<custom-block-time> -rpcport=38332
```

### ❌ [RN] Importing generated bindings directly before generation

```typescript
import { Taker } from 'coinswap-react-native/src/generated/coinswap'
```

```typescript
import { CoinswapTaker } from 'coinswap-react-native'
```

### ❌ [RN] Running without new architecture

```properties
newArchEnabled=false
```

```properties
newArchEnabled=true
android.useAndroidX=true
```

### ❌ [Ruby] Calling stale `do_coinswap`

```ruby
result = taker.do_coinswap(swap_params)
```

```ruby
swap_id = taker.prepare_coinswap(swap_params)
result = taker.start_coinswap(swap_id)
```

## build and test

| Binding | Release build command | Test / integration check command | Smoke test |
| --- | --- | --- | --- |
| `ffi-commons` | `cargo build --package coinswap-ffi --profile release-smaller --target x86_64-unknown-linux-gnu` | `cargo test` | `cargo run --bin uniffi-bindgen generate --library ./target/x86_64-unknown-linux-gnu/release-smaller/libcoinswap_ffi.so --language python --out-dir ../coinswap-python/src/coinswap/native/linux-x86_64 --no-format` |
| `coinswap-js` | `yarn build` | `yarn test` | `node -e "const m=require('./'); console.log(typeof m.Taker)"` |
| `coinswap-react-native` | `npm run ubrn:android`; `npm run ubrn:ios` | `COINSWAP_LIVE_TESTS=1 npm run test:live` | after build: `node -e "const m=require('./src'); console.log(typeof m.isNativeCoinswapAvailable)"` |
| `coinswap-kotlin` | `bash ./build-scripts/release/build-release-linux-arm64_v8a.sh`; `./gradlew :lib:assembleRelease` | `./gradlew test` | after build: `./gradlew :lib:test --tests org.coinswap.StandardSwap` |
| `coinswap-swift` | `bash ./build-xcframework.sh`; `swift build` | `swift test` | after build: `swift test --filter LiveStandardSwapTests/testLiveTakerFlow` |
| `coinswap-python` | `bash ./build-scripts/release/build-release-linux-x86_64.sh`; `python -m build` | `python coinswap-python/test/standard_swap.py` | after build: `python -c "from coinswap import Taker; print(Taker)"` |
| `coinswap-ruby` | `bash ./build-scripts/release/build-release-linux-x86_64.sh` | `ruby test/standard_swap.rb` | after build: `ruby -I. -e "require 'coinswap'; p Coinswap::Taker"` |

## source file index

| File path | What an agent finds there |
| --- | --- |
| `README.md` | Development status and runtime requirement summary; not binding inventory. |
| `ffi-commons/Cargo.toml` | Rust crate name, edition, dependency versions, crate type, release profile, panic behavior. |
| `ffi-commons/README.md` | Shared UniFFI build model, downstream binding outputs, Rust minimum. |
| `ffi-commons/ffi-docker-setup` | Regtest Bitcoin Core, Tor, maker, port, fee, and funding configuration. |
| `ffi-commons/signet-docker-script` | Mutinynet custom Signet Bitcoin Core, Tor, maker, and funding configuration. |
| `ffi-commons/uniffi.toml` | Kotlin package name, Swift module name, TypeScript generated-binding hint. |
| `ffi-commons/src/lib.rs` | UniFFI scaffolding and module exports. |
| `ffi-commons/src/taker.rs` | Shared Rust `Taker`, constructor, swap methods, wallet methods, offer methods. |
| `ffi-commons/src/types.rs` | Public records, enums, errors, fee helpers, restore helpers, default RPC config. |
| `coinswap-js/package.json` | NPM package metadata, Node engine, NAPI targets, scripts. |
| `coinswap-js/index.d.ts` | Generated TypeScript API names, method signatures, object field casing. |
| `coinswap-js/src/taker.rs` | NAPI `Taker` implementation, sync/blocking methods, thrown error strings. |
| `coinswap-js/src/types.rs` | NAPI object type mapping and enums. |
| `coinswap-js/README.md` | JS install, build, usage, and API examples. |
| `coinswap-react-native/package.json` | RN package name, peer dependencies, scripts, codegen config. |
| `coinswap-react-native/README.md` | RN Turbo Module architecture, build commands, usage, requirements, tests. |
| `coinswap-react-native/src/index.ts` | Checked-in JS wrapper, Promise API, native availability check, exported `AddressType`. |
| `coinswap-react-native/ubrn.config.yaml` | Rust source path, generated output paths, Android/iOS targets, TurboModule config. |
| `coinswap-react-native/CoinswapReactNative.podspec` | iOS source files, vendored framework, TurboModule dependencies. |
| `coinswap-react-native/android/build.gradle` | Android new-architecture detection, namespace, min/target SDK, ABI filters, React dependency. |
| `coinswap-react-native/android/consumer-rules.pro` | RN keep rule for generated UniFFI classes. |
| `coinswap-react-native/android/gradle.properties` | RN Android Kotlin version and SDK versions. |
| `coinswap-react-native/react-native.config.js` | RN package import and package instance. |
| `coinswap-react-native/plugin/withAndroid.js` | Expo plugin mutation for AndroidX and `newArchEnabled=true`. |
| `coinswap-react-native/plugin/withBinaryArtifacts.js` | Native artifact presence checks for Android and iOS. |
| `coinswap-react-native/build-scripts/development/build-dev-android-x86_64.sh` | RN Android x86_64 build and Kotlin generation fallback. |
| `coinswap-react-native/build-scripts/development/build-dev-ios.sh` | RN iOS debug build and Swift/header generation fallback. |
| `coinswap-react-native/build-scripts/release/build-release-android-arm64_v8a.sh` | RN Android arm64 release build fallback. |
| `coinswap-react-native/build-scripts/release/build-release-android-armeabi-v7a.sh` | RN Android armv7 release build fallback. |
| `coinswap-react-native/build-scripts/release/build-release-ios.sh` | RN iOS release XCFramework fallback build. |
| `coinswap-kotlin/README.md` | Kotlin build, package, usage, constructor and method examples. |
| `coinswap-kotlin/build.gradle.kts` | Root Gradle plugin versions and aggregate `test` task. |
| `coinswap-kotlin/lib/build.gradle.kts` | Maven coordinates, Android namespace, SDK levels, dependencies, publishing. |
| `coinswap-kotlin/lib/proguard-rules.pro` | Kotlin consumer ProGuard baseline. |
| `coinswap-kotlin/build-scripts/development/build-dev-linux-jvm.sh` | Kotlin JVM test native build and binding generation. |
| `coinswap-kotlin/build-scripts/development/build-dev-linux-x86_64.sh` | Kotlin Android x86_64 development build. |
| `coinswap-kotlin/build-scripts/development/build-dev-macos-x86_64.sh` | Kotlin macOS-host Android x86_64 development build. |
| `coinswap-kotlin/build-scripts/release/build-release-linux-arm64_v8a.sh` | Kotlin Linux-host arm64 release build. |
| `coinswap-kotlin/build-scripts/release/build-release-linux-armeabi-v7a.sh` | Kotlin Linux-host armv7 release build. |
| `coinswap-kotlin/build-scripts/release/build-release-macos-arm64_v8a.sh` | Kotlin macOS-host arm64 release build. |
| `coinswap-kotlin/build-scripts/release/build-release-macos-armeabi-v7a.sh` | Kotlin macOS-host armv7 release build. |
| `coinswap-kotlin/build-scripts/release/build-release-win-arm64_v8a.sh` | Kotlin Windows-host arm64 release build. |
| `coinswap-kotlin/build-scripts/release/build-release-win-armeabi-v7a.sh` | Kotlin Windows-host armv7 release build. |
| `coinswap-swift/README.md` | Swift package build, usage, API examples, requirements, tests. |
| `coinswap-swift/Package.swift` | Swift package name, products, binary target, platform versions. |
| `coinswap-swift/build-xcframework.sh` | Swift release XCFramework build. |
| `coinswap-swift/build-xcframework-dev.sh` | Swift debug XCFramework build. |
| `coinswap-swift/build-xcframework-ci.sh` | Swift CI XCFramework build. |
| `coinswap-swift/Tests/CoinswapTests/LiveTestSupport.swift` | Swift live test RPC, Tor, ZMQ, wallet, funding values. |
| `coinswap-swift/Tests/CoinswapTests/LiveStandardSwapTests.swift` | Swift legacy swap call sequence. |
| `coinswap-swift/Tests/CoinswapTests/LiveTaprootSwapTests.swift` | Swift taproot swap call sequence. |
| `coinswap-python/README.md` | Python build, package, usage, API examples, requirements. |
| `coinswap-python/pyproject.toml` | Python package name, version, Python minimum, native package data. |
| `coinswap-python/src/coinswap/__init__.py` | Python import shim and generated-binding absence error. |
| `coinswap-python/build-scripts/development/build-dev-linux-x86_64.sh` | Python Linux x86_64 debug build. |
| `coinswap-python/build-scripts/development/build-dev-macos-x86_64.sh` | Python macOS x86_64 debug build. |
| `coinswap-python/build-scripts/release/build-release-linux-x86_64.sh` | Python Linux x86_64 release build. |
| `coinswap-python/build-scripts/release/build-release-linux-aarch64.sh` | Python Linux aarch64 release build. |
| `coinswap-python/build-scripts/release/build-release-macos-x86_64.sh` | Python macOS x86_64 release build. |
| `coinswap-python/build-scripts/release/build-release-macos-aarch64.sh` | Python macOS arm64 release build. |
| `coinswap-ruby/README.md` | Ruby build, direct require path, usage, API examples, requirements. |
| `coinswap-ruby/build-scripts/development/build-dev-linux-x86_64.sh` | Ruby Linux x86_64 debug build. |
| `coinswap-ruby/build-scripts/development/build-dev-macos-x86_64.sh` | Ruby macOS x86_64 debug build. |
| `coinswap-ruby/build-scripts/release/build-release-linux-x86_64.sh` | Ruby Linux x86_64 release build. |
| `coinswap-ruby/build-scripts/release/build-release-linux-aarch64.sh` | Ruby Linux aarch64 release build. |
| `coinswap-ruby/build-scripts/release/build-release-macos-x86_64.sh` | Ruby macOS x86_64 release build. |
| `coinswap-ruby/build-scripts/release/build-release-macos-aarch64.sh` | Ruby macOS arm64 release build. |
| `coinswap-ruby/test/standard_swap.rb` | Ruby checked-in test flow; stale `do_coinswap` usage conflict. |
