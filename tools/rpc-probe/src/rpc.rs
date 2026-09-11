//! Minimal Stellar JSON-RPC client for the feasibility probe.
//!
//! Deliberately separate from `soroban-failure-rpc`: this tool measures what an
//! endpoint *returns*, so it must stay close to the raw JSON and must not be
//! constrained by the product crate's typed model. It keeps the untouched
//! `result` object so the probe can report which shape a provider actually used.

use std::time::Duration;

use serde_json::{json, Value};

/// Something went wrong talking to the endpoint.
#[derive(Debug)]
pub enum RpcError {
    /// The HTTP request itself failed (DNS, TLS, timeout, connection refused).
    Transport(String),
    /// The endpoint answered, but not with JSON we could parse.
    BadResponse(String),
    /// The endpoint returned a JSON-RPC `error` member.
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "transport error: {e}"),
            Self::BadResponse(e) => write!(f, "malformed response: {e}"),
            Self::Rpc { code, message } => write!(f, "rpc error {code}: {message}"),
        }
    }
}

impl std::error::Error for RpcError {}

/// A JSON-RPC client bound to one endpoint.
pub struct RpcClient {
    url: String,
    agent: ureq::Agent,
}

impl RpcClient {
    /// Create a client with a total per-request timeout.
    pub fn new(url: impl Into<String>, timeout: Duration) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .user_agent(concat!("sdo-probe/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        Self {
            url: url.into(),
            agent,
        }
    }

    /// The endpoint this client talks to.
    #[allow(dead_code)] // used by future multi-provider batch runs
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Issue a JSON-RPC call and return the `result` member.
    pub fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

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

    /// `getTransaction` for a single hex transaction hash.
    pub fn get_transaction(&self, tx_hash: &str) -> Result<Value, RpcError> {
        self.call("getTransaction", json!({ "hash": tx_hash }))
    }

    /// `getTransactions` starting at a ledger.
    pub fn get_transactions(&self, start_ledger: u32, limit: u32) -> Result<Value, RpcError> {
        self.call(
            "getTransactions",
            json!({
                "startLedger": start_ledger,
                "pagination": { "limit": limit },
            }),
        )
    }

    /// `getHealth`, used to discover the endpoint's retention window.
    pub fn get_health(&self) -> Result<Value, RpcError> {
        self.call("getHealth", json!({}))
    }

    /// `getNetwork`, used to identify which network an endpoint serves.
    pub fn get_network(&self) -> Result<Value, RpcError> {
        self.call("getNetwork", json!({}))
    }
}
