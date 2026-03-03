use crate::types::{Amount, FidelityBond, LockTime, OutPoint, PublicKey};
use coinswap::bitcoin::absolute::LockTime as csLockTime;
use coinswap::bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use std::process::Command;
use std::sync::Arc;

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

const DOCKER_BITCOIN_RPC_URL: &str = "http://localhost:18442";
const DOCKER_BITCOIN_RPC_USER: &str = "user";
const DOCKER_BITCOIN_RPC_PASS: &str = "password";
const DOCKER_BITCOIN_ZMQ: &str = "tcp://127.0.0.1:28332";

struct DockerBitcoind {
  client: Client,
}

impl DockerBitcoind {
  fn connect() -> Result<Self, String> {
    let client = Client::new(
      DOCKER_BITCOIN_RPC_URL,
      Auth::UserPass(
        DOCKER_BITCOIN_RPC_USER.to_string(),
        DOCKER_BITCOIN_RPC_PASS.to_string(),
      ),
    )
    .map_err(|e| format!("Failed to connect to Docker bitcoind: {}", e))?;

    client
      .get_blockchain_info()
      .map_err(|e| format!("Failed to get blockchain info: {}", e))?;

    Ok(Self { client })
  }
}

fn get_docker_rpc_config(wallet_name: &str) -> crate::types::RPCConfig {
  crate::types::RPCConfig {
    url: "localhost:18442".to_string(),
    username: DOCKER_BITCOIN_RPC_USER.to_string(),
    password: DOCKER_BITCOIN_RPC_PASS.to_string(),
    wallet_name: wallet_name.to_string(),
  }
}

fn setup_bitcoind_and_taker(wallet_name: &str) -> (super::Taker, DockerBitcoind) {
  let bitcoind = DockerBitcoind::connect().expect("Failed to connect to Docker bitcoind");

  let rpc_config = get_docker_rpc_config(wallet_name);

  let taker = super::Taker::init(
    None,
    Some(wallet_name.to_string()),
    Some(rpc_config),
    Some(9051),
    Some("coinswap".to_string()),
    DOCKER_BITCOIN_ZMQ.to_string(),
    None,
  )
  .unwrap();

  (taker, bitcoind)
}

fn cleanup_wallet(wallet_name: &str) {
  use std::fs;
  use std::path::PathBuf;

  let mut wallet_dir = PathBuf::from(env!("HOME"));
  wallet_dir.push(".coinswap");

  if wallet_dir.exists() {
    let _ = fs::remove_dir_all(&wallet_dir);
  }

  if let Ok(bitcoind) = DockerBitcoind::connect() {
    let _ = bitcoind.client.unload_wallet(Some(wallet_name));
  }

  let _ = Command::new("docker")
    .args([
      "exec",
      "coinswap-ffi-bitcoind",
      "rm",
      "-rf",
      &format!("/home/bitcoin/.bitcoin/wallets/{}", wallet_name),
    ])
    .output();
}

#[test]
fn test_mutex_blocks_concurrent_access_with_docker_setup() {
  use std::thread;
  use std::time::{Duration, Instant};

  let wallet_name = "test-js-taker-mutex";
  cleanup_wallet(wallet_name);

  let (taker, _bitcoind) = setup_bitcoind_and_taker(wallet_name);
  let taker = Arc::new(taker);

  let holder = Arc::clone(&taker);
  let hold_duration = Duration::from_secs(2);
  let holder_thread = thread::spawn(move || {
    let _guard = holder.inner.lock().expect("Failed to acquire taker lock");
    thread::sleep(hold_duration);
  });

  thread::sleep(Duration::from_millis(100));

  let reader = Arc::clone(&taker);
  let start = Instant::now();
  let reader_thread = thread::spawn(move || {
    let result = reader.get_balances();
    let elapsed = start.elapsed();
    (result, elapsed)
  });

  holder_thread.join().expect("Holder thread panicked");
  let (balances_result, elapsed) = reader_thread.join().expect("Reader thread panicked");

  let min_blocked_time = Duration::from_millis(1700);
  assert!(
    elapsed >= min_blocked_time,
    "get_balances was not blocked by taker mutex: {:?} < {:?}",
    elapsed,
    min_blocked_time
  );

  assert!(
    balances_result.is_ok(),
    "get_balances failed after lock release: {:?}",
    balances_result.err()
  );
}
