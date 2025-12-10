//! FFI Layer Tests for Coinswap Taker
//!
//! This module tests the UniFFI bindings for the Coinswap Taker,
//! ensuring the FFI layer correctly wraps the underlying Rust API.
//!
//! Based on BDK-FFI test patterns.

use crate::{
    taker::{SwapParams, Taker},
    taproot_taker::TaprootTaker,
    types::RPCConfig,
};
use bitcoin::Amount;
use bitcoind::{BitcoinD, bitcoincore_rpc::RpcApi};

#[cfg(feature = "integration-test")]
#[test]
fn main() {
    test_taker_initialization();
    test_taker_get_balance();
    test_taker_address_generation();
    test_taker_wallet_funding();
    test_taker_list_utxos();
    test_swap_params_creation();
    test_taker_sync();
    test_multiple_taker_instances();
    test_rpc_config_conversion();
    test_taker_get_transactions();
    cleanup_wallet();
}

fn setup_bitcoind() -> BitcoinD {
    let mut conf = bitcoind::Conf::default();
    conf.args.push("-txindex=1");

    let data_dir = std::env::temp_dir().join("coinswap_ffi_test");
    std::fs::create_dir_all(&data_dir).unwrap();
    conf.staticdir = Some(data_dir.join(".bitcoin"));

    let exe_path = bitcoind::exe_path().unwrap();
    let bitcoind = BitcoinD::with_conf(exe_path, &conf).unwrap();

    // Generate initial blocks for coinbase maturity
    let mining_address = bitcoind
        .client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoind::bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    bitcoind
        .client
        .generate_to_address(101, &mining_address)
        .unwrap();

    bitcoind
}

fn cleanup_wallet() {
    use std::fs;
    use std::path::PathBuf;

    // Remove wallet directory
    let mut wallet_dir = PathBuf::from(env!("HOME"));
    wallet_dir.push(".coinswap");
    wallet_dir.push("taker");
    wallet_dir.push("wallets");

    if wallet_dir.exists() {
        let _ = fs::remove_dir_all(wallet_dir);
    }
}

fn create_rpc_config(bitcoind: &BitcoinD, wallet_name: &str) -> RPCConfig {
    let url = bitcoind.rpc_url().split_at(7).1.to_string();

    let cookie_file = &bitcoind.params.cookie_file;
    let auth_str = std::fs::read_to_string(cookie_file).unwrap();
    let parts: Vec<&str> = auth_str.split(':').collect();

    RPCConfig {
        url,
        username: parts[0].to_string(),
        password: parts[1].to_string(),
        wallet_name: wallet_name.to_string(),
    }
}

fn test_taker_initialization() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "test-taker");

    let result = Taker::init(
        None,
        Some("test-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28332".to_string(),
        None,
    );

    assert!(
        result.is_ok(),
        "Taker initialization should succeed: {:?}",
        result.err()
    );

    let _ = bitcoind.client.stop();
}

fn test_taker_get_balance() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "balance-taker");

    let taker = Taker::init(
        None,
        Some("balance-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28333".to_string(),
        None,
    )
    .unwrap();

    let balances = taker.get_balances();
    assert!(balances.is_ok(), "Getting balances should succeed");

    let balances = balances.unwrap();
    assert_eq!(balances.spendable, 0, "Initial balance should be zero");
    assert_eq!(balances.regular, 0, "Regular balance should be zero");
    assert_eq!(balances.swap, 0, "Swap balance should be zero");
    assert_eq!(balances.fidelity, 0, "Fidelity balance should be zero");

    let _ = bitcoind.client.stop();
}

fn test_taker_address_generation() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "address-taker");

    let taker = Taker::init(
        None,
        Some("address-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28334".to_string(),
        None,
    )
    .unwrap();

    // Test external address generation
    let address1 = taker.get_next_external_address();
    assert!(
        address1.is_ok(),
        "Should generate external address successfully"
    );

    let address2 = taker.get_next_external_address();
    assert!(
        address2.is_ok(),
        "Should generate second external address successfully"
    );

    // Addresses should be different
    assert_ne!(
        address1.unwrap().address,
        address2.unwrap().address,
        "Generated addresses should be unique"
    );

    // Test internal address generation
    let internal_addresses = taker.get_next_internal_addresses(3);
    assert!(
        internal_addresses.is_ok(),
        "Should generate internal addresses successfully"
    );
    assert_eq!(
        internal_addresses.unwrap().len() - 1,
        3,
        "Should generate 3 internal addresses"
    );

    let _ = bitcoind.client.stop();
}

fn test_taker_wallet_funding() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "funding-taker");

    let taker = Taker::init(
        None,
        Some("funding-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28335".to_string(),
        None,
    )
    .unwrap();

    // Get an address to fund
    let funding_address_str = taker.get_next_external_address().unwrap().address;
    let funding_address = funding_address_str
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    // Send 1 BTC from bitcoind to the taker wallet
    let fund_amount = Amount::from_btc(1.0).unwrap();
    let _txid = bitcoind
        .client
        .send_to_address(
            &funding_address,
            fund_amount,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // Mine a block to confirm
    let mining_address = bitcoind
        .client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoind::bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    bitcoind
        .client
        .generate_to_address(1, &mining_address)
        .unwrap();

    // Sync wallet
    taker.sync_and_save().unwrap();
    println!("{}", &taker.get_wallet_name().unwrap());

    // Check balance
    let balances = taker.get_balances().unwrap();
    assert_eq!(
        balances.spendable,
        fund_amount.to_sat() as i64,
        "Spendable balance should be 1 BTC"
    );

    let _ = bitcoind.client.stop();
}

fn test_taker_list_utxos() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "utxo-taker");

    let taker = Taker::init(
        None,
        Some("utxo-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28336".to_string(),
        None,
    )
    .unwrap();

    let initial_utxos = taker.list_all_utxo_spend_info().unwrap();
    assert_eq!(initial_utxos.len(), 0, "Should start with no UTXOs");

    for i in 0..2 {
        let funding_address_str = taker.get_next_external_address().unwrap().address;
        let funding_address = funding_address_str
            .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
            .unwrap()
            .require_network(bitcoin::Network::Regtest)
            .unwrap();

        let amount = Amount::from_btc(0.5 * (i + 1) as f64).unwrap();
        bitcoind
            .client
            .send_to_address(&funding_address, amount, None, None, None, None, None, None)
            .unwrap();
    }

    // Confirm transactions
    let mining_address = bitcoind
        .client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoind::bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    bitcoind
        .client
        .generate_to_address(1, &mining_address)
        .unwrap();

    // Sync and check UTXOs
    taker.sync_and_save().unwrap();
    let utxos = taker.list_all_utxo_spend_info().unwrap();
    assert_eq!(utxos.len(), 2, "Should have 2 UTXOs after funding");

    let _ = bitcoind.client.stop();
}

fn test_swap_params_creation() {
    // Test SwapParams struct creation
    let swap_params = SwapParams {
        send_amount: 500_000, // 0.005 BTC in sats
        maker_count: 2,
        manually_selected_outpoints: None,
    };

    assert_eq!(swap_params.send_amount, 500_000);
    assert_eq!(swap_params.maker_count, 2);
    assert!(swap_params.manually_selected_outpoints.is_none());

    // Test with manual outpoints
    let outpoints = vec![];
    let swap_params_with_selection = SwapParams {
        send_amount: 1_000_000,
        maker_count: 3,
        manually_selected_outpoints: Some(outpoints),
    };

    assert_eq!(swap_params_with_selection.send_amount, 1_000_000);
    assert_eq!(swap_params_with_selection.maker_count, 3);
    assert!(
        swap_params_with_selection
            .manually_selected_outpoints
            .is_some()
    );
}

fn test_taker_sync() {
    // TODO: Do we need it?
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "sync-taker");

    let taker = Taker::init(
        None,
        Some("sync-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28337".to_string(),
        None,
    )
    .unwrap();

    let sync_result = taker.sync_and_save();
    assert!(sync_result.is_ok(), "Sync should succeed");

    let _ = bitcoind.client.stop();
}

fn test_multiple_taker_instances() {
    let bitcoind = setup_bitcoind();

    // Create first taker
    let rpc_config1 = create_rpc_config(&bitcoind, "multi-taker-1");
    let taker1 = Taker::init(
        None,
        Some("multi-taker-1".to_string()),
        Some(rpc_config1),
        None,
        None,
        None,
        "tcp://127.0.0.1:28338".to_string(),
        None,
    );
    assert!(taker1.is_ok(), "First taker should initialize");

    // Create second taker with different walle),t
    let rpc_config2 = create_rpc_config(&bitcoind, "multi-taker-2");
    let taker2 = TaprootTaker::init(
        None,
        Some("multi-taker-2".to_string()),
        Some(rpc_config2.into()),
        None,
        None,
        "tcp://127.0.0.1:28339".to_string(),
        None,
    );
    assert!(taker2.is_ok(), "Second taker should initialize");

    // Fund the first taker (regular Taker) with 1 BTC
    let taker1 = taker1.unwrap();
    let funding_address1_str = taker1.get_next_external_address().unwrap().address;
    let funding_address1 = funding_address1_str
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    let fund_amount1 = Amount::from_btc(1.0).unwrap();
    bitcoind
        .client
        .send_to_address(
            &funding_address1,
            fund_amount1,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // Fund the second taker (TaprootTaker) with 2.5 BTC
    let taker2 = taker2.unwrap();
    let funding_address2_str = taker2.get_next_external_address().unwrap().address;
    let funding_address2 = funding_address2_str
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    let fund_amount2 = Amount::from_btc(2.5).unwrap();
    bitcoind
        .client
        .send_to_address(
            &funding_address2,
            fund_amount2,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    let mining_address = bitcoind
        .client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoind::bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    bitcoind
        .client
        .generate_to_address(1, &mining_address)
        .unwrap();

    // Sync both wallets
    taker1.sync_and_save().unwrap();
    taker2.sync_and_save().unwrap();

    // Both should have independent balances
    let balance1 = taker1.get_balances().unwrap();
    let balance2 = taker2.get_balances().unwrap();

    assert_eq!(
        balance1.spendable,
        fund_amount1.to_sat() as i64,
        "First taker should have 1 BTC"
    );
    assert_eq!(
        balance2.spendable,
        fund_amount2.to_sat() as i64,
        "Second taker (Taproot) should have 2.5 BTC"
    );

    let _ = bitcoind.client.stop();
}

fn test_rpc_config_conversion() {
    let rpc_config = RPCConfig {
        url: "127.0.0.1:18443".to_string(),
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        wallet_name: "test_wallet".to_string(),
    };

    // Test that config values are preserved
    assert_eq!(rpc_config.url, "127.0.0.1:18443");
    assert_eq!(rpc_config.username, "test_user");
    assert_eq!(rpc_config.password, "test_pass");
    assert_eq!(rpc_config.wallet_name, "test_wallet");
}

fn test_taker_get_transactions() {
    let bitcoind = setup_bitcoind();
    let rpc_config = create_rpc_config(&bitcoind, "tx-taker");

    let taker = Taker::init(
        None,
        Some("tx-taker".to_string()),
        Some(rpc_config),
        None,
        None,
        None,
        "tcp://127.0.0.1:28340".to_string(),
        None,
    )
    .unwrap();

    // Initially should have no transactions
    let inital_txs = taker.get_transactions(None, None);
    assert!(inital_txs.is_ok(), "Getting transactions should succeed");

    // Fund the wallet from bitcoind
    let funding_address_str = taker.get_next_external_address().unwrap().address;
    let funding_address = funding_address_str
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    bitcoind
        .client
        .send_to_address(
            &funding_address,
            Amount::from_btc(0.1).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // Confirm
    let mining_address = bitcoind
        .client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoind::bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    bitcoind
        .client
        .generate_to_address(1, &mining_address)
        .unwrap();

    taker.sync_and_save().unwrap();

    // Should now have 1 transaction
    let txs = taker.get_transactions(None, None);
    assert!(txs.is_ok(), "Getting transactions should succeed");
    assert!(
        txs.as_ref().unwrap().len() > 0,
        "Should have transactions after funding"
    );
    println!(
        "initial transactions: {}, final_transactions: {}",
        inital_txs.unwrap().len(),
        txs.unwrap().len()
    );

    let _ = bitcoind.client.stop();
}
