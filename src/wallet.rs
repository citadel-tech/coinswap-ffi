//! Coinswap Wallet FFI bindings
//!
//! This module provides UniFFI bindings for the coinswap wallet functionality.

use bitcoin::ScriptBuf as csScriptBuf;
use bitcoin::address::NetworkUnchecked;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use bitcoin::{hashes::hash160::Hash, Address, Amount as coinswapAmount};
use bitcoind::bitcoincore_rpc::{json::ListUnspentResultEntry as csListUnspentResultEntry, Auth};
use coinswap::wallet::{
    Balances as CoinswapBalances, Destination as CoinswapDestination,
    RPCConfig as CoinswapRPCConfig, Wallet as CoinswapWallet,
    WalletError as CoinswapWalletError
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

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct Amount(pub coinswapAmount);

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct ScriptBuf(pub csScriptBuf);

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct ListUnspentResultEntry(pub csListUnspentResultEntry);

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
pub struct UTXO {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,  // satoshis
    pub confirmations: u32,
    pub spendable: bool,
    pub solvable: bool,
    pub safe: bool,
}

#[derive(uniffi::Record)]
pub struct UTXOInfo {
    pub utxo: UTXO,
    pub spend_info: UTXOSpendInfo,
}

#[derive(uniffi::Record)]
pub struct RPCConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub wallet_name: String,
}

#[derive(uniffi::Enum)]
pub enum UTXOSpendInfo {
    /// Seed Coin
    SeedCoin { path: String, input_value: Arc<Amount> },
    /// Coins that we have received in a swap
    IncomingSwapCoin { multisig_redeemscript: Arc<ScriptBuf> },
    /// Coins that we have sent in a swap
    OutgoingSwapCoin { multisig_redeemscript: Arc<ScriptBuf> },
    /// Timelock Contract
    TimelockContract {
        swapcoin_multisig_redeemscript: Arc<ScriptBuf>,
        input_value: Arc<Amount>,
    },
    /// HashLockContract
    HashlockContract {
        swapcoin_multisig_redeemscript: Arc<ScriptBuf>,
        input_value: Arc<Amount>,
    },
    /// Fidelity Bond Coin
    FidelityBondCoin { index: u32, input_value: Arc<Amount> },
    ///Swept incoming swap coin
    SweptCoin {
        path: String,
        input_value: Arc<Amount>,
        original_multisig_redeemscript: Arc<ScriptBuf>,
    },
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

#[derive(uniffi::Record)]
pub struct WalletBackup {
    pub file_name: String,
}

// Main wallet wrapper
#[derive(uniffi::Object)]
pub struct Wallet {
    inner: CoinswapWallet,
}

#[uniffi::export]
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
        Err(WalletError::General { 
            msg: "Address generation requires mutable access - not implemented in immutable context".to_string()
        })
    }

    /// Get the wallet name
    pub fn get_name(&self) -> String {
        self.inner.get_name().to_string()
    }

    pub fn get_external_index(&self) -> u32 {
        *self.inner.get_external_index()
    }

    pub fn list_all_utxos(&self) -> Vec<ListUnspentResultEntry, UTXOSpendInfo> {
        self.inner.list_all_utxo_spend_info();
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

// // Note: Since coinswap::wallet::UTXOSpendInfo is private, we need to implement 
// // conversion logic manually based on the wallet's list_all_utxo_spend_info method
// impl UTXOSpendInfo {
//     /// Convert from the internal representation that would come from the wallet
//     /// This is a placeholder - the actual conversion would need to be implemented
//     /// based on how the wallet exposes this information
//     pub fn from_raw_data(
//         utxo: &ListUnspentResultEntry,
//         // Additional parameters would be needed based on the wallet's internal logic
//     ) -> Self {
//         // This is a simplified conversion - in practice, you'd need access to
//         // the wallet's internal state to determine the correct UTXOSpendInfo type
//         UTXOSpendInfo::SeedCoin {
//             path: "unknown".to_string(),
//             input_value: utxo.amount.to_sat(),
//         }
//     }
// }

// impl From<&ListUnspentResultEntry> for UTXO {
//     fn from(entry: &ListUnspentResultEntry) -> Self {
//         Self {
//             txid: entry.txid.to_string(),
//             vout: entry.vout,
//             amount: entry.amount.to_sat(),
//             confirmations: entry.confirmations,
//             spendable: entry.spendable,
//             solvable: entry.solvable,
//             safe: entry.safe,
//         }
//     }
// }
