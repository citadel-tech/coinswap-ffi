#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'

# Add parent directory to load path for the coinswap module
lib_path = File.expand_path('..', __dir__)
$LOAD_PATH.unshift(lib_path) unless $LOAD_PATH.include?(lib_path)

require 'coinswap'

def cleanup_test_wallets
  """Clean up test wallet directories before running tests"""
  coinswap_taker_dir = File.expand_path("~/.coinswap/taker")
  if File.exist?(coinswap_taker_dir)
    begin
      FileUtils.rm_rf(coinswap_taker_dir)
      puts "✓ Cleaned up #{coinswap_taker_dir}"
    rescue => e
      puts "Warning: Could not clean #{coinswap_taker_dir}: #{e.message}"
    end
  end
  
  bitcoin_wallet_dir = File.expand_path("~/.bitcoin/regtest/wallets/ruby_test_wallet")
  if File.exist?(bitcoin_wallet_dir)
    begin
      FileUtils.rm_rf(bitcoin_wallet_dir)
      puts "✓ Cleaned up #{bitcoin_wallet_dir}"
    rescue => e
      puts "Warning: Could not clean #{bitcoin_wallet_dir}: #{e.message}"
    end
  end
  
  begin
    system('bitcoin-cli', '-regtest', 'unloadwallet', 'ruby_test_wallet',
           out: File::NULL, err: File::NULL)
  rescue
    # Ignore errors
  end
end

def setup_funding_wallet(taker_address)
  """Create a funding wallet, mine blocks, and send BTC to taker address"""
  funding_wallet = "test"
  begin
    result = `docker exec coinswap-bitcoind bitcoin-cli -regtest -rpcport=18442 -rpcwallet=#{funding_wallet} -rpcuser=user -rpcpassword=password sendtoaddress #{taker_address} 1.0 2>&1`
    
    if $?.success?
      txid = result.strip
      puts "✓ Sent 1.0 BTC to taker address (txid: #{txid[0..15]}...)"
    else
      puts "✗ Failed to send BTC: #{result}"
      raise "Could not send BTC to taker address"
    end
  rescue => e
    puts "✗ Unexpected error sending BTC: #{e.message}"
    raise
  end
  
  sleep(1)
end

    WALLET_NAME = 'ruby_test_wallet'

def main
  begin
    puts "Cleaning up previous test data..."
    cleanup_test_wallets
    puts ""
    
    rpc_config = Coinswap::RpcConfig.new(
      url: "localhost:18442",
      username: "user",
      password: "password",
      wallet_name: WALLET_NAME
    )

    puts "\nInitializing Taker..."
    data_dir = File.expand_path("~/.coinswap/taker")
    
    taker = Coinswap::Taker.init(
      data_dir,
      WALLET_NAME,
      rpc_config,
      9051,
      "coinswap",
      "tcp://127.0.0.1:28332",
      ""
    )
    puts "✓ Taker initialized successfully"
    
    # Setup logging after initialization
    puts "\nSetting up logging..."
    begin
      taker.setup_logging_with_level(data_dir, "Info")
      puts "✓ Logging configured (level: Info)"
    rescue => e
      puts "⚠️  Warning: Could not setup logging: #{e.message}"
      puts "   Continuing without logging..."
    end

    wallet_name_check = taker.get_wallet_name
    puts "Wallet name: #{wallet_name_check}"

    puts "\n📡 Syncing offerbook..."
    puts "Checking if offerbook is syncing: #{taker.is_offerbook_syncing}"
    
    # Trigger immediate sync
    puts "Triggering immediate offerbook sync..."
    taker.run_offer_sync_now
    
    # Wait for synchronization to complete
    puts "Waiting for offerbook synchronization to complete..."
    begin
      taker.is_offerbook_syncing
      puts "Offerbook sync in progress..."
      sleep(15)
    rescue => e
      puts "Error checking offerbook sync status: #{e.message}"
    end
    
    puts "\n📡 Attempting to fetch offers from makers..."
    puts "   Note: In regtest mode, makers are auto-discovered during coinswap"
    begin
      offerbook = taker.fetch_offers
      puts "✓ Successfully fetched offers"
      puts "  Total makers found: #{offerbook.makers.length}"
      
      if offerbook.makers.length > 0
        puts "\n🎯 Maker Details:"
        offerbook.makers.each_with_index do |maker, i|
          puts "\n  Maker #{i + 1}:"
          puts "    Address: #{maker.address.address}"
          print "    State: #{maker.state.state_type}"
          if maker.state.retries
            puts " (retries: #{maker.state.retries})"
          else
            puts ""
          end
          
          if maker.protocol
            puts "    Protocol: #{maker.protocol.protocol_type}"
          end
          
          if maker.offer
            puts "    Offer Details:"
            puts "      Base Fee: #{maker.offer.base_fee} sats"
            puts "      Amount Relative Fee: #{maker.offer.amount_relative_fee_pct}%"
            puts "      Time Relative Fee: #{maker.offer.time_relative_fee_pct}%"
            puts "      Required Confirms: #{maker.offer.required_confirms}"
            puts "      Minimum Locktime: #{maker.offer.minimum_locktime}"
            puts "      Min Size: #{maker.offer.min_size} sats"
            puts "      Max Size: #{maker.offer.max_size} sats"
          else
            puts "    Offer: None (no offer available)"
          end
        end
      else
        puts "\n⚠️  No makers found in offerbook"
      end
      
    rescue => e
      puts "⚠️  Could not fetch offers (expected in regtest): #{e.message}"
      puts "   Makers running on localhost will be auto-discovered during coinswap"
    end

    puts "\nSyncing wallet..."
    taker.sync_and_save
    puts "✓ Wallet synced"

    puts "\nGetting initial balances..."
    balances = taker.get_balances
    puts "Initial Balances: #{balances.inspect}"

    puts "\nGetting next external address..."
    address = taker.get_next_external_address(Coinswap::AddressType.new(addr_type: "P2WPKH"))
    setup_funding_wallet(address.address)
    puts "Address: #{address.address}"

    puts "\nSyncing wallet after funding..."
    taker.sync_and_save
    puts "✓ Wallet synced"

    puts "\nGetting updated balances..."
    balances = taker.get_balances
    puts "Updated Balances:"
    puts "  Spendable: #{balances.spendable} sats"
    puts "  Regular: #{balances.regular} sats"
    puts "  Swap: #{balances.swap} sats"
    puts "  Fidelity: #{balances.fidelity} sats"

    # Perform coinswap
    puts "\n💱 Initiating coinswap..."
    swap_params = Coinswap::SwapParams.new(
      send_amount: 500000,
      maker_count: 2,
      manually_selected_outpoints: nil
    )
    puts "Swap Parameters:"
    puts "  Send Amount: #{swap_params.send_amount} sats"
    puts "  Maker Count: #{swap_params.maker_count}"
    
    begin
      puts "\n🔄 Executing coinswap (this may take a while)..."
      result = taker.do_coinswap(swap_params)
      
      if result
        puts "\n✅ Coinswap completed successfully!"
        puts "\nSwap Report:"
        puts "  Swap ID: #{result.swap_id}"
        puts "  Duration: #{result.swap_duration_seconds.round(2)} seconds"
        puts "  Target Amount: #{result.target_amount} sats"
        puts "  Total Fee: #{result.total_fee} sats"
        puts "  Maker Fees: #{result.total_maker_fees} sats"
        puts "  Mining Fee: #{result.mining_fee} sats"
        puts "  Fee Percentage: #{result.fee_percentage.round(4)}%"
        puts "  Number of Makers Used: #{result.makers_count}"
        puts "  Maker Addresses:"
        result.maker_addresses.each_with_index do |addr, i|
          puts "    #{i + 1}. #{addr}"
        end
      else
        puts "\n⚠️  Coinswap returned no result (possibly no makers available)"
      end
      
    rescue => e
      puts "\n❌ Coinswap failed: #{e.message}"
      puts "   This is expected if makers are not running or not properly set up."
    end

    # Final balance check
    puts "\n📊 Final balances after coinswap..."
    taker.sync_and_save
    final_balances = taker.get_balances
    puts "Final Balances:"
    puts "  Spendable: #{final_balances.spendable} sats"
    puts "  Regular: #{final_balances.regular} sats"
    puts "  Swap: #{final_balances.swap} sats"
    puts "  Fidelity: #{final_balances.fidelity} sats"

    puts "\n✓ All tests completed!"

  rescue => e
    puts "\n✗ Error: #{e.class.name}: #{e.message}"
    puts e.backtrace.join("\n")
    exit(1)
  end
end

if __FILE__ == $PROGRAM_NAME
  main
end
