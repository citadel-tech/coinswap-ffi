pub mod taker;
pub mod wallet;
#[path = "wallet-napi.rs"]
pub mod wallet_napi;
#[path = "taker-napi.rs"]
pub mod taker_napi;

pub use taker::*;
pub use wallet::*;

uniffi::setup_scaffolding!("coinswap");
