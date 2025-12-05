//! Shared types for coinswap UniFFI bindings
//!
//! This module contains types that are used across multiple modules
//! to avoid duplicate type definitions across language bindings.

use coinswap::{
    bitcoin::{
        Address as csAddress, Amount as csAmount, OutPoint as coinswapOutPoint,
        PublicKey as csPublicKey, ScriptBuf as csScriptBuf, SignedAmount, Txid as csTxid,
        absolute::LockTime as csLocktime,
    },
    bitcoind::bitcoincore_rpc::Auth,
    protocol::messages::{FidelityProof as csFidelityProof, Offer as csOffer},
    taker::{
        error::TakerError as CoinswapTakerError,
        offers::{
            MakerAddress as csMakerAddress, OfferAndAddress as csOfferAndAddress,
            OfferBook as csOfferBook,
        },
    },
    wallet::{
        Balances as CoinswapBalances, FidelityBond as csFidelityBond,
        RPCConfig as CoinswapRPCConfig,
        ffi::{MakerFeeInfo as csMakerFeeInfo, SwapReport as csSwapReport},
    },
};

#[derive(Clone, uniffi::Record)]
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
                msg: format!("{:?}", e),
            },
            CoinswapTakerError::General(msg) => TakerError::General { msg },
            CoinswapTakerError::IO(e) => TakerError::IO { msg: e.to_string() },
            _ => TakerError::General {
                msg: format!("Taker error: {:?}", error),
            },
        }
    }
}

#[derive(uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl From<coinswapOutPoint> for OutPoint {
    fn from(value: coinswapOutPoint) -> Self {
        Self {
            txid: value.txid.into(),
            vout: value.vout,
        }
    }
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct ListTransactionResult {
    pub info: WalletTxInfo,
    pub detail: GetTransactionResultDetail,
    pub trusted: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct GetTransactionResultDetail {
    pub address: Option<Address>,
    pub category: String,
    pub amount: SignedAmountSats,
    pub label: Option<String>,
    pub vout: u32,
    pub fee: Option<SignedAmountSats>,
    pub abandoned: Option<bool>,
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct Txid {
    pub value: String,
}

impl From<csTxid> for Txid {
    fn from(txid: csTxid) -> Self {
        Self {
            value: txid.to_string(),
        }
    }
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct UtxoSpendInfo {
    pub spend_type: String,
    pub path: Option<String>,
    pub multisig_redeemscript: Option<ScriptBuf>,
    pub input_value: Option<Amount>,
    pub index: Option<u32>,
    pub original_multisig_redeemscript: Option<ScriptBuf>,
}

#[derive(Clone, uniffi::Record)]
pub struct WalletTxInfo2 {
    pub outpoint: OutPoint,
    pub listunspent: ListUnspentResultEntry,
    pub spend_info: UtxoSpendInfo,
}

#[derive(Clone, uniffi::Record)]
pub struct FeeRates {
    pub fastest: f64,
    pub standard: f64,
    pub economy: f64,
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct FidelityProof {
    pub bond: FidelityBond,
    pub cert_hash: Vec<u8>,
    pub cert_sig: Vec<u8>,
}

impl From<csFidelityProof> for FidelityProof {
    fn from(fidelityproof: csFidelityProof) -> Self {
        Self {
            bond: fidelityproof.bond.into(),
            cert_hash: <_ as AsRef<[u8]>>::as_ref(&fidelityproof.cert_hash).to_vec(),
            cert_sig: fidelityproof.cert_sig.serialize_compact().to_vec(),
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct FidelityBond {
    pub amount: Amount,
    pub lock_time: LockTime,
    pub pubkey: PublicKey,
}

impl From<csFidelityBond> for FidelityBond {
    fn from(bond: csFidelityBond) -> Self {
        Self {
            amount: Amount::from(bond.amount),
            lock_time: LockTime::from(bond.lock_time),
            pubkey: PublicKey {
                compressed: true,
                inner: vec![],
            },
        }
    }
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct OfferAndAddress {
    pub offer: Offer,
    pub address: MakerAddress,
}

impl From<csOfferAndAddress> for OfferAndAddress {
    fn from(offer_and_addr: csOfferAndAddress) -> Self {
        Self {
            offer: Offer::from(offer_and_addr.offer),
            address: MakerAddress::from(offer_and_addr.address),
        }
    }
}

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
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

#[derive(Clone, uniffi::Record)]
pub struct MakerFeeInfo {
    pub maker_index: u32,
    pub maker_address: String,
    pub base_fee: f64,
    pub amount_relative_fee: f64,
    pub time_relative_fee: f64,
    pub total_fee: f64,
}

impl From<csMakerFeeInfo> for MakerFeeInfo {
    fn from(info: csMakerFeeInfo) -> Self {
        Self {
            maker_index: info.maker_index as u32,
            maker_address: info.maker_address,
            base_fee: info.base_fee,
            amount_relative_fee: info.amount_relative_fee,
            time_relative_fee: info.time_relative_fee,
            total_fee: info.total_fee,
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct SwapReport {
    pub swap_id: String,
    pub swap_duration_seconds: f64,
    pub target_amount: i64,
    pub total_input_amount: i64,
    pub total_output_amount: i64,
    pub makers_count: u32,
    pub maker_addresses: Vec<String>,
    pub total_funding_txs: i64,
    pub funding_txids_by_hop: Vec<Vec<String>>,
    pub total_fee: i64,
    pub total_maker_fees: i64,
    pub mining_fee: i64,
    pub fee_percentage: f64,
    pub maker_fee_info: Vec<MakerFeeInfo>,
    pub input_utxos: Vec<i64>,
    pub output_change_amounts: Vec<i64>,
    pub output_swap_amounts: Vec<i64>,
    pub output_change_utxos: Vec<UtxoWithAddress>,
    pub output_swap_utxos: Vec<UtxoWithAddress>,
}

#[derive(Clone, uniffi::Record)]
pub struct UtxoWithAddress {
    pub amount: i64,
    pub address: String,
}

impl From<csSwapReport> for SwapReport {
    fn from(report: csSwapReport) -> Self {
        Self {
            swap_id: report.swap_id,
            swap_duration_seconds: report.swap_duration_seconds,
            target_amount: report.target_amount as i64,
            total_input_amount: report.total_input_amount as i64,
            total_output_amount: report.total_output_amount as i64,
            makers_count: report.makers_count as u32,
            maker_addresses: report.maker_addresses,
            total_funding_txs: report.total_funding_txs as i64,
            funding_txids_by_hop: report.funding_txids_by_hop,
            total_fee: report.total_fee as i64,
            total_maker_fees: report.total_maker_fees as i64,
            mining_fee: report.mining_fee as i64,
            fee_percentage: report.fee_percentage,
            maker_fee_info: report
                .maker_fee_info
                .into_iter()
                .map(MakerFeeInfo::from)
                .collect(),
            input_utxos: report.input_utxos.into_iter().map(|v| v as i64).collect(),
            output_change_amounts: report
                .output_change_amounts
                .into_iter()
                .map(|v| v as i64)
                .collect(),
            output_swap_amounts: report
                .output_swap_amounts
                .into_iter()
                .map(|v| v as i64)
                .collect(),
            output_change_utxos: report
                .output_change_utxos
                .into_iter()
                .map(|(amount, address)| UtxoWithAddress {
                    amount: amount as i64,
                    address,
                })
                .collect(),
            output_swap_utxos: report
                .output_swap_utxos
                .into_iter()
                .map(|(amount, address)| UtxoWithAddress {
                    amount: amount as i64,
                    address,
                })
                .collect(),
        }
    }
}
