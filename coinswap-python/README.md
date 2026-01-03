<div align="center">

# Coinswap Python

**Python bindings for the Coinswap Bitcoin privacy protocol**

</div>

## Overview

Python bindings for [Coinswap](https://github.com/citadel-tech/coinswap), enabling cross-platform integration with the Bitcoin coinswap privacy protocol. Built using [UniFFI](https://mozilla.github.io/uniffi-rs/).

## Quick Start

### Prerequisites

- Python 3.8 or higher
- pip or poetry for package management
- Generated bindings (see [Building](#building))

### Building

Generate the Python bindings from the UniFFI core:

```bash
cd ../ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This generates:
- `coinswap.py` - Python binding module
- `libcoinswap_ffi.so` - Native library (Linux)
- `libcoinswap_ffi.dylib` - Native library (macOS)
- `coinswap_ffi.dll` - Native library (Windows)

### Installation

#### Option 1: Direct Usage

Add the coinswap-python directory to your Python path:

```python
import sys
sys.path.append('/path/to/coinswap-ffi/coinswap-python')

import coinswap
```

#### Option 2: Virtual Environment

```bash
cd coinswap-python
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install --upgrade pip
```

Then use the module:

```python
import coinswap
```

### Basic Usage

```python
import coinswap

# Initialize a Taker
taker = coinswap.Taker.init(
    data_dir="/path/to/data",
    wallet_file_name="taker_wallet",
    rpc_config=coinswap.RPCConfig(
        url="http://localhost:18443",
        user="bitcoin",
        password="bitcoin",
        wallet_name="taker_wallet"
    ),
    control_port=9051,
    tor_auth_password=None,
    zmq_addr="tcp://localhost:28332",
    password="your_secure_password"
)

# Setup logging
taker.setup_logging(data_dir="/path/to/data")

# Sync wallet
taker.sync_and_save()

# Get balances
balances = taker.get_balances()
print(f"Total Balance: {balances.total} sats")

# Get a new receiving address
address = taker.get_next_external_address(
    address_type=coinswap.AddressType.P2WPKH
)
print(f"Receive to: {address.value}")

# Perform a coinswap
swap_params = coinswap.SwapParams(
    send_amount=1_000_000,  # 0.01 BTC in sats
    maker_count=2,
    manually_selected_outpoints=None
)

report = taker.do_coinswap(swap_params=swap_params)
if report:
    print("Swap completed!")
    print(f"Amount swapped: {report.amount_swapped} sats")
    print(f"Routing fee paid: {report.routing_fees_paid} sats")
```

## API Reference

### Taker Class

Initialize and manage a coinswap taker:

```python
# Initialize
taker = coinswap.Taker.init(
    data_dir: str | None,
    wallet_file_name: str | None,
    rpc_config: RPCConfig | None,
    control_port: int | None,
    tor_auth_password: str | None,
    zmq_addr: str,
    password: str | None
)

# Wallet operations
balances = taker.get_balances()
address = taker.get_next_external_address(address_type: AddressType)
transactions = taker.get_transactions(count: int | None, skip: int | None)
utxos = taker.list_all_utxo_spend_info()
txid = taker.send_to_address(
    address: str,
    amount: int,
    fee_rate: float | None,
    manually_selected_outpoints: list[OutPoint] | None
)

# Swap operations
report = taker.do_coinswap(swap_params: SwapParams)
offers = taker.fetch_offers()
is_syncing = taker.is_offerbook_syncing()

# Maintenance
taker.sync_and_save()
taker.backup(destination_path: str, password: str | None)
taker.recover_from_swap()
```

### Data Types

```python
from dataclasses import dataclass
from enum import Enum

@dataclass
class SwapParams:
    send_amount: int              # Amount to swap in satoshis
    maker_count: int              # Number of makers (hops)
    manually_selected_outpoints: list[OutPoint] | None

@dataclass
class Balances:
    total: int                    # Total balance in sats
    confirmed: int                # Confirmed balance
    unconfirmed: int              # Unconfirmed balance

@dataclass
class SwapReport:
    amount_swapped: int           # Amount successfully swapped
    routing_fees_paid: int        # Total routing fees
    num_successful_swaps: int     # Number of successful hops
    total_swap_time: int          # Time taken in seconds

class AddressType(Enum):
    P2WPKH = "P2WPKH"            # Native SegWit (bech32)
    P2TR = "P2TR"                # Taproot (bech32m)
```

## Complete Example

```python
#!/usr/bin/env python3
import coinswap
import sys
from pathlib import Path

class CoinswapWallet:
    def __init__(self, data_dir: str):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.taker = None
        
    def initialize(self):
        """Initialize the taker wallet"""
        try:
            self.taker = coinswap.Taker.init(
                data_dir=str(self.data_dir),
                wallet_file_name="wallet",
                rpc_config=coinswap.RPCConfig(
                    url="http://localhost:18443",
                    user="bitcoin",
                    password="bitcoin",
                    wallet_name="taker_wallet"
                ),
                control_port=9051,
                tor_auth_password=None,
                zmq_addr="tcp://localhost:28332",
                password="secure_password_123"
            )
            
            self.taker.setup_logging(str(self.data_dir))
            print("✓ Wallet initialized")
            
        except coinswap.TakerError as e:
            print(f"✗ Initialization error: {e}")
            sys.exit(1)
    
    def sync(self):
        """Sync wallet with blockchain"""
        try:
            self.taker.sync_and_save()
            print("✓ Wallet synced")
        except coinswap.TakerError as e:
            print(f"✗ Sync error: {e}")
    
    def show_balance(self):
        """Display wallet balance"""
        try:
            balances = self.taker.get_balances()
            print(f"\nWallet Balance:")
            print(f"  Total:       {balances.total:,} sats")
            print(f"  Confirmed:   {balances.confirmed:,} sats")
            print(f"  Unconfirmed: {balances.unconfirmed:,} sats")
        except coinswap.TakerError as e:
            print(f"✗ Error getting balance: {e}")
    
    def get_new_address(self):
        """Get a new receiving address"""
        try:
            address = self.taker.get_next_external_address(
                coinswap.AddressType.P2WPKH
            )
            print(f"\nNew receiving address: {address.value}")
            return address.value
        except coinswap.TakerError as e:
            print(f"✗ Error getting address: {e}")
            return None
    
    def perform_swap(self, amount_sats: int, maker_count: int = 2):
        """Perform a coinswap"""
        try:
            print(f"\nStarting coinswap...")
            print(f"  Amount: {amount_sats:,} sats")
            print(f"  Makers: {maker_count}")
            
            params = coinswap.SwapParams(
                send_amount=amount_sats,
                maker_count=maker_count,
                manually_selected_outpoints=None
            )
            
            report = self.taker.do_coinswap(params)
            
            if report:
                print(f"\n✓ Swap completed successfully!")
                print(f"  Amount swapped: {report.amount_swapped:,} sats")
                print(f"  Routing fees:   {report.routing_fees_paid:,} sats")
                print(f"  Successful hops: {report.num_successful_swaps}")
                print(f"  Time taken:     {report.total_swap_time} seconds")
                return True
            else:
                print("✗ Swap failed")
                return False
                
        except coinswap.TakerError as e:
            print(f"✗ Swap error: {e}")
            return False
    
    def list_transactions(self, count: int = 10):
        """List recent transactions"""
        try:
            txs = self.taker.get_transactions(count=count, skip=0)
            print(f"\nRecent Transactions ({len(txs)}):")
            
            for tx in txs:
                print(f"\n  TXID: {tx.info.txid.value}")
                print(f"  Confirmations: {tx.info.confirmations}")
                print(f"  Amount: {tx.detail.amount.value:,} sats")
                print(f"  Category: {tx.detail.category}")
                
        except coinswap.TakerError as e:
            print(f"✗ Error listing transactions: {e}")
    
    def fetch_makers(self):
        """Fetch available makers"""
        try:
            print("\nFetching available makers...")
            offers = self.taker.fetch_offers()
            
            if offers.offers:
                print(f"✓ Found {len(offers.offers)} makers")
                for i, offer in enumerate(offers.offers[:5], 1):
                    print(f"\n  Maker {i}:")
                    print(f"    Min: {offer.min_size:,} sats")
                    print(f"    Max: {offer.max_size:,} sats")
                    print(f"    Fee: {offer.amount_relative_fee_pct}%")
            else:
                print("No makers currently available")
                
        except coinswap.TakerError as e:
            print(f"✗ Error fetching makers: {e}")

def main():
    # Initialize wallet
    wallet = CoinswapWallet(data_dir="./coinswap_data")
    wallet.initialize()
    
    # Sync wallet
    wallet.sync()
    
    # Show balance
    wallet.show_balance()
    
    # Get new address
    wallet.get_new_address()
    
    # List transactions
    wallet.list_transactions(count=5)
    
    # Fetch makers
    wallet.fetch_makers()
    
    # Perform a small test swap (uncomment to use)
    # wallet.perform_swap(amount_sats=100_000, maker_count=2)

if __name__ == "__main__":
    main()
```

## Error Handling

All operations that can fail raise `TakerError`:

```python
import coinswap

try:
    balances = taker.get_balances()
    print(f"Balance: {balances.total}")
except coinswap.TakerError as e:
    # Handle all taker errors
    print(f"Error: {e}")
except Exception as e:
    # Handle unexpected errors
    print(f"Unexpected error: {e}")
```

## Async/Await Support

For async applications, wrap blocking calls in an executor:

```python
import asyncio
import coinswap
from concurrent.futures import ThreadPoolExecutor

class AsyncCoinswapWallet:
    def __init__(self):
        self.taker = None
        self.executor = ThreadPoolExecutor(max_workers=4)
    
    async def initialize(self, data_dir: str):
        """Async initialization"""
        loop = asyncio.get_event_loop()
        self.taker = await loop.run_in_executor(
            self.executor,
            lambda: coinswap.Taker.init(
                data_dir=data_dir,
                wallet_file_name="wallet",
                rpc_config=coinswap.RPCConfig(
                    url="http://localhost:18443",
                    user="bitcoin",
                    password="bitcoin",
                    wallet_name="taker_wallet"
                ),
                control_port=9051,
                tor_auth_password=None,
                zmq_addr="tcp://localhost:28332",
                password="secure_password"
            )
        )
    
    async def get_balances(self):
        """Async get balances"""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            self.taker.get_balances
        )
    
    async def perform_swap(self, amount: int, maker_count: int):
        """Async coinswap"""
        params = coinswap.SwapParams(
            send_amount=amount,
            maker_count=maker_count,
            manually_selected_outpoints=None
        )
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            self.executor,
            self.taker.do_coinswap,
            params
        )

# Usage
async def main():
    wallet = AsyncCoinswapWallet()
    await wallet.initialize("/path/to/data")
    
    balances = await wallet.get_balances()
    print(f"Balance: {balances.total} sats")
    
    report = await wallet.perform_swap(1_000_000, 2)
    if report:
        print(f"Swap completed: {report.amount_swapped} sats")

asyncio.run(main())
```

## Requirements

- Python 3.8 or higher
- Works with CPython and PyPy
- Bitcoin Core with RPC enabled
- Synced, non-pruned node with `-txindex`
- Tor daemon for privacy

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
