# Diagnostic-event availability across public Stellar mainnet RPC providers

**Measured:** 2026-09-10
**Tool:** `sdo-probe` ([`tools/rpc-probe`](../../tools/rpc-probe)), commit at time of measurement
**Network:** Public Global Stellar Network ; September 2015 (mainnet)

## Why this was measured

Diagnostic events are only emitted by a node started with
`--enable-soroban-diagnostic-events`. They are unmetered, they are **not part of
consensus**, and Stellar's own guidance is to enable them only on watcher nodes
(`NODE_IS_VALIDATOR=false`). Nothing guarantees a public provider runs with them.

This matters more than it first appears, because for a **failed** transaction
`sorobanMeta.events` is not populated. Diagnostic events are therefore the *only*
source of execution-flow evidence for exactly the transactions this project
exists to analyse. If they were broadly unavailable, the project as designed
would not be viable.

[`docs/research/00-validation.md`](00-validation.md) flagged this as the single
largest feasibility risk. This document is the measurement.

## Method

One real failed Soroban transaction was probed against every free public mainnet
endpoint listed in the [Stellar RPC providers
documentation](https://developers.stellar.org/docs/data/apis/rpc/providers).

Test transaction:
`2514224d35758cdaa2dd7b6ec1c091369fc34b2facfdbd9d384ff322883b34ae`
(ledger 64365919, fee-bumped, `invoke_host_function_trapped`).

For each endpoint the probe called `getTransaction` and recorded whether
`diagnosticEventsXdr` was present, how many events were returned, and whether
every one of them decoded against `stellar-xdr 28.0.0`.

```bash
cargo run -p sdo-probe -- probe --rpc <URL> --tx <HASH> --json
```

The Liquify endpoint was **excluded**: the URL published in the docs embeds
someone else's API key, and this project does not use credentials it was not
given.

## Results

| Provider | Endpoint | Diagnostic events | Count | Decoded | Location | Verdict |
|---|---|---|---|---|---|---|
| sorobanrpc.com | `https://mainnet.sorobanrpc.com` | **yes** | 24 | 24/24 | `diagnosticEventsXdr` (top level) | `SUFFICIENT_FOR_ANALYSIS` |
| Gateway | `https://soroban-rpc.mainnet.stellar.gateway.fm` | **yes** | 24 | 24/24 | `diagnosticEventsXdr` (top level) | `SUFFICIENT_FOR_ANALYSIS` |
| Lightsail Quasar | `https://rpc.lightsail.network/` | **yes** | 24 | 24/24 | `diagnosticEventsXdr` (top level) | `SUFFICIENT_FOR_ANALYSIS` |
| Ankr | `https://rpc.ankr.com/stellar_soroban` | **yes** | 24 | 24/24 | `diagnosticEventsXdr` (top level) | `SUFFICIENT_FOR_ANALYSIS` |
| Nodies | `https://stellar-soroban-public.nodies.app` | *inconclusive* | — | — | — | HTTP 500 |
| OnFinality | `https://stellar.api.onfinality.io/public` | *inconclusive* | — | — | — | HTTP 429 (rate limited) |
| Lightsail Quasar (archive) | `https://archive-rpc.lightsail.network/` | *inconclusive* | — | — | — | `NOT_FOUND` for a 3000-ledger-old transaction |

**4 of 4 endpoints that answered returned diagnostic events, all of them
complete and decodable, all at the same location, all agreeing on the count.**

No endpoint answered successfully *without* diagnostic events. The three
inconclusive rows are transport or coverage problems, not evidence of absence —
they are recorded as unknown rather than counted either way.

## Secondary findings

These were not what the probe was looking for, but they change the design.

### 1. Transaction meta is V4, and diagnostic events moved

Every sampled transaction returned `TransactionMetaV4` (protocol 23). The layout
differs from the widely-documented V3:

| | V3 | V4 |
|---|---|---|
| Diagnostic events | inside `sorobanMeta.diagnosticEvents` | **top level** of the meta |
| Soroban events | inside `sorobanMeta.events` | `SorobanTransactionMetaV2` no longer carries events |

An analyzer written only against V3 will silently find nothing on current
mainnet. `soroban-failure-rpc` checks all three known locations
(top-level field, `events` object, embedded meta for both V3 and V4).

### 2. `stellar-xdr` dropped the `curr` module

Tutorials and older code use `stellar_xdr::curr::*`. In **28.0.0** the
`curr`/`next` split is gone and the XDR types sit at the crate root
(`stellar_xdr::TransactionEnvelope`). Copying an older example will not compile.

### 3. Failed Soroban transactions on mainnet are overwhelmingly fee-bumped

All 10 sampled Soroban failures were fee-bumped. Their outer result is
`TxFeeBumpInnerFailed`, which says nothing about the cause — the real result is
in `InnerTransactionResultPair`. **Reading only the outer union loses the
failure entirely.** This is handled in
[`tools/rpc-probe/src/result_codes.rs`](../../tools/rpc-probe/src/result_codes.rs)
and covered by a regression test.

## Limitations of this measurement

Stated plainly, because they bound how far the conclusion goes:

- **One transaction across providers.** Provider comparison used a single
  transaction. A provider could in principle behave differently on others.
- **Three providers untested.** Nodies and OnFinality never answered; the
  Lightsail archive endpoint did not have a transaction only ~3000 ledgers old,
  which is unexpected for an archive node and was not investigated further.
- **Paid tiers untested.** Only free public endpoints were probed.
- **A point-in-time result.** Providers can change node configuration at any
  time. Nothing here is a guarantee about future behaviour.
- **No testnet measurement.** Mainnet only.

## Reproducing

```bash
cargo run -p sdo-probe -- scan --want 5
cargo run -p sdo-probe -- probe --rpc https://mainnet.sorobanrpc.com --tx <HASH>
```

Mainnet RPC retains ~7 days (120,960 ledgers), so the specific transaction above
is no longer fetchable. The recorded responses are committed under
[`fixtures/`](../../fixtures) for exactly this reason.
