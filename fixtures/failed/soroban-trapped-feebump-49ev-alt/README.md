# Fixture: soroban-trapped-feebump-49ev-alt

| | |
|---|---|
| Transaction hash | `01285524f481e3b14f4ba1dad3a6e6360e2feabd3882847a53678a34a3171b9f` |
| Network | Public Global Stellar Network ; September 2015 |
| Ledger | 64358921 |
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

A third trapped fee-bumped failure from a different ledger again. Present so that a rule cannot pass by memorising one transaction.

## Fields available in the recorded response

`applicationOrder, createdAt, diagnosticEventsXdr, envelopeXdr, events, feeBump, latestLedger, latestLedgerCloseTime, ledger, oldestLedger, oldestLedgerCloseTime, resultMetaXdr, resultXdr, status, txHash`

## Files

- `rpc-response.json` — the `result` member of a `getTransaction` JSON-RPC response, recorded verbatim.
- `probe.json` — what `sdo-probe` observed about this response at capture time.

## Reproducing

```bash
cargo run -p sdo-probe -- capture --rpc https://rpc.lightsail.network/ --tx 01285524f481e3b14f4ba1dad3a6e6360e2feabd3882847a53678a34a3171b9f --out fixtures/failed/soroban-trapped-feebump-49ev-alt
```

> Mainnet RPC retains roughly 7 days of history, so this transaction is no
> longer fetchable from a standard endpoint. That is precisely why the
> response is committed here.
