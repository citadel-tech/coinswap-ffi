//! Coinswap Wallet N-API bindings
//!
//! This module provides N-API bindings for the coinswap wallet functionality.

use coinswap::wallet::{
    UTXOSpendInfo as csUtxoSpendInfo,
    Wallet as CoinswapWallet, 
    WalletError as CoinswapWalletError,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;

// Import shared types
use crate::types::{Amount, Balances, RPCConfig, ScriptBuf, Txid};

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
pub struct Address {
    pub address: String,
}

impl From<bitcoin::Address> for Address {
    fn from(addr: bitcoin::Address) -> Self {
        Self {
            address: addr.to_string(),
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

#[napi(object)]
pub struct UtxoWithSpendInfo {
    pub utxo: ListUnspentResultEntry,
    pub spend_info: UtxoSpendInfo,
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
        let config = rpc_config.into();

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
    pub fn get_next_internal_addresses(&self, count: u32) -> Result<Vec<Address>> {
        let internal_addresses = self.inner.get_next_internal_addresses(count)
            .map_err(|e| napi::Error::from_reason(format!("Get internal addresses error: {:?}", e)))?;
        Ok(internal_addresses
            .into_iter()
            .map(Address::from)
            .collect())
    }

    #[napi]
    pub fn get_next_external_address(&mut self) -> Result<Address> {
        let external_address = self
            .inner
            .get_next_external_address()
            .map_err(|e| napi::Error::from_reason(format!("Get next external address error: {:?}", e)))?;
        Ok(Address::from(external_address))
    }

    /// Get the wallet name
    #[napi]
    pub fn get_name(&self) -> String {
        self.inner.get_name().to_string()
    }

    #[napi]
    pub fn list_all_utxos(&self) -> Vec<UtxoWithSpendInfo> {
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
                    csUtxoSpendInfo::SeedCoin { path, input_value } => UtxoSpendInfo {
                        spend_type: "SeedCoin".to_string(),
                        path: Some(path),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::IncomingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "IncomingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::OutgoingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "OutgoingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::TimelockContract {
                        swapcoin_multisig_redeemscript,
                        input_value,
                    } => UtxoSpendInfo {
                        spend_type: "TimelockContract".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(
                            swapcoin_multisig_redeemscript,
                        )),
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::HashlockContract {
                        swapcoin_multisig_redeemscript,
                        input_value,
                    } => UtxoSpendInfo {
                        spend_type: "HashlockContract".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(
                            swapcoin_multisig_redeemscript,
                        )),
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::FidelityBondCoin { index, input_value } => UtxoSpendInfo {
                        spend_type: "FidelityBondCoin".to_string(),
                        path: None,
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: Some(index),
                        original_multisig_redeemscript: None,
                    },
                    csUtxoSpendInfo::SweptCoin {
                        path,
                        input_value,
                        original_multisig_redeemscript,
                    } => UtxoSpendInfo {
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
                UtxoWithSpendInfo { utxo, spend_info }
            })
            .collect()
    }

    #[napi]
    pub fn sync_and_save(&mut self) -> Result<()> {
        self.inner
            .sync_and_save()
            .map_err(|e| napi::Error::from_reason(format!("Sync and save error: {:?}", e)))?;
        Ok(())
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
pub struct UtxoSpendInfo {
    pub spend_type: String,
    pub path: Option<String>,
    pub multisig_redeemscript: Option<ScriptBuf>,
    pub input_value: Option<Amount>,
    pub index: Option<u32>,
    pub original_multisig_redeemscript: Option<ScriptBuf>,
}
