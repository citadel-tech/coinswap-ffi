//! Coinswap Taker N-API bindings
//!
//! This module provides N-API bindings for the coinswap taker functionality.

use bitcoin::absolute::LockTime as csLocktime;
use bitcoin::PublicKey as csPublicKey;
use bitcoin::{OutPoint as BitcoinOutPoint, Txid};
use coinswap::protocol::messages::FidelityProof as csFidelityProof;
use coinswap::protocol::messages::Offer as csOffer;
use coinswap::taker::offers::MakerAddress as csMakerAddress;
use coinswap::taker::{
  api::{SwapParams as CoinswapSwapParams, Taker as CoinswapTaker},
  offers::{OfferAndAddress as csOfferAndAddress, OfferBook},
};
use coinswap::wallet::FidelityBond as csFidelityBond;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::error::Error;
use std::sync::Mutex;
use std::{fmt, string};
use std::{path::PathBuf, str::FromStr};

// Import shared types
use crate::types::{Amount, Balances, OutPoint, RPCConfig};

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
  /// Total Amount
  pub send_amount: Amount,
  /// How many hops (number of makers)
  pub maker_count: u32,
  /// User selected UTXOs (optional)
  pub manually_selected_outpoints: Option<Vec<OutPoint>>,
}

impl TryFrom<SwapParams> for CoinswapSwapParams {
  type Error = napi::Error;

  fn try_from(params: SwapParams) -> Result<Self> {
    let send_amount = bitcoin::Amount::from_sat(params.send_amount.sats as u64);

    let manually_selected_outpoints = params
      .manually_selected_outpoints
      .map(|outpoints| {
        outpoints
          .into_iter()
          .map(|outpoint| {
            let txid = Txid::from_str(&outpoint.txid)
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
    rpc_config: Option<RPCConfig>,
    behavior: Option<TakerBehavior>,
    control_port: Option<u16>,
    tor_auth_password: Option<String>,
  ) -> Result<Self> {
    let data_dir = data_dir.map(PathBuf::from);
    let rpc_config = rpc_config.map(|cfg| cfg.into());

    let taker = CoinswapTaker::init(
      data_dir,
      wallet_file_name,
      rpc_config,
      // #[cfg(feature = "integration-test")]
      // behavior.unwrap_or(TakerBehavior::Normal).into(),
      control_port,
      tor_auth_password,
    )
    .map_err(|e| napi::Error::from_reason(format!("Init error: {:?}", e)))?;

    Ok(Self {
      inner: Mutex::new(taker),
    })
  }

  #[napi]
  pub fn send_coinswap(&self, swap_params: SwapParams) -> Result<()> {
    let params = CoinswapSwapParams::try_from(swap_params)?;
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    taker
      .do_coinswap(params)
      .map_err(|e| napi::Error::from_reason(format!("Send coinswap error: {:?}", e)))?;
    Ok(())
  }

  #[napi]
  pub fn get_wallet_name(&self) -> Result<String> {
    let taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
    Ok(taker.get_wallet().get_name().to_string())
  }

  /// Get wallet balances
  #[napi]
  pub fn get_wallet_balances(&self) -> Result<Balances> {
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
  pub fn sync_wallet(&mut self) -> Result<()> {
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
  pub fn fetch_offers(&self) -> Result<OfferBookNapi> {
    let mut taker = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

    let offerbook = taker
      .fetch_offers()
      .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;

    Ok(OfferBookNapi::from(offerbook))
  }
}

#[napi]
pub fn create_swap_params(
  send_amount: Amount,
  maker_count: u32,
  outpoints: Vec<OutPoint>,
) -> SwapParams {
  SwapParams {
    send_amount,
    maker_count,
    manually_selected_outpoints: Some(outpoints),
  }
}

#[napi(object)]
pub struct PublicKey {
  pub compressed: bool,
  pub inner: Vec<u8>,
}

impl From<csPublicKey> for PublicKey {
  fn from(publickey: csPublicKey) -> Self {
    Self {
      compressed: publickey.compressed,
      inner: publickey.inner.serialize().to_vec(),
    }
  }
}

#[napi(object)]
pub struct FidelityProof {
  pub bond: FidelityBond,
  pub cert_hash: String,
  pub cert_sig: u8,
}

impl From<csFidelityProof> for FidelityProof {
  fn from(fidelityproof: csFidelityProof) -> Self {
    Self {
      bond: fidelityproof.bond.into(),
      cert_hash: "".to_string(),
      cert_sig: 0,
    }
  }
}

#[napi(object)]
pub struct FidelityBond {
  pub outpoint: OutPoint,
  pub amount: Amount,
  pub lock_time: LockTime,
  pub pubkey: PublicKey,
  pub conf_height: Option<u32>,
  pub cert_expiry: Option<u32>,
  pub is_spent: bool,
}

impl From<csFidelityBond> for FidelityBond {
  fn from(bond: csFidelityBond) -> Self {
    Self {
      outpoint: OutPoint {
        txid: "".to_string(),
        vout: 0,
      },
      amount: Amount::from(bond.amount),
      lock_time: LockTime::from(bond.lock_time),
      pubkey: PublicKey {
        compressed: true,
        inner: vec![],
      },
      conf_height: None,
      cert_expiry: None,
      is_spent: false,
    }
  }
}

#[napi(object)]
pub struct MakerStats {
  pub total_makers: u32,
  pub online_makers: u32,
  pub avg_base_fee: i64,
  pub avg_amount_relative_fee_pct: f64,
  pub avg_time_relative_fee_pct: f64,
  pub total_liquidity: i64,
  pub avg_min_size: i64,
  pub avg_max_size: i64,
}

#[napi(object)]
pub struct Offer {
  pub base_fee: i64,
  pub amount_relative_fee_pct: f64,
  pub time_relative_fee_pct: f64,
  pub required_confirms: u32,
  pub minimum_locktime: u16,
  pub max_size: i64,
  pub min_size: i64,
  pub tweakable_point: PublicKey,
  pub fidelity: FidelityProof,
}

impl From<csOffer> for Offer {
  fn from(offer: csOffer) -> Self {
    Self {
      base_fee: offer.base_fee as i64,
      amount_relative_fee_pct: offer.amount_relative_fee_pct,
      time_relative_fee_pct: offer.time_relative_fee_pct,
      required_confirms: offer.required_confirms,
      minimum_locktime: offer.minimum_locktime,
      max_size: offer.max_size as i64,
      min_size: offer.min_size as i64,
      tweakable_point: offer.tweakable_point.into(),
      fidelity: offer.fidelity.into(),
    }
  }
}

#[napi(object)]
pub struct OfferAndAddress {
  pub offer: Offer,
  pub address: MakerAddress,
  pub timestamp: String,
}

impl From<csOfferAndAddress> for OfferAndAddress {
  fn from(offer_and_addr: csOfferAndAddress) -> Self {
    Self {
      offer: Offer::from(offer_and_addr.offer),
      address: MakerAddress::from(offer_and_addr.address),
      timestamp: "".to_string(), // Static null value since we don't need it
    }
  }
}

#[napi(object)]
pub struct MakerAddress {
  pub address: String,
}

impl From<csMakerAddress> for MakerAddress {
  fn from(addr: csMakerAddress) -> Self {
    Self {
      address: addr.to_string(),
    }
  }
}

#[napi(object)]
pub struct OfferBookNapi {
  pub good_makers: Vec<OfferAndAddress>,
  pub all_makers: Vec<OfferAndAddress>,
}

impl From<&OfferBook> for OfferBookNapi {
  fn from(offerbook: &OfferBook) -> Self {
    Self {
      good_makers: offerbook
        .all_good_makers()
        .into_iter()
        .cloned()
        .map(OfferAndAddress::from)
        .collect(),
      all_makers: offerbook
        .all_makers()
        .into_iter()
        .cloned()
        .map(OfferAndAddress::from)
        .collect(),
    }
  }
}

#[napi(string_enum)]
#[derive(Debug)]
pub enum LockTime {
  Blocks,
  Seconds,
}

impl From<csLocktime> for LockTime {
  fn from(locktime: csLocktime) -> Self {
    match locktime {
      csLocktime::Blocks(_) => LockTime::Blocks,
      csLocktime::Seconds(_) => LockTime::Seconds,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
//   use crate::taker::LockTime::{Blocks, Seconds};
  use bitcoin::absolute::LockTime;

  #[test]
  fn test_locktime_conversion() {
    // Test block-based locktime
    let block_locktime = LockTime::from_height(500000).unwrap();
    let napi_block = super::LockTime::from(block_locktime);
    println!("Block locktime: {:?} -> {:?}", block_locktime, napi_block);

    // Test time-based locktime
    let time_locktime = LockTime::from_time(1234567890).unwrap();
    let napi_time = super::LockTime::from(time_locktime);
    println!("Time locktime: {:?} -> {:?}", time_locktime, napi_time);

    // Test the actual values
    assert!(matches!(napi_block, super::LockTime::Blocks));
    assert!(matches!(napi_time, super::LockTime::Seconds));
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
      lock_time: super::LockTime::Blocks,
      pubkey: PublicKey {
        compressed: true,
        inner: vec![2, 123, 45, 67, 89], // Mock compressed pubkey bytes
      },
      conf_height: Some(500000),
      cert_expiry: Some(144), // 1 difficulty period
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

  #[test]
  fn test_enum_string_representation() {
    // Show how the enum will appear in JavaScript
    let blocks_variant = super::LockTime::Blocks;
    let seconds_variant = super::LockTime::Seconds;

    println!("NAPI enum variants:");
    println!("  Blocks variant: {:?}", blocks_variant);
    println!("  Seconds variant: {:?}", seconds_variant);

    // In JavaScript, these will be:
    println!("\nIn JavaScript/TypeScript:");
    println!("  LockTime.Blocks = 'Blocks'");
    println!("  LockTime.Seconds = 'Seconds'");
  }
}
