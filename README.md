# Stellar Developer Observatory

**An open-source engine that explains why Soroban transactions fail.**

[![CI](https://github.com/The-Big-Danny/Stellar-Developer-Observatory/actions/workflows/ci.yml/badge.svg)](https://github.com/The-Big-Danny/Stellar-Developer-Observatory/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> ### ⚠️ Status: early development / experimental
>
> **This project cannot yet tell you why your transaction failed.**
>
> Milestones M0 (feasibility) and M1 (foundation) are complete. The engine
> decodes transactions, structures the evidence, and states what it cannot
> determine — but **no failure rules are implemented yet** (milestone M4), and
> failure-stage classification is not implemented yet (milestone M2).
>
> Nothing in this README describes a capability that does not exist. See
> [ROADMAP.md](ROADMAP.md).

---

## The problem

When a Soroban transaction fails, the result code is usually
`invoke_host_function_trapped`. That one code covers a missing authorization
entry, an under-declared footprint, an archived ledger entry, an exhausted
resource limit, and a contract's own error — causes that have nothing to do with
each other and completely different fixes.

This is not a novel complaint.
[`stellar-core#3816`](https://github.com/stellar/stellar-core/issues/3816),
opened by Stellar's own team in 2023, asked for errors that name the missing auth
entry, the missing footprint entry, and the depleted resource. It was closed
without a linked implementation.

Our own measurement in [M0](docs/research/m0-report.md) found 10 consecutive
mainnet Soroban failures reporting one identical result code, with no way to
distinguish them without reading diagnostic events by hand.

## The approach

Existing Stellar tools — Stellar Lab, StellarExpert, the various explorers —
**render** execution data, and several do it very well. None of them state a
**cause**.

This project is the interpretation layer, and it is a **library first, not a
website**:

```
                    ┌──────────────────────────────┐
   Stellar RPC ───► │  soroban-failure-rpc         │  fetch, decode XDR
                    │  (the only I/O in the repo)  │
                    └─────────────┬────────────────┘
                                  │  typed AnalysisInput
                                  ▼
                    ┌──────────────────────────────┐
                    │ soroban-failure-analysis     │  NO I/O. Pure. Deterministic.
                    │   stage classifier   (M2)    │
                    │   error resolver     (M3)    │
                    │   rule engine        (M4)    │
                    │   ranker                     │
                    └─────────────┬────────────────┘
                                  │  Diagnosis
                    ┌─────────────┴────────────────┐
                    ▼                              ▼
                 sdo CLI                    JSON / other tools
```

A website would compete with Stellar Lab. A crate is something Lab — or any
explorer, or a CI pipeline — could **consume**. Being the dependency beats being
the alternative.

### Design commitments

- **Ranked candidates, never a single verdict.** Diagnostic events are unmetered
  and are *not part of consensus*. Claiming certainty we do not have would
  destroy trust the first time it is wrong.
- **Every claim cites evidence.** A cause points at the exact diagnostic event,
  auth entry, or footprint entry that supports it.
- **Absence of evidence is not evidence.** The engine distinguishes "this node
  emits no diagnostic events" from "this transaction had none", and rules may not
  conclude anything from silence.
- **The core crate performs no I/O**, so the whole rule corpus is testable from
  committed fixtures with no network.

## What this is *not*

- ❌ Not a block explorer
- ❌ Not an indexer — use [Mercury](https://mercurydata.app/) or Stellar's CDP
- ❌ Not a replacement for [Stellar Lab](https://lab.stellar.org), which already
  renders transactions well and which we would rather be used *by*
- ❌ Not an AI chatbot — this is a deterministic rule engine
- ❌ Not a monitoring or alerting platform — see
  [OpenZeppelin Monitor](https://docs.openzeppelin.com/monitor)

## Repository layout

| Path | What it is |
|---|---|
| [`crates/soroban-failure-analysis`](crates/soroban-failure-analysis) | The pure analysis engine. No I/O, ever. |
| [`crates/soroban-failure-rpc`](crates/soroban-failure-rpc) | RPC access, XDR decoding, fixture loading |
| [`crates/sdo`](crates/sdo) | The `sdo` command line interface |
| [`tools/rpc-probe`](tools/rpc-probe) | `sdo-probe`, the M0 research instrument |
| [`fixtures/`](fixtures) | Real recorded mainnet responses for offline tests |
| [`docs/research/`](docs/research) | Validation and measurement write-ups |

## Try it

Requires Rust 1.88 or newer.

```bash
git clone https://github.com/The-Big-Danny/Stellar-Developer-Observatory
cd Stellar-Developer-Observatory
cargo test --workspace
```

Analyse a committed fixture — no network needed:

```bash
cargo run -p sdo -- explain --fixture fixtures/failed/soroban-trapped-feebump-49ev
```

```
Transaction
  6d597ca6a4d168770b91d89769b8343a6d5ef11d78201c606ffd7caecc3058bc

Status
  FAILED

Failure stage
  unknown

Evidence available
  diagnostic events: 49 (from TopLevel)
  transaction metadata: present

Candidate causes
  none — this build cannot yet attribute a cause

Limitations
  - No failure rules are implemented yet (milestone M4). ...
  - Failure-stage classification is not implemented yet (milestone M2) ...
```

That output is the honest current state of the project.

Measure an RPC endpoint yourself:

```bash
cargo run -p sdo-probe -- scan --want 5
cargo run -p sdo-probe -- probe --tx <TRANSACTION_HASH>
```

## Research

M0 settled the question that could have killed the project — whether public
mainnet RPC actually returns diagnostic events. It does: 4 of 4 responding
providers, 13 of 13 transactions, 437 of 437 events decoded.

- [Pre-build validation](docs/research/00-validation.md) — competitor analysis
  and why this is library-first
- [M0 feasibility report](docs/research/m0-report.md) — method, results, and the
  **GO** decision
- [RPC diagnostic-event availability](docs/research/rpc-diagnostic-events.md) —
  per-provider measurements
- [Failure taxonomy](docs/research/failure-taxonomy.md) — including the honest
  finding that our fixture corpus currently covers only one failure category

## Contributing

Contributions are genuinely wanted, and the architecture exists to make them
possible: a new failure mode should be **one rule file, one fixture, one test**.

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the
[`good first issue`](https://github.com/The-Big-Danny/Stellar-Developer-Observatory/labels/good%20first%20issue)
label.

## License

[MIT](LICENSE).
