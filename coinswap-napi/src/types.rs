//! Shared types for coinswap N-API bindings
//!
//! This module contains types that are used across multiple modules
//! to avoid duplicate type definitions in TypeScript.

use coinswap::{
  bitcoin::{
    absolute::LockTime as csLocktime, Address as csAddress, Amount as csAmount,
    PublicKey as csPublicKey, ScriptBuf as csScriptBuf, SignedAmount, Txid as csTxid,
  },
  bitcoind::bitcoincore_rpc::{
    json::{
      GetTransactionResultDetail as csGetTransactionResultDetail,
      ListTransactionResult as csListTransactionResult, WalletTxInfo as csWalletTxInfo,
    },
    Auth,
  },
  protocol::messages::{FidelityProof as csFidelityProof, Offer as csOffer},
  taker::offers::{
    MakerAddress as csMakerAddress, OfferAndAddress as csOfferAndAddress, OfferBook as csOfferBook,
  },
  wallet::{
    Balances as CoinswapBalances, FidelityBond as csFidelityBond, RPCConfig as CoinswapRPCConfig,
  },
};
use napi_derive::napi;

#[napi(object)]
pub struct Balances {
  pub regular: i64,
  pub swap: i64,
  pub contract: i64,
  pub fidelity: i64,
  pub spendable: i64,
}

impl From<CoinswapBalances> for Balances {
  fn from(balances: CoinswapBalances) -> Self {
    Self {
      regular: balances.regular.to_sat() as i64,
      swap: balances.swap.to_sat() as i64,
      contract: balances.contract.to_sat() as i64,
      fidelity: balances.fidelity.to_sat() as i64,
      spendable: balances.spendable.to_sat() as i64,
    }
  }
}

#[napi(object)]
pub struct Address {
  pub address: String,
}

impl From<csAddress> for Address {
  fn from(addr: csAddress) -> Self {
    Self {
      address: addr.to_string(),
    }
  }
}

#[napi(object)]
pub struct ListTransactionResult {
  pub info: WalletTxInfo,
  pub detail: GetTransactionResultDetail,
  pub trusted: Option<bool>,
  pub comment: Option<String>,
}

impl From<csListTransactionResult> for ListTransactionResult {
  fn from(result: csListTransactionResult) -> Self {
    Self {
      info: WalletTxInfo::from(result.info),
      detail: GetTransactionResultDetail::from(result.detail),
      trusted: result.trusted,
      comment: result.comment,
    }
  }
}

#[napi(object)]
pub struct WalletTxInfo {
  pub confirmations: i32,
  pub blockhash: Option<String>,
  pub blockindex: Option<u32>,
  pub blocktime: Option<i64>,
  pub blockheight: Option<u32>,
  pub txid: Txid,
  pub time: i64,
  pub timereceived: i64,
  pub bip125_replaceable: String,
  pub wallet_conflicts: Vec<Txid>,
}

impl From<csWalletTxInfo> for WalletTxInfo {
  fn from(info: csWalletTxInfo) -> Self {
    Self {
      confirmations: info.confirmations,
      blockhash: info.blockhash.map(|h| h.to_string()),
      blockindex: info.blockindex.map(|i| i as u32),
      blocktime: info.blocktime.map(|t| t as i64),
      blockheight: info.blockheight,
      txid: Txid::from(info.txid),
      time: info.time as i64,
      timereceived: info.timereceived as i64,
      bip125_replaceable: format!("{:?}", info.bip125_replaceable),
      wallet_conflicts: info.wallet_conflicts.into_iter().map(Txid::from).collect(),
    }
  }
}

#[napi(object)]
pub struct GetTransactionResultDetail {
  pub address: Option<Address>,
  pub category: String,
  pub amount: SignedAmountSats,
  pub label: Option<String>,
  pub vout: u32,
  pub fee: Option<SignedAmountSats>,
  pub abandoned: Option<bool>,
}

impl From<csGetTransactionResultDetail> for GetTransactionResultDetail {
  fn from(detail: csGetTransactionResultDetail) -> Self {
    Self {
      address: detail
        .address
        .map(|addr| Address::from(addr.assume_checked())),
      category: format!("{:?}", detail.category),
      amount: SignedAmountSats::from(detail.amount),
      label: detail.label,
      vout: detail.vout,
      fee: detail.fee.map(SignedAmountSats::from),
      abandoned: detail.abandoned,
    }
  }
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

#[napi(object)]
pub struct Amount {
  pub sats: i64,
}

impl From<csAmount> for Amount {
  fn from(amount: csAmount) -> Self {
    Self {
      sats: amount.to_sat() as i64,
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
pub struct OutPoint {
  pub txid: String,
  pub vout: u32,
}

#[napi(object)]
pub struct SignedAmountSats {
  pub sats: i64,
}

impl From<SignedAmount> for SignedAmountSats {
  fn from(amount: SignedAmount) -> Self {
    Self {
      sats: amount.to_sat(),
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
pub struct UtxoSpendInfo {
  pub spend_type: String,
  pub path: Option<String>,
  pub multisig_redeemscript: Option<ScriptBuf>,
  pub input_value: Option<Amount>,
  pub index: Option<u32>,
  pub original_multisig_redeemscript: Option<ScriptBuf>,
}

#[napi(object)]
#[derive(Debug)]
pub struct LockTime {
  pub lock_type: String,
  pub value: u32,
}

impl From<csLocktime> for LockTime {
  fn from(locktime: csLocktime) -> Self {
    match locktime {
      csLocktime::Blocks(height) => LockTime {
        lock_type: "Blocks".to_string(),
        value: height.to_consensus_u32(),
      },
      csLocktime::Seconds(time) => LockTime {
        lock_type: "Seconds".to_string(),
        value: time.to_consensus_u32(),
      },
    }
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

// #[napi(object)]
// pub struct MakerStats {
//   pub total_makers: u32,
//   pub online_makers: u32,
//   pub avg_base_fee: i64,
//   pub avg_amount_relative_fee_pct: f64,
//   pub avg_time_relative_fee_pct: f64,
//   pub total_liquidity: i64,
//   pub avg_min_size: i64,
//   pub avg_max_size: i64,
// }

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
pub struct OfferBook {
  pub good_makers: Vec<OfferAndAddress>,
  pub all_makers: Vec<OfferAndAddress>,
}

impl From<&csOfferBook> for OfferBook {
  fn from(offerbook: &csOfferBook) -> Self {
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

#[napi(object)]
#[allow(unused)]
pub struct WalletBackup {
  pub file_name: String,
}
