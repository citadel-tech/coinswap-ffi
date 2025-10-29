//! Coinswap Taker FFI bindings
//!
//! This module provides UniFFI bindings for the coinswap taker functionality.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use coinswap::taker::{
    api::{SwapParams as CoinswapSwapParams, Taker as CoinswapTaker},
    error::TakerError as CoinswapTakerError,
    offers::{OfferAndAddress, OfferBook}
};
use coinswap::wallet::RPCConfig as CoinswapRPCConfig;
use bitcoin::{Amount, OutPoint};
use crate::RPCConfig;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TakerError {
    #[error("Wallet error: {msg}")]
    Wallet { msg: String },
    #[error("Protocol error: {msg}")]
    Protocol { msg: String },
    #[error("Network error: {msg}")]
    Network { msg: String },
    #[error("General error: {msg}")]
    General { msg: String },
    #[error("IO error: {msg}")]
    IO { msg: String },
}

impl From<CoinswapTakerError> for TakerError {
    fn from(error: CoinswapTakerError) -> Self {
        match error {
            CoinswapTakerError::Wallet(e) => TakerError::Wallet { 
                msg: format!("{:?}", e) 
            },
            CoinswapTakerError::General(msg) => TakerError::General { msg },
            CoinswapTakerError::IO(e) => TakerError::IO { 
                msg: e.to_string() 
            },
            _ => TakerError::General { 
                msg: format!("Taker error: {:?}", error) 
            },
        }
    }
}

#[derive(uniffi::Record)]
pub struct SwapParams {
    /// Total Amount
    pub send_amount: u64,
    /// How many hops (number of makers)
    pub maker_count: u32,
    /// User selected UTXOs (optional)
    pub manually_selected_outpoints: Option<Vec<String>>,
}

impl TryFrom<SwapParams> for CoinswapSwapParams {
    type Error = TakerError;
    
    fn try_from(params: SwapParams) -> Result<Self, Self::Error> {
        let send_amount = Amount::from_sat(params.send_amount);
        
        let manually_selected_outpoints = if let Some(outpoints) = params.manually_selected_outpoints {
            let mut parsed_outpoints = Vec::new();
            for outpoint_str in outpoints {
                let outpoint = outpoint_str.parse::<OutPoint>()
                    .map_err(|e| TakerError::General { 
                        msg: format!("Invalid outpoint format: {}", e) 
                    })?;
                parsed_outpoints.push(outpoint);
            }
            Some(parsed_outpoints)
        } else {
            None
        };
        
        Ok(CoinswapSwapParams {
            send_amount,
            maker_count: params.maker_count as usize,
            manually_selected_outpoints,
        })
    }
}

#[derive(uniffi::Enum)]
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
            },
            TakerBehavior::BroadcastContractAfterFullSetup => {
                coinswap::taker::api::TakerBehavior::BroadcastContractAfterFullSetup
            },
        }
    }
}

#[derive(uniffi::Object)]
pub struct Taker {
    taker: Mutex<CoinswapTaker>,
}

#[uniffi::export]
impl Taker {
    #[uniffi::constructor]
    pub fn init(
        data_dir: Option<String>,
        wallet_file_name: Option<String>,
        rpc_config: Option<RPCConfig>,
        _behavior: Option<TakerBehavior>,
        control_port: Option<u16>,
        tor_auth_password: Option<String>,
    ) -> Result<Arc<Self>, TakerError> {
        let data_dir = data_dir.map(PathBuf::from);
        let rpc_config = rpc_config.map(CoinswapRPCConfig::from);
        
        let taker = CoinswapTaker::init(
            data_dir,
            wallet_file_name,
            rpc_config,
            #[cfg(feature = "integration-test")]
            _behavior.unwrap_or(TakerBehavior::Normal).into(),
            control_port,
            tor_auth_password,
        )?;
        
        Ok(Arc::new(Self { taker: Mutex::new(taker) }))
    }
    
    pub fn send_coinswap(&self, swap_params: SwapParams) -> Result<(), TakerError> {
        let params = CoinswapSwapParams::try_from(swap_params)?;
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        taker.do_coinswap(params)?;
        Ok(())
    }
    
    pub fn get_wallet_name(&self) -> Result<String, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        Ok(taker.get_wallet().get_name().to_string())
    }
    
    /// Get wallet balances
    pub fn get_wallet_balances(&self) -> Result<crate::Balances, TakerError> {
        let taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        let balances = taker.get_wallet().get_balances()
            .map_err(|e| TakerError::Wallet { msg: format!("{:?}", e) })?;
        Ok(crate::Balances::from(balances))
    }
    
    pub fn sync_wallet(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        taker.get_wallet_mut().sync_and_save()
            .map_err(|e| TakerError::Wallet { msg: format!("{:?}", e) })?;
        Ok(())
    }

    /// Sync the offerbook with available makers
    pub fn sync_offerbook(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        taker.sync_offerbook()?;
        Ok(())
    }

    /// Get basic information about all good makers (limited due to private fields)
    pub fn get_all_good_makers(&self) -> Result<Vec<String>, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        
        // Fetch fresh offers
        let offerbook = taker.fetch_offers()?;
        let good_makers = offerbook.all_good_makers();
        
        // Since fields are private, we can only return addresses
        let addresses = good_makers
            .into_iter()
            .map(|maker| maker.address.to_string())
            .collect();
        
        Ok(addresses)
    }

    // /// Get detailed information about all good makers
    // pub fn get_all_good_makers(&self) -> Result<Vec<MakerOffer>, TakerError> {
    //     let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
    //         msg: "Failed to acquire taker lock".to_string() 
    //     })?;
        
    //     // Fetch fresh offers
    //     let offerbook = taker.fetch_offers()?;
    //     let good_makers = offerbook.all_good_makers();
        
    //     let offers = good_makers
    //         .into_iter()
    //         .map(|maker| MakerOffer {
    //             base_fee: maker.offer.base_fee,
    //             amount_relative_fee_pct: maker.offer.amount_relative_fee_pct,
    //             time_relative_fee_pct: maker.offer.time_relative_fee_pct,
    //             required_confirms: maker.offer.required_confirms,
    //             minimum_locktime: maker.offer.minimum_locktime,
    //             max_size: maker.offer.max_size,
    //             min_size: maker.offer.min_size,
    //             address: maker.address.to_string(),
    //         })
    //         .collect();
        
    //     Ok(offers)
    // }

    /// Display detailed information about a specific maker offer
    pub fn display_offer(&self, maker_offer: &MakerOffer) -> Result<String, TakerError> {
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
            .map_err(|e| TakerError::General { msg: e.to_string() })
    }

    /// Recover from a failed swap
    pub fn recover_from_swap(&self) -> Result<(), TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        taker.recover_from_swap()?;
        Ok(())
    }

    pub fn fetch_good_makers(&self) -> Result<Vec<String>, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
        })?;
        
        let offerbook = taker.fetch_offers()?;
        let all_good_makers = offerbook.all_good_makers();
        
        let addresses = all_good_makers
            .into_iter()
            .map(|maker| maker.address.to_string())
            .collect();
        
        Ok(addresses)
    }

        pub fn fetch_all_makers(&self) -> Result<Vec<String>, TakerError> {
        let mut taker = self.taker.lock().map_err(|_| TakerError::General { 
            msg: "Failed to acquire taker lock".to_string() 
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

#[uniffi::export]
pub fn create_swap_params(
    send_amount_sats: u64,
    maker_count: u32,
    outpoints: Vec<String>,
) -> SwapParams {
    SwapParams {
        send_amount: send_amount_sats,
        maker_count,
        manually_selected_outpoints: Some(outpoints),
    }
}

#[derive(uniffi::Record)]
pub struct MakerOffer {
    pub base_fee: u64,
    pub amount_relative_fee_pct: f64,
    pub time_relative_fee_pct: f64,
    pub required_confirms: u32,
    pub minimum_locktime: u16,
    pub max_size: u64,
    pub min_size: u64,
    pub address: String,
}

#[derive(uniffi::Record)]
pub struct MakerStats {
    pub total_makers: u32,
    pub online_makers: u32,
    pub avg_base_fee: u64,
    pub avg_amount_relative_fee_pct: f64,
    pub avg_time_relative_fee_pct: f64,
    pub total_liquidity: u64,
    pub avg_min_size: u64,
    pub avg_max_size: u64,
}