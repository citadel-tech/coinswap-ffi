//! Shared types for coinswap N-API bindings
//!
//! This module contains types that are used across multiple modules
//! to avoid duplicate type definitions in TypeScript.

use bitcoin::{Amount as BitcoinAmount, ScriptBuf as BitcoinScriptBuf, Txid as BitcoinTxid};
use bitcoind::bitcoincore_rpc::Auth;
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
        username: "regtestrpcuser".to_string(),
        password: "regtestrpcpass".to_string(),
        wallet_name: "coinswap-wallet".to_string(),
    }
}
