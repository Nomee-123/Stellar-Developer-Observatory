//! Error types for fetching and decoding.

use thiserror::Error;

/// Something went wrong turning an RPC response into typed analysis input.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// A required field was absent from the RPC result.
    #[error("RPC result is missing required field `{0}`")]
    MissingField(&'static str),

    /// The endpoint reported a status this crate does not understand.
    #[error("unrecognised transaction status `{0}`")]
    UnknownStatus(String),

    /// The endpoint does not have this transaction.
    ///
    /// Usually means it fell outside the history retention window of the node
    /// (about 7 days by default), or the endpoint serves a different network.
    #[error("transaction `{0}` not found on this endpoint")]
    TransactionNotFound(String),

    /// A base64 XDR field was present but would not decode.
    #[error("failed to decode `{0}` as XDR: {1}")]
    Xdr(&'static str, String),
}

/// Something went wrong talking to an RPC endpoint.
#[derive(Debug, Error)]
pub enum RpcError {
    /// The HTTP request failed.
    #[error("transport error: {0}")]
    Transport(String),

    /// The response was not valid JSON-RPC.
    #[error("malformed response: {0}")]
    BadResponse(String),

    /// The endpoint returned a JSON-RPC error member.
    #[error("rpc error {code}: {message}")]
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },

    /// The response arrived but could not be decoded.
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
