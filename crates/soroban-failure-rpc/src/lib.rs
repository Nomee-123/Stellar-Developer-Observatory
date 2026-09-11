//! Fetching and decoding Stellar transactions for failure analysis.
//!
//! This crate is the I/O boundary of the project. It has two halves, and the
//! split is deliberate:
//!
//! * [`decode`] and [`fixture`] are **pure** — they turn JSON into a typed
//!   [`soroban_failure_analysis::AnalysisInput`] and touch no network.
//! * [`client`] is the only part that makes network calls.
//!
//! Everything downstream can therefore be tested against committed fixtures
//! with no RPC endpoint, no rate limit, and no flakiness.
//!
//! # Status
//!
//! **Milestone M1 (foundation).** Fetching, decoding and fixture loading work.
//! Contract-spec resolution — turning `Error(Contract, #3)` into a name — is
//! milestone M3 and is not implemented. See `ROADMAP.md`.

#![doc(html_root_url = "https://docs.rs/soroban-failure-rpc")]

pub mod client;
pub mod decode;
pub mod error;
pub mod fixture;

pub use client::RpcClient;
pub use decode::{decode_get_transaction, DecodedTransaction, DiagnosticSource, TransactionStatus};
pub use error::{DecodeError, RpcError};
