//! Docker test helpers for the FFI taker integration test.
//!
//! Backs the single `swap_test` against the regtest stack in
//! `docker-compose.electrum-regtest.yml` (bitcoind + electrs + tor + 1 RPC
//! maker + 1 Electrum maker). Provides the connection config for both backends
//! and one `run_swap` flow used by every taker in the test.

use crate::{
    taker::{SwapParams, Taker},
    types::{AddressType, BackendConfig, Balances},
};
use bitcoin::Amount;
use bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use std::{path::PathBuf, sync::Arc, thread, time::Duration};

pub const BITCOIN_RPC_URL: &str = "http://localhost:18442";
pub const BITCOIN_RPC_USER: &str = "user";
pub const BITCOIN_RPC_PASS: &str = "password";
pub const BITCOIN_ZMQ: &str = "tcp://127.0.0.1:28332";
pub const ELECTRUM_URL: &str = "tcp://localhost:50001";
pub const BITCOIND_CONTAINER: &str = "coinswap-bitcoind";
/// Public Nostr relays for maker discovery. Passed explicitly so the test does
/// not depend on the crate's compiled-in default (which is a local relay when
/// built with the `integration-test` feature) or on a persisted config file.
pub const NOSTR_RELAYS: &[&str] = &["wss://nos.lol", "wss://relay.damus.io"];

/// Which backend a taker connects through.
#[derive(Clone, Copy)]
pub enum Backend {
    /// Bitcoin Core RPC (makerd1).
    Rpc,
    /// Electrum / electrs (makerd2).
    Electrum,
}

/// One taker scenario: backend + swap protocol.
pub struct Swap {
    pub name: &'static str,
    pub wallet: &'static str,
    pub backend: Backend,
    /// "Legacy" or "Taproot".
    pub protocol: &'static str,
    /// "P2WPKH" (Legacy) or "P2TR" (Taproot).
    pub addr_type: &'static str,
}

/// Connect to the Docker bitcoind's `test` (funding) wallet.
fn funding_client() -> Client {
    Client::new(
        &format!("{BITCOIN_RPC_URL}/wallet/test"),
        Auth::UserPass(BITCOIN_RPC_USER.into(), BITCOIN_RPC_PASS.into()),
    )
    .expect("connect to Docker bitcoind test wallet")
}

/// Send `amount` to `address` from the funding wallet and mine one block.
fn fund_address(client: &Client, address: &str, amount: Amount) {
    let addr = address
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();
    client
        .send_to_address(&addr, amount, None, None, None, None, None, None)
        .expect("send funding");
    let mine_to = client
        .get_new_address(None, None)
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();
    client.generate_to_address(1, &mine_to).unwrap();
}

/// Remove any wallet artifacts for `wallet` (local dirs + Docker bitcoind + container).
pub fn cleanup_wallet(wallet: &str) {
    use std::{fs, process::Command};

    let home = PathBuf::from(env!("HOME"));
    for dir in [
        home.join(".coinswap"),
        home.join(".coinswap/taker"),
        home.join(".coinswap/taker/wallets"),
    ] {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(wallet) {
                    let p = entry.path();
                    let _ = if p.is_dir() {
                        fs::remove_dir_all(&p)
                    } else {
                        fs::remove_file(&p)
                    };
                }
            }
        }
    }
    let _ = Command::new("docker")
        .args([
            "exec",
            BITCOIND_CONTAINER,
            "rm",
            "-rf",
            &format!("/home/bitcoin/.bitcoin/wallets/{wallet}"),
        ])
        .output();
    let _ = fs::remove_dir_all(test_data_dir(wallet));
}

/// Initialise a taker for `swap`'s backend.
fn init_taker(swap: &Swap) -> Arc<Taker> {
    let (rpc_config, backend_config) = match swap.backend {
        Backend::Rpc => (
            Some(crate::types::RPCConfig {
                url: "localhost:18442".into(),
                username: BITCOIN_RPC_USER.into(),
                password: BITCOIN_RPC_PASS.into(),
                wallet_name: swap.wallet.into(),
            }),
            None,
        ),
        Backend::Electrum => (
            None,
            Some(BackendConfig {
                kind: "electrum".into(),
                url: Some(ELECTRUM_URL.into()),
                username: None,
                password: None,
                wallet_name: None,
                zmq_addr: None,
                socks5: None,
                timeout: None,
                poll_interval_secs: None,
                max_retries: None,
            }),
        ),
    };

    Taker::init(
        Some(test_data_dir(swap.wallet).display().to_string()),
        Some(swap.wallet.into()),
        rpc_config,
        Some(9051),
        Some("coinswap".into()),
        BITCOIN_ZMQ.into(),
        None,
        // Explicitly use the public Nostr relays for maker discovery.
        Some(NOSTR_RELAYS.iter().map(|r| r.to_string()).collect()),
        backend_config,
    )
    .expect("init taker")
}

fn test_data_dir(wallet: &str) -> PathBuf {
    std::env::temp_dir().join("coinswap-ffi").join(wallet)
}

/// Fund the taker with `total` sats across 4 fresh external addresses.
fn fund_taker(taker: &Taker, funding: &Client, addr_type: &str, total: u64) {
    let quarter = total / 4;
    for i in 0..4 {
        let part = if i == 3 { total - quarter * 3 } else { quarter };
        let addr = taker
            .get_next_external_address(AddressType {
                addr_type: addr_type.into(),
            })
            .expect("funding address")
            .addr;
        fund_address(funding, &addr, Amount::from_sat(part));
    }
}

/// Sync until spendable reaches `target` (tolerates Electrum indexing lag).
fn wait_for_spendable(taker: &Taker, target: i64) -> Balances {
    for _ in 0..30 {
        taker.sync_and_save().unwrap();
        let b = taker.get_balances().unwrap();
        if b.spendable >= target {
            return b;
        }
        thread::sleep(Duration::from_secs(3));
    }
    taker.get_balances().unwrap()
}

/// Run one taker end-to-end: init → fund → sync → 2-maker coinswap → assert.
/// `send` sats are swapped; the taker is funded with `2 * send`.
pub fn run_swap(swap: &Swap, send: u64) {
    println!(
        "\n=== {} ({} / {}) ===",
        swap.name, swap.protocol, swap.addr_type
    );
    cleanup_wallet(swap.wallet);

    let funding = funding_client();
    let taker = init_taker(swap);
    taker.sync_offerbook_and_wait().unwrap();
    let offers = taker.fetch_offers().expect("fetch offers");
    let active_count = offers
        .makers
        .iter()
        .filter(|maker| maker.state.state_type == "Good")
        .filter(|maker| {
            maker
                .protocol
                .as_ref()
                .map(|protocol| {
                    protocol.protocol_type == swap.protocol || protocol.protocol_type == "Unified"
                })
                .unwrap_or(false)
        })
        .count();
    println!(
        "{}: offerbook has {} total makers, {} active {} makers",
        swap.name,
        offers.makers.len(),
        active_count,
        swap.protocol
    );
    for maker in &offers.makers {
        println!(
            "{}: maker {} state={} protocol={}",
            swap.name,
            maker.address.address,
            maker.state.state_type,
            maker
                .protocol
                .as_ref()
                .map(|protocol| protocol.protocol_type.as_str())
                .unwrap_or("None")
        );
    }

    assert_eq!(taker.get_wallet_name().unwrap(), swap.wallet);

    // Fund with 2x the swap amount and wait for it to be visible.
    fund_taker(&taker, &funding, swap.addr_type, send * 2);
    let funded = wait_for_spendable(&taker, (send * 2) as i64);
    assert_eq!(
        funded.spendable,
        (send * 2) as i64,
        "{}: spendable should equal funded amount",
        swap.name
    );

    // 2-maker coinswap, single funding tx (tx_count = 1).
    let swap_id = taker
        .prepare_coinswap(SwapParams {
            protocol: Some(swap.protocol.into()),
            send_amount: send,
            maker_count: 2,
            tx_count: Some(1),
            required_confirms: Some(1),
            manually_selected_outpoints: None,
            preferred_makers: None,
        })
        .expect("prepare_coinswap");
    let report = taker.start_coinswap(swap_id).expect("start_coinswap");
    assert_eq!(
        report.makers_count,
        Some(2),
        "{}: swap should route through 2 makers",
        swap.name
    );
    // `status` is a display string (may carry ANSI color); match on content.
    assert!(
        report.status.to_uppercase().contains("SUCCESS"),
        "{}: swap status was {:?}",
        swap.name,
        report.status
    );
    println!("✓ {} passed (swap_id {})", swap.name, report.swap_id);
}
