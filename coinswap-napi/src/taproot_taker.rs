//! Coinswap Taproot Taker N-API bindings
//!
//! This module provides N-API bindings for the coinswap taproot taker functionality.

use crate::types::{
  Address, Amount, Balances, FeeRates, GetTransactionResultDetail, ListTransactionResult,
  ListUnspentResultEntry, Offer, OfferBook, OutPoint, RPCConfig as RpcConfig, ScriptBuf,
  SignedAmountSats, SwapReport, Txid, UtxoSpendInfo, WalletTxInfo,
};
use coinswap::{
  bitcoin::{Amount as csAmount, OutPoint as BitcoinOutPoint, Txid as csTxid},
  fee_estimation::{BlockTarget, FeeEstimator},
  taker::api2::{SwapParams as CoinswapSwapParams, Taker as CoinswapTaker},
  wallet::{UTXOSpendInfo as csUtxoSpendInfo, ffi},
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{
  error::Error,
  fmt,
  path::PathBuf,
  str::FromStr,
  sync::Mutex,
};

#[napi]
#[derive(Debug)]
pub enum TakerError {
  Wallet,
  Protocol,
  Network,
  General,
  IO,
}

impl fmt::Display for TakerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TakerError::Wallet => write!(f, "Wallet error"),
      TakerError::Protocol => write!(f, "Protocol error"),
      TakerError::Network => write!(f, "Network error"),
      TakerError::General => write!(f, "General error"),
      TakerError::IO => write!(f, "IO error"),
    }
  }
}

impl AsRef<str> for TakerError {
  fn as_ref(&self) -> &str {
    match self {
      TakerError::Wallet => "Wallet error",
      TakerError::Protocol => "Protocol error",
      TakerError::Network => "Network error",
      TakerError::General => "General error",
      TakerError::IO => "IO error",
    }
  }
}

impl Error for TakerError {}

#[napi(object)]
pub struct SwapParams {
  pub send_amount: i64,
  pub maker_count: u32,
  pub manually_selected_outpoints: Option<Vec<OutPoint>>,
}

impl TryFrom<SwapParams> for CoinswapSwapParams {
  type Error = napi::Error;

  fn try_from(params: SwapParams) -> Result<Self> {
    let send_amount = csAmount::from_sat(params.send_amount as u64);

    let manually_selected_outpoints = params
      .manually_selected_outpoints
      .map(|outpoints| {
        outpoints
          .into_iter()
          .map(|outpoint| {
            let txid = csTxid::from_str(&outpoint.txid)
              .map_err(|e| napi::Error::from_reason(format!("Invalid txid: {:?}", e)))?;
            Ok(BitcoinOutPoint::new(txid, outpoint.vout))
          })
          .collect::<Result<Vec<_>, _>>()
      })
      .transpose()?;

    Ok(CoinswapSwapParams {
      send_amount,
      maker_count: params.maker_count as usize,
      manually_selected_outpoints,
    })
  }
}

#[napi]
pub enum TakerBehavior {
  Normal,
  DropConnectionAfterFullSetup,
  BroadcastContractAfterFullSetup,
}

impl From<TakerBehavior> for coinswap::taker::api::TakerBehavior {
  fn from(behavior: TakerBehavior) -> Self {
    match behavior {
      TakerBehavior::Normal => coinswap::taker::api::TakerBehavior::Normal,
      TakerBehavior::DropConnectionAfterFullSetup => {
        coinswap::taker::api::TakerBehavior::DropConnectionAfterFullSetup
      }
      TakerBehavior::BroadcastContractAfterFullSetup => {
        coinswap::taker::api::TakerBehavior::BroadcastContractAfterFullSetup
      }
    }
  }
}

#[napi]
pub struct Taker {
  inner: Mutex<CoinswapTaker>,
}

#[napi]
impl Taker {
  #[napi(constructor)]
  pub fn init(
    data_dir: Option<String>,
    wallet_file_name: Option<String>,
    rpc_config: Option<RpcConfig>,
    control_port: Option<u16>,
    tor_auth_password: Option<String>,
    zmq_addr: String,
    password: Option<String>
  ) -> Result<Self> {
    let data_dir = data_dir.map(PathBuf::from);
    let rpc_config = rpc_config.map(|cfg| cfg.into());

    let taker = CoinswapTaker::init(
      data_dir,
      wallet_file_name,
      rpc_config,
      control_port,
      tor_auth_password,
      zmq_addr,
      password,
    )
    .map_err(|e| napi::Error::from_reason(format!("Init error: {:?}", e)))?;

    Ok(Self {
      inner: Mutex::new(taker),
    })
  }

  #[napi]
  pub fn init_native_logging() {
    // For full backtrace panics
    console_error_panic_hook::set_once();
    // This makes ALL log:: macros from any crate go to the JS console
    console_log::init_with_level(log::Level::Trace).expect("Failed to initialize console_log");
    log::info!("Rust logging → Electron console is ready!");
  }

  /// Fetch fee estimates from Mempool.space API with automatic fallback to Esplora
  #[napi]
  pub fn fetch_mempool_fees() -> Result<FeeRates> {
    // mempool.space serves live data and is recommended for user facing apps over esplora, the latter serving historical(mov_avg)+live
    let fees = FeeEstimator::fetch_mempool_fees()
      .or_else(|mempool_err| {
        log::warn!(
          "Mempool.space API failed: {:?}, falling back to Esplora",
          mempool_err
        );
        FeeEstimator::fetch_esplora_fees()
      })
      .map_err(|e| napi::Error::from_reason(format!("Both fee APIs failed: {:?}", e)))?;

    let get = |target| {
      fees
        .get(&target)
        .ok_or_else(|| napi::Error::from_reason(format!("Missing fee for {:?}", target)))
    };

    Ok(FeeRates {
      fastest: *get(BlockTarget::Fastest)?,
      standard: *get(BlockTarget::Standard)?,
      economy: *get(BlockTarget::Economy)?,
    })
  }

  #[napi]
  pub fn do_coinswap(&self, swap_params: SwapParams) -> Result<Option<SwapReport>> {
    let params = CoinswapSwapParams::try_from(swap_params)?;
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    let swap_report = taker
      .do_coinswap(params)
      .map_err(|e| napi::Error::from_reason(format!("Send coinswap error: {:?}", e)))?;
    Ok(swap_report.map(SwapReport::from))
  }

  #[napi]
  pub fn get_transactions(
    &self,
    count: Option<u32>,
    skip: Option<u32>,
  ) -> Result<Vec<ListTransactionResult>> {
    let txns = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet()
      .get_transactions(count.map(|c| c as usize), skip.map(|s| s as usize))
      .map_err(|e| napi::Error::from_reason(format!("Get Transactions Error: {:?}", e)))?;

    Ok(
      txns
        .into_iter()
        .map(|tx| ListTransactionResult {
          info: {
            WalletTxInfo {
              confirmations: tx.info.confirmations,
              blockhash: tx.info.blockhash.map(|h| h.to_string()),
              blockindex: tx.info.blockindex.map(|i| i as u32),
              blocktime: tx.info.blocktime.map(|t| t as i64),
              blockheight: tx.info.blockheight,
              txid: Txid {
                hex: tx.info.txid.to_string(),
              },
              time: tx.info.time as i64,
              timereceived: tx.info.timereceived as i64,
              bip125_replaceable: format!("{:?}", tx.info.bip125_replaceable),
              wallet_conflicts: tx
                .info
                .wallet_conflicts
                .into_iter()
                .map(|txid| Txid {
                  hex: txid.to_string(),
                })
                .collect(),
            }
          },
          detail: {
            GetTransactionResultDetail {
              address: tx.detail.address.map(|addr| Address {
                address: addr.assume_checked().to_string(),
              }),
              category: format!("{:?}", tx.detail.category),
              amount: SignedAmountSats {
                sats: tx.detail.amount.to_sat(),
              },
              label: tx.detail.label,
              vout: tx.detail.vout,
              fee: tx.detail.fee.map(|f| SignedAmountSats { sats: f.to_sat() }),
              abandoned: tx.detail.abandoned,
            }
          },
          trusted: tx.trusted,
          comment: tx.comment,
        })
        .collect(),
    )
  }

  #[napi]
  pub fn get_next_internal_addresses(&self, count: u32) -> Result<Vec<Address>> {
    let internal_addresses = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet()
      .get_next_internal_addresses(count)
      .map_err(|e| napi::Error::from_reason(format!("Get internal addresses error: {:?}", e)))?;
    Ok(internal_addresses.into_iter().map(Address::from).collect())
  }

  #[napi]
  pub fn get_next_external_address(&mut self) -> Result<Address> {
    let external_address = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet_mut()
      .get_next_external_address()
      .map_err(|e| napi::Error::from_reason(format!("Get next external address error: {:?}", e)))?;
    Ok(Address::from(external_address))
  }

  // Get Name of the Wallet
  #[napi]
  pub fn get_name(&self) -> Result<String> {
    Ok(
      self
        .inner
        .lock()
        .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
        .get_wallet()
        .get_name()
        .to_string(),
    )
  }

  #[napi]
  pub fn list_all_utxo_spend_info(&self) -> Result<Vec<(ListUnspentResultEntry, UtxoSpendInfo)>> {
    let entries = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet()
      .list_all_utxo_spend_info();
    Ok(
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
              multisig_redeemscript: Some(ScriptBuf::from(swapcoin_multisig_redeemscript)),
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
              multisig_redeemscript: Some(ScriptBuf::from(swapcoin_multisig_redeemscript)),
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
              original_multisig_redeemscript: Some(ScriptBuf::from(original_multisig_redeemscript)),
            },
          };
          (utxo, spend_info)
        })
        .collect(),
    )
  }

  #[napi]
  pub fn backup(&self, destination_path: String, password: Option<String>) -> Result<()> {
    self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet_mut()
      .backup_wallet_gui_app(destination_path, password)
      .map_err(|e| napi::Error::from_reason(format!("App's Backup error: {:?}", e)))?;

    Ok(())
  }

  #[napi]
  pub fn restore_wallet_gui_app(
    data_dir: Option<String>,
    wallet_file_name: Option<String>,
    rpc_config: RpcConfig,
    backup_file: String,
    password: Option<String>,
  ) {
    let data_dir = data_dir.map(PathBuf::from);

    ffi::restore_wallet_gui_app(
      data_dir,
      wallet_file_name,
      rpc_config.into(),
      backup_file,
      password,
    );
  }

  #[napi]
  pub fn lock_unspendable_utxos(&self) -> Result<()> {
    self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet()
      .lock_unspendable_utxos()
      .map_err(|e| napi::Error::from_reason(format!("Lock error: {:?}", e)))?;
    Ok(())
  }

  #[napi]
  pub fn send_to_address(
    &mut self,
    address: String,
    amount: i64,
    fee_rate: Option<f64>,
    manually_selected_outpoints: Option<Vec<OutPoint>>,
  ) -> Result<Txid> {
    let manually_selected_outpoints = manually_selected_outpoints
      .map(|outpoints| {
        outpoints
          .into_iter()
          .map(|outpoint| {
            let txid = csTxid::from_str(&outpoint.txid)
              .map_err(|e| napi::Error::from_reason(format!("Invalid txid: {:?}", e)))?;
            Ok(BitcoinOutPoint::new(txid, outpoint.vout))
          })
          .collect::<Result<Vec<_>, _>>()
      })
      .transpose()?;
    let txid = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?
      .get_wallet_mut()
      .send_to_address(
        amount as u64,
        address,
        fee_rate,
        manually_selected_outpoints,
      )
      .map_err(|e| napi::Error::from_reason(format!("Send to Address error: {:?}", e)))?;
    Ok(txid.into())
  }

  /// Get wallet balances
  #[napi]
  pub fn get_balances(&self) -> Result<Balances> {
    let taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    let balances = taker
      .get_wallet()
      .get_balances()
      .map_err(|e| napi::Error::from_reason(format!("Get balances error: {:?}", e)))?;
    Ok(Balances::from(balances))
  }

  #[napi]
  pub fn sync_and_save(&mut self) -> Result<()> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    taker
      .get_wallet_mut()
      .sync_and_save()
      .map_err(|e| napi::Error::from_reason(format!("Sync wallet error: {:?}", e)))?;
    Ok(())
  }

  /// Sync the offerbook with available makers
  #[napi]
  pub fn sync_offerbook(&mut self) -> Result<()> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    taker
      .sync_offerbook()
      .map_err(|e| napi::Error::from_reason(format!("Sync offerbook error: {:?}", e)))?;
    Ok(())
  }

  /// Get basic information about all good makers (limited due to private fields)
  #[napi]
  pub fn get_all_good_makers(&self) -> Result<Vec<String>> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

    // Fetch fresh offers
    let offerbook = taker
      .fetch_offers()
      .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;
    let good_makers = offerbook.all_good_makers();

    // Since fields are private, we can only return addresses
    let addresses = good_makers
      .into_iter()
      .map(|maker| maker.address.to_string())
      .collect();

    Ok(addresses)
  }

  #[napi]
  pub fn display_offer(&self, maker_offer: Offer) -> Result<String> {
    let offer_json = serde_json::json!({
        "base_fee": maker_offer.base_fee,
        "amount_relative_fee_pct": maker_offer.amount_relative_fee_pct,
        "time_relative_fee_pct": maker_offer.time_relative_fee_pct,
        "required_confirms": maker_offer.required_confirms,
        "minimum_locktime": maker_offer.minimum_locktime,
        "max_size": maker_offer.max_size,
        "min_size": maker_offer.min_size,
        // "tweakable_point": maker_offer.tweakable_point,
        // "fidelity": maker_offer.fidelity
    });

    serde_json::to_string_pretty(&offer_json)
      .map_err(|e| napi::Error::from_reason(format!("JSON error: {:?}", e)))
  }

  /// Recover from a failed swap
  #[napi]
  pub fn recover_from_swap(&mut self) -> Result<()> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    taker
      .recover_from_swap()
      .map_err(|e| napi::Error::from_reason(format!("Recover error: {:?}", e)))?;
    Ok(())
  }

  #[napi]
  pub fn fetch_good_makers(&self) -> Result<Vec<String>> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

    let offerbook = taker
      .fetch_offers()
      .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;
    let all_good_makers = offerbook.all_good_makers();

    let addresses = all_good_makers
      .into_iter()
      .map(|maker| maker.address.to_string())
      .collect();

    Ok(addresses)
  }

  #[napi]
  pub fn fetch_all_makers(&self) -> Result<Vec<String>> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

    let offerbook = taker
      .fetch_offers()
      .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;
    let all_makers = offerbook.all_makers();

    let addresses = all_makers
      .into_iter()
      .map(|maker| maker.address.to_string())
      .collect();

    Ok(addresses)
  }

  #[napi]
  pub fn fetch_offers(&self) -> Result<OfferBook> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

    let offerbook = taker
      .fetch_offers()
      .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;

    Ok(OfferBook::from(offerbook))
  }
}

#[cfg(test)]
mod tests {
  use crate::types::{Amount, FidelityBond, LockTime, OutPoint, PublicKey};
  use coinswap::bitcoin::absolute::LockTime as csLockTime;

  #[test]
  fn test_locktime_conversion_basic() {
    let block_locktime = csLockTime::from_height(500000).unwrap();
    let napi_block = LockTime::from(block_locktime);

    let time_locktime = csLockTime::from_time(1234567890).unwrap();
    let napi_time = LockTime::from(time_locktime);

    println!("From Rust -> Javascript : ");
    println!("Block locktime: {:?} -> {:?}", block_locktime, napi_block);
    println!("Time locktime: {:?} -> {:?}", time_locktime, napi_time);
  }

  #[test]
  fn test_fidelity_bond_creation() {
    // Create a mock fidelity bond to see the structure
    let bond = FidelityBond {
      outpoint: OutPoint {
        txid: "abc123def456789".to_string(),
        vout: 0,
      },
      amount: Amount { sats: 100000 },
      lock_time: LockTime {
        lock_type: "Blocks".to_string(),
        value: 750000,
      },
      pubkey: PublicKey {
        compressed: true,
        inner: vec![2, 123, 45, 67, 89],
      },
      conf_height: Some(500000),
      cert_expiry: Some(144),
      is_spent: false,
    };

    println!("FidelityBond structure:");
    println!("  outpoint: {}:{}", bond.outpoint.txid, bond.outpoint.vout);
    println!("  amount: {} sats", bond.amount.sats);
    println!("  lock_time: {:?}", bond.lock_time);
    println!("  pubkey compressed: {}", bond.pubkey.compressed);
    println!("  pubkey bytes: {:?}", bond.pubkey.inner);
    println!("  conf_height: {:?}", bond.conf_height);
    println!("  cert_expiry: {:?}", bond.cert_expiry);
    println!("  is_spent: {}", bond.is_spent);
  }
}
