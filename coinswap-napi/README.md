<div align="center">

# Coinswap NAPI

**Node.js bindings for the Coinswap Bitcoin privacy protocol**

[![MIT Licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Node.js >= 18](https://img.shields.io/badge/node-%3E%3D18.0.0-brightgreen.svg)](https://nodejs.org)

</div>

## Overview

Coinswap NAPI provides high-performance Node.js bindings for the [Coinswap protocol](https://github.com/citadel-tech/coinswap), enabling JavaScript and TypeScript applications to perform trustless, privacy-preserving Bitcoin atomic swaps. Built with [NAPI-RS](https://napi.rs) for native performance and cross-platform compatibility.

> **See it in action:** Check out [taker-app](https://github.com/citadel-tech/taker-app) - a reference GUI application built with these bindings that demonstrates wallet management, swap execution, and market analytics.

## Installation

```bash
npm install coinswap-napi
# or
yarn add coinswap-napi
# or
pnpm add coinswap-napi
```

## Quick Start

### Create a New Wallet

```typescript
import { Taker } from 'coinswap-napi';

// Configure RPC connection to Bitcoin Core
const rpcConfig = {
  url: 'http://127.0.0.1:38332',
  username: 'user',
  password: 'password'
  walletName: 'my_wallet'
};

// Create a new wallet
const taker = await Taker.init(
  null,                    // data_dir (defaults to ~/.coinswap/taker)
  'taker_wallet',             // wallet name
  rpcConfig,
  [control_port],
  null,               // Tor Auth Password
  [zmq_addr]
  'secure_password'     // password for encryption
);

console.log('Wallet created successfully!');
```

### Restore from Backup

```typescript
import { Taker } from 'coinswap-napi';

const backupPath = './backups/wallet_backup.json';
const password = 'secure_password';

await Taker.restoreWallet(
  null,              // data_dir
  'restored_wallet', // wallet name
  rpcConfig,
  backupPath,
  password
);
```

### Query Wallet Balance

```typescript
const balances = await taker.getBalances();

console.log(`Seed Balance: ${balances.seedBalance} sats`);
console.log(`Swap Balance: ${balances.swapBalance} sats`);
console.log(`Live Balance: ${balances.liveBalance} sats`);
console.log(`Fidelity Bonds: ${balances.fidelityBalance} sats`);
```

### Fetch Maker Offers

```typescript
const offerBook = await taker.fetchOffers();

console.log(`Found ${offerBook.offers.length} makers`);
console.log(`Total Liquidity: ${offerBook.totalOfferAmount} sats`);
console.log(`Average Maker Fee: ${offerBook.avgMakerFee.toFixed(2)}%`);
console.log(`Online Makers: ${offerBook.onlineMakers}`);

// Filter offers
const filteredOffers = offerBook.offers.filter(offer => 
  offer.minSize <= 1_000_000 && offer.maxSize >= 1_000_000
);
```

### Execute a Coinswap

```typescript
const swapParams = {
  sendAmount: 1_000_000,  // 0.01 BTC in satoshis
  makerCount: 3,          // Route through 3 makers
  manuallySelectedOutpoints: null  // Auto UTXO selection
};

console.log('Starting coinswap...');
const swapReport = await taker.doSwap(swapParams);

console.log(`Swap completed in ${swapReport.swapDurationSeconds}s`);
console.log(`Total Fee: ${swapReport.totalFee} sats (${swapReport.feePercentage.toFixed(2)}%)`);
console.log(`Makers Used: ${swapReport.makersCount}`);
console.log(`Funding Txs: ${swapReport.totalFundingTxs}`);

// Detailed maker fees
swapReport.makerFeeInfo.forEach((maker, index) => {
  console.log(`Maker ${index + 1}: ${maker.makerAddress}`);
  console.log(`  Total Fee: ${maker.totalFee} sats`);
});
```

## Platform Support

Pre-built binaries are available for:

| Platform | Architecture               | Status |
| -------- | -------------------------- | ------ |
| Linux    | x64, ARM64                 | ✅      |
| macOS    | x64 (Intel), ARM64 (M1/M2) | ✅      |
| Windows  | x64, ARM64                 | ✅      |
| FreeBSD  | x64                        | ✅      |
| Android  | ARM64                      | ✅      |

## Build from Source

### Prerequisites

- Rust 1.75.0 or higher
- Node.js 18.0.0 or higher
- Build tools: `build-essential`, `automake`, `libtool` (Linux)

### Build Steps

```bash
# Clone repository
git clone https://github.com/citadel-tech/coinswap-ffi.git
cd coinswap-ffi/coinswap-napi

# Install dependencies
yarn install

# Build native module
yarn run build

# Build for all platforms (requires cross-compilation setup)
yarn run build:release
```

## Requirements

See the [main Coinswap documentation](https://github.com/citadel-tech/coinswap/blob/master/docs/) for setup instructions.

## Examples & Reference Applications

### Taker App (GUI Reference Implementation)

The [taker-app](../taker-app) is a full-featured desktop application that demonstrates production usage of these bindings. It includes:

- Complete wallet management interface
- Real-time swap execution with progress tracking
- Market analytics and maker discovery
- UTXO visualization and control
- Transaction history and reporting

Use it as a reference for building your own applications or as a testing environment for the bindings.

```bash
# Run the taker app
cd ../taker-app
npm install
npm start
```

## Performance

NAPI bindings provide near-native performance:
- Wallet operations: < 10ms
- UTXO queries: < 50ms
- Swap execution: 30-120s (depends on maker count and network)

## Troubleshooting

### Module not found

Ensure native binaries are built:
```bash
npm run build
```

### RPC connection errors

Check Bitcoin Core is running and RPC credentials are correct:
```bash
bitcoin-cli -signet getblockchaininfo
```

### Tor connection issues

Verify Tor daemon is running:
```bash
systemctl status tor
```

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/citadel-tech/coinswap) for guidelines.

## Resources

- [Coinswap Protocol](https://gist.github.com/chris-belcher/9144bd57a91c194e332fb5ca371d0964)
- [Coinswap Implementation](https://github.com/citadel-tech/coinswap)
- [NAPI-RS Documentation](https://napi.rs)
- [Bitcoin Core RPC](https://developer.bitcoin.org/reference/rpc/)

## Support

- GitHub Issues: [Report bugs](https://github.com/citadel-tech/coinswap-ffi/issues)
- Community: [Discussions](https://discord.gg/K9BX4EGn)