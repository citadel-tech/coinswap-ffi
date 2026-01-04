//! Coinswap Taker UniFFI bindings
//!
//! This module provides UniFFI bindings for the coinswap taker functionality.

use crate::{
    AddressType,
    types::{
        Address, Amount, Balances, GetTransactionResultDetail, ListTransactionResult,
        ListUnspentResultEntry, Offer, OfferBook, OutPoint, RPCConfig, ScriptBuf, SignedAmountSats,
        SwapReport, TakerError, TotalUtxoInfo, Txid, UtxoSpendInfo, WalletTxInfo,
    },
};
use coinswap::{
    bitcoin::{Amount as coinswapAmount, OutPoint as coinswapOutPoint, Txid as coinswapTxid},
    taker::api::{SwapParams as CoinswapSwapParams, Taker as CoinswapTaker},
    wallet::{RPCConfig as CoinswapRPCConfig, UTXOSpendInfo as csUtxoSpendInfo},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(uniffi::Record)]
pub struct SwapParams {
    /// Total Amount
    pub send_amount: u64,
    /// How many hops (number of makers)
    pub maker_count: u32,
    /// User selected UTXOs (optional)
    pub manually_selected_outpoints: Option<Vec<OutPoint>>,
}

impl TryFrom<SwapParams> for CoinswapSwapParams {
    type Error = TakerError;

    fn try_from(params: SwapParams) -> Result<Self, Self::Error> {
        let send_amount = coinswapAmount::from_sat(params.send_amount);

        let manually_selected_outpoints = params
            .manually_selected_outpoints
            .map(|outpoints| -> Result<Vec<coinswapOutPoint>, TakerError> {
                outpoints
                    .into_iter()
                    .map(|op| {
                        let txid = op.txid.value.parse::<coinswapTxid>().map_err(|e| {
                            TakerError::General {
                                msg: format!("Invalid txid: {}", e),
                            }
                        })?;
                        Ok(coinswapOutPoint::new(txid, op.vout))
                    })
                    .collect()
            })
            .transpose()?;

        Ok(CoinswapSwapParams {
            send_amount,
            maker_count: params.maker_count as usize,
            manually_selected_outpoints,
        })
    }
}

#[derive(uniffi::Object)]
pub struct Taker {
    taker: Mutex<CoinswapTaker>,
}

#[uniffi::export]
impl Taker {
    #[uniffi::constructor]
    // #[allow(clippy::too_many_arguments)]
    pub fn init(
        data_dir: Option<String>,
        wallet_file_name: Option<String>,
        rpc_config: Option<RPCConfig>,
        // _behavior: Option<TakerBehavior>,
        control_port: Option<u16>,
        tor_auth_password: Option<String>,
        zmq_addr: String,
        password: Option<String>,
    ) -> Result<Arc<Self>, TakerError> {
        let data_dir = data_dir.map(PathBuf::from);
        let rpc_config = rpc_config.map(CoinswapRPCConfig::from);

        let taker = CoinswapTaker::init(
            data_dir,
            wallet_file_name,
            rpc_config,
            // #[cfg(feature = "integration-test")]
            // _behavior.unwrap_or(TakerBehavior::Normal).into(),
            control_port,
            tor_auth_password,
            zmq_addr,
            password,
        )?;

        Ok(Arc::new(Self {
            taker: Mutex::new(taker),
        }))
    }

    pub fn setup_logging(
        &self,
        data_dir: Option<String>,
        log_level: String,
    ) -> Result<(), TakerError> {
        let path = data_dir.map(PathBuf::from);
        let level = match log_level.to_lowercase().as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        };
        coinswap::utill::setup_taker_logger(level, true, path);
        Ok(())
    }

    pub fn do_coinswap(&self, swap_params: SwapParams) -> Result<Option<SwapReport>, TakerError> {
        let params = CoinswapSwapParams::try_from(swap_params)?;
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let swap_report = taker.do_coinswap(params)?;
        Ok(swap_report.map(SwapReport::from))
    }

    pub fn get_transactions(
        &self,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<ListTransactionResult>, TakerError> {
        let txns = self
            .taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet()
            .get_transactions(count.map(|c| c as usize), skip.map(|s| s as usize))
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get Transactions Error: {:?}", e),
            })?;

        Ok(txns
            .into_iter()
            .map(|tx| ListTransactionResult {
                info: WalletTxInfo {
                    confirmations: tx.info.confirmations,
                    blockhash: tx.info.blockhash.map(|h| h.to_string()),
                    blockindex: tx.info.blockindex.map(|i| i as u32),
                    blocktime: tx.info.blocktime.map(|t| t as i64),
                    blockheight: tx.info.blockheight,
                    txid: Txid::from(tx.info.txid),
                    time: tx.info.time as i64,
                    timereceived: tx.info.timereceived as i64,
                    bip125_replaceable: format!("{:?}", tx.info.bip125_replaceable),
                    wallet_conflicts: tx
                        .info
                        .wallet_conflicts
                        .into_iter()
                        .map(Txid::from)
                        .collect(),
                },
                detail: GetTransactionResultDetail {
                    address: tx.detail.address.map(|a| Address::from(a.assume_checked())),
                    category: format!("{:?}", tx.detail.category),
                    amount: SignedAmountSats::from(tx.detail.amount),
                    label: tx.detail.label,
                    vout: tx.detail.vout,
                    fee: tx.detail.fee.map(SignedAmountSats::from),
                    abandoned: tx.detail.abandoned,
                },
                trusted: tx.trusted,
                comment: tx.comment,
            })
            .collect())
    }

    pub fn get_next_internal_addresses(
        &self,
        count: u32,
        address_type: AddressType,
    ) -> Result<Vec<Address>, TakerError> {
        let cs_address_type = coinswap::wallet::AddressType::try_from(address_type)?;
        let internal_addresses = self
            .taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet()
            .get_next_internal_addresses(count, cs_address_type)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get internal addresses error: {:?}", e),
            })?;
        Ok(internal_addresses.into_iter().map(Address::from).collect())
    }

    pub fn get_next_external_address(
        &self,
        address_type: AddressType,
    ) -> Result<Address, TakerError> {
        let cs_address_type = coinswap::wallet::AddressType::try_from(address_type)?;
        let external_address = self
            .taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet_mut()
            .get_next_external_address(cs_address_type)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get next external address error: {:?}", e),
            })?;
        Ok(Address::from(external_address))
    }

    pub fn list_all_utxo_spend_info(&self) -> Result<Vec<TotalUtxoInfo>, TakerError> {
        let entries = self
            .taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet()
            .list_all_utxo_spend_info();

        Ok(entries
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
                    csUtxoSpendInfo::SeedCoin {
                        path,
                        input_value,
                        address_type: _,
                    } => UtxoSpendInfo {
                        spend_type: "SeedCoin".to_string(),
                        path: Some(path),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                    },
                    csUtxoSpendInfo::IncomingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "IncomingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
                    },
                    csUtxoSpendInfo::OutgoingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "OutgoingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript)),
                        input_value: None,
                        index: None,
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
                    },
                    csUtxoSpendInfo::FidelityBondCoin { index, input_value } => UtxoSpendInfo {
                        spend_type: "FidelityBondCoin".to_string(),
                        path: None,
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: Some(index),
                    },
                    csUtxoSpendInfo::SweptCoin {
                        path,
                        input_value,
                        address_type: _,
                    } => UtxoSpendInfo {
                        spend_type: "SweptCoin".to_string(),
                        path: Some(path),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                    },
                };

                TotalUtxoInfo {
                    list_unspent_result_entry: utxo,
                    utxo_spend_info: spend_info,
                }
            })
            .collect())
    }

    pub fn backup(
        &self,
        destination_path: String,
        password: Option<String>,
    ) -> Result<(), TakerError> {
        self.taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet_mut()
            .backup_wallet_gui_app(destination_path, password)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Backup error: {:?}", e),
            })?;
        Ok(())
    }

    pub fn lock_unspendable_utxos(&self) -> Result<(), TakerError> {
        self.taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet()
            .lock_unspendable_utxos()
            .map_err(|e| TakerError::Wallet {
                msg: format!("Lock error: {:?}", e),
            })?;
        Ok(())
    }

    pub fn send_to_address(
        &self,
        address: String,
        amount: i64,
        fee_rate: Option<f64>,
        manually_selected_outpoints: Option<Vec<OutPoint>>,
    ) -> Result<Txid, TakerError> {
        let manually_selected_outpoints = manually_selected_outpoints
            .map(|outpoints| -> Result<Vec<coinswapOutPoint>, TakerError> {
                outpoints
                    .into_iter()
                    .map(|op| {
                        let txid = op.txid.value.parse::<coinswapTxid>().map_err(|e| {
                            TakerError::General {
                                msg: format!("Invalid txid: {}", e),
                            }
                        })?;
                        Ok(coinswapOutPoint::new(txid, op.vout))
                    })
                    .collect()
            })
            .transpose()?;

        let txid = self
            .taker
            .lock()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire taker lock".to_string(),
            })?
            .get_wallet_mut()
            .send_to_address(
                amount as u64,
                address,
                fee_rate,
                manually_selected_outpoints,
            )
            .map_err(|e| TakerError::Wallet {
                msg: format!("Send to Address error: {:?}", e),
            })?;
        Ok(txid.into())
    }

    pub fn get_balances(&self) -> Result<Balances, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let balances = taker
            .get_wallet()
            .get_balances()
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get balances error: {:?}", e),
            })?;
        Ok(Balances::from(balances))
    }

    pub fn sync_and_save(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker
            .get_wallet_mut()
            .sync_and_save()
            .map_err(|e| TakerError::Wallet {
                msg: format!("Sync wallet error: {:?}", e),
            })?;
        Ok(())
    }

    pub fn is_offerbook_syncing(&self) -> Result<bool, TakerError> {
        let taker = self.taker.lock().map_err(|e| TakerError::General {
            msg: format!(
                "Failed to acquire taker lock for offerbook sync check: {:?}",
                e
            ),
        })?;
        Ok(taker.is_offerbook_syncing())
    }

    pub fn run_offer_sync_now(&self) -> Result<(), TakerError> {
        let taker = self.taker.lock().map_err(|e| TakerError::General {
            msg: format!(
                "Failed to acquire taker lock for offerbook sync check: {:?}",
                e
            ),
        })?;
        taker.run_offer_sync_now();
        Ok(())
    }

    pub fn fetch_offers(&self) -> Result<OfferBook, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;

        let offerbook = taker.fetch_offers().map_err(|e| TakerError::Network {
            msg: format!("Fetch offers error: {:?}", e),
        })?;

        Ok(OfferBook::from(&offerbook))
    }

    pub fn display_offer(&self, maker_offer: &Offer) -> Result<String, TakerError> {
        let offer_json = serde_json::json!({
            "base_fee": maker_offer.base_fee,
            "amount_relative_fee_pct": maker_offer.amount_relative_fee_pct,
            "time_relative_fee_pct": maker_offer.time_relative_fee_pct,
            "required_confirms": maker_offer.required_confirms,
            "minimum_locktime": maker_offer.minimum_locktime,
            "max_size": maker_offer.max_size,
            "min_size": maker_offer.min_size,
        });

        serde_json::to_string_pretty(&offer_json)
            .map_err(|e| TakerError::General { msg: e.to_string() })
    }

    pub fn get_wallet_name(&self) -> Result<String, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        Ok(taker.get_wallet().get_name().to_string())
    }

    pub fn recover_from_swap(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker.recover_from_swap()?;
        Ok(())
    }

    pub fn fetch_all_makers(&self) -> Result<Vec<String>, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;

        let offerbook = taker.fetch_offers()?;
        let all_makers = offerbook.all_makers();

        let addresses = all_makers
            .into_iter()
            .map(|maker| maker.address.to_string())
            .collect();

        Ok(addresses)
    }
}
