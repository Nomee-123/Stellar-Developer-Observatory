//! Network access.
//!
//! This is the **only** module in the product crates that performs I/O. It is
//! kept deliberately thin: fetch JSON, hand it to [`crate::decode`]. No analysis
//! happens here, and nothing here is required in order to test the analyzer.

use std::time::Duration;

use serde_json::{json, Value};

use crate::decode::{decode_get_transaction, DecodedTransaction};
use crate::error::RpcError;

/// Default per-request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A Stellar JSON-RPC client.
pub struct RpcClient {
    url: String,
    agent: ureq::Agent,
}

impl RpcClient {
    /// Create a client for an endpoint with the default timeout.
    pub fn new(url: impl Into<String>) -> Self {
        Self::with_timeout(url, DEFAULT_TIMEOUT)
    }

    /// Create a client with an explicit total per-request timeout.
    pub fn with_timeout(url: impl Into<String>, timeout: Duration) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .user_agent(concat!("soroban-failure-rpc/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        Self {
            url: url.into(),
            agent,
        }
    }

    /// The endpoint this client talks to.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Issue a JSON-RPC call, returning the `result` member.
    pub fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });

        let mut response = self
            .agent
            .post(&self.url)
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        let value: Value = response
            .body_mut()
            .read_json()
            .map_err(|e| RpcError::BadResponse(e.to_string()))?;

        if let Some(err) = value.get("error") {
            return Err(RpcError::Rpc {
                code: err.get("code").and_then(Value::as_i64).unwrap_or(0),
                message: err
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("<no message>")
                    .to_string(),
            });
        }

        value.get("result").cloned().ok_or_else(|| {
            RpcError::BadResponse("response had neither `result` nor `error`".into())
        })
    }

    /// Fetch a transaction and decode it into typed analysis input.
    pub fn fetch_transaction(&self, tx_hash: &str) -> Result<DecodedTransaction, RpcError> {
        let result = self.call("getTransaction", json!({ "hash": tx_hash }))?;
        Ok(decode_get_transaction(tx_hash, &result)?)
    }
}
