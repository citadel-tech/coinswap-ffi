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
    fee_estimation::{BlockTarget, FeeEstimator},
    protocol::messages::{FidelityProof as csFidelityProof, Offer as csOffer},
    taker::{
        error::TakerError as CoinswapTakerError,
        offers::{
            MakerAddress as csMakerAddress, MakerOfferCandidate as csMakerOfferCandidate,
            MakerProtocol as csMakerProtocol, MakerState as csMakerState, OfferBook as csOfferBook,
        },
    },
    wallet::{
        AddressType as csAddressType, Balances as CoinswapBalances, FidelityBond as csFidelityBond,
        RPCConfig as CoinswapRPCConfig,
        ffi::{
            MakerFeeInfo as csMakerFeeInfo, SwapReport as csSwapReport,
            restore_wallet_gui_app as cs_restore_wallet_gui_app,
        },
    },
};
use std::path::PathBuf;

/// Configuration parameters for connecting to a Bitcoin node via RPC.
#[derive(Debug, Clone, uniffi::Record)]
pub struct RPCConfig {
    /// The bitcoin node url
    pub url: String,
    /// The bitcoin node username
    pub username: String,
    /// The bitcoin node password
    pub password: String,
    /// The wallet name in the bitcoin node, derive this from the descriptor.
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

/// Represents errors that can occur during Taker operations.
///
/// This enum covers a range of errors related to I/O, wallet operations, network communication,
/// and other Taker-specific scenarios.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TakerError {
    /// Error related to wallet operations.
    #[error("Wallet error: {msg}")]
    Wallet { msg: String },
    /// Protocol error during coinswap operations.
    #[error("Protocol error: {msg}")]
    Protocol { msg: String },
    /// Error related to network operations.
    #[error("Network error: {msg}")]
    Network { msg: String },
    /// General error with a custom message
    #[error("General error: {msg}")]
    General { msg: String },
    /// Standard input/output error.
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

/// Represents different behaviors taker can have during the swap.
/// Used for testing various possible scenarios that can happen during a swap.
#[derive(uniffi::Enum)]
pub enum TakerBehavior {
    /// Normal behaviour
    Normal,
    /// This depicts the behavior when the taker drops connections after the full coinswap setup.
    DropConnectionAfterFullSetup,
    /// Behavior to broadcast the contract after the full coinswap setup.
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

/// Represents total wallet balances of different categories.
#[derive(Debug, uniffi::Record)]
pub struct Balances {
    /// All single signature regular wallet coins (seed balance).
    pub regular: i64,
    /// All 2of2 multisig coins received in swaps.
    pub swap: i64,
    /// All live contract transaction balance locked in timelocks.
    pub contract: i64,
    /// All coins locked in fidelity bonds.
    pub fidelity: i64,
    /// Spendable amount in wallet (regular + swap balance).
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

#[derive(Debug, Clone, uniffi::Record)]
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
}

#[derive(uniffi::Record)]
pub struct TotalUtxoInfo {
    pub list_unspent_result_entry: ListUnspentResultEntry,
    pub utxo_spend_info: UtxoSpendInfo,
}

#[derive(Clone, uniffi::Record)]
pub struct FeeRates {
    pub fastest: f64,
    pub standard: f64,
    pub economy: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
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

#[derive(Debug, Clone, uniffi::Record)]
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

/// Contains proof data related to fidelity bond.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FidelityProof {
    /// Details for Fidelity Bond
    pub bond: FidelityBond,
    /// Double SHA256 hash of certificate message proving bond ownership and binding to maker address
    pub cert_hash: Vec<u8>,
    /// ECDSA signature over cert_hash using the bond's private key
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

#[derive(Clone, Debug, uniffi::Record)]
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

/// Represents an offer in the context of the Coinswap protocol.
#[derive(Debug, Clone, uniffi::Record)]
pub struct Offer {
    /// Base fee charged per swap in satoshis (fixed cost component)
    pub base_fee: i64,
    /// Percentage fee relative to swap amount
    pub amount_relative_fee_pct: f64,
    /// Percentage fee for time-locked funds
    pub time_relative_fee_pct: f64,
    /// Minimum confirmations required before proceeding with swap
    pub required_confirms: u32,
    /// Minimum timelock duration in blocks for contract transactions
    pub minimum_locktime: u16,
    /// Maximum swap amount accepted in sats
    pub max_size: i64,
    /// Minimum swap amount accepted in sats
    pub min_size: i64,
    /// Displayed public key of makers, for receiving swaps.
    /// Actual swap addresses are derived from this public key using unique nonces per swap.
    pub tweakable_point: PublicKey,
    /// Cryptographic proof of fidelity bond for Sybil resistance
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

#[derive(Debug, Clone, uniffi::Record)]
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

/// Represents the Maker connection state
#[derive(Debug, Clone, uniffi::Record)]
pub struct MakerState {
    /// State type: "Good", "Unresponsive", or "Bad"
    pub state_type: String,
    /// Number of retries (only for Unresponsive state). We allow only 10 retries before marking a maker as bad.
    pub retries: Option<u8>,
}

impl From<csMakerState> for MakerState {
    fn from(state: csMakerState) -> Self {
        match state {
            csMakerState::Good => MakerState {
                state_type: "Good".to_string(),
                retries: None,
            },
            csMakerState::Unresponsive { retries } => MakerState {
                state_type: "Unresponsive".to_string(),
                retries: Some(retries),
            },
            csMakerState::Bad => MakerState {
                state_type: "Bad".to_string(),
                retries: None,
            },
        }
    }
}

/// Protocol which maker follows
#[derive(Debug, Clone, uniffi::Record)]
pub struct MakerProtocol {
    /// Protocol type: "Legacy" or "Taproot"
    pub protocol_type: String,
}

impl From<csMakerProtocol> for MakerProtocol {
    fn from(protocol: csMakerProtocol) -> Self {
        match protocol {
            csMakerProtocol::Legacy => MakerProtocol {
                protocol_type: "Legacy".to_string(),
            },
            csMakerProtocol::Taproot => MakerProtocol {
                protocol_type: "Taproot".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AddressType {
    /// P2WPKH or P2TR
    pub addr_type: String,
}

impl TryFrom<AddressType> for csAddressType {
    type Error = TakerError;

    fn try_from(addr: AddressType) -> Result<Self, Self::Error> {
        match addr.addr_type.as_str() {
            "P2TR" => Ok(csAddressType::P2TR),
            "P2WPKH" => Ok(csAddressType::P2WPKH),
            _ => Err(TakerError::General {
                msg: format!("Invalid address type: {}", addr.addr_type),
            }),
        }
    }
}

/// Canonical maker record.
/// A maker may or may not currently have an offer.
#[derive(Debug, Clone, uniffi::Record)]
pub struct MakerOfferCandidate {
    /// Maker Address: onion_addr:port
    pub address: MakerAddress,
    /// Latest offer, if successfully fetched
    pub offer: Option<Offer>,
    /// Current state of maker
    pub state: MakerState,
    /// Supporting protocol (Legacy or Taproot), if known
    pub protocol: Option<MakerProtocol>,
}

impl From<csMakerOfferCandidate> for MakerOfferCandidate {
    fn from(maker: csMakerOfferCandidate) -> Self {
        Self {
            address: maker.address.into(),
            offer: maker.offer.map(Offer::from),
            state: maker.state.into(),
            protocol: maker.protocol.map(|p| p.into()),
        }
    }
}

/// Contains all maker offers in the network
#[derive(Debug, Clone, uniffi::Record)]
pub struct OfferBook {
    /// All makers in the offerbook (good, bad, and unresponsive)
    pub makers: Vec<MakerOfferCandidate>,
}

impl From<&csOfferBook> for OfferBook {
    fn from(offerbook: &csOfferBook) -> Self {
        Self {
            makers: offerbook
                .all_makers()
                .into_iter()
                .map(MakerOfferCandidate::from)
                .collect(),
        }
    }
}

/// Information about individual maker fees in a swap
#[derive(Debug, Clone, uniffi::Record)]
pub struct MakerFeeInfo {
    /// Index of maker in the swap route
    pub maker_index: u32,
    /// Maker Addresses (Onion:Port)
    pub maker_address: String,
    /// The fixed Base Fee for each maker
    pub base_fee: f64,
    /// Dynamic Amount Fee for each maker
    pub amount_relative_fee: f64,
    /// Dynamic Time Fee(Decreases for subsequent makers) for each maker
    pub time_relative_fee: f64,
    /// All inclusive fee for each maker
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

/// Complete swap report containing all swap information
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapReport {
    /// Unique swap ID
    pub swap_id: String,
    /// Duration of the swap in seconds
    pub swap_duration_seconds: f64,
    /// Target amount for the swap
    pub target_amount: i64,
    /// Total input amount
    pub total_input_amount: i64,
    /// Total output amount
    pub total_output_amount: i64,
    /// Number of makers involved
    pub makers_count: u32,
    /// List of maker addresses used
    pub maker_addresses: Vec<String>,
    /// Total number of funding transactions
    pub total_funding_txs: i64,
    /// Funding transaction IDs organized by hops
    pub funding_txids_by_hop: Vec<Vec<String>>,
    /// Total fees paid
    pub total_fee: i64,
    /// Total maker fees
    pub total_maker_fees: i64,
    /// Mining fees
    pub mining_fee: i64,
    /// Fee percentage relative to target amount
    pub fee_percentage: f64,
    /// Individual maker fee information
    pub maker_fee_info: Vec<MakerFeeInfo>,
    /// Input UTXOs amounts
    pub input_utxos: Vec<i64>,
    /// Output change UTXOs amounts
    pub output_change_amounts: Vec<i64>,
    /// Output swap coin UTXOs amounts
    pub output_swap_amounts: Vec<i64>,
    /// Output change coin UTXOs with amounts and addresses (amount, address)
    pub output_change_utxos: Vec<UtxoWithAddress>,
    /// Output swap coin UTXOs with amounts and addresses (amount, address)
    pub output_swap_utxos: Vec<UtxoWithAddress>,
}

#[derive(Debug, Clone, uniffi::Record)]
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

/// Fetches current network fee estimates from mempool.space or esplora as fallback.
/// Returns fee rates for fastest, standard, and economy confirmation targets.
#[uniffi::export]
pub fn fetch_mempool_fees() -> Result<FeeRates, TakerError> {
    let fees = FeeEstimator::fetch_mempool_fees()
        .or_else(|_mempool_err| FeeEstimator::fetch_esplora_fees())
        .map_err(|e| TakerError::Network {
            msg: format!("Both fee APIs failed: {:?}", e),
        })?;

    let get = |target| {
        fees.get(&target).ok_or_else(|| TakerError::General {
            msg: format!("Missing fee for {:?}", target),
        })
    };

    Ok(FeeRates {
        fastest: *get(BlockTarget::Fastest)?,
        standard: *get(BlockTarget::Standard)?,
        economy: *get(BlockTarget::Economy)?,
    })
}

/// Restores a wallet from an encrypted or unencrypted JSON backup file for GUI/FFI applications.
///
/// This is a non-interactive restore method designed for programmatic use via FFI bindings.
/// Unlike `restore_wallet`, this function accepts a path to a JSON backup file and handles both
/// encrypted and unencrypted backups using [`load_sensitive_struct_from_value`].
///
/// # Behavior
///
/// 1. Reads and parses the JSON backup file into a [`WalletBackup`] structure
/// 2. If encrypted, decrypts using the provided password and preserves encryption material
/// 3. Constructs the wallet path: `{data_dir_or_default}/wallets/{wallet_file_name_or_default}`
/// 4. Calls [`Wallet::restore`] to reconstruct the wallet with all UTXOs and metadata
///
/// # Parameters
///
/// - `data_dir`: Target directory, defaults to `~/.coinswap/taker`
/// - `wallet_file_name`: Restored wallet filename, defaults to name from backup if empty
/// - `backup_file_path`: Path to the JSON file containing the wallet backup (encrypted or plain)
/// - `password`: Required if backup is encrypted, ignored otherwise
#[uniffi::export]
pub fn restore_wallet_gui_app(
    data_dir: Option<String>,
    wallet_file_name: Option<String>,
    rpc_config: RPCConfig,
    backup_file_path: String,
    password: Option<String>,
) {
    let data_dir = data_dir.map(PathBuf::from);

    cs_restore_wallet_gui_app(
        data_dir,
        wallet_file_name,
        rpc_config.into(),
        backup_file_path.into(),
        password,
    );
}

/// Checks whether wallet is encrypted or not.
#[uniffi::export]
pub fn is_wallet_encrypted(wallet_path: String) -> Result<bool, TakerError> {
    let path = PathBuf::from(wallet_path);

    coinswap::wallet::Wallet::is_wallet_encrypted(&path).map_err(|e| TakerError::Wallet {
        msg: format!("Failed to check wallet encryption: {:?}", e),
    })
}

#[uniffi::export]
pub fn create_default_rpc_config() -> RPCConfig {
    RPCConfig {
        url: "http://127.0.0.1:38332".to_string(),
        username: "user".to_string(),
        password: "password".to_string(),
        wallet_name: "coinswap_wallet".to_string(),
    }
}

/// Sets up the logger for the taker component.
///
/// This method initializes the logging configuration for the taker, directing logs to both
/// the console and a file. It sets the `RUST_LOG` environment variable to provide default
/// log levels and configures log4rs with the specified filter level for fine-grained control
/// of log verbosity.
#[uniffi::export]
pub fn setup_logging(data_dir: Option<String>) -> Result<(), TakerError> {
    let path = data_dir.map(PathBuf::from);
    coinswap::utill::setup_taker_logger(log::LevelFilter::Info, false, path);
    Ok(())
}
