//! Coinswap Taker N-API bindings
//!
//! This module provides N-API bindings for the coinswap taker functionality.

use bitcoin::{Amount, OutPoint as BitcoinOutPoint, Txid};
use coinswap::taker::{
    api::{SwapParams as CoinswapSwapParams, Taker as CoinswapTaker},
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{path::PathBuf, str::FromStr};
use std::sync::Mutex;
use std::fmt;
use std::error::Error;

// Import shared types
use crate::types::{Balances, RPCConfig, OutPoint};


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
    pub send_amount: i64,
    /// How many hops (number of makers)
    pub maker_count: u32,
    /// User selected UTXOs (optional)
    pub manually_selected_outpoints: Option<Vec<OutPoint>>,
}

impl TryFrom<SwapParams> for CoinswapSwapParams {
    type Error = napi::Error;

    fn try_from(params: SwapParams) -> Result<Self> {
        let send_amount = Amount::from_sat(params.send_amount as u64);

        let manually_selected_outpoints = params.manually_selected_outpoints.map(|outpoints| {
            outpoints
                .into_iter()
                .map(|outpoint| {
                    let txid = Txid::from_str(&outpoint.txid).map_err(|e| napi::Error::from_reason(format!("Invalid txid: {:?}", e)))?;
                    Ok(BitcoinOutPoint::new(txid, outpoint.vout))
                })
                .collect::<Result<Vec<_>, _>>()
        }).transpose()?;

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
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        taker.do_coinswap(params)
            .map_err(|e| napi::Error::from_reason(format!("Send coinswap error: {:?}", e)))?;
        Ok(())
    }

    #[napi]
    pub fn get_wallet_name(&self) -> Result<String> {
        let taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        Ok(taker.get_wallet().get_name().to_string())
    }

    /// Get wallet balances
    #[napi]
    pub fn get_wallet_balances(&self) -> Result<Balances> {
        let taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        let balances = taker.get_wallet().get_balances()
            .map_err(|e| napi::Error::from_reason(format!("Get balances error: {:?}", e)))?;
        Ok(Balances::from(balances))
    }

    #[napi]
    pub fn sync_wallet(&mut self) -> Result<()> {
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        taker.get_wallet_mut().sync_and_save()
            .map_err(|e| napi::Error::from_reason(format!("Sync wallet error: {:?}", e)))?;
        Ok(())
    }

    /// Sync the offerbook with available makers
    #[napi]
    pub fn sync_offerbook(&mut self) -> Result<()> {
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        taker.sync_offerbook()
            .map_err(|e| napi::Error::from_reason(format!("Sync offerbook error: {:?}", e)))?;
        Ok(())
    }

    /// Get basic information about all good makers (limited due to private fields)
    #[napi]
    pub fn get_all_good_makers(&self) -> Result<Vec<String>> {
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

        // Fetch fresh offers
        let offerbook = taker.fetch_offers()
            .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;
        let good_makers = offerbook.all_good_makers();

        // Since fields are private, we can only return addresses
        let addresses = good_makers
            .into_iter()
            .map(|maker| maker.address.to_string())
            .collect();

        Ok(addresses)
    }

    /// Display detailed information about a specific maker offer
    #[napi]
    pub fn display_offer(&self, maker_offer: MakerOffer) -> Result<String> {
        let offer_json = serde_json::json!({
            "base_fee": maker_offer.base_fee,
            "amount_relative_fee_pct": maker_offer.amount_relative_fee_pct,
            "time_relative_fee_pct": maker_offer.time_relative_fee_pct,
            "required_confirms": maker_offer.required_confirms,
            "minimum_locktime": maker_offer.minimum_locktime,
            "max_size": maker_offer.max_size,
            "min_size": maker_offer.min_size,
            "address": maker_offer.address
        });

        serde_json::to_string_pretty(&offer_json)
            .map_err(|e| napi::Error::from_reason(format!("JSON error: {:?}", e)))
    }

    /// Recover from a failed swap
    #[napi]
    pub fn recover_from_swap(&mut self) -> Result<()> {
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;
        taker.recover_from_swap()
            .map_err(|e| napi::Error::from_reason(format!("Recover error: {:?}", e)))?;
        Ok(())
    }

    #[napi]
    pub fn fetch_good_makers(&self) -> Result<Vec<String>> {
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

        let offerbook = taker.fetch_offers()
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
        let mut taker = self.inner.lock()
            .map_err(|e| napi::Error::from_reason(format!("Failed to acquire taker lock: {}", e)))?;

        let offerbook = taker.fetch_offers()
            .map_err(|e| napi::Error::from_reason(format!("Fetch offers error: {:?}", e)))?;
        let all_makers = offerbook.all_makers();

        let addresses = all_makers
            .into_iter()
            .map(|maker| maker.address.to_string())
            .collect();

        Ok(addresses)
    }
}

#[napi]
pub fn create_swap_params(
    send_amount: i64,
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
pub struct MakerOffer {
    pub base_fee: i64,
    pub amount_relative_fee_pct: f64,
    pub time_relative_fee_pct: f64,
    pub required_confirms: u32,
    pub minimum_locktime: u16,
    pub max_size: i64,
    pub min_size: i64,
    pub address: String,
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
