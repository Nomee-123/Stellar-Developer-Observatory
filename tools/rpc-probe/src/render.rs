//! Human-readable rendering of a [`ProbeReport`].

use crate::probe::{ArtifactState, ProbeReport};

const RULE: &str = "────────────────────────────────";

fn yes_no(b: bool) -> &'static str {
    if b {
        "YES"
    } else {
        "NO"
    }
}

/// Render a probe report for a terminal.
pub fn render(r: &ProbeReport) -> String {
    let mut s = String::new();

    s.push_str("Soroban Diagnostic Probe\n");
    s.push_str(RULE);
    s.push_str("\n\n");

    s.push_str(&format!("RPC:\n{}\n\n", r.rpc_url));
    s.push_str(&format!("Transaction:\n{}\n\n", r.transaction));

    if let Some(net) = &r.network_passphrase {
        s.push_str(&format!("Network:\n{net}\n\n"));
    }

    s.push_str(&format!("Status:\n{}\n\n", r.status));

    if let Some(ledger) = r.ledger {
        s.push_str(&format!("Ledger:\n{ledger}\n\n"));
    }

    s.push_str(&format!(
        "Transaction Type:\n{}\n\n",
        if r.is_soroban { "Soroban" } else { "Classic" }
    ));

    if let Some(op) = &r.soroban_operation {
        s.push_str(&format!("Soroban Operation:\n{op}\n\n"));
    }

    s.push_str(&format!("Envelope XDR:\n{}\n\n", r.envelope_xdr.label()));
    s.push_str(&format!("Result XDR:\n{}\n\n", r.result_xdr.label()));
    s.push_str(&format!(
        "Result Meta XDR:\n{}\n\n",
        r.result_meta_xdr.label()
    ));
    if let Some(v) = r.meta_version {
        s.push_str(&format!("Transaction Meta Version:\nV{v}\n\n"));
    }

    s.push_str(&format!(
        "Soroban Metadata:\n{}\n\n",
        if r.soroban_meta_present {
            "AVAILABLE"
        } else {
            "NOT AVAILABLE"
        }
    ));

    if let Some(sig) = &r.failure_signal {
        s.push_str(&format!(
            "Transaction Result:\n{}\n\n",
            sig.transaction_result
        ));
        if sig.fee_bumped {
            s.push_str(&format!(
                "Inner Transaction Result:\n{}\n\n",
                sig.inner_transaction_result.as_deref().unwrap_or("unknown")
            ));
        }
        s.push_str(&format!(
            "Effective Result:\n{}\n\n",
            sig.effective_result()
        ));
        if !sig.operation_results.is_empty() {
            s.push_str(&format!(
                "Operation Results:\n{}\n\n",
                sig.operation_results.join(", ")
            ));
        }
        s.push_str(&format!(
            "Failed In Contract Execution:\n{}\n\n",
            yes_no(sig.failed_in_contract_execution)
        ));
    }

    s.push_str(&format!(
        "Diagnostic Events:\n{}\n\n",
        if r.diagnostic_events.available {
            "AVAILABLE"
        } else {
            "NOT AVAILABLE"
        }
    ));

    if r.diagnostic_events.available {
        s.push_str(&format!(
            "Diagnostic Event Count:\n{}\n\n",
            r.diagnostic_events.count
        ));
        s.push_str(&format!(
            "Diagnostic Events Decoded:\n{} ({}/{})\n\n",
            yes_no(r.diagnostic_events.decoded),
            r.diagnostic_events.decoded_count,
            r.diagnostic_events.count
        ));
        if let Some(loc) = r.diagnostic_events.location {
            s.push_str(&format!("Found In:\n{loc:?}\n\n"));
        }
    }

    for state in [&r.envelope_xdr, &r.result_xdr, &r.result_meta_xdr] {
        if let ArtifactState::DecodeFailed(e) = state {
            s.push_str(&format!("Decode error:\n{e}\n\n"));
        }
    }

    for e in &r.diagnostic_events.decode_errors {
        s.push_str(&format!("Diagnostic decode error:\n{e}\n\n"));
    }

    if !r.notes.is_empty() {
        s.push_str("Notes:\n");
        for n in &r.notes {
            s.push_str(&format!("  - {n}\n"));
        }
        s.push('\n');
    }

    s.push_str(RULE);
    s.push_str(&format!("\nVerdict:\n{}\n", r.verdict.label()));
    s.push_str(RULE);
    s.push('\n');

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_reports_absence_without_claiming_availability() {
        let report = crate::probe::probe_transaction(
            "http://example.invalid",
            None,
            "deadbeef",
            &json!({ "status": "FAILED" }),
        );
        let out = render(&report);
        assert!(out.contains("Diagnostic Events:\nNOT AVAILABLE"));
        assert!(out.contains("UNUSABLE"));
        // A report with nothing available must never print an availability claim.
        assert!(!out.contains("SUFFICIENT_FOR_ANALYSIS"));
    }
}
