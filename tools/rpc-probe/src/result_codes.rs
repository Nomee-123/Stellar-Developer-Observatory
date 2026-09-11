//! Turning XDR result unions into stable code names.
//!
//! No interpretation happens here — this is naming, not diagnosis. It answers
//! the M0 question "did this fail during contract execution?" by reading result
//! codes, and it is the factual ground the failure taxonomy is built on.
//!
//! One thing this module handles that a naive reader would get wrong: a large
//! share of real failed Soroban transactions on mainnet are **fee-bumped**. For
//! those the outer result is `TxFeeBumpInnerFailed` and the actual failure lives
//! in the inner result. Reading only the outer union reports nothing useful.

use serde::Serialize;
use stellar_xdr::{
    InnerTransactionResultResult, InvokeHostFunctionResult, OperationResult, OperationResultTr,
    TransactionResult, TransactionResultResult, VecM,
};

/// Result-code names for one transaction.
#[derive(Debug, Clone, Serialize)]
pub struct FailureSignal {
    /// Name of the outer `TransactionResultResult` arm.
    pub transaction_result: String,
    /// Whether the transaction was fee-bumped.
    pub fee_bumped: bool,
    /// Name of the inner result arm, when fee-bumped.
    pub inner_transaction_result: Option<String>,
    /// Names of the per-operation result arms, in order, from whichever level
    /// actually carried operations.
    pub operation_results: Vec<String>,
    /// Whether an `InvokeHostFunction` operation reported a non-success arm.
    pub failed_in_contract_execution: bool,
}

fn invoke_host_function_arm(r: &InvokeHostFunctionResult) -> &'static str {
    match r {
        InvokeHostFunctionResult::Success(_) => "invoke_host_function_success",
        InvokeHostFunctionResult::Malformed => "invoke_host_function_malformed",
        InvokeHostFunctionResult::Trapped => "invoke_host_function_trapped",
        InvokeHostFunctionResult::ResourceLimitExceeded => {
            "invoke_host_function_resource_limit_exceeded"
        }
        InvokeHostFunctionResult::EntryArchived => "invoke_host_function_entry_archived",
        InvokeHostFunctionResult::InsufficientRefundableFee => {
            "invoke_host_function_insufficient_refundable_fee"
        }
    }
}

fn operation_result_name(op: &OperationResult) -> (String, bool) {
    match op {
        OperationResult::OpInner(OperationResultTr::InvokeHostFunction(r)) => (
            invoke_host_function_arm(r).to_string(),
            !matches!(r, InvokeHostFunctionResult::Success(_)),
        ),
        OperationResult::OpInner(OperationResultTr::ExtendFootprintTtl(_)) => {
            ("extend_footprint_ttl".to_string(), false)
        }
        OperationResult::OpInner(OperationResultTr::RestoreFootprint(_)) => {
            ("restore_footprint".to_string(), false)
        }
        OperationResult::OpInner(_) => ("op_inner_classic".to_string(), false),
        OperationResult::OpBadAuth => ("op_bad_auth".to_string(), false),
        OperationResult::OpNoAccount => ("op_no_account".to_string(), false),
        OperationResult::OpNotSupported => ("op_not_supported".to_string(), false),
        OperationResult::OpTooManySubentries => ("op_too_many_subentries".to_string(), false),
        OperationResult::OpExceededWorkLimit => ("op_exceeded_work_limit".to_string(), false),
        OperationResult::OpTooManySponsoring => ("op_too_many_sponsoring".to_string(), false),
    }
}

fn collect_ops(ops: &VecM<OperationResult>) -> (Vec<String>, bool) {
    let mut names = Vec::new();
    let mut contract_exec_failure = false;
    for op in ops.iter() {
        let (name, failed) = operation_result_name(op);
        contract_exec_failure |= failed;
        names.push(name);
    }
    (names, contract_exec_failure)
}

/// Name the arms of a `TransactionResultResult` that carries no operations.
fn bare_outer_arm(r: &TransactionResultResult) -> &'static str {
    match r {
        TransactionResultResult::TxTooEarly => "tx_too_early",
        TransactionResultResult::TxTooLate => "tx_too_late",
        TransactionResultResult::TxMissingOperation => "tx_missing_operation",
        TransactionResultResult::TxBadSeq => "tx_bad_seq",
        TransactionResultResult::TxBadAuth => "tx_bad_auth",
        TransactionResultResult::TxInsufficientBalance => "tx_insufficient_balance",
        TransactionResultResult::TxNoAccount => "tx_no_account",
        TransactionResultResult::TxInsufficientFee => "tx_insufficient_fee",
        TransactionResultResult::TxBadAuthExtra => "tx_bad_auth_extra",
        TransactionResultResult::TxInternalError => "tx_internal_error",
        TransactionResultResult::TxNotSupported => "tx_not_supported",
        TransactionResultResult::TxBadSponsorship => "tx_bad_sponsorship",
        TransactionResultResult::TxBadMinSeqAgeOrGap => "tx_bad_min_seq_age_or_gap",
        TransactionResultResult::TxMalformed => "tx_malformed",
        TransactionResultResult::TxSorobanInvalid => "tx_soroban_invalid",
        TransactionResultResult::TxFrozenKeyAccessed => "tx_frozen_key_accessed",
        // The operation-carrying and fee-bump arms are handled by the caller.
        TransactionResultResult::TxSuccess(_) => "tx_success",
        TransactionResultResult::TxFailed(_) => "tx_failed",
        TransactionResultResult::TxFeeBumpInnerSuccess(_) => "tx_fee_bump_inner_success",
        TransactionResultResult::TxFeeBumpInnerFailed(_) => "tx_fee_bump_inner_failed",
    }
}

/// Name the arms of an `InnerTransactionResultResult`.
fn inner_arm(r: &InnerTransactionResultResult) -> &'static str {
    match r {
        InnerTransactionResultResult::TxSuccess(_) => "tx_success",
        InnerTransactionResultResult::TxFailed(_) => "tx_failed",
        InnerTransactionResultResult::TxTooEarly => "tx_too_early",
        InnerTransactionResultResult::TxTooLate => "tx_too_late",
        InnerTransactionResultResult::TxMissingOperation => "tx_missing_operation",
        InnerTransactionResultResult::TxBadSeq => "tx_bad_seq",
        InnerTransactionResultResult::TxBadAuth => "tx_bad_auth",
        InnerTransactionResultResult::TxInsufficientBalance => "tx_insufficient_balance",
        InnerTransactionResultResult::TxNoAccount => "tx_no_account",
        InnerTransactionResultResult::TxInsufficientFee => "tx_insufficient_fee",
        InnerTransactionResultResult::TxBadAuthExtra => "tx_bad_auth_extra",
        InnerTransactionResultResult::TxInternalError => "tx_internal_error",
        InnerTransactionResultResult::TxNotSupported => "tx_not_supported",
        InnerTransactionResultResult::TxBadSponsorship => "tx_bad_sponsorship",
        InnerTransactionResultResult::TxBadMinSeqAgeOrGap => "tx_bad_min_seq_age_or_gap",
        InnerTransactionResultResult::TxMalformed => "tx_malformed",
        InnerTransactionResultResult::TxSorobanInvalid => "tx_soroban_invalid",
        InnerTransactionResultResult::TxFrozenKeyAccessed => "tx_frozen_key_accessed",
    }
}

/// Extract result-code names, unwrapping fee bumps.
pub fn failure_signal(result: &TransactionResult) -> FailureSignal {
    let outer = bare_outer_arm(&result.result).to_string();

    match &result.result {
        TransactionResultResult::TxFailed(ops) | TransactionResultResult::TxSuccess(ops) => {
            let (operation_results, failed_in_contract_execution) = collect_ops(ops);
            FailureSignal {
                transaction_result: outer,
                fee_bumped: false,
                inner_transaction_result: None,
                operation_results,
                failed_in_contract_execution,
            }
        }

        TransactionResultResult::TxFeeBumpInnerFailed(pair)
        | TransactionResultResult::TxFeeBumpInnerSuccess(pair) => {
            let inner = &pair.result.result;
            let (operation_results, failed_in_contract_execution) = match inner {
                InnerTransactionResultResult::TxFailed(ops)
                | InnerTransactionResultResult::TxSuccess(ops) => collect_ops(ops),
                _ => (Vec::new(), false),
            };
            FailureSignal {
                transaction_result: outer,
                fee_bumped: true,
                inner_transaction_result: Some(inner_arm(inner).to_string()),
                operation_results,
                failed_in_contract_execution,
            }
        }

        _ => FailureSignal {
            transaction_result: outer,
            fee_bumped: false,
            inner_transaction_result: None,
            operation_results: Vec::new(),
            failed_in_contract_execution: false,
        },
    }
}

/// The result arm that actually describes the failure.
///
/// For a fee-bumped transaction that is the inner arm; otherwise the outer one.
impl FailureSignal {
    /// The effective result code name, unwrapping fee bumps.
    pub fn effective_result(&self) -> &str {
        self.inner_transaction_result
            .as_deref()
            .unwrap_or(&self.transaction_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{
        Hash, InnerTransactionResult, InnerTransactionResultExt, InnerTransactionResultPair,
        TransactionResultExt,
    };

    fn wrap(result: TransactionResultResult) -> TransactionResult {
        TransactionResult {
            fee_charged: 100,
            result,
            ext: TransactionResultExt::V0,
        }
    }

    fn invoke_op(r: InvokeHostFunctionResult) -> OperationResult {
        OperationResult::OpInner(OperationResultTr::InvokeHostFunction(r))
    }

    #[test]
    fn plain_failed_transaction_reports_operation_arms() {
        let ops: VecM<OperationResult> = vec![invoke_op(InvokeHostFunctionResult::Trapped)]
            .try_into()
            .unwrap();
        let sig = failure_signal(&wrap(TransactionResultResult::TxFailed(ops)));

        assert_eq!(sig.transaction_result, "tx_failed");
        assert!(!sig.fee_bumped);
        assert_eq!(sig.operation_results, ["invoke_host_function_trapped"]);
        assert!(sig.failed_in_contract_execution);
        assert_eq!(sig.effective_result(), "tx_failed");
    }

    #[test]
    fn fee_bumped_failure_is_unwrapped_to_the_inner_result() {
        // Regression guard for the M0 finding: reading only the outer union
        // reports `tx_fee_bump_inner_failed` and loses the real cause.
        let ops: VecM<OperationResult> =
            vec![invoke_op(InvokeHostFunctionResult::ResourceLimitExceeded)]
                .try_into()
                .unwrap();
        let pair = InnerTransactionResultPair {
            transaction_hash: Hash([0; 32]),
            result: InnerTransactionResult {
                fee_charged: 100,
                result: InnerTransactionResultResult::TxFailed(ops),
                ext: InnerTransactionResultExt::V0,
            },
        };
        let sig = failure_signal(&wrap(TransactionResultResult::TxFeeBumpInnerFailed(pair)));

        assert_eq!(sig.transaction_result, "tx_fee_bump_inner_failed");
        assert!(sig.fee_bumped);
        assert_eq!(sig.inner_transaction_result.as_deref(), Some("tx_failed"));
        assert_eq!(
            sig.operation_results,
            ["invoke_host_function_resource_limit_exceeded"]
        );
        assert!(sig.failed_in_contract_execution);
        assert_eq!(sig.effective_result(), "tx_failed");
    }

    #[test]
    fn successful_invoke_is_not_flagged_as_contract_failure() {
        let ops: VecM<OperationResult> =
            vec![invoke_op(InvokeHostFunctionResult::Success(Hash([1; 32])))]
                .try_into()
                .unwrap();
        let sig = failure_signal(&wrap(TransactionResultResult::TxSuccess(ops)));
        assert!(!sig.failed_in_contract_execution);
    }

    #[test]
    fn operationless_arms_are_named_not_swallowed() {
        let sig = failure_signal(&wrap(TransactionResultResult::TxSorobanInvalid));
        assert_eq!(sig.transaction_result, "tx_soroban_invalid");
        assert!(sig.operation_results.is_empty());
        assert!(!sig.failed_in_contract_execution);
    }
}
