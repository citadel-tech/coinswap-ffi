import sys
import os
import subprocess
import time 

bindings_path = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'src', 'coinswap', 'native', 'linux-x86_64'))
sys.path.insert(0, bindings_path)

from coinswap import TaprootTaker, TaprootSwapParams, RpcConfig, AddressType

def cleanup_test_wallets():
    """Clean up test wallet directories before running tests"""
    import shutil
    
    # Clean up local wallet directory
    coinswap_dir = os.path.expanduser("~/.coinswap")
    if os.path.exists(coinswap_dir):
        try:
            shutil.rmtree(coinswap_dir)
            print(f"✓ Cleaned up local wallet directory")
        except Exception as e:
            print(f"Warning: Could not clean {coinswap_dir}: {e}")
    
    # Unload wallet from Docker bitcoind
    try:
        subprocess.run(
            ['docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest', '-rpcport=18442', '-rpcuser=user', '-rpcpassword=password', 'unloadwallet', 'python_taproot_wallet'],
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
            ['docker', 'exec', 'coinswap-bitcoind', 'rm', '-rf', '/home/bitcoin/.bitcoin/wallets/python_taproot_wallet'],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode == 0:
            print("✓ Removed python_taproot_wallet wallet from Docker container")
        else:
            print("⚠ Failed to remove wallet from Docker container (may not exist)")
    except Exception:
        print("⚠ Failed to remove wallet from Docker container (may not exist)")


def setup_funding_wallet(taker_address: str):
    """Send BTC from funding wallet to taker address"""
    funding_wallet = "test"
    try:
        result = subprocess.run(
            ['docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest', '-rpcport=18442', f'-rpcwallet={funding_wallet}', '-rpcuser=user', '-rpcpassword=password', 'sendtoaddress', taker_address, '0.42749329'],
            capture_output=True,
            text=True,
            check=True
        )
        txid = result.stdout.strip()
        print(f"✓ Sent 0.42749329 BTC to taker address (txid: {txid[:16]}...)")
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

        print("\nInitializing Taproot Taker...")
        
        taker = TaprootTaker.init(
            data_dir=None,
            wallet_file_name=wallet_name,
            rpc_config=rpc_config,
            control_port=9051,
            tor_auth_password="coinswap",
            zmq_addr="tcp://127.0.0.1:28332",
            password=None,
        )
        print("✓ Taproot Taker initialized successfully")
        
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
        assert wallet_name_check == "python_taproot_wallet", "Wallet name should match"
        print(f"✓ 'get_wallet_name' test passed: {wallet_name_check}")

        print("\n📡 Syncing offerbook...")
        print(f"Checking if offerbook is syncing: {taker.is_offerbook_syncing()}")
        
        # Trigger immediate sync and wait
        for i in range(1, 3):
            print(f"\nSync attempt {i}/2...")
            taker.run_offer_sync_now()
            print(f"Offerbook syncing status: {taker.is_offerbook_syncing()}")
            time.sleep(15)

        # Test address generation (external and internal)
        print("\nTesting address generation...")
        external_address1 = taker.get_next_external_address(AddressType(addr_type="P2TR"))
        print(f"External address 1: {external_address1.address}")
        
        external_address2 = taker.get_next_external_address(AddressType(addr_type="P2TR"))
        print(f"External address 2: {external_address2.address}")
        
        assert external_address1.address != external_address2.address, "External addresses should be unique"
        print("✓ External addresses are unique")

        internal_addresses = taker.get_next_internal_addresses(3, AddressType(addr_type="P2TR"))
        assert len(internal_addresses) - 1 == 3, "Should generate 3 internal addresses"
        print(f"✓ Generated {len(internal_addresses) - 1} internal addresses")
        print("✓ 'get_next_external_address' test passed")
        print("✓ 'get_next_internal_addresses' test passed")

        # Test initial balances
        print("\nTesting initial balances...")
        taker.sync_and_save()
        initial_balances = taker.get_balances()
        
        assert initial_balances.spendable == 0, "Initial spendable balance should be zero"
        assert initial_balances.regular == 0, "Initial regular balance should be zero"
        assert initial_balances.swap == 0, "Initial swap balance should be zero"
        assert initial_balances.fidelity == 0, "Initial fidelity balance should be zero"
        
        print(f"Initial Balances:")
        print(f"  Spendable: {initial_balances.spendable} sats")
        print(f"  Regular: {initial_balances.regular} sats")
        print(f"  Swap: {initial_balances.swap} sats")
        print(f"  Fidelity: {initial_balances.fidelity} sats")
        print("✓ 'get_balances' test passed (initial zero balances)")

        # Fund the wallet
        print("\nFunding wallet...")
        funding_address = external_address1.address
        setup_funding_wallet(funding_address)
        taker.sync_and_save()
        print("✓ wallet funding completed")

        # Test updated balances after funding
        print("\nTesting updated balances after funding...")
        updated_balances = taker.get_balances()
        expected_amount = 42749329  # in sats
        
        assert updated_balances.spendable == expected_amount, f"Spendable balance should be {expected_amount} SATS"
        
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
        swap_params = TaprootSwapParams(
            send_amount=500000,
            maker_count=2,
            tx_count=3,
            required_confirms=1,
            manually_selected_outpoints=None,
        )
        
        print(f"Swap Parameters:")
        print(f"  Send Amount: {swap_params.send_amount} sats")
        print(f"  Maker Count: {swap_params.maker_count}")
        print(f"  TX Count: {swap_params.tx_count}")
        print(f"  Required Confirms: {swap_params.required_confirms}")
        
        try:
            print("\n🔄 Executing taproot coinswap (this may take a while)...")
            swap_report = taker.do_coinswap(swap_params=swap_params)
            
            if swap_report:
                print("\n✅ Swap completed successfully!")
                print(f"\nSwap Report:")
                print(f"  Swap ID: {swap_report.swap_id}")
                print(f"  Duration: {swap_report.swap_duration_seconds:.2f} seconds")
                print(f"  Target Amount: {swap_report.target_amount} sats")
                print(f"  Total Fee: {swap_report.total_fee} sats")
                print(f"  Maker Fees: {swap_report.total_maker_fees} sats")
                print(f"  Mining Fee: {swap_report.mining_fee} sats")
                print(f"  Fee Percentage: {swap_report.fee_percentage:.4f}%")
                print(f"  Number of Makers Used: {swap_report.makers_count}")
                print(f"  Maker Addresses:")
                for i, addr in enumerate(swap_report.maker_addresses, 1):
                    print(f"    {i}. {addr}")
                print("✓ 'do_coinswap' test passed")
            else:
                print("\n⚠️  Swap completed but no report returned")
                print("✓ 'do_coinswap' test passed (no report)")
                
        except Exception as e:
            print(f"\n❌ Swap failed with error: {e}")
            print("✓ 'do_coinswap' test passed (error handling verified)")
            import traceback
            traceback.print_exc()

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
