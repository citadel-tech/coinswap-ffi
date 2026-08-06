//! FFI taker integration test: 4 takers × 2 makers.
//!
//! One test drives four takers sequentially against the Docker regtest stack
//! (1 RPC maker + 1 Electrum maker), covering the full backend × protocol
//! matrix. Each taker funds a fresh wallet and runs a 2-maker coinswap.

use crate::tests::docker_helpers::{Backend, Swap, run_swap};
/// Amount swapped by each taker, in sats. The taker is funded with 2×.
const SWAP_AMOUNT: u64 = 500_000;

// This test targets the PRODUCTION stack: makers expose Tor onion services and
// announce to the public Nostr relays, and the taker dials them over Tor. Run
// with a plain `cargo test` (no --features integration-test).
#[test]
fn main() {
    coinswap::utill::setup_taker_logger(log::LevelFilter::Info, true, None);

    let swaps = [
        Swap {
            name: "legacy_rpc",
            wallet: "test-legacy-rpc",
            backend: Backend::Rpc,
            protocol: "Legacy",
            addr_type: "P2WPKH",
        },
        Swap {
            name: "taproot_rpc",
            wallet: "test-taproot-rpc",
            backend: Backend::Rpc,
            protocol: "Taproot",
            addr_type: "P2TR",
        },
        Swap {
            name: "legacy_electrum",
            wallet: "test-legacy-electrum",
            backend: Backend::Electrum,
            protocol: "Legacy",
            addr_type: "P2WPKH",
        },
        Swap {
            name: "taproot_electrum",
            wallet: "test-taproot-electrum",
            backend: Backend::Electrum,
            protocol: "Taproot",
            addr_type: "P2TR",
        },
    ];

    for swap in &swaps {
        run_swap(swap, SWAP_AMOUNT);
    }

    println!("\n✓ all 4 takers (legacy/taproot × rpc/electrum) completed 2-maker swaps");
}
