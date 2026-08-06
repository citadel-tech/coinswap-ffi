"""FFI taker integration test: 4 takers × 2 makers.

Mirrors the Rust `swap_test`: one test driving four takers sequentially against
the Docker regtest stack (1 RPC maker + 1 Electrum maker), covering the full
backend × protocol matrix — legacy/taproot over rpc/electrum. Each taker funds a
fresh wallet and runs a 2-maker coinswap.
"""

import os
import subprocess
import sys
import time

bindings_path = os.path.abspath(
    os.path.join(os.path.dirname(__file__), '..', 'src', 'coinswap', 'native', 'linux-x86_64')
)
sys.path.insert(0, bindings_path)

from coinswap import Taker, SwapParams, RpcConfig, AddressType, BackendConfig

# Amount swapped by each taker, in sats. The taker is funded with 2×.
SWAP_AMOUNT = 500_000


def fund(address):
    """Fund `address` with 0.25 BTC from the Docker bitcoind `test` wallet."""
    subprocess.run(
        [
            'docker', 'exec', 'coinswap-bitcoind',
            'bitcoin-cli', '-regtest', '-rpcport=18442',
            '-rpcwallet=test', '-rpcuser=user', '-rpcpassword=password',
            'sendtoaddress', address, '0.25',
        ],
        capture_output=True, text=True, check=True,
    )


def wait_for_spendable(taker, target):
    """Sync until spendable reaches `target`, tolerating Electrum indexing lag."""
    for _ in range(30):
        taker.sync_and_save()
        balances = taker.get_balances()
        if balances.spendable >= target:
            return balances
        time.sleep(3)
    return taker.get_balances()


def run_swap(name, data_dir, backend, protocol, addr_type):
    """Run one taker end-to-end: init → fund → sync → 2-maker coinswap → assert."""
    print(f"\n=== {name} ({protocol}) ===")

    rpc_config = (
        RpcConfig(url="localhost:18442", username="user", password="password", wallet_name=f"python_{name}")
        if backend == "rpc" else None
    )
    backend_config = (
        BackendConfig(
            kind="electrum",
            url="tcp://localhost:50001",
            username=None,
            password=None,
            wallet_name=None,
            zmq_addr=None,
            socks5=None,
            timeout=None,
            poll_interval_secs=None,
            max_retries=None,
        )
        if backend == "electrum" else None
    )

    taker = Taker.init(
        data_dir=data_dir,
        wallet_file_name=name,
        rpc_config=rpc_config,
        control_port=9051,
        tor_auth_password="coinswap",
        zmq_addr="tcp://localhost:28332",
        password="",
        nostr_relays=None,
        backend_config=backend_config,
    )

    taker.sync_offerbook_and_wait()

    # Fund with 2× the swap amount across 4 fresh external addresses.
    for _ in range(4):
        addr = taker.get_next_external_address(AddressType(addr_type=addr_type)).addr
        fund(addr)

    target = SWAP_AMOUNT * 2
    funded = wait_for_spendable(taker, target)
    assert funded.spendable >= target, (
        f"{name}: spendable {funded.spendable} < target {target}"
    )

    swap_id = taker.prepare_coinswap(
        swap_params=SwapParams(
            protocol=protocol,
            send_amount=SWAP_AMOUNT,
            maker_count=2,
            tx_count=1,
            required_confirms=1,
            manually_selected_outpoints=None,
            preferred_makers=None,
        )
    )
    report = taker.start_coinswap(swap_id=swap_id)
    assert report is not None, f"{name}: coinswap should return a swap report"
    assert report.makers_count == 2, f"{name}: should route through 2 makers, got {report.makers_count}"
    assert "SUCCESS" in report.status.upper(), f"{name}: swap status was {report.status}"

    print(f"✓ {name} passed (swap_id {report.swap_id})")


SWAPS = [
    # (name, backend, protocol, addr_type)
    ("legacy_rpc", "rpc", "Legacy", "P2WPKH"),
    ("taproot_rpc", "rpc", "Taproot", "P2TR"),
    ("legacy_electrum", "electrum", "Legacy", "P2WPKH"),
    ("taproot_electrum", "electrum", "Taproot", "P2TR"),
]


def main():
    base_dir = os.path.expanduser("~/.coinswap/taker")
    try:
        for name, backend, protocol, addr_type in SWAPS:
            data_dir = os.path.join(base_dir, name)
            run_swap(name, data_dir, backend, protocol, addr_type)
        print("\n✓ all 4 takers (legacy/taproot × rpc/electrum) completed 2-maker swaps")
    except Exception as e:
        print(f"\n✗ Error: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
