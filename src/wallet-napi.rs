//! Coinswap Wallet N-API bindings
//!
//! This module provides N-API bindings for the coinswap wallet functionality.

use bitcoin::Amount as coinswapAmount;
use bitcoin::{ScriptBuf as csScriptBuf, Txid as csTxid};
use bitcoind::bitcoincore_rpc::Auth;
use coinswap::wallet::{
    Balances as CoinswapBalances, RPCConfig as CoinswapRPCConfig, UTXOSpendInfo as csUTXOSpendInfo,
    Wallet as CoinswapWallet, WalletError as CoinswapWalletError,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;

#[napi]
pub enum WalletError {
    IO,
    RPC,
    General,
    JSON,
    Network,
    AddressParse,
}

impl From<CoinswapWalletError> for WalletError {
    fn from(error: CoinswapWalletError) -> Self {
        match error {
            CoinswapWalletError::IO(_) => WalletError::IO,
            CoinswapWalletError::Rpc(_) => WalletError::RPC,
            CoinswapWalletError::Json(_) => WalletError::JSON,
            CoinswapWalletError::General(_) => WalletError::General,
            _ => WalletError::General,
        }
    }
}

#[napi(object)]
pub struct Balances {
    pub regular: u32,
    pub swap: u32,
    pub contract: u32,
    pub fidelity: u32,
    pub spendable: u32,
}

#[napi(object)]
pub struct Amount {
    pub sats: u32,
}

impl From<coinswapAmount> for Amount {
    fn from(amount: coinswapAmount) -> Self {
        Self {
            sats: amount.to_sat() as u32,
        }
    }
}

#[napi(object)]
pub struct Txid {
    pub hex: String,
}

impl From<csTxid> for Txid {
    fn from(txid: csTxid) -> Self {
        Self {
            hex: txid.to_string(),
        }
    }
}

#[napi(object)]
pub struct ScriptBuf {
    pub hex: String,
}

impl From<csScriptBuf> for ScriptBuf {
    fn from(script: csScriptBuf) -> Self {
        Self {
            hex: hex::encode(script.as_bytes()),
        }
    }
}

#[napi(object)]
pub struct ListUnspentResultEntry {
    pub txid: Txid,
    pub vout: u32,
    pub address: Option<String>,
    pub label: Option<String>,
    pub script_pub_key: ScriptBuf,
    pub amount: Amount,
    pub confirmations: u32,
    pub redeem_script: Option<ScriptBuf>,
    pub witness_script: Option<ScriptBuf>,
    pub spendable: bool,
    pub solvable: bool,
    pub desc: Option<String>,
    pub safe: bool,
}

impl From<CoinswapBalances> for Balances {
    fn from(balances: CoinswapBalances) -> Self {
        Self {
            regular: balances.regular.to_sat() as u32,
            swap: balances.swap.to_sat() as u32,
            contract: balances.contract.to_sat() as u32,
            fidelity: balances.fidelity.to_sat() as u32,
            spendable: balances.spendable.to_sat() as u32,
        }
    }
}

#[napi(object)]
pub struct UTXOWithSpendInfo {
    pub utxo: ListUnspentResultEntry,
    pub spend_info: UTXOSpendInfo,
}

#[napi(object)]
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

#[napi]
pub struct Wallet {
    inner: CoinswapWallet,
}

#[napi]
impl Wallet {
    #[napi(constructor)]
    pub fn init(path: String, rpc_config: RPCConfig) -> Result<Self> {
        let path = Path::new(&path);
        let config = CoinswapRPCConfig::from(rpc_config);

        let wallet = CoinswapWallet::init(path, &config, None)
            .map_err(|e| napi::Error::from_reason(format!("Init error: {:?}", e)))?;

        Ok(Self { inner: wallet })
    }

    #[napi]
    pub fn get_balances(&self) -> Result<Balances> {
        let balances = self
            .inner
            .get_balances()
            .map_err(|e| napi::Error::from_reason(format!("Get balances error: {:?}", e)))?;
        Ok(Balances::from(balances))
    }

    #[napi]
    pub fn get_next_external_address(&self) -> Result<String> {
        Err(napi::Error::from_reason(
            "Address generation requires mutable access - not implemented in immutable context",
        ))
    }

    /// Get the wallet name
    #[napi]
    pub fn get_name(&self) -> String {
        self.inner.get_name().to_string()
    }

    #[napi]
    pub fn list_all_utxos(&self) -> Vec<UTXOWithSpendInfo> {
        let entries = self.inner.list_all_utxo_spend_info();
        entries
            .into_iter()
            .map(|(cs_utxo, cs_info)| {
                let utxo = ListUnspentResultEntry {
                    txid: Txid::from(cs_utxo.txid),
                    vout: cs_utxo.vout,
                    address: cs_utxo.address.map(|a| a.assume_checked().to_string()),
                    label: cs_utxo.label,
                    script_pub_key: ScriptBuf::from(cs_utxo.script_pub_key),
                    amount: Amount::from(cs_utxo.amount),
                    confirmations: cs_utxo.confirmations,
                    redeem_script: cs_utxo.redeem_script.map(ScriptBuf::from),
                    witness_script: cs_utxo.witness_script.map(ScriptBuf::from),
                    spendable: cs_utxo.spendable,
                    solvable: cs_utxo.solvable,
                    desc: cs_utxo.descriptor,
                    safe: cs_utxo.safe,
                };
                let spend_info = match cs_info {
                    csUTXOSpendInfo::SeedCoin { path, input_value } => UTXOSpendInfo {
                        spend_type: "SeedCoin".to_string(),
                        path: Some(path),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::IncomingSwapCoin {
                        multisig_redeemscript,
                    } => UTXOSpendInfo {
                        spend_type: "IncomingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::OutgoingSwapCoin {
                        multisig_redeemscript,
                    } => UTXOSpendInfo {
                        spend_type: "OutgoingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::TimelockContract {
                        swapcoin_multisig_redeemscript,
                        input_value,
                    } => UTXOSpendInfo {
                        spend_type: "TimelockContract".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(
                            swapcoin_multisig_redeemscript,
                        )),
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::HashlockContract {
                        swapcoin_multisig_redeemscript,
                        input_value,
                    } => UTXOSpendInfo {
                        spend_type: "HashlockContract".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(
                            swapcoin_multisig_redeemscript,
                        )),
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::FidelityBondCoin { index, input_value } => UTXOSpendInfo {
                        spend_type: "FidelityBondCoin".to_string(),
                        path: None,
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: Some(index),
                        original_multisig_redeemscript: None,
                    },
                    csUTXOSpendInfo::SweptCoin {
                        path,
                        input_value,
                        original_multisig_redeemscript,
                    } => UTXOSpendInfo {
                        spend_type: "SweptCoin".to_string(),
                        path: Some(path),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: Some(ScriptBuf::from(
                            original_multisig_redeemscript,
                        )),
                    },
                };
                UTXOWithSpendInfo { utxo, spend_info }
            })
            .collect()
    }

    #[napi]
    pub fn sync(&self) -> Result<()> {
        Err(napi::Error::from_reason(
            "Sync requires mutable access - use sync_and_save from external interface",
        ))
    }

    #[napi]
    pub fn backup(&self, path: String) -> Result<()> {
        let backup_path = Path::new(&path);
        self.inner
            .backup(backup_path, None)
            .map_err(|e| napi::Error::from_reason(format!("Backup error: {:?}", e)))?;
        Ok(())
    }

    #[napi]
    pub fn lock_unspendable_utxos(&self) -> Result<()> {
        self.inner
            .lock_unspendable_utxos()
            .map_err(|e| napi::Error::from_reason(format!("Lock error: {:?}", e)))?;
        Ok(())
    }
}

#[napi(object)]
pub struct WalletBackup {
    pub file_name: String,
}

#[napi(object)]
pub struct UTXOSpendInfo {
    pub spend_type: String,
    pub path: Option<String>,
    pub multisig_redeemscript: Option<ScriptBuf>,
    pub input_value: Option<Amount>,
    pub index: Option<u32>,
    pub original_multisig_redeemscript: Option<ScriptBuf>,
}

#[napi]
pub fn create_default_rpc_config() -> RPCConfig {
    RPCConfig {
        url: "localhost:18443".to_string(),
        username: "regtestrpcuser".to_string(),
        password: "regtestrpcpass".to_string(),
        wallet_name: "coinswap-wallet".to_string(),
    }
}
