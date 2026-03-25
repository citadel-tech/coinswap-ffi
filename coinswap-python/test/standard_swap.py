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
    
    coinswap_taker_dir = os.path.expanduser("~/.coinswap/taker")
    if os.path.exists(coinswap_taker_dir):
        try:
            shutil.rmtree(coinswap_taker_dir)
            print(f"✓ Cleaned up {coinswap_taker_dir}")
        except Exception as e:
            print(f"Warning: Could not clean {coinswap_taker_dir}: {e}")
    
    bitcoin_wallet_dir = os.path.expanduser("~/.bitcoin/regtest/wallets/python_legacy_wallet")
    if os.path.exists(bitcoin_wallet_dir):
        try:
            shutil.rmtree(bitcoin_wallet_dir)
            print(f"✓ Cleaned up {bitcoin_wallet_dir}")
        except Exception as e:
            print(f"Warning: Could not clean {bitcoin_wallet_dir}: {e}")
    
    try:
        subprocess.run(
            ['bitcoin-cli', '-regtest', 'unloadwallet', 'python_legacy_wallet'],
            capture_output=True,
            text=True,
            check=False
        )
    except Exception:
        pass
def setup_funding_wallet(taker_address: str):
    """Create a funding wallet, mine blocks, and send BTC to taker address"""
    funding_wallet = "test"
    try:
        result = subprocess.run(
            ['docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest', '-rpcport=18442', f'-rpcwallet={funding_wallet}', '-rpcuser=user', '-rpcpassword=password', 'sendtoaddress', taker_address, '1.0'],
            capture_output=True,
            text=True,
            check=True
        )
        txid = result.stdout.strip()
        print(f"✓ Sent 1.0 BTC to taker address (txid: {txid[:16]}...)")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to send BTC: {e.stderr}")
        raise Exception("Could not send BTC to taker address") from e
    except Exception as e:
        print(f"✗ Unexpected error sending BTC: {e}")
        raise

    time.sleep(1)
def main():
    try:
        print("Cleaning up previous test data...")
        cleanup_test_wallets()
        print()

        wallet_name = 'python_legacy_wallet'
        
        rpc_config = RpcConfig(
            url="localhost:18442",
            username="user",
            password="password",
            wallet_name=wallet_name,
        )

        print("\nInitializing Taker...")
        data_dir = os.path.expanduser("~/.coinswap/taker")
        
        taker = Taker.init(
                data_dir=data_dir,
                wallet_file_name=wallet_name,
                rpc_config=rpc_config,
                control_port=9051,
                tor_auth_password="coinswap",
                zmq_addr="tcp://127.0.0.1:28332",
                password="",
            )
        print("✓ Taker initialized successfully")
        
        # Setup logging after initialization
        print("\nSetting up logging...")
        try:
            taker.setup_logging(data_dir=data_dir, log_level="Info")
            print("✓ Logging configured (level: Info)")
        except Exception as e:
            print(f"⚠️  Warning: Could not setup logging: {e}")
            print("   Continuing without logging...")

        wallet_name_check = taker.get_wallet_name()
        print(f"Wallet name: {wallet_name_check}")


        print("\n📡 Syncing offerbook...")
        print("Waiting for offerbook synchronization to complete...")
        try:
            taker.sync_offerbook_and_wait()
            print("Offerbook synchronized")
        except Exception as e:
            print(f"Error during offerbook sync: {e}")
        
        print("\n📡 Attempting to fetch offers from makers...")
        try:
            offerbook = taker.fetch_offers()
            print(f"✓ Successfully fetched offers")
            print(f"  Total makers found: {len(offerbook.makers)}")
            
            if len(offerbook.makers) > 0:
                print("\n🎯 Maker Details:")
                for i, maker in enumerate(offerbook.makers, 1):
                    print(f"\n  Maker {i}:")
                    print(f"    Address: {maker.address.address}")
                    print(f"    State: {maker.state.state_type}", end="")
                    if maker.state.retries is not None:
                        print(f" (retries: {maker.state.retries})")
                    else:
                        print()
                    
                    if maker.protocol:
                        print(f"    Protocol: {maker.protocol.protocol_type}")
                    
                    if maker.offer:
                        print(f"    Offer Details:")
                        print(f"      Base Fee: {maker.offer.base_fee} sats")
                        print(f"      Amount Relative Fee: {maker.offer.amount_relative_fee_pct}%")
                        print(f"      Time Relative Fee: {maker.offer.time_relative_fee_pct}%")
                        print(f"      Required Confirms: {maker.offer.required_confirms}")
                        print(f"      Minimum Locktime: {maker.offer.minimum_locktime}")
                        print(f"      Min Size: {maker.offer.min_size} sats")
                        print(f"      Max Size: {maker.offer.max_size} sats")
                    else:
                        print(f"    Offer: None (no offer available)")
            else:
                print("\n⚠️  No makers found in offerbook")
                
        except Exception as e:
            print(f"⚠️  Could not fetch offers: {e}")

        print("\nSyncing wallet...")
        taker.sync_and_save()
        print("✓ Wallet synced")

        print("\nGetting initial balances...")
        balances = taker.get_balances()
        print(f"Initial Balances: {balances}")

        print("\nGetting next external address...")
        address = taker.get_next_external_address(AddressType(addr_type="P2WPKH"))
        setup_funding_wallet(address.address)
        print(f"Address: {address.address}")

        print("\nSyncing wallet after funding...")
        taker.sync_and_save()
        print("✓ Wallet synced")

        print("\nGetting updated balances...")
        balances = taker.get_balances()
        print(f"Updated Balances:")
        print(f"  Spendable: {balances.spendable} sats")
        print(f"  Regular: {balances.regular} sats")
        print(f"  Swap: {balances.swap} sats")
        print(f"  Fidelity: {balances.fidelity} sats")

        # Perform coinswap
        print("\n💱 Initiating coinswap...")
        swap_params = SwapParams(
            protocol="Legacy",
            send_amount=500000,
            maker_count=2,
            tx_count=1,
            required_confirms=1,
            manually_selected_outpoints=None,
            preferred_makers=None,
        )
        print(f"Swap Parameters:")
        print(f"  Send Amount: {swap_params.send_amount} sats")
        print(f"  Maker Count: {swap_params.maker_count}")
        print(f"  Protocol: {swap_params.protocol}")

        print("\n🔄 Executing coinswap (this may take a while)...")
        swap_id = taker.prepare_coinswap(swap_params=swap_params)
        result = taker.start_coinswap(swap_id=swap_id)
        assert result is not None, "Coinswap should return a swap report"

        print(f"\n✅ Coinswap completed successfully!")
        print(f"\nSwap Report:")
        outgoing_amount = getattr(result, "outgoing_amount", getattr(result, "target_amount", None))
        fee_value = getattr(result, "fee_paid_or_earned", getattr(result, "total_fee", None))
        total_fee_paid = abs(fee_value) if fee_value is not None else None
        print(f"  Swap ID: {result.swap_id}")
        print(f"  Duration: {result.swap_duration_seconds:.2f} seconds")
        print(f"  Outgoing/Target Amount: {outgoing_amount} sats")
        print(f"  Total Fee Paid: {total_fee_paid} sats")
        print(f"  Maker Fees: {result.total_maker_fees} sats")
        print(f"  Mining Fee: {result.mining_fee} sats")
        print(f"  Fee Percentage: {result.fee_percentage:.4f}%")
        print(f"  Number of Makers Used: {result.makers_count}")
        print(f"  Maker Addresses:")
        for i, addr in enumerate(result.maker_addresses, 1):
            print(f"    {i}. {addr}")

        # Final balance check
        print("\n📊 Final balances after coinswap...")
        taker.sync_and_save()
        final_balances = taker.get_balances()
        print(f"Final Balances:")
        print(f"  Spendable: {final_balances.spendable} sats")
        print(f"  Regular: {final_balances.regular} sats")
        print(f"  Swap: {final_balances.swap} sats")
        print(f"  Fidelity: {final_balances.fidelity} sats")

        print("\n✓ All tests completed!")

    except Exception as e:
        print(f"\n✗ Error: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
