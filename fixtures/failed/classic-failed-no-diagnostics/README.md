# Fixture: classic-failed-no-diagnostics

| | |
|---|---|
| Transaction hash | `c90eab4b76a34d20206a137892cb4d0f69ab101cd74cce790d3b956d311b7fb2` |
| Network | Public Global Stellar Network ; September 2015 |
| Ledger | 64368700 |
| Captured | 2026-09-10 |
| RPC provider used | https://rpc.lightsail.network/ |
| Status | FAILED |
| Soroban | no |
| Transaction meta version | V4 |
| Fee bumped | no |
| Outer result | `tx_failed` |
| Inner result | n/a |
| Operation results | `op_inner_classic` |
| Failed in contract execution | no |
| Diagnostic events | 0 (NOT all decoded) |
| Probe verdict | `INSUFFICIENT_FOR_FULL_ANALYSIS` |

## Preliminary failure category

`Validation` / classic — not a Soroban failure at all.

See [docs/research/failure-taxonomy.md](../../../docs/research/failure-taxonomy.md).

## Why this fixture is useful

Negative control. A failed CLASSIC transaction with no Soroban operation and zero diagnostic events. Any rule that fires on this is wrong, and any code path that assumes diagnostic events exist will break here.

## Fields available in the recorded response

`applicationOrder, createdAt, envelopeXdr, events, feeBump, latestLedger, latestLedgerCloseTime, ledger, oldestLedger, oldestLedgerCloseTime, resultMetaXdr, resultXdr, status, txHash`

## Files

- `rpc-response.json` — the `result` member of a `getTransaction` JSON-RPC response, recorded verbatim.
- `probe.json` — what `sdo-probe` observed about this response at capture time.

## Reproducing

```bash
cargo run -p sdo-probe -- capture --rpc https://rpc.lightsail.network/ --tx c90eab4b76a34d20206a137892cb4d0f69ab101cd74cce790d3b956d311b7fb2 --out fixtures/failed/classic-failed-no-diagnostics
```

> Mainnet RPC retains roughly 7 days of history, so this transaction is no
> longer fetchable from a standard endpoint. That is precisely why the
> response is committed here.
