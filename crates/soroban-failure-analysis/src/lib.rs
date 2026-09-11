//! Pure, deterministic analysis of failed Soroban transactions.
//!
//! # Contract of this crate
//!
//! This crate performs **no I/O**. No network, no filesystem, no clock, no
//! environment variables, no global state. Given the same [`AnalysisInput`] it
//! always produces the same [`Diagnosis`]. That is what makes the whole rule
//! corpus testable from committed fixtures without mainnet access, which is in
//! turn what makes outside contribution possible.
//!
//! Fetching and decoding live under [`soroban-failure-rpc`][rpc]; presentation
//! lives in the `sdo` CLI.
//!
//! # Status
//!
//! **Milestone M1 (foundation).** The engine, taxonomy, and rule interface are
//! in place. **No failure rules are implemented yet** — [`analyze`] will run an
//! empty registry and return a [`Diagnosis`] with no candidate causes and an
//! explicit limitation saying so. Rules are milestone M4; see `ROADMAP.md`.
//!
//! This crate would rather return "undetermined" than a guess.
//!
//! # Example
//!
//! ```no_run
//! use soroban_failure_analysis::{analyze, AnalysisInput};
//! use stellar_xdr::{Limits, ReadXdr, TransactionEnvelope, TransactionResult};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let (envelope_b64, result_b64) = ("", "");
//! let envelope = TransactionEnvelope::from_xdr_base64(envelope_b64, Limits::none())?;
//! let result = TransactionResult::from_xdr_base64(result_b64, Limits::none())?;
//!
//! let input = AnalysisInput::builder(envelope, result).build();
//! let diagnosis = analyze(&input);
//!
//! for limitation in &diagnosis.limitations {
//!     eprintln!("note: {limitation}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [rpc]: https://docs.rs/soroban-failure-rpc

#![doc(html_root_url = "https://docs.rs/soroban-failure-analysis")]

pub mod diagnosis;
pub mod input;
pub mod rule;
pub mod taxonomy;

pub use diagnosis::{CandidateCause, Confidence, Diagnosis, Evidence, EvidenceSource};
pub use input::{AnalysisInput, AnalysisInputBuilder};
pub use rule::{FailureContext, Rule, RuleRegistry};
pub use taxonomy::{CauseClass, FailureStage};

/// Analyse a failed transaction with the built-in rules.
///
/// See [`analyze_with`] to supply your own registry.
pub fn analyze(input: &AnalysisInput) -> Diagnosis {
    analyze_with(input, &RuleRegistry::builtin())
}

/// Analyse a failed transaction against a specific rule registry.
///
/// Candidate causes are returned ranked by [`Confidence`], strongest first.
/// Ranking is stable: rules of equal confidence keep their registration order,
/// so output does not shift between runs.
pub fn analyze_with(input: &AnalysisInput, registry: &RuleRegistry) -> Diagnosis {
    // Stage classification reads the transaction result and is milestone M2.
    // Until then, report honestly that it is not determined rather than
    // inventing a stage.
    let stage = FailureStage::Unknown;

    let ctx = FailureContext { input, stage };

    let mut candidate_causes: Vec<CandidateCause> = registry
        .iter()
        .filter_map(|rule| rule.evaluate(&ctx))
        .collect();

    // Strongest confidence first. `sort_by_key` is stable, so equal-confidence
    // candidates retain registration order and output stays deterministic.
    candidate_causes.sort_by_key(|c| core::cmp::Reverse(c.confidence));

    let mut limitations = Vec::new();

    if registry.is_empty() {
        limitations.push(
            "No failure rules are implemented yet (milestone M4). This build can \
             decode and structure a transaction but cannot yet attribute a cause."
                .to_string(),
        );
    }

    limitations.push(
        "Failure-stage classification is not implemented yet (milestone M2); \
         stage is reported as `unknown`."
            .to_string(),
    );

    if !input.diagnostics_enabled {
        limitations.push(
            "Diagnostic events were not available from the source. Their absence \
             carries no information: the node may simply not emit them."
                .to_string(),
        );
    }

    Diagnosis {
        transaction_hash: input.transaction_hash.clone(),
        stage,
        candidate_causes,
        limitations,
        rules_evaluated: registry.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{
        Limits, Memo, MuxedAccount, Operation, Preconditions, ReadXdr, SequenceNumber, Transaction,
        TransactionEnvelope, TransactionExt, TransactionResult, TransactionResultExt,
        TransactionResultResult, TransactionV1Envelope, Uint256, WriteXdr,
    };

    /// A minimal, structurally valid envelope/result pair.
    ///
    /// Built in code rather than pasted as base64 so the test stays readable and
    /// does not silently depend on an opaque blob.
    fn minimal_input() -> AnalysisInput {
        let tx = Transaction {
            source_account: MuxedAccount::Ed25519(Uint256([0; 32])),
            fee: 100,
            seq_num: SequenceNumber(1),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: Vec::<Operation>::new().try_into().unwrap(),
            ext: TransactionExt::V0,
        };
        let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
            tx,
            signatures: Vec::new().try_into().unwrap(),
        });
        let result = TransactionResult {
            fee_charged: 100,
            result: TransactionResultResult::TxFailed(Vec::new().try_into().unwrap()),
            ext: TransactionResultExt::V0,
        };
        AnalysisInput::builder(envelope, result).build()
    }

    #[test]
    fn analyze_is_undetermined_with_no_rules() {
        let diagnosis = analyze(&minimal_input());
        assert!(diagnosis.is_undetermined());
        assert_eq!(diagnosis.rules_evaluated, 0);
        assert_eq!(diagnosis.stage, FailureStage::Unknown);
    }

    #[test]
    fn analyze_states_its_limitations_rather_than_degrading_silently() {
        let diagnosis = analyze(&minimal_input());
        assert!(
            diagnosis.limitations.iter().any(|l| l.contains("M4")),
            "must disclose that no rules exist yet"
        );
        assert!(
            diagnosis
                .limitations
                .iter()
                .any(|l| l.contains("Diagnostic events were not available")),
            "must disclose missing diagnostic events"
        );
    }

    #[test]
    fn diagnostics_flag_distinguishes_absent_from_empty() {
        let base = minimal_input();
        assert!(!base.diagnostics_enabled);
        assert!(!base.has_diagnostic_evidence());

        let with_empty = AnalysisInput::builder(base.envelope.clone(), base.result.clone())
            .diagnostic_events(Vec::new())
            .build();
        // Node emits diagnostics, but this transaction had none. That is a real,
        // usable observation -- distinct from the node not emitting any.
        assert!(with_empty.diagnostics_enabled);
        assert!(!with_empty.has_diagnostic_evidence());
    }

    #[test]
    fn analysis_is_deterministic() {
        let input = minimal_input();
        assert_eq!(analyze(&input), analyze(&input));
    }

    #[test]
    fn minimal_fixture_round_trips_through_xdr() {
        // Guards against the hand-built test envelope drifting out of shape.
        let input = minimal_input();
        let b64 = input.envelope.to_xdr_base64(Limits::none()).unwrap();
        let decoded = TransactionEnvelope::from_xdr_base64(&b64, Limits::none()).unwrap();
        assert_eq!(decoded, input.envelope);
    }
}
