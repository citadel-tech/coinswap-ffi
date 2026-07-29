//! Docker Test Environment Helpers
//!
//! Utilities for connecting FFI tests to Docker-based test services

use bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use std::thread;
use std::time::Duration;

pub const DOCKER_BITCOIN_RPC_URL: &str = "http://localhost:18442";
pub const DOCKER_BITCOIN_RPC_USER: &str = "user";
pub const DOCKER_BITCOIN_RPC_PASS: &str = "password";
pub const DOCKER_BITCOIN_ZMQ: &str = "tcp://127.0.0.1:28332";

pub struct DockerBitcoind {
    pub client: Client,
}

impl DockerBitcoind {
    /// Connect to the Docker bitcoind instance
    pub fn connect() -> Result<Self, String> {
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

    /// Send funds to an address using the 'test' wallet
    pub fn send_to_address_from_funding_wallet(
        &self,
        address: &bitcoin::Address,
        amount: bitcoin::Amount,
    ) -> Result<bitcoin::Txid, String> {
        let test_wallet_url = format!("{}/wallet/{}", DOCKER_BITCOIN_RPC_URL, "test");
        let test_client = Client::new(
            &test_wallet_url,
            Auth::UserPass(
                DOCKER_BITCOIN_RPC_USER.to_string(),
                DOCKER_BITCOIN_RPC_PASS.to_string(),
            ),
        )
        .map_err(|e| format!("Failed to connect to test wallet: {}", e))?;

        let txid = test_client
            .send_to_address(address, amount, None, None, None, None, None, None)
            .map_err(|e| format!("Failed to send to address from test wallet: {}", e))?;

        let address = test_client
            .get_new_address(None, None)
            .map_err(|e| format!("Failed to get new address: {}", e))?
            .require_network(bitcoin::Network::Regtest)
            .map_err(|e| format!("Failed to require network: {}", e))?;

        test_client
            .generate_to_address(1, &address)
            .map_err(|e| format!("Failed to generate blocks: {}", e))?;

        Ok(txid)
    }

    #[allow(dead_code)]
    pub fn wait_for_ready(max_attempts: u32) -> Result<(), String> {
        for attempt in 1..=max_attempts {
            match Self::connect() {
                Ok(_) => return Ok(()),
                Err(e) if attempt == max_attempts => {
                    return Err(format!(
                        "Failed to connect after {} attempts: {}",
                        max_attempts, e
                    ));
                }
                _ => {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
        Ok(())
    }
}

pub fn get_docker_rpc_config(wallet_name: &str) -> crate::types::RPCConfig {
    crate::types::RPCConfig {
        url: "localhost:18442".to_string(),
        username: DOCKER_BITCOIN_RPC_USER.to_string(),
        password: DOCKER_BITCOIN_RPC_PASS.to_string(),
        wallet_name: wallet_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_connection() {
        let bitcoind = DockerBitcoind::connect().expect("Should connect to Docker bitcoind");
        let info = bitcoind
            .client
            .get_blockchain_info()
            .expect("Should get blockchain info");
        assert_eq!(info.chain, bitcoin::Network::Regtest);
    }
}
