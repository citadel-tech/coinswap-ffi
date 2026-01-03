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
        url="http://localhost:18442",
        user="user",
        password="password",
        wallet_name="taker_wallet"
    ),
    control_port=9051,
    tor_auth_password="your_tor_auth_pass",
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

# Wait for offerbook to sync
print("Waiting for offerbook synchronization...")
while taker.is_offerbook_syncing():
    print("Offerbook sync in progress...")
    import time
    time.sleep(2)
print("Offerbook synchronized!")

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
    regular: int                  # Regular wallet balance in sats
    swap: int                     # Swap balance in sats
    contract: int                 # Contract balance in sats
    spendable: int                # Spendable balance in sats

@dataclass
class SwapReport:
    swap_id: str                  # Unique swap identifier
    swap_duration_seconds: float  # Duration of swap in seconds
    target_amount: int            # Target swap amount in sats
    total_input_amount: int       # Total input amount in sats
    total_output_amount: int      # Total output amount in sats
    makers_count: int             # Number of makers in swap
    maker_addresses: list[str]    # List of maker addresses
    total_funding_txs: int        # Total number of funding transactions
    funding_txids_by_hop: list[list[str]]  # Funding TXIDs grouped by hop
    total_fee: int                # Total fees paid in sats
    total_maker_fees: int         # Total maker fees in sats
    mining_fee: int               # Mining fees in sats
    fee_percentage: float         # Fee as percentage of amount
    maker_fee_info: list[MakerFeeInfo]  # Detailed fee info per maker
    input_utxos: list[int]        # Input UTXO amounts
    output_change_amounts: list[int]    # Change output amounts
    output_swap_amounts: list[int]      # Swap output amounts
    output_change_utxos: list[UtxoWithAddress]  # Change UTXOs with addresses
    output_swap_utxos: list[UtxoWithAddress]    # Swap UTXOs with addresses

class AddressType(Enum):
    P2WPKH = "P2WPKH"            # Native SegWit (bech32)
    P2TR = "P2TR"                # Taproot (bech32m)
```

## Examples

Complete example are available in the [`test/`](test/) directory:
- [`coinswap.py`](test/coinswap.py) - Full wallet implementation

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
