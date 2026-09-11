//! Loading recorded RPC responses from disk.
//!
//! A fixture is a verbatim recording of a `getTransaction` result. Fixtures are
//! what make the analysis engine testable without network access, which in turn
//! is what makes this project contributable by people who do not want to run a
//! node or spend a rate limit. See `fixtures/README.md`.

use std::path::Path;

use serde_json::Value;

use crate::decode::{decode_get_transaction, DecodedTransaction};
use crate::error::DecodeError;

/// Something went wrong loading a fixture from disk.
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    /// The file could not be read.
    #[error("could not read fixture at {path}: {source}")]
    Io {
        /// The path that failed.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The file was not valid JSON.
    #[error("fixture at {path} is not valid JSON: {source}")]
    Json {
        /// The path that failed.
        path: String,
        /// The underlying parse error.
        source: serde_json::Error,
    },

    /// The JSON was valid but could not be decoded as a transaction.
    #[error(transparent)]
    Decode(#[from] DecodeError),
}

/// The filename every fixture directory must contain.
pub const RESPONSE_FILE: &str = "rpc-response.json";

/// Read and parse the recorded response of a fixture without decoding it.
pub fn load_raw(dir: impl AsRef<Path>) -> Result<Value, FixtureError> {
    let path = dir.as_ref().join(RESPONSE_FILE);
    let display = path.display().to_string();

    let bytes = std::fs::read(&path).map_err(|source| FixtureError::Io {
        path: display.clone(),
        source,
    })?;

    serde_json::from_slice(&bytes).map_err(|source| FixtureError::Json {
        path: display,
        source,
    })
}

/// Load a fixture directory and decode it into typed analysis input.
///
/// The transaction hash is read from the recorded response itself, so a fixture
/// cannot drift out of sync with its own directory name.
pub fn load(dir: impl AsRef<Path>) -> Result<DecodedTransaction, FixtureError> {
    let raw = load_raw(&dir)?;
    let hash = raw
        .get("txHash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Ok(decode_get_transaction(&hash, &raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fixture_reports_the_path_it_looked_for() {
        let err = load_raw("does/not/exist").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(RESPONSE_FILE),
            "error should name the file: {msg}"
        );
    }
}
