//! Coinswap Wallet FFI bindings
//!
//! This module provides UniFFI bindings for the coinswap wallet functionality.

use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use bitcoin::{Address, Amount};
use bitcoind::bitcoincore_rpc::Auth;
use coinswap::wallet::{
    Balances as CoinswapBalances, Destination as CoinswapDestination,
    RPCConfig as CoinswapRPCConfig, Wallet as CoinswapWallet,
    WalletError as CoinswapWalletError,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum WalletError {
    #[error("IO error: {msg}")]
    IO { msg: String },
    #[error("RPC error: {msg}")]
    RPC { msg: String },
    #[error("General error: {msg}")]
    General { msg: String },
    #[error("JSON error: {msg}")]
    JSON { msg: String },
    #[error("Network error: {msg}")]
    Network { msg: String },
    #[error("Address parsing error: {msg}")]
    AddressParse { msg: String },
}

impl From<CoinswapWalletError> for WalletError {
    fn from(error: CoinswapWalletError) -> Self {
        match error {
            CoinswapWalletError::IO(e) => WalletError::IO { msg: e.to_string() },
            CoinswapWalletError::Rpc(e) => WalletError::RPC { msg: e.to_string() },
            CoinswapWalletError::Json(e) => WalletError::JSON { msg: e.to_string() },
            CoinswapWalletError::General(msg) => WalletError::General { msg },
            _ => WalletError::General {
                msg: format!("Wallet error: {:?}", error),
            },
        }
    }
}

#[derive(uniffi::Record)]
pub struct Balances {
    pub regular: u64,
    pub swap: u64,
    pub contract: u64,
    pub fidelity: u64,
    pub spendable: u64,
}

impl From<CoinswapBalances> for Balances {
    fn from(balances: CoinswapBalances) -> Self {
        Self {
            regular: balances.regular.to_sat(),
            swap: balances.swap.to_sat(),
            contract: balances.contract.to_sat(),
            fidelity: balances.fidelity.to_sat(),
            spendable: balances.spendable.to_sat(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct RPCConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub wallet_name: String,
}

impl From<RPCConfig> for CoinswapRPCConfig {
    fn from(config: RPCConfig) -> Self {
        Self {
            url: config.url,
            auth: Auth::UserPass(config.username, config.password),
            wallet_name: config.wallet_name,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum Destination {
    Sweep {
        address: String,
    },
    Multi {
        outputs: HashMap<String, u64>,  // Vec<(String, u64)> cant work
        op_return_data: Option<Vec<u8>>,
    },
    MultiDynamic {
        amount: u64,
        addresses: Vec<String>,
    },
}

impl TryFrom<Destination> for CoinswapDestination {
    type Error = WalletError;

    fn try_from(dest: Destination) -> Result<Self, Self::Error> {
        match dest {
            Destination::Sweep { address } => {
                let addr = address
                    .parse::<Address<bitcoin::address::NetworkUnchecked>>()
                    .map_err(|e| WalletError::AddressParse { msg: e.to_string() })?
                    .assume_checked();
                Ok(CoinswapDestination::Sweep(addr))
            }
            Destination::Multi {
                outputs,
                op_return_data,
            } => {
                let mut parsed_outputs = Vec::new();
                for (addr_str, amount_sats) in outputs {
                    let addr = addr_str
                        .parse::<Address<bitcoin::address::NetworkUnchecked>>()
                        .map_err(|e| WalletError::AddressParse { msg: e.to_string() })?
                        .assume_checked();
                    let amount = Amount::from_sat(amount_sats);
                    parsed_outputs.push((addr, amount));
                }
                let op_return = op_return_data.map(|data| data.into_boxed_slice());
                Ok(CoinswapDestination::Multi {
                    outputs: parsed_outputs,
                    op_return_data: op_return,
                })
            }
            Destination::MultiDynamic { amount, addresses } => {
                let amount = Amount::from_sat(amount);
                let mut parsed_addresses = Vec::new();
                for addr_str in addresses {
                    let addr = addr_str
                        .parse::<Address<bitcoin::address::NetworkUnchecked>>()
                        .map_err(|e| WalletError::AddressParse { msg: e.to_string() })?
                        .assume_checked();
                    parsed_addresses.push(addr);
                }
                Ok(CoinswapDestination::MultiDynamic(amount, parsed_addresses))
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct WalletBackup {
    pub file_name: String,
}

// Main wallet wrapper
#[derive(uniffi::Object)]
pub struct Wallet {
    inner: CoinswapWallet,
}

impl Wallet {
    #[uniffi::constructor]
    pub fn init(path: String, rpc_config: RPCConfig) -> Result<Arc<Self>, WalletError> {
        let path = Path::new(&path);
        let config = CoinswapRPCConfig::from(rpc_config);

        let wallet = CoinswapWallet::init(path, &config, None)?;

        Ok(Arc::new(Self { inner: wallet }))
    }

    pub fn get_balances(&self) -> Result<Balances, WalletError> {
        let balances = self.inner.get_balances()?;
        Ok(Balances::from(balances))
    }

    pub fn get_next_external_address(&self) -> Result<String, WalletError> {
        let addr = format!(
            "Address generation requires mutable access - not implemented in immutable context"
        );
        Err(WalletError::General { msg: addr })
    }

    /// Get the wallet name
    pub fn get_name(&self) -> String {
        self.inner.get_name().to_string()
    }

    pub fn get_external_index(&self) -> u32 {
        *self.inner.get_external_index()
    }

    pub fn send_transaction(
        &mut self,
        destination: Destination,
        fee_rate: Option<f64>,
    ) -> Result<String, WalletError> {
        let dest = CoinswapDestination::try_from(destination)?;

        let utxos = self.inner.list_descriptor_utxo_spend_info();

        if utxos.is_empty() {
            return Err(WalletError::General {
                msg: "No UTXOs available for spending".to_string(),
            });
        }

        let tx = self.inner.spend_from_wallet(fee_rate.unwrap(), dest, &utxos)?;
        let txid = self.inner.send_tx(&tx)?;

        Ok(txid.to_string())
    }

    pub fn sync(&self) -> Result<(), WalletError> {
        // This method requires mutable access in the original
        Err(WalletError::General {
            msg: "Sync requires mutable access - use sync_and_save from external interface"
                .to_string(),
        })
    }

    pub fn backup(&self, path: String) -> Result<(), WalletError> {
        let backup_path = Path::new(&path);
        self.inner.backup(backup_path, None)?;
        Ok(())
    }

    pub fn lock_unspendable_utxos(&self) -> Result<(), WalletError> {
        self.inner.lock_unspendable_utxos()?;
        Ok(())
    }
}

#[uniffi::export]
pub fn create_default_rpc_config() -> RPCConfig {
    RPCConfig {
        url: "localhost:18443".to_string(),
        username: "regtestrpcuser".to_string(),
        password: "regtestrpcpass".to_string(),
        wallet_name: "coinswap-wallet".to_string(),
    }
}
