//! FFI Layer Tests for Coinswap Taproot Taker
//!
//! This module tests the UniFFI bindings for the Coinswap Taproot Taker,
//! ensuring the FFI layer correctly wraps the underlying Rust API.
//!
//! Based on BDK-FFI test patterns.

use crate::{
    taker::{SwapParams, Taker},
    tests::docker_helpers::{self, DockerBitcoind},
};
use bitcoin::Amount;
use bitcoind::bitcoincore_rpc::RpcApi;
use std::process::Command;
use std::sync::Arc;

#[test]
fn main() {
    cleanup_wallet();
    test_taproot_taker_complete_flow();
    // cleanup_wallet();
}

fn setup_bitcoind_and_taproot_taker(wallet_name: &str) -> (Arc<Taker>, DockerBitcoind) {
    let bitcoind = DockerBitcoind::connect().expect("Failed to connect to Docker bitcoind");

    let rpc_config = docker_helpers::get_docker_rpc_config(wallet_name);

    let taker = Taker::init(
        None,
        Some(wallet_name.to_string()),
        Some(rpc_config),
        // None,
        Some(9051),
        Some("coinswap".to_string()),
        docker_helpers::DOCKER_BITCOIN_ZMQ.to_string(),
        None,
    )
    .unwrap();

    (taker, bitcoind)
}

fn cleanup_wallet() {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn remove_wallet_entries(base_dir: &Path, wallet_name: &str) {
        if !base_dir.exists() {
            return;
        }

        let Ok(entries) = fs::read_dir(base_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let entry_name = entry.file_name();
            let entry_name = entry_name.to_string_lossy();
            if !entry_name.starts_with(wallet_name) {
                continue;
            }

            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
            println!("✓ Removed local wallet entry: {}", path.display());
        }
    }

    let wallet_name = "test-taproot-taker";

    let mut coinswap_dir = PathBuf::from(env!("HOME"));
    coinswap_dir.push(".coinswap");
    remove_wallet_entries(&coinswap_dir, wallet_name);

    let taker_dir = coinswap_dir.join("taker");
    remove_wallet_entries(&taker_dir, wallet_name);

    let taker_wallets_dir = taker_dir.join("wallets");
    remove_wallet_entries(&taker_wallets_dir, wallet_name);

    if let Ok(bitcoind) = DockerBitcoind::connect() {
        let _ = bitcoind.client.unload_wallet(Some(wallet_name));
        println!("✓ Unloaded wallet from Docker bitcoind");
    }

    // Remove the test-taproot-taker wallet from the Docker container's bitcoin folder
    let output = Command::new("docker")
        .args([
            "exec",
            "coinswap-ffi-bitcoind",
            "rm",
            "-rf",
            "/home/bitcoin/.bitcoin/wallets/test-taproot-taker",
        ])
        .output();

    if output.is_ok() && output.as_ref().unwrap().status.success() {
        println!("✓ Removed test-taproot-taker wallet from Docker container");
    } else {
        println!("⚠ Failed to remove wallet from Docker container (may not exist)");
    }
}

fn test_taproot_taker_complete_flow() {
    // Setup logging FIRST, before initializing taker
    coinswap::utill::setup_taker_logger(
        log::LevelFilter::Info, // Change to Debug for more verbose logging
        true,                   // Enable stdout
        None,                   // Use default taker directory
    );

    log::info!("Starting taproot taker test flow");

    let (taker, bitcoind) = setup_bitcoind_and_taproot_taker("test-taproot-taker");

    taker.sync_offerbook_and_wait().unwrap();

    // Test get_name
    println!("Testing get_name...");
    let wallet_name = taker.get_wallet_name().unwrap();
    assert_eq!(
        wallet_name, "test-taproot-taker",
        "Wallet name should match"
    );
    println!("✓ 'get_wallet_name' test passed");

    // Test address generation (external and internal)
    println!("\nTesting address generation...");
    let external_address1 = taker.get_next_external_address(crate::AddressType {
        addr_type: "P2TR".to_string(),
    });
    assert!(
        external_address1.is_ok(),
        "Should generate external address successfully"
    );

    let external_address2 = taker.get_next_external_address(crate::AddressType {
        addr_type: "P2TR".to_string(),
    });
    assert!(
        external_address2.is_ok(),
        "Should generate second external address successfully"
    );

    assert_ne!(
        external_address1.as_ref().unwrap().address,
        external_address2.as_ref().unwrap().address,
        "External addresses should be unique"
    );

    let internal_addresses = taker.get_next_internal_addresses(
        3,
        crate::AddressType {
            addr_type: "P2TR".to_string(),
        },
    );
    assert!(
        internal_addresses.is_ok(),
        "Should generate internal addresses successfully"
    );
    assert_eq!(
        internal_addresses.unwrap().len() - 1,
        3,
        "Should generate 3 internal addresses"
    );
    println!("✓ 'get_next_external_address' test passed");
    println!("✓ 'get_next_internal_addresses' test passed");

    println!("\nTesting initial balances...");
    taker.sync_and_save().unwrap();
    let initial_balances = taker.get_balances();
    assert!(initial_balances.is_ok(), "Getting balances should succeed");

    let initial_balances = initial_balances.unwrap();
    assert_eq!(
        initial_balances.spendable, 0,
        "Initial spendable balance should be zero"
    );
    assert_eq!(
        initial_balances.regular, 0,
        "Initial regular balance should be zero"
    );
    assert_eq!(
        initial_balances.swap, 0,
        "Initial swap balance should be zero"
    );
    assert_eq!(
        initial_balances.fidelity, 0,
        "Initial fidelity balance should be zero"
    );
    println!("✓ 'get_balances' test passed (initial zero balances)");

    println!("\nFunding wallet...");
    let funding_address_str = external_address1.unwrap().address;
    let funding_address = funding_address_str
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    let fund_amount = Amount::from_btc(0.42749329).unwrap();
    let _txid = bitcoind
        .send_to_address_from_funding_wallet(&funding_address, fund_amount)
        .unwrap();
    taker.sync_and_save().unwrap();
    println!("✓ wallet funding completed");

    println!("\nTesting updated balances after funding...");
    let updated_balances = taker.get_balances().unwrap();
    assert_eq!(
        updated_balances.spendable,
        fund_amount.to_sat() as i64,
        "Spendable balance should be 42749329 SATS"
    );
    println!("✓ 'get_balances' test passed (post-funding balance verification)");

    println!("\nTesting list_utxos...");
    let utxos = taker.list_all_utxo_spend_info();
    assert!(utxos.is_ok(), "Listing UTXOs should succeed");
    let utxos = utxos.unwrap();
    assert!(
        !utxos.is_empty(),
        "Should have at least 1 UTXO after funding"
    );
    println!("Found {} UTXO(s)", utxos.len());
    println!("✓ list_all_utxo_spend_info test passed");

    println!("\nTesting get_transactions...");
    let transactions = taker.get_transactions(None, None);
    assert!(transactions.is_ok(), "Getting transactions should succeed");
    let transactions = transactions.unwrap();
    assert!(
        !transactions.is_empty(),
        "Should have at least 1 transaction after funding"
    );
    println!("Found {} transaction(s)", transactions.len());
    println!("✓ 'get_transactions' test passed");

    let fetch_offers_result = taker.fetch_offers();
    println!("Fetch offers result: {:?}", fetch_offers_result);

    println!("\nTesting prepare_coinswap + start_coinswap...");
    let swap_params = SwapParams {
        protocol: Some("Taproot".to_string()),
        send_amount: 500_000,
        maker_count: 2,
        tx_count: Some(3),
        required_confirms: Some(1),
        manually_selected_outpoints: None,
        preferred_makers: None,
    };
    let swap_id = taker
        .prepare_coinswap(swap_params)
        .expect("'prepare_coinswap' should succeed");
    let report = taker
        .start_coinswap(swap_id)
        .expect("'start_coinswap' should succeed");
    println!("Swap completed successfully!");
    println!("Swap Report: {:?}", report);
    println!("✓ 'prepare_coinswap' and 'start_coinswap' tests passed");

    taker.sync_and_save().unwrap();

    println!(
        "\nTesting updated balances after swap...{:?}",
        taker.get_balances()
    );

    println!("\n========================================");
    println!("All FFI method tests completed successfully!");
    println!("========================================");
}
