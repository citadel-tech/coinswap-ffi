#!/usr/bin/env ruby
# frozen_string_literal: true

# Complete example of using the Coinswap Ruby bindings.
#
# This script demonstrates how to:
# - Initialize a taker wallet
# - Sync with the blockchain
# - Check balances
# - Get receiving addresses
# - List transactions
# - Fetch available makers
# - Perform a coinswap

require 'coinswap'
require 'fileutils'

class CoinswapWallet
  attr_reader :taker
  
  def initialize(data_dir)
    @data_dir = data_dir
    FileUtils.mkdir_p(@data_dir)
    @taker = nil
  end
  
  def initialize_wallet
    puts "Initializing wallet..."
    
    begin
      @taker = Coinswap::Taker.init(
        data_dir: @data_dir,
        wallet_file_name: 'wallet',
        rpc_config: Coinswap::RPCConfig.new(
          url: 'http://localhost:18443',
          user: 'bitcoin',
          password: 'bitcoin',
          wallet_name: 'taker_wallet'
        ),
        control_port: 9051,
        tor_auth_password: nil,
        zmq_addr: 'tcp://localhost:28332',
        password: 'secure_password_123'
      )
      
      @taker.setup_logging(@data_dir)
      puts "✓ Wallet initialized"
      
    rescue Coinswap::TakerError => e
      puts "✗ Initialization error: #{e.message}"
      exit 1
    end
  end
  
  def sync
    puts "Syncing wallet..."
    begin
      @taker.sync_and_save
      puts "✓ Wallet synced"
    rescue Coinswap::TakerError => e
      puts "✗ Sync error: #{e.message}"
    end
  end
  
  def show_balance
    begin
      balances = @taker.get_balances
      puts "\nWallet Balance:"
      puts "  Total:       #{format_sats(balances.total)} sats"
      puts "  Confirmed:   #{format_sats(balances.confirmed)} sats"
      puts "  Unconfirmed: #{format_sats(balances.unconfirmed)} sats"
    rescue Coinswap::TakerError => e
      puts "✗ Error getting balance: #{e.message}"
    end
  end
  
  def get_new_address
    begin
      address = @taker.get_next_external_address(
        address_type: Coinswap::AddressType::P2WPKH
      )
      puts "\nNew receiving address: #{address.value}"
      address.value
    rescue Coinswap::TakerError => e
      puts "✗ Error getting address: #{e.message}"
      nil
    end
  end
  
  def perform_swap(amount_sats, maker_count = 2)
    begin
      puts "\nStarting coinswap..."
      puts "  Amount: #{format_sats(amount_sats)} sats"
      puts "  Makers: #{maker_count}"
      
      # Wait for offerbook to sync
      puts "Waiting for offerbook synchronization..."
      while @taker.is_offerbook_syncing
        puts "Offerbook sync in progress..."
        sleep 2
      end
      puts "Offerbook synchronized!"
      
      params = Coinswap::SwapParams.new(
        send_amount: amount_sats,
        maker_count: maker_count,
        manually_selected_outpoints: nil
      )
      
      report = @taker.do_coinswap(swap_params: params)
      
      if report
        puts "\n✓ Swap completed successfully!"
        puts "  Amount swapped: #{format_sats(report.amount_swapped)} sats"
        puts "  Routing fees:   #{format_sats(report.routing_fees_paid)} sats"
        puts "  Successful hops: #{report.num_successful_swaps}"
        puts "  Time taken:     #{report.total_swap_time} seconds"
        true
      else
        puts "✗ Swap failed"
        false
      end
      
    rescue Coinswap::TakerError => e
      puts "✗ Swap error: #{e.message}"
      false
    end
  end
  
  def list_transactions(count = 10)
    begin
      txs = @taker.get_transactions(count: count, skip: 0)
      puts "\nRecent Transactions (#{txs.length}):"
      
      txs.each do |tx|
        puts "\n  TXID: #{tx.info.txid.value}"
        puts "  Confirmations: #{tx.info.confirmations}"
        puts "  Amount: #{format_sats(tx.detail.amount.value)} sats"
        puts "  Category: #{tx.detail.category}"
      end
      
    rescue Coinswap::TakerError => e
      puts "✗ Error listing transactions: #{e.message}"
    end
  end
  
  def fetch_makers
    begin
      puts "\nFetching available makers..."
      offers = @taker.fetch_offers
      
      if offers.offers && !offers.offers.empty?
        puts "✓ Found #{offers.offers.length} makers"
        
        offers.offers.first(5).each_with_index do |offer, i|
          puts "\n  Maker #{i + 1}:"
          puts "    Min: #{format_sats(offer.min_size)} sats"
          puts "    Max: #{format_sats(offer.max_size)} sats"
          puts "    Fee: #{offer.amount_relative_fee_pct}%"
        end
      else
        puts "No makers currently available"
      end
      
    rescue Coinswap::TakerError => e
      puts "✗ Error fetching makers: #{e.message}"
    end
  end
  
  private
  
  def format_sats(sats)
    sats.to_s.reverse.gsub(/(\d{3})(?=\d)/, '\\1,').reverse
  end
end

# Main execution
if __FILE__ == $0
  wallet = CoinswapWallet.new('./coinswap_data')
  
  # Initialize wallet
  wallet.initialize_wallet
  
  # Sync wallet
  wallet.sync
  
  # Show balance
  wallet.show_balance
  
  # Get new address
  wallet.get_new_address
  
  # List transactions
  wallet.list_transactions(5)
  
  # Fetch makers
  wallet.fetch_makers
  
  # Perform a test swap (uncomment to use)
  # wallet.perform_swap(100_000, 2)
end
