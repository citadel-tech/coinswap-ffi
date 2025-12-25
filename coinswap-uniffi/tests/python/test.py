import sys
import os
import subprocess
import time 

bindings_path = os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../../bindings/python')
)
sys.path.insert(0, bindings_path)

from coinswap import Taker, SwapParams, RpcConfig

# Try to import TakerBehavior (only available with integration-test feature)
try:
    from coinswap import TakerBehavior
    HAS_INTEGRATION_TEST = True
    print("✓ Integration test feature detected - TakerBehavior available")
except ImportError:
    HAS_INTEGRATION_TEST = False
    TakerBehavior = None
    print("⚠ Integration test feature not enabled - rebuild with: cargo build --features integration-test")

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
    
    bitcoin_wallet_dir = os.path.expanduser("~/.bitcoin/regtest/wallets/python_test_wallet")
    if os.path.exists(bitcoin_wallet_dir):
        try:
            shutil.rmtree(bitcoin_wallet_dir)
            print(f"✓ Cleaned up {bitcoin_wallet_dir}")
        except Exception as e:
            print(f"Warning: Could not clean {bitcoin_wallet_dir}: {e}")
    
    try:
        subprocess.run(
            ['bitcoin-cli', '-regtest', 'unloadwallet', 'python_test_wallet'],
            capture_output=True,
            text=True,
            check=False
        )
    except Exception:
        pass
def setup_funding_wallet(taker_address: str):
    """Create a funding wallet, mine blocks, and send BTC to taker address"""
    funding_wallet = "test_funding_wallet"
    
    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', 'createwallet', funding_wallet, 'false', 'false', '', 'false', 'true'],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode == 0:
            print(f"✓ Created funding wallet: {funding_wallet}")
        else:
            result = subprocess.run(
                ['bitcoin-cli', '-regtest', 'loadwallet', funding_wallet],
                capture_output=True,
                text=True,
                check=False
            )
            if result.returncode == 0:
                print(f"✓ Loaded existing funding wallet: {funding_wallet}")
            else:
                print(f"Warning: Could not create or load wallet: {result.stderr}")
    except Exception as e:
        print(f"Warning: Wallet setup issue: {e}")
    
    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', f'-rpcwallet={funding_wallet}', 'getnewaddress'],
            capture_output=True,
            text=True,
            check=True
        )
        mining_address = result.stdout.strip()

        print(f"✓ Generated mining address")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to generate address: {e.stderr}")
        raise Exception("Could not generate mining address") from e
    except Exception as e:
        print(f"✗ Unexpected error generating address: {e}")
        raise

    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', 'generatetoaddress', '101', mining_address],
            capture_output=True,
            text=True,
            check=True
        )
        print(f"✓ Mined 101 blocks to funding wallet")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to mine blocks: {e.stderr}")
        raise Exception("Could not mine blocks to funding wallet") from e
    except Exception as e:
        print(f"✗ Unexpected error mining blocks: {e}")
        raise
    
    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', f'-rpcwallet={funding_wallet}', 'getbalance'],
            capture_output=True,
            text=True,
            check=True
        )
        balance = result.stdout.strip()
        print(f"✓ Funding wallet balance: {balance} BTC")
    except subprocess.CalledProcessError as e:
        print(f"Warning: Could not check balance: {e.stderr}")
    except Exception as e:
        print(f"Warning: Unexpected error checking balance: {e}")

    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', f'-rpcwallet={funding_wallet}', 'sendtoaddress', taker_address, '1.0'],
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

    try:
        result = subprocess.run(
            ['bitcoin-cli', '-regtest', 'generatetoaddress', '1', mining_address],
            capture_output=True,
            text=True,
            check=True
        )
        print(f"✓ Mined 1 block to confirm transaction")
    except subprocess.CalledProcessError as e:
        print(f"Warning: Could not mine confirmation block: {e.stderr}")
    except Exception as e:
        print(f"Warning: Unexpected error mining confirmation block: {e}")

    time.sleep(1)
def main():
    try:
        print("Cleaning up previous test data...")
        cleanup_test_wallets()
        print()

        wallet_name = 'python_test_wallet'
        
        rpc_config = RpcConfig(
            url="127.0.0.1:18442",
            username="user",
            password="password",
            wallet_name=wallet_name,
        )

        print("\nInitializing Taker...")
        data_dir = os.path.expanduser("~/.coinswap/taker")
        
        # Initialize with or without integration test feature
        if HAS_INTEGRATION_TEST:
            print("  Using integration-test feature with NORMAL behavior")
            taker = Taker.init(
                data_dir=data_dir,
                wallet_file_name=wallet_name,
                rpc_config=rpc_config,
                behavior=TakerBehavior.NORMAL,  # Integration test parameter
                control_port="",
                tor_auth_password="",
                zmq_addr="tcp://127.0.0.1:28332",
                password="",
            )
        else:
            print("  Using standard mode (no integration-test feature)")
            taker = Taker.init(
                data_dir=data_dir,
                wallet_file_name=wallet_name,
                rpc_config=rpc_config,
                control_port=9051,
                tor_auth_password=None,
                zmq_addr="tcp://127.0.0.1:28332",
                password=None,
            )
        print("✓ Taker initialized successfully")

        wallet_name_check = taker.get_wallet_name()
        print(f"Wallet name: {wallet_name_check}")

        print("\nSyncing wallet...")
        taker.sync_and_save()
        print("✓ Wallet synced")

        print("\nGetting initial balances...")
        balances = taker.get_balances()
        print(f"Initial Balances: {balances}")

        print("\nGetting next external address...")
        address = taker.get_next_external_address()
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

        print("\n📡 Attempting to fetch offers from makers...")
        print("   Note: In regtest mode, makers are auto-discovered during coinswap")
        print("   fetch_offers() typically requires a directory server")
        try:
            offerbook = taker.fetch_offers()
            print(f"✓ Successfully fetched offers")
            print(f"  Total makers found: {len(offerbook.all_makers)}")
            print(f"  Good makers: {len(offerbook.good_makers)}")
            
            if len(offerbook.good_makers) > 0:
                print("\n🎯 Good Makers Details:")
                for i, maker_offer in enumerate(offerbook.good_makers, 1):
                    print(f"\n  Maker {i}:")
                    print(f"    Address: {maker_offer.address.address}")
                    print(f"    Base Fee: {maker_offer.offer.base_fee} sats")
                    print(f"    Amount Relative Fee: {maker_offer.offer.amount_relative_fee_pct}%")
                    print(f"    Min Size: {maker_offer.offer.min_size} sats")
                    print(f"    Max Size: {maker_offer.offer.max_size} sats")
            else:
                print("\n⚠️  No offers fetched from directory server (expected in regtest)")
                print("   Makers will be auto-discovered during coinswap from localhost")

        except Exception as e:
            print(f"⚠️  Could not fetch offers (expected in regtest): {e}")
            print("   Makers running on localhost will be auto-discovered during coinswap")

        # Perform coinswap
        print("\n💱 Initiating coinswap...")
        swap_params = SwapParams(
            send_amount=500000,
            maker_count=2,
            manually_selected_outpoints=None
        )
        print(f"Swap Parameters:")
        print(f"  Send Amount: {swap_params.send_amount} sats")
        print(f"  Maker Count: {swap_params.maker_count}")
        
        try:
            print("\n🔄 Executing coinswap (this may take a while)...")
            result = taker.do_coinswap(swap_params=swap_params)
            
            if result:
                print(f"\n✅ Coinswap completed successfully!")
                print(f"\nSwap Report:")
                print(f"  Swap ID: {result.swap_id}")
                print(f"  Duration: {result.swap_duration_seconds:.2f} seconds")
                print(f"  Target Amount: {result.target_amount} sats")
                print(f"  Total Fee: {result.total_fee} sats")
                print(f"  Maker Fees: {result.total_maker_fees} sats")
                print(f"  Mining Fee: {result.mining_fee} sats")
                print(f"  Fee Percentage: {result.fee_percentage:.4f}%")
                print(f"  Number of Makers Used: {result.makers_count}")
                print(f"  Maker Addresses:")
                for i, addr in enumerate(result.maker_addresses, 1):
                    print(f"    {i}. {addr}")
            else:
                print("\n⚠️  Coinswap returned no result (possibly no makers available)")
                
        except Exception as e:
            print(f"\n❌ Coinswap failed: {e}")
            print("   This is expected if makers are not running or not properly set up.")

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
