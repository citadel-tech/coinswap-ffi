#!/usr/bin/env python3
"""
Complete example of using the Coinswap Python bindings.

This script demonstrates how to:
- Initialize a taker wallet
- Sync with the blockchain
- Check balances
- Get receiving addresses
- List transactions
- Fetch available makers
- Perform a coinswap
"""

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
            
            # Wait for offerbook to sync
            print("Waiting for offerbook synchronization...")
            while self.taker.is_offerbook_syncing():
                print("Offerbook sync in progress...")
                import time
                time.sleep(2)
            print("Offerbook synchronized!")
            
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
