# Fixture: soroban-trapped-feebump-24ev

| | |
|---|---|
| Transaction hash | `2514224d35758cdaa2dd7b6ec1c091369fc34b2facfdbd9d384ff322883b34ae` |
| Network | Public Global Stellar Network ; September 2015 |
| Ledger | 64365919 |
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
| Diagnostic events | 24 (all decoded, found at top_level) |
| Probe verdict | `SUFFICIENT_FOR_ANALYSIS` |

## Preliminary failure category

`ContractTrap` — the operation result is `invoke_host_function_trapped`. The *specific* reason for the trap has not been determined; that requires reading the diagnostic events, which is milestone M4.

See [docs/research/failure-taxonomy.md](../../../docs/research/failure-taxonomy.md).

## Why this fixture is useful

Baseline Soroban failure. Fee-bumped, trapped inside contract execution, 24 diagnostic events. Represents the dominant failure shape observed on mainnet during M0.

## Fields available in the recorded response

`applicationOrder, createdAt, diagnosticEventsXdr, envelopeXdr, events, feeBump, latestLedger, latestLedgerCloseTime, ledger, oldestLedger, oldestLedgerCloseTime, resultMetaXdr, resultXdr, status, txHash`

## Files

- `rpc-response.json` — the `result` member of a `getTransaction` JSON-RPC response, recorded verbatim.
- `probe.json` — what `sdo-probe` observed about this response at capture time.

## Reproducing

```bash
cargo run -p sdo-probe -- capture --rpc https://rpc.lightsail.network/ --tx 2514224d35758cdaa2dd7b6ec1c091369fc34b2facfdbd9d384ff322883b34ae --out fixtures/failed/soroban-trapped-feebump-24ev
```

> Mainnet RPC retains roughly 7 days of history, so this transaction is no
> longer fetchable from a standard endpoint. That is precisely why the
> response is committed here.
