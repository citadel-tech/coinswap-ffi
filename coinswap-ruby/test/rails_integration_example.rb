# frozen_string_literal: true

# Rails Integration Example for Coinswap Ruby bindings
#
# This file demonstrates how to integrate coinswap into a Ruby on Rails application.

# config/initializers/coinswap.rb
require 'coinswap'

module CoinswapConfig
  DATA_DIR = Rails.root.join('tmp', 'coinswap')
  
  def self.taker
    @taker ||= Coinswap::Taker.init(
      data_dir: DATA_DIR.to_s,
      wallet_file_name: 'rails_wallet',
      rpc_config: Coinswap::RPCConfig.new(
        url: ENV['BITCOIN_RPC_URL'] || 'http://localhost:18442',
        user: ENV['BITCOIN_RPC_USER'] || 'user',
        password: ENV['BITCOIN_RPC_PASSWORD'] || 'password',
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
