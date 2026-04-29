<div align="center">

# Coinswap React Native

React Native TurboModule bindings for the Coinswap protocol

</div>

## Overview

`coinswap-react-native` exposes the shared `ffi-commons` taker API through a React Native TurboModule.

This package wraps native UniFFI outputs on:
- Android via generated Kotlin bindings and `.so` libraries.
- iOS via generated Swift bindings and `coinswap_ffi.xcframework`.

## Build Scripts

```bash
cd coinswap-react-native

# Android development build (x86_64 emulator)
bash ./build-scripts/development/build-dev-android-x86_64.sh

# Android release build (arm64-v8a)
bash ./build-scripts/release/build-release-android-arm64_v8a.sh

# iOS development build
bash ./build-scripts/development/build-dev-ios.sh

# iOS release build
bash ./build-scripts/release/build-release-ios.sh
```

## TurboModule API

The package exports a high-level class API:

```ts
import { AddressType, CoinswapTaker } from 'coinswap-react-native'

await CoinswapTaker.setupLogging(null, 'info')

const taker = await CoinswapTaker.init({
  dataDir: null,
  walletFileName: 'rn_wallet',
  rpcConfig: {
    url: 'localhost:18442',
    username: 'user',
    password: 'password',
    walletName: 'rn_wallet',
  },
  controlPort: 9051,
  torAuthPassword: 'coinswap',
  zmqAddr: 'tcp://127.0.0.1:28332',
  password: '',
})

await taker.syncOfferbookAndWait()
await taker.syncAndSave()

const address = await taker.getNextExternalAddress(AddressType.P2WPKH)
const swapId = await taker.prepareCoinswap({
  protocol: 'Legacy',
  sendAmount: 500_000,
  makerCount: 2,
  txCount: 1,
  requiredConfirms: 1,
})

const report = await taker.startCoinswap(swapId)
await taker.dispose()
```

## Tests

Legacy and Taproot live tests are included:
- `__tests__/live.standard-swap.test.ts`
- `__tests__/live.taproot-swap.test.ts`

Run them with:

```bash
COINSWAP_LIVE_TESTS=1 yarn test:live
```

These tests require the shared regtest docker stack from `ffi-commons`.

## Expo Plugin

This package also ships an Expo config plugin at [app.plugin.js](app.plugin.js) that mirrors the Fedimint-style setup flow.

In an Expo app, you can enable it with:

```json
{
  "expo": {
    "plugins": ["coinswap-react-native"]
  }
}
```

The plugin keeps the package ready for AndroidX and the new React Native architecture, and it is where future binary-artifact download logic would live if we start publishing prebuilt RN outputs.
