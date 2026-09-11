# Fixture: soroban-trapped-feebump-49ev

| | |
|---|---|
| Transaction hash | `6d597ca6a4d168770b91d89769b8343a6d5ef11d78201c606ffd7caecc3058bc` |
| Network | Public Global Stellar Network ; September 2015 |
| Ledger | 64308924 |
| Captured | 2026-09-10 |
| RPC provider used | https://rpc.lightsail.network/ |
| Status | FAILED |
| Soroban | yes (invoke_host_function) |
| Transaction meta version | V4 |
| Fee bumped | yes |
| Outer result | `tx_fee_bump_inner_failed` |
| Inner result | `tx_failed` |
| Operation results | `invoke_host_function_trapped` |
| Failed in contract execution | yes |
| Diagnostic events | 49 (all decoded, found at top_level) |
| Probe verdict | `SUFFICIENT_FOR_ANALYSIS` |

## Preliminary failure category

`ContractTrap` — the operation result is `invoke_host_function_trapped`. The *specific* reason for the trap has not been determined; that requires reading the diagnostic events, which is milestone M4.

See [docs/research/failure-taxonomy.md](../../../docs/research/failure-taxonomy.md).

## Why this fixture is useful

Same failure class as the 24-event fixture but with a materially larger diagnostic-event list (49), captured ~57k ledgers earlier. Guards against rules that accidentally depend on event-list length or on a single contract.

## Fields available in the recorded response

`applicationOrder, createdAt, diagnosticEventsXdr, envelopeXdr, events, feeBump, latestLedger, latestLedgerCloseTime, ledger, oldestLedger, oldestLedgerCloseTime, resultMetaXdr, resultXdr, status, txHash`

## Files

- `rpc-response.json` — the `result` member of a `getTransaction` JSON-RPC response, recorded verbatim.
- `probe.json` — what `sdo-probe` observed about this response at capture time.

## Reproducing

```bash
cargo run -p sdo-probe -- capture --rpc https://rpc.lightsail.network/ --tx 6d597ca6a4d168770b91d89769b8343a6d5ef11d78201c606ffd7caecc3058bc --out fixtures/failed/soroban-trapped-feebump-49ev
```

> Mainnet RPC retains roughly 7 days of history, so this transaction is no
> longer fetchable from a standard endpoint. That is precisely why the
> response is committed here.
