<div align="center">

# Coinswap Ruby

**Ruby bindings for the Coinswap Bitcoin privacy protocol**

</div>

## Overview

Ruby bindings for [Coinswap](https://github.com/citadel-tech/coinswap), enabling integration with the Bitcoin coinswap privacy protocol in Ruby applications. Built using [UniFFI](https://mozilla.github.io/uniffi-rs/).

## Quick Start

### Prerequisites

- Ruby 2.7 or higher
- Bundler for gem management
- FFI gem (automatically installed)
- Generated bindings (see [Building](#building))

### Building

Generate the Ruby bindings from the UniFFI core:

```bash
cd ../ffi-commons
chmod +x create_bindings.sh
./create_bindings.sh
```

This generates:
- `coinswap.rb` - Ruby binding module
- `libcoinswap_ffi.so` - Native library (Linux)
- `libcoinswap_ffi.dylib` - Native library (macOS)

### Installation

#### Option 1: Direct Usage

Add the coinswap-ruby directory to your Ruby load path:

```ruby
$LOAD_PATH.unshift('/path/to/coinswap-ffi/coinswap-ruby')
require 'coinswap'
```

#### Option 2: Gemfile

Create a `Gemfile` in your project:

```ruby
source 'https://rubygems.org'

gem 'ffi', '~> 1.15'

# Add local path to coinswap
gem 'coinswap', path: '/path/to/coinswap-ffi/coinswap-ruby'
```

Then run:
```bash
bundle install
```

### Basic Usage

```ruby
require 'coinswap'

# Initialize a Taker
taker = Coinswap::Taker.init(
  data_dir: '/path/to/data',
  wallet_file_name: 'taker_wallet',
  rpc_config: Coinswap::RPCConfig.new(
    url: 'http://localhost:18443',
    user: 'bitcoin',
    password: 'bitcoin',
    wallet_name: 'taker_wallet'
  ),
  control_port: 9051,
  tor_auth_password: nil,
  zmq_addr: 'tcp://localhost:28332',
  password: 'your_secure_password'
)

# Setup logging
taker.setup_logging(data_dir: '/path/to/data')

# Sync wallet
taker.sync_and_save

# Get balances
balances = taker.get_balances
puts "Total Balance: #{balances.total} sats"

# Get a new receiving address
address = taker.get_next_external_address(
  address_type: Coinswap::AddressType::P2WPKH
)
puts "Receive to: #{address.value}"

# Perform a coinswap
swap_params = Coinswap::SwapParams.new(
  send_amount: 1_000_000,  # 0.01 BTC in sats
  maker_count: 2,
  manually_selected_outpoints: nil
)

report = taker.do_coinswap(swap_params: swap_params)
if report
  puts "Swap completed!"
  puts "Amount swapped: #{report.amount_swapped} sats"
  puts "Routing fee paid: #{report.routing_fees_paid} sats"
end
```

## API Reference

### Taker Class

Initialize and manage a coinswap taker:

```ruby
# Initialize
taker = Coinswap::Taker.init(
  data_dir: String | nil,
  wallet_file_name: String | nil,
  rpc_config: RPCConfig | nil,
  control_port: Integer | nil,
  tor_auth_password: String | nil,
  zmq_addr: String,
  password: String | nil
)

# Wallet operations
balances = taker.get_balances
address = taker.get_next_external_address(address_type: AddressType)
transactions = taker.get_transactions(count: Integer | nil, skip: Integer | nil)
utxos = taker.list_all_utxo_spend_info
txid = taker.send_to_address(
  address: String,
  amount: Integer,
  fee_rate: Float | nil,
  manually_selected_outpoints: Array<OutPoint> | nil
)

# Swap operations
report = taker.do_coinswap(swap_params: SwapParams)
offers = taker.fetch_offers
is_syncing = taker.is_offerbook_syncing

# Maintenance
taker.sync_and_save
taker.backup(destination_path: String, password: String | nil)
taker.recover_from_swap
```

### Data Types

```ruby
module Coinswap
  # Swap parameters
  SwapParams = Struct.new(
    :send_amount,                    # Amount to swap in satoshis
    :maker_count,                    # Number of makers (hops)
    :manually_selected_outpoints     # Array of OutPoint or nil
  )
  
  # Balance information
  Balances = Struct.new(
    :total,                          # Total balance in sats
    :confirmed,                      # Confirmed balance
    :unconfirmed                     # Unconfirmed balance
  )
  
  # Swap report
  SwapReport = Struct.new(
    :amount_swapped,                 # Amount successfully swapped
    :routing_fees_paid,              # Total routing fees
    :num_successful_swaps,           # Number of successful hops
    :total_swap_time                 # Time taken in seconds
  )
  
  # Address types
  module AddressType
    P2WPKH = :p2wpkh                # Native SegWit (bech32)
    P2TR = :p2tr                    # Taproot (bech32m)
  end
end
```

## Complete Example

```ruby
#!/usr/bin/env ruby
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
```

## Rails Integration

For Ruby on Rails applications:

```ruby
# config/initializers/coinswap.rb
require 'coinswap'

module CoinswapConfig
  DATA_DIR = Rails.root.join('tmp', 'coinswap')
  
  def self.taker
    @taker ||= Coinswap::Taker.init(
      data_dir: DATA_DIR.to_s,
      wallet_file_name: 'rails_wallet',
      rpc_config: Coinswap::RPCConfig.new(
        url: ENV['BITCOIN_RPC_URL'] || 'http://localhost:18443',
        user: ENV['BITCOIN_RPC_USER'] || 'bitcoin',
        password: ENV['BITCOIN_RPC_PASSWORD'] || 'bitcoin',
        wallet_name: 'taker_wallet'
      ),
      control_port: ENV['TOR_CONTROL_PORT']&.to_i || 9051,
      tor_auth_password: ENV['TOR_AUTH_PASSWORD'],
      zmq_addr: ENV['ZMQ_ADDR'] || 'tcp://localhost:28332',
      password: ENV['WALLET_PASSWORD']
    )
  end
end

# app/services/coinswap_service.rb
class CoinswapService
  def self.perform_swap(amount_sats, maker_count = 2)
    params = Coinswap::SwapParams.new(
      send_amount: amount_sats,
      maker_count: maker_count,
      manually_selected_outpoints: nil
    )
    
    CoinswapConfig.taker.do_coinswap(swap_params: params)
  rescue Coinswap::TakerError => e
    Rails.logger.error "Coinswap error: #{e.message}"
    nil
  end
  
  def self.get_balances
    CoinswapConfig.taker.get_balances
  rescue Coinswap::TakerError => e
    Rails.logger.error "Balance error: #{e.message}"
    nil
  end
end

# Usage in controller
class WalletsController < ApplicationController
  def balance
    balances = CoinswapService.get_balances
    render json: balances
  end
  
  def swap
    amount = params[:amount].to_i
    report = CoinswapService.perform_swap(amount)
    
    if report
      render json: { success: true, report: report }
    else
      render json: { success: false }, status: :unprocessable_entity
    end
  end
end
```

## Error Handling

All operations that can fail raise `Coinswap::TakerError`:

```ruby
begin
  balances = taker.get_balances
  puts "Balance: #{balances.total}"
rescue Coinswap::TakerError => e
  puts "Taker error: #{e.message}"
rescue StandardError => e
  puts "Unexpected error: #{e.message}"
end
```

## Thread Safety

For multi-threaded applications, wrap the taker in a mutex:

```ruby
require 'thread'

class ThreadSafeTaker
  def initialize(taker)
    @taker = taker
    @mutex = Mutex.new
  end
  
  def get_balances
    @mutex.synchronize { @taker.get_balances }
  end
  
  def do_coinswap(params)
    @mutex.synchronize { @taker.do_coinswap(swap_params: params) }
  end
  
  def sync_and_save
    @mutex.synchronize { @taker.sync_and_save }
  end
end

# Usage
taker = Coinswap::Taker.init(...)
safe_taker = ThreadSafeTaker.new(taker)

threads = 5.times.map do
  Thread.new { safe_taker.get_balances }
end

threads.each(&:join)
```

## Requirements

- Ruby 2.7 or higher
- FFI gem (automatically installed)
- Bitcoin Core with RPC enabled
- Synced, non-pruned node with `-txindex`
- Tor daemon for privacy

## Testing

Create a test file:

```ruby
# test_coinswap.rb
require 'minitest/autorun'
require 'coinswap'

class TestCoinswap < Minitest::Test
  def setup
    @taker = Coinswap::Taker.init(
      data_dir: './test_data',
      wallet_file_name: 'test_wallet',
      rpc_config: test_rpc_config,
      control_port: 9051,
      tor_auth_password: nil,
      zmq_addr: 'tcp://localhost:28332',
      password: 'test_password'
    )
  end
  
  def test_get_balances
    balances = @taker.get_balances
    assert balances.total >= 0
    assert balances.confirmed >= 0
  end
  
  def test_get_address
    address = @taker.get_next_external_address(
      address_type: Coinswap::AddressType::P2WPKH
    )
    assert address.value.start_with?('bcrt1')
  end
  
  private
  
  def test_rpc_config
    Coinswap::RPCConfig.new(
      url: 'http://localhost:18443',
      user: 'bitcoin',
      password: 'bitcoin',
      wallet_name: 'test_wallet'
    )
  end
end
```

Run tests:
```bash
ruby test_coinswap.rb
```

## Support

- [Main Coinswap Repository](https://github.com/citadel-tech/coinswap)
- [FFI Commons](../ffi-commons) - Build and binding generation
- [Coinswap Documentation](https://github.com/citadel-tech/coinswap/docs)

## License

MIT License - see [LICENSE](../LICENSE) for details
