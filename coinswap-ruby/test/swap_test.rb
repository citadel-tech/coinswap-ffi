#!/usr/bin/env ruby
# frozen_string_literal: true

# FFI taker integration test: 4 takers x 2 makers.
#
# Mirrors the Rust/Python/Swift swap tests: one run drives four takers
# sequentially against the Docker regtest stack (1 RPC maker + 1 Electrum
# maker), covering the full backend x protocol matrix -- legacy/taproot over
# rpc/electrum. Each taker funds a fresh wallet and runs a 2-maker coinswap.

require 'fileutils'

# Add parent directory to load path for the coinswap module
lib_path = File.expand_path('..', __dir__)
$LOAD_PATH.unshift(lib_path) unless $LOAD_PATH.include?(lib_path)

require 'coinswap'

# Amount swapped by each taker, in sats. The taker is funded with 4x this.
SWAP_AMOUNT = 500_000

# (name, backend, protocol, addr_type)
SWAPS = [
  ['legacy_rpc',       'rpc',      'Legacy',  'P2WPKH'],
  ['taproot_rpc',      'rpc',      'Taproot', 'P2TR'],
  ['legacy_electrum',  'electrum', 'Legacy',  'P2WPKH'],
  ['taproot_electrum', 'electrum', 'Taproot', 'P2TR']
].freeze

def cleanup_wallet(wallet_name)
  wallets_dir = File.expand_path('~/.coinswap/taker/wallets')
  if Dir.exist?(wallets_dir)
    Dir.children(wallets_dir).each do |entry|
      next unless entry.start_with?(wallet_name)

      wallet_path = File.join(wallets_dir, entry)
      begin
        FileUtils.rm_rf(wallet_path)
      rescue StandardError => e
        puts "Warning: Could not clean #{wallet_path}: #{e.message}"
      end
    end
  end

  begin
    system('docker', 'exec', 'coinswap-bitcoind', 'bitcoin-cli', '-regtest',
           '-rpcport=18442', '-rpcuser=user', '-rpcpassword=password',
           'unloadwallet', wallet_name,
           out: File::NULL, err: File::NULL)
  rescue StandardError
    # Ignore missing wallet errors.
  end
end

def fund(address)
  result = `docker exec coinswap-bitcoind bitcoin-cli -regtest -rpcport=18442 -rpcwallet=test -rpcuser=user -rpcpassword=password sendtoaddress #{address} 0.25 2>&1`
  raise "Could not send BTC to #{address}: #{result}" unless $?.success?
end

def wait_for_spendable(taker, target)
  # Sync until spendable reaches `target`, tolerating Electrum indexing lag.
  30.times do
    taker.sync_and_save
    balances = taker.get_balances
    return balances if balances.spendable >= target

    sleep(3)
  end
  taker.get_balances
end

def run_swap(name, backend, protocol, addr_type)
  puts "\n=== #{name} (#{backend} / #{protocol} / #{addr_type}) ==="
  cleanup_wallet(name)

  rpc_config =
    if backend == 'rpc'
      Coinswap::RpcConfig.new(
        url: 'localhost:18442',
        username: 'user',
        password: 'password',
        wallet_name: name
      )
    end

  backend_config =
    if backend == 'electrum'
      Coinswap::BackendConfig.new(kind: 'electrum', url: 'tcp://localhost:50001')
    end

  data_dir = File.expand_path("~/.coinswap/taker/#{name}")

  taker = Coinswap::Taker.init(
    data_dir,                  # taker data directory
    name,                      # wallet file name
    rpc_config,                # Bitcoin Core RPC settings (nil for electrum)
    9051,                      # Tor control port
    'coinswap',                # Tor control password
    'tcp://127.0.0.1:28332',   # Bitcoin Core ZMQ endpoint
    '',                        # optional wallet encryption password
    nil,                       # nostr relays (nil keeps defaults)
    backend_config             # backend selection (nil for rpc)
  )

  taker.sync_offerbook_and_wait

  # Fund with 0.25 BTC across 4 fresh external addresses (1.0 BTC total).
  4.times do
    addr = taker.get_next_external_address(
      Coinswap::AddressType.new(addr_type: addr_type)
    ).addr
    fund(addr)
  end

  target = SWAP_AMOUNT * 2
  funded = wait_for_spendable(taker, target)
  raise "#{name}: spendable #{funded.spendable} < target #{target}" unless funded.spendable >= target

  swap_params = Coinswap::SwapParams.new(
    protocol: protocol,
    send_amount: SWAP_AMOUNT,
    maker_count: 2,
    tx_count: 1,
    required_confirms: 1,
    manually_selected_outpoints: nil,
    preferred_makers: nil
  )
  swap_id = taker.prepare_coinswap(swap_params)
  report = taker.start_coinswap(swap_id)

  raise "#{name}: coinswap should return a swap report" if report.nil?
  raise "#{name}: should route through 2 makers, got #{report.makers_count}" unless report.makers_count == 2
  raise "#{name}: swap status was #{report.status}" unless report.status.upcase.include?('SUCCESS')

  puts "✓ #{name} passed (swap_id #{report.swap_id})"
end

def main
  SWAPS.each do |name, backend, protocol, addr_type|
    run_swap(name, backend, protocol, addr_type)
  end
  puts "\n✓ all 4 takers (legacy/taproot × rpc/electrum) completed 2-maker swaps"
rescue StandardError => e
  puts "\n✗ Error: #{e.class.name}: #{e.message}"
  puts e.backtrace.join("\n")
  exit(1)
end

main if __FILE__ == $PROGRAM_NAME
