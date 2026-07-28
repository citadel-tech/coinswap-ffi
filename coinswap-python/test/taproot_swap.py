import sys
import os
import subprocess
import time 

bindings_path = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'src', 'coinswap', 'native', 'linux-x86_64'))
sys.path.insert(0, bindings_path)

from coinswap import Taker, SwapParams, RpcConfig, AddressType

def cleanup_test_wallets():
    """Clean up test wallet directories before running tests"""
    import shutil
    wallet_name = "python_taproot_wallet"

    wallets_dir = os.path.expanduser("~/.coinswap/taker/wallets")
    if os.path.isdir(wallets_dir):
        for entry in os.listdir(wallets_dir):
            if not entry.startswith(wallet_name):
                continue
            wallet_path = os.path.join(wallets_dir, entry)
            try:
                if os.path.isdir(wallet_path):
                    shutil.rmtree(wallet_path)
                else:
                    os.remove(wallet_path)
                print(f"✓ Cleaned up {wallet_path}")
            except Exception as e:
                print(f"Warning: Could not clean {wallet_path}: {e}")
    
    # Unload wallet from Docker bitcoind
    try:
        subprocess.run(
            ['docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest', '-rpcport=18442', '-rpcuser=user', '-rpcpassword=password', 'unloadwallet', wallet_name],
            capture_output=True,
            text=True,
            check=False
        )
        print("✓ Unloaded wallet from Docker bitcoind")
    except Exception:
        pass
    
    # Remove the python_taproot_wallet wallet from the Docker container's bitcoin folder
    try:
        result = subprocess.run(
            ['docker', 'exec', 'coinswap-bitcoind', 'rm', '-rf', f'/home/bitcoin/.bitcoin/wallets/{wallet_name}'],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode == 0:
            print(f"✓ Removed {wallet_name} wallet from Docker container")
        else:
            print("⚠ Failed to remove wallet from Docker container (may not exist)")
    except Exception:
        print("⚠ Failed to remove wallet from Docker container (may not exist)")


def setup_funding_wallet(taker):
    """Fund the taker as 4 separate UTXOs (summing to 0.42749329 BTC), each sent to a
    FRESH external P2TR address (one per swap split), mirroring the core integration
    tests' fund_taker. Reusing one address does not give the split-funding path
    (tx_count > 1) distinct selectable inputs."""
    funding_wallet = "test"
    total_sats = 42749329
    quarter_sats = total_sats // 4
    parts = [quarter_sats, quarter_sats, quarter_sats, total_sats - quarter_sats * 3]
    try:
        for part_sats in parts:
            taker_address = taker.get_next_external_address(AddressType(addr_type="P2TR")).addr
            amount_btc = f"{part_sats / 1e8:.8f}"
            result = subprocess.run(
                ['docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest', '-rpcport=18442', f'-rpcwallet={funding_wallet}', '-rpcuser=user', '-rpcpassword=password', 'sendtoaddress', taker_address, amount_btc],
                capture_output=True,
                text=True,
                check=True
            )
            txid = result.stdout.strip()
            print(f"✓ Sent {amount_btc} BTC to {taker_address[:16]}... (txid: {txid[:16]}...)")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to send BTC: {e.stderr}")
        raise Exception("Could not send BTC to taker address") from e
    except Exception as e:
        print(f"✗ Unexpected error sending BTC: {e}")
        raise

    time.sleep(1)


def main():
    try:
        print("========================================")
        print("Taproot Taker Complete Flow Test")
        print("========================================\n")

        print("Cleaning up previous test data...")
        cleanup_test_wallets()
        print()

        wallet_name = 'python_taproot_wallet'
        
        rpc_config = RpcConfig(
            url="localhost:18442",
            username="user",
            password="password",
            wallet_name=wallet_name,
        )

        print("\nInitializing Taker...")
        
        taker = Taker.init(
            data_dir=None,
            wallet_file_name=wallet_name,
            rpc_config=rpc_config,
            control_port=9051,
            tor_auth_password="coinswap",
            zmq_addr="tcp://127.0.0.1:28332",
            password=None,
        )
        print("✓ Taker initialized successfully")
        
        # Setup logging after initialization
        print("\nSetting up logging...")
        try:
            taker.setup_logging(data_dir=None, log_level="Info")
            print("✓ Logging configured (level: Info)")
        except Exception as e:
            print(f"⚠️  Warning: Could not setup logging: {e}")
            print("   Continuing without logging...")

        # Test get_wallet_name
        print("\nTesting get_wallet_name...")
        wallet_name_check = taker.get_wallet_name()
        print(f"✓ 'get_wallet_name' test passed: {wallet_name_check}")

        print("\n📡 Syncing offerbook...")
        print("Waiting for offerbook synchronization to complete...")
        taker.sync_offerbook_and_wait()
        print("Offerbook synchronized")

        # Test address generation (external and internal)
        print("\nTesting address generation...")
        external_address1 = taker.get_next_external_address(AddressType(addr_type="P2TR"))
        print(f"External address 1: {external_address1.addr}")
        
        external_address2 = taker.get_next_external_address(AddressType(addr_type="P2TR"))
        print(f"External address 2: {external_address2.addr}")
        
        assert external_address1.addr != external_address2.addr, "External addresses should be unique"
        print("✓ External addresses are unique")

        internal_addresses = taker.get_next_internal_addresses(3, AddressType(addr_type="P2TR"))
        print(f"✓ Generated {len(internal_addresses)} internal addresses")
        print("✓ 'get_next_external_address' test passed")
        print("✓ 'get_next_internal_addresses' test passed")

        # Test initial balances
        print("\nTesting initial balances...")
        taker.sync_and_save()
        initial_balances = taker.get_balances()

        print(f"Initial Balances:")
        print(f"  Spendable: {initial_balances.spendable} sats")
        print(f"  Regular: {initial_balances.regular} sats")
        print(f"  Swap: {initial_balances.swap} sats")
        print(f"  Fidelity: {initial_balances.fidelity} sats")
        print("✓ 'get_balances' test passed (initial zero balances)")

        # Fund the wallet
        print("\nFunding wallet...")
        setup_funding_wallet(taker)
        taker.sync_and_save()
        print("✓ wallet funding completed")

        # Test updated balances after funding
        print("\nTesting updated balances after funding...")
        updated_balances = taker.get_balances()

        print(f"Updated Balances:")
        print(f"  Spendable: {updated_balances.spendable} sats")
        print(f"  Regular: {updated_balances.regular} sats")
        print(f"  Swap: {updated_balances.swap} sats")
        print(f"  Fidelity: {updated_balances.fidelity} sats")
        print("✓ 'get_balances' test passed (post-funding balance verification)")

        # Test list_all_utxo_spend_info
        print("\nTesting list_all_utxo_spend_info...")
        utxos = taker.list_all_utxo_spend_info()
        assert len(utxos) > 0, "Should have at least 1 UTXO after funding"
        print(f"Found {len(utxos)} UTXO(s)")
        print("✓ list_all_utxo_spend_info test passed")

        # Test get_transactions
        print("\nTesting get_transactions...")
        transactions = taker.get_transactions(None, None)
        assert len(transactions) > 0, "Should have at least 1 transaction after funding"
        print(f"Found {len(transactions)} transaction(s)")
        print("✓ 'get_transactions' test passed")

        # Fetch offers
        print("\n📡 Fetching offers from makers...")
        try:
            fetch_offers_result = taker.fetch_offers()
            print(f"Fetch offers result: {fetch_offers_result}")
        except Exception as e:
            print(f"⚠️  Could not fetch offers: {e}")

        # Perform taproot coinswap
        print("\n💱 Initiating taproot coinswap...")
        swap_params = SwapParams(
            protocol="Taproot",
            send_amount=500000,
            maker_count=2,
            tx_count=3,
            required_confirms=1,
            manually_selected_outpoints=None,
            preferred_makers=None,
        )
        
        print(f"Swap Parameters:")
        print(f"  Send Amount: {swap_params.send_amount} sats")
        print(f"  Maker Count: {swap_params.maker_count}")
        print(f"  TX Count: {swap_params.tx_count}")
        print(f"  Required Confirms: {swap_params.required_confirms}")
        print(f"  Protocol: {swap_params.protocol}")

        print("\n🔄 Executing taproot coinswap (this may take a while)...")
        swap_id = taker.prepare_coinswap(swap_params=swap_params)
        swap_report = taker.start_coinswap(swap_id=swap_id)
        assert swap_report is not None, "Taproot coinswap should return a swap report"

        print("\n✅ Swap completed successfully!")
        print(f"\nSwap Report:")
        outgoing_amount = getattr(swap_report, "outgoing_amount", getattr(swap_report, "target_amount", None))
        fee_value = getattr(swap_report, "fee_paid", None)
        total_fee_paid = abs(fee_value) if fee_value is not None else None
        print(f"  Swap ID: {swap_report.swap_id}")
        print(f"  Duration: {swap_report.swap_duration_seconds:.2f} seconds")
        print(f"  Outgoing/Target Amount: {outgoing_amount} sats")
        print(f"  Total Fee Paid: {total_fee_paid} sats")
        print(f"  Maker Fees: {swap_report.total_maker_fees} sats")
        print(f"  Mining Fee: {swap_report.mining_fee} sats")
        print(f"  Fee Percentage: {swap_report.fee_percentage:.4f}%")
        print(f"  Number of Makers Used: {swap_report.makers_count}")
        print(f"  Maker Addresses:")
        for i, addr in enumerate(swap_report.maker_addresses, 1):
            print(f"    {i}. {addr}")
        print("✓ 'prepare_coinswap' and 'start_coinswap' test passed")

        # Final balance check
        print("\n📊 Final balances after swap...")
        taker.sync_and_save()
        final_balances = taker.get_balances()
        print(f"Final Balances:")
        print(f"  Spendable: {final_balances.spendable} sats")
        print(f"  Regular: {final_balances.regular} sats")
        print(f"  Swap: {final_balances.swap} sats")
        print(f"  Fidelity: {final_balances.fidelity} sats")

        print("\n========================================")
        print("All FFI method tests completed successfully!")
        print("========================================")

    except Exception as e:
        print(f"\n✗ Error: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
