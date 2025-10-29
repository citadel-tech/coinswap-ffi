//! Coinswap Wallet FFI bindings
//!
//! This module provides UniFFI bindings for the coinswap wallet functionality.

use bitcoin::{ScriptBuf as csScriptBuf, Txid as csTxid};
use std::path::Path;
use std::sync::Arc;
use bitcoin::Amount as coinswapAmount;
use bitcoind::bitcoincore_rpc::Auth;
use coinswap::wallet::{
    Balances as CoinswapBalances, 
    RPCConfig as CoinswapRPCConfig, Wallet as CoinswapWallet,
    WalletError as CoinswapWalletError, UTXOSpendInfo as csUTXOSpendInfo
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
pub struct Txid(pub csTxid);

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct ScriptBuf(pub csScriptBuf);

// #[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
// pub struct ListUnspentResultEntry(pub csListUnspentResultEntry);

#[derive(uniffi::Record)]
pub struct ListUnspentResultEntry {
    pub txid: Arc<Txid>,
    pub vout: u32,
    pub address: Option<String>,
    pub label: Option<String>,
    pub script_pub_key: Arc<ScriptBuf>,
    pub amount: Arc<Amount>,
    pub confirmations: u32,
    pub redeem_script: Option<Arc<ScriptBuf>>,
    pub witness_script: Option<Arc<ScriptBuf>>,
    pub spendable: bool,
    pub solvable: bool,
    pub desc: Option<String>,
    pub safe: bool,
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
pub struct UTXOWithSpendInfo {
    pub utxo: ListUnspentResultEntry,
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

    pub fn list_all_utxos(&self) -> Vec<UTXOWithSpendInfo> {
        let entries = self.inner.list_all_utxo_spend_info();
        entries
            .into_iter()
            .map(|(cs_utxo, cs_info)| {
                let utxo = ListUnspentResultEntry {
                    txid: Arc::new(Txid(cs_utxo.txid)),
                    vout: cs_utxo.vout,
                    address: cs_utxo.address.map(|a| a.assume_checked().to_string()),
                    label: cs_utxo.label,
                    script_pub_key: Arc::new(ScriptBuf(cs_utxo.script_pub_key)),
                    amount: Arc::new(Amount(cs_utxo.amount)),
                    confirmations: cs_utxo.confirmations,
                    redeem_script: cs_utxo.redeem_script.map(|s| Arc::new(ScriptBuf(s))),
                    witness_script: cs_utxo.witness_script.map(|s| Arc::new(ScriptBuf(s))),
                    spendable: cs_utxo.spendable,
                    solvable: cs_utxo.solvable,
                    desc: cs_utxo.descriptor,
                    safe: cs_utxo.safe,
                };
                let spend_info = match cs_info {
                    csUTXOSpendInfo::SeedCoin { path, input_value } => {
                        UTXOSpendInfo::SeedCoin { path, input_value: Arc::new(Amount(input_value)) }
                    }
                    csUTXOSpendInfo::IncomingSwapCoin { multisig_redeemscript } => {
                        UTXOSpendInfo::IncomingSwapCoin { multisig_redeemscript: Arc::new(ScriptBuf(multisig_redeemscript)) }
                    }
                    csUTXOSpendInfo::OutgoingSwapCoin { multisig_redeemscript } => {
                        UTXOSpendInfo::OutgoingSwapCoin { multisig_redeemscript: Arc::new(ScriptBuf(multisig_redeemscript)) }
                    }
                    csUTXOSpendInfo::TimelockContract { swapcoin_multisig_redeemscript, input_value } => {
                        UTXOSpendInfo::TimelockContract {
                            swapcoin_multisig_redeemscript: Arc::new(ScriptBuf(swapcoin_multisig_redeemscript)),
                            input_value: Arc::new(Amount(input_value)),
                        }
                    }
                    csUTXOSpendInfo::HashlockContract { swapcoin_multisig_redeemscript, input_value } => {
                        UTXOSpendInfo::HashlockContract {
                            swapcoin_multisig_redeemscript: Arc::new(ScriptBuf(swapcoin_multisig_redeemscript)),
                            input_value: Arc::new(Amount(input_value)),
                        }
                    }
                    csUTXOSpendInfo::FidelityBondCoin { index, input_value } => {
                        UTXOSpendInfo::FidelityBondCoin { index, input_value: Arc::new(Amount(input_value)) }
                    }
                    csUTXOSpendInfo::SweptCoin { path, input_value, original_multisig_redeemscript } => {
                        UTXOSpendInfo::SweptCoin {
                            path,
                            input_value: Arc::new(Amount(input_value)),
                            original_multisig_redeemscript: Arc::new(ScriptBuf(original_multisig_redeemscript)),
                        }
                    }
                };
                UTXOWithSpendInfo { utxo, spend_info }
            })
            .collect()
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