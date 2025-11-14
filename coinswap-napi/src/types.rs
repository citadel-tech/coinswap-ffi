//! Shared types for coinswap N-API bindings
//!
//! This module contains types that are used across multiple modules
//! to avoid duplicate type definitions in TypeScript.

use coinswap::bitcoin::{SignedAmount, Amount as BitcoinAmount, ScriptBuf as BitcoinScriptBuf, Txid as BitcoinTxid, Address as BitcoinAddress};
use coinswap::bitcoind::bitcoincore_rpc::{json::{GetTransactionResultDetail as csGetTransactionResultDetail, ListTransactionResult as csListTransactionResult, WalletTxInfo as csWalletTxInfo}, Auth};
use coinswap::wallet::{Balances as CoinswapBalances, RPCConfig as CoinswapRPCConfig};
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

impl From<BitcoinAddress> for Address {
  fn from(addr: BitcoinAddress) -> Self {
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

impl From<BitcoinAmount> for Amount {
  fn from(amount: BitcoinAmount) -> Self {
    Self {
      sats: amount.to_sat() as i64,
    }
  }
}

#[napi(object)]
pub struct Txid {
  pub hex: String,
}

impl From<BitcoinTxid> for Txid {
  fn from(txid: BitcoinTxid) -> Self {
    Self {
      hex: txid.to_string(),
    }
  }
}

#[napi(object)]
pub struct ScriptBuf {
  pub hex: String,
}

impl From<BitcoinScriptBuf> for ScriptBuf {
  fn from(script: BitcoinScriptBuf) -> Self {
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

#[napi]
pub fn create_default_rpc_config() -> RPCConfig {
  RPCConfig {
    url: "localhost:18443".to_string(),
    username: "user".to_string(),
    password: "password".to_string(),
    wallet_name: "coinswap-wallet".to_string(),
  }
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
