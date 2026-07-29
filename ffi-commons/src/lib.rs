pub mod taker;
pub mod types;

#[cfg(test)]
mod tests;

pub use taker::*;
pub use types::*;

#[uniffi::export]
pub fn coinswap_ffi_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

uniffi::setup_scaffolding!("coinswap");
