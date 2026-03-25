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
    protocol::ProtocolVersion,
    taker::api::{
        ConnectionType, SwapParams as CoinswapSwapParams, Taker as CoinswapTaker, TakerInitConfig,
    },
    wallet::{RPCConfig as CoinswapRPCConfig, UTXOSpendInfo as csUtxoSpendInfo},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Swap specific parameters. These are user's policy and can differ among swaps.
/// SwapParams govern the criteria to find suitable set of makers from the offerbook.
///
/// If no maker matches with a given SwapParam, that coinswap round will fail.
#[derive(uniffi::Record)]
pub struct SwapParams {
    /// Protocol to use: Legacy or Taproot.
    pub protocol: Option<String>,
    /// Total Amount to Swap.
    pub send_amount: u64,
    /// How many hops.
    pub maker_count: u32,
    /// Number of transaction splits.
    pub tx_count: Option<u32>,
    /// Required funding confirmations.
    pub required_confirms: Option<u32>,
    /// User selected UTXOs
    pub manually_selected_outpoints: Option<Vec<OutPoint>>,
    /// Optional explicit maker addresses.
    pub preferred_makers: Option<Vec<String>>,
}

/// SwapParams govern the criteria to find suitable set of makers from the offerbook.
impl TryFrom<SwapParams> for CoinswapSwapParams {
    type Error = TakerError;

    /// Swap specific parameters. These are user's policy and can differ among swaps.
    fn try_from(params: SwapParams) -> Result<Self, Self::Error> {
        let protocol = match params.protocol.as_deref().unwrap_or("Legacy") {
            "Legacy" | "legacy" => ProtocolVersion::Legacy,
            "Taproot" | "taproot" => ProtocolVersion::Taproot,
            other => {
                return Err(TakerError::General {
                    msg: format!("Invalid protocol: {} (expected legacy or taproot)", other),
                });
            }
        };

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
            protocol,
            send_amount,
            maker_count: params.maker_count as usize,
            tx_count: params.tx_count.unwrap_or(1),
            required_confirms: params.required_confirms.unwrap_or(1),
            manually_selected_outpoints,
            preferred_makers: params.preferred_makers,
        })
    }
}

/// The Taker structure that performs bulk of the coinswap protocol. Taker connects
/// to multiple Makers and send protocol messages sequentially to them. The communication
/// sequence and corresponding SwapCoin infos are stored in `ongoing_swap_state`.
#[derive(uniffi::Object)]
pub struct Taker {
    /// The Taker structure that performs bulk of the coinswap protocol.
    taker: Mutex<CoinswapTaker>,
}

#[uniffi::export]
impl Taker {
    #[uniffi::constructor]
    // #[allow(clippy::too_many_arguments)]
    ///  Initializes a Taker structure.
    ///
    /// This function sets up a Taker instance with configurable parameters.
    /// It handles the initialization of data directories, wallet files, and RPC configurations.
    ///
    /// ### Parameters:
    /// - `data_dir`:
    ///   - `Some(value)`: Use the specified directory for storing data.
    ///   - `None`: Use the default data directory (e.g., for Linux: `~/.coinswap/taker`).
    /// - `wallet_file_name`:
    ///   - `Some(value)`: Attempt to load a wallet file named `value`. If it does not exist,
    ///     a new wallet with the given name will be created.
    ///   - `None`: Create a new wallet file with the default name `taker-wallet`.
    /// - If `rpc_config` = `None`: Use the default [`RPCConfig`]
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

        let init_config = TakerInitConfig {
            data_dir,
            wallet_file_name,
            rpc_config,
            control_port,
            tor_auth_password,
            socks_port: 9050,
            zmq_addr,
            password,
            connection_type: ConnectionType::Tor,
            nostr_relays: TakerInitConfig::default().nostr_relays,
        };

        let taker = CoinswapTaker::init(init_config)?;

        Ok(Arc::new(Self {
            taker: Mutex::new(taker),
        }))
    }

    /// Sets up the logger for the taker component.
    ///
    /// This method initializes the logging configuration for the taker, directing logs to both
    /// the console and a file. It sets the `RUST_LOG` environment variable to provide default
    /// log levels and configures log4rs with the specified filter level for fine-grained control
    /// of log verbosity.
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

    /// Prepares a coinswap and returns a swap id.
    pub fn prepare_coinswap(&self, swap_params: SwapParams) -> Result<String, TakerError> {
        let params = CoinswapSwapParams::try_from(swap_params)?;
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let summary = taker.prepare_coinswap(params)?;
        Ok(summary.swap_id)
    }

    /// Starts execution for a prepared coinswap.
    pub fn start_coinswap(&self, swap_id: String) -> Result<SwapReport, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let report = taker.start_coinswap(&swap_id)?;
        Ok(SwapReport::from(report))
    }

    /// Returns a list of recent Incoming Transactions (bydefault last 10)
    pub fn get_transactions(
        &self,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<ListTransactionResult>, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let wallet = taker.get_wallet().read().map_err(|_| TakerError::General {
            msg: "Failed to acquire wallet read lock".to_string(),
        })?;
        let txns = wallet
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

    /// Gets the next internal addresses from the HD keychain.
    pub fn get_next_internal_addresses(
        &self,
        count: u32,
        address_type: AddressType,
    ) -> Result<Vec<Address>, TakerError> {
        let cs_address_type = coinswap::wallet::AddressType::try_from(address_type)?;
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let wallet = taker.get_wallet().read().map_err(|_| TakerError::General {
            msg: "Failed to acquire wallet read lock".to_string(),
        })?;
        let internal_addresses = wallet
            .get_next_internal_addresses(count, cs_address_type)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get internal addresses error: {:?}", e),
            })?;
        Ok(internal_addresses.into_iter().map(Address::from).collect())
    }

    /// Gets the next external address from the HD keychain. Saves the wallet to disk
    pub fn get_next_external_address(
        &self,
        address_type: AddressType,
    ) -> Result<Address, TakerError> {
        let cs_address_type = coinswap::wallet::AddressType::try_from(address_type)?;
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let mut wallet = taker
            .get_wallet()
            .write()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire wallet write lock".to_string(),
            })?;
        let external_address = wallet
            .get_next_external_address(cs_address_type)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Get next external address error: {:?}", e),
            })?;
        Ok(Address::from(external_address))
    }

    /// Returns a list all utxos with their spend info tracked by the wallet.
    /// Optionally takes in an Utxo list to reduce RPC calls. If None is given, the
    /// full list of utxo is fetched from core rpc.
    pub fn list_all_utxo_spend_info(&self) -> Result<Vec<TotalUtxoInfo>, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let wallet = taker.get_wallet().read().map_err(|_| TakerError::General {
            msg: "Failed to acquire wallet read lock".to_string(),
        })?;
        let entries = wallet.list_all_utxo_spend_info();

        Ok(entries
            .into_iter()
            .map(|(cs_utxo, cs_info)| {
                let utxo = ListUnspentResultEntry {
                    txid: Txid::from(cs_utxo.txid),
                    vout: cs_utxo.vout,
                    address: cs_utxo
                        .address
                        .as_ref()
                        .map(|a| a.clone().assume_checked().to_string()),
                    label: cs_utxo.label.clone(),
                    script_pub_key: ScriptBuf::from(cs_utxo.script_pub_key.clone()),
                    amount: Amount::from(cs_utxo.amount),
                    confirmations: cs_utxo.confirmations,
                    redeem_script: cs_utxo.redeem_script.clone().map(ScriptBuf::from),
                    witness_script: cs_utxo.witness_script.clone().map(ScriptBuf::from),
                    spendable: cs_utxo.spendable,
                    solvable: cs_utxo.solvable,
                    desc: cs_utxo.descriptor.clone(),
                    safe: cs_utxo.safe,
                };
                let spend_info = match cs_info {
                    csUtxoSpendInfo::SeedCoin {
                        path,
                        input_value,
                        address_type: _,
                    } => UtxoSpendInfo {
                        spend_type: "SeedCoin".to_string(),
                        path: Some(path.to_string()),
                        multisig_redeemscript: None,
                        input_value: Some(Amount::from(input_value)),
                        index: None,
                    },
                    csUtxoSpendInfo::IncomingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "IncomingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript.clone())),
                        input_value: None,
                        index: None,
                    },
                    csUtxoSpendInfo::OutgoingSwapCoin {
                        multisig_redeemscript,
                    } => UtxoSpendInfo {
                        spend_type: "OutgoingSwapCoin".to_string(),
                        path: None,
                        multisig_redeemscript: Some(ScriptBuf::from(multisig_redeemscript.clone())),
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
                            swapcoin_multisig_redeemscript.clone(),
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
                            swapcoin_multisig_redeemscript.clone(),
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
                        path: Some(path.to_string()),
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

    /// Creates a wallet backup for GUI/FFI applications with optional encryption.
    ///
    /// This is a ffi-only wrapper around [`Wallet::backup`] that handles encryption
    /// material generation internally based on whether a password is provided.
    ///
    /// # Behavior
    ///
    /// - If `password` is `Some(pwd)` and not empty: Creates encrypted backup using the password
    /// - If `password` is `None` or empty string: Creates unencrypted backup (logs warning)
    /// - The backup is written as a `.json` file at the specified path
    ///
    /// # Parameters
    ///
    /// - `destination_path`: Destination file path for the backup (`.json`)
    /// - `password`: Optional password for encryption. Use `None` or empty string for plaintext backup
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Encrypted backup
    /// wallet.backup_gui_app("/path/to/backup".to_string(), Some("my_password".to_string()))?;
    ///
    /// // Unencrypted backup
    /// wallet.backup_gui_app("/path/to/backup".to_string(), None)?;
    /// ```
    pub fn backup(
        &self,
        destination_path: String,
        password: Option<String>,
    ) -> Result<(), TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker
            .get_wallet()
            .write()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire wallet write lock".to_string(),
            })?
            .backup_wallet_gui_app(destination_path, password)
            .map_err(|e| TakerError::Wallet {
                msg: format!("Backup error: {:?}", e),
            })?;
        Ok(())
    }

    /// Locks the fidelity and live_contract utxos which are not considered for spending from the wallet.
    pub fn lock_unspendable_utxos(&self) -> Result<(), TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker
            .get_wallet()
            .read()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire wallet read lock".to_string(),
            })?
            .lock_unspendable_utxos()
            .map_err(|e| TakerError::Wallet {
                msg: format!("Lock error: {:?}", e),
            })?;
        Ok(())
    }

    /// Sends specified Amount of Satoshis to an External Address
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

        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let txid = taker
            .get_wallet()
            .write()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire wallet write lock".to_string(),
            })?
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

    /// Calculates the total balances of different categories in the wallet.
    /// Includes regular, swap, contract, fidelity, and spendable (regular + swap) utxos.
    /// Optionally takes in a list of UTXOs to reduce rpc call. If None is provided,
    /// the full list is fetched from core rpc.
    pub fn get_balances(&self) -> Result<Balances, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let wallet = taker.get_wallet().read().map_err(|_| TakerError::General {
            msg: "Failed to acquire wallet read lock".to_string(),
        })?;
        let balances = wallet.get_balances().map_err(|e| TakerError::Wallet {
            msg: format!("Get balances error: {:?}", e),
        })?;
        Ok(Balances::from(balances))
    }

    /// Wrapper around Self::sync that also saves the wallet to disk.
    ///
    /// This method first synchronizes the wallet with the Bitcoin Core node,
    /// then persists the wallet state in the disk.
    pub fn sync_and_save(&self) -> Result<(), TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker
            .get_wallet()
            .write()
            .map_err(|_| TakerError::General {
                msg: "Failed to acquire wallet write lock".to_string(),
            })?
            .sync_and_save()
            .map_err(|e| TakerError::Wallet {
                msg: format!("Sync wallet error: {:?}", e),
            })?;
        Ok(())
    }

    pub fn sync_offerbook_and_wait(&self) -> Result<(), TakerError> {
        let taker = self.taker.lock().map_err(|e| TakerError::General {
            msg: format!(
                "Failed to acquire taker lock for offerbook sync check: {:?}",
                e
            ),
        })?;
        taker
            .sync_offerbook_and_wait()
            .map_err(|e| TakerError::Network {
                msg: format!("Offerbook sync error: {:?}", e),
            })?;
        Ok(())
    }

    /// Returns the OfferBook.
    pub fn fetch_offers(&self) -> Result<OfferBook, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;

        let offerbook = taker.fetch_offers().map_err(|e| TakerError::Network {
            msg: format!("Fetch offers error: {:?}", e),
        })?;

        Ok(OfferBook::from(&offerbook))
    }

    /// Displays a maker offer candidate in a human-readable format.
    /// If the maker does not yet have an offer, a partial view is shown.
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

    /// Get the wallet name
    pub fn get_wallet_name(&self) -> Result<String, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        let wallet = taker.get_wallet().read().map_err(|_| TakerError::General {
            msg: "Failed to acquire wallet read lock".to_string(),
        })?;
        Ok(wallet.get_name().to_string())
    }

    /// Recover from a bad swap
    pub fn recover_active_swap(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General {
            msg: "Failed to acquire taker lock".to_string(),
        })?;
        taker.recover_active_swap()?;
        Ok(())
    }

    /// Fetch all makers good, bad, and unresponsive
    pub fn fetch_all_makers(&self) -> Result<Vec<String>, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General {
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
