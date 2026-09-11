# Roadmap

A milestone is marked complete only when the work exists in the repository and
is covered by tests. Nothing below is aspirational marketing.

| Milestone | Status |
|---|---|
| **M0 — Feasibility** | ✅ **Complete** (2026-09-10) |
| **M1 — Foundation** | ✅ **Complete** (2026-09-10) |
| M2 — Transaction & XDR engine | ⬜ Planned — next |
| M3 — Contract error resolution | ⬜ Planned |
| M4 — Failure classification | ⬜ Planned — **blocked on fixture diversity** |
| M5 — Accuracy & reliability | ⬜ Planned |
| M6 — Developer experience | ⬜ Planned |
| M7 — Ecosystem integration | ⬜ Planned |
| M8 — Observatory expansion | ⬜ Future |

---

## ✅ M0 — Feasibility

**Question:** do public mainnet RPC providers actually return decodable
diagnostic events for real failed Soroban transactions? If not, the project as
designed is dead.

**Answer: yes.** 4 of 4 responding providers, 13 of 13 transactions, 437 of 437
diagnostic events decoded. Decision: **GO**.

Delivered:

- `sdo-probe` with `probe`, `scan` and `capture` subcommands
- Per-provider measurements across every free public mainnet endpoint
- 4 real recorded fixtures, including a negative control
- A preliminary failure taxonomy grounded in XDR result types
- [M0 report](docs/research/m0-report.md),
  [RPC measurements](docs/research/rpc-diagnostic-events.md),
  [taxonomy](docs/research/failure-taxonomy.md)

Findings that changed the design: current mainnet returns `TransactionMetaV4`
(diagnostic events moved out of `sorobanMeta`); failed Soroban transactions are
overwhelmingly **fee-bumped**, so the real result is in the inner result pair;
and `stellar-xdr` 28 removed the `curr` module that every tutorial uses.

## ✅ M1 — Foundation

A Rust workspace with clear boundaries, offline tests, and CI.

Delivered:

- `soroban-failure-analysis` — taxonomy, `Diagnosis`, `Evidence`, `Rule` trait,
  `RuleRegistry`, ranking. **No I/O. No rules yet.**
- `soroban-failure-rpc` — RPC client, pure decoding, fixture loading, handling
  both `TransactionMeta` V3 and V4
- `sdo` CLI with `explain`, including `--fixture` for fully offline use
- 40 tests, all passing, **none requiring network access**
- CI running fmt, clippy (`-D warnings`), and tests on stable and MSRV (1.88)
- README, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, LICENSE, architecture docs

The engine runs end to end and reports honestly that it cannot yet attribute a
cause.

---

## ⬜ M2 — Transaction & XDR engine

**Goal:** determine *where* a transaction failed.

- Implement `FailureStage` classification from `TransactionResult`
- **Unwrap fee-bumped results** — a hard requirement, not an edge case (M0)
- Reconstruct the contract invocation tree from diagnostic events
- Extract declared vs. consumed resources, auth entries, and footprint entries
- Surface all of it through `sdo explain`

**Done when:** every fixture reports a correct, non-`Unknown` stage, and the
`FailureStage` limitation is removed from `analyze`.

## ⬜ M3 — Contract error resolution

**Goal:** turn `Error(Contract, #3)` into `InsufficientBalance`.

A Soroban contract error arrives as a bare `u32`. The name lives in the
contract's spec, which is retrievable on-chain. As far as our research found,
**nobody ships this** — it is the project's sharpest early differentiator.

- Decode `ScError` type/code pairs into human-readable strings
- Fetch the contract instance and WASM via `getLedgerEntries`
- Parse the contract spec and map error codes to their declared names
- Cache specs by WASM hash

**Done when:** `sdo explain` names a contract-defined error from a real fixture.

## ⬜ M4 — Failure classification

**Goal:** the first real rules.

> ### 🚧 Blocked on fixture diversity
>
> M0 found that **every** Soroban failure in our corpus is
> `invoke_host_function_trapped` — almost certainly the same arbitrage bots
> failing repeatedly. Five of the six planned rule categories have **no fixture
> at all**.
>
> Rules cannot be written against a corpus containing one category. Before M4
> starts, failures must be deliberately produced on testnet and captured:
> omit an auth entry, truncate a footprint, let an entry expire, under-declare
> resources, and return a contract error.
>
> **Any category without a fixture is not implementable.** Tracked in
> [issue #1](https://github.com/The-Big-Danny/Stellar-Developer-Observatory/issues/1).

Target rules: missing/invalid authorization entry, footprint entry missing,
archived entry requiring restore, resource limit exceeded, insufficient resource
fee, contract-defined error.

**Done when:** each rule has a fixture, fires on it, and stays silent on the
negative control.

## ⬜ M5 — Accuracy & reliability

**Goal:** know how often we are right, and publish it.

- Grow the corpus to ~100 real failed transactions, deduplicated by contract
- Measure top-ranked-cause accuracy against hand-labelled ground truth
- **Publish the number including the misses** — this is itself a differentiator
- Calibrate confidence levels against observed accuracy
- Harden decoding: explicit XDR depth and length limits, fuzzing

## ⬜ M6 — Developer experience

- JSON output and a published `Diagnosis` schema
- Better CLI formatting; colour, quiet and verbose modes
- Published docs and usage examples
- First crates.io release

## ⬜ M7 — Ecosystem integration

Adoption by an existing tool is worth more than any UI we could build.

- WASM build so JavaScript tooling can use the engine
- A GitHub Action asserting failure classes in CI
- Approach the Stellar Lab, Erst, and OpenZeppelin teams about consuming the crate

## ⬜ M8 — Observatory expansion

Only after the analyzer is genuinely good. Possibly a thin web UI, aggregate
failure statistics, or historical analysis — each judged on whether it is
already well served elsewhere. See
[the abandoned-pillars list](docs/research/00-validation.md) before proposing
anything here.

---

## Non-goals

Deliberately out of scope, because each is already well served:

| Not building | Use instead |
|---|---|
| An indexer | [Mercury](https://mercurydata.app/), Stellar CDP |
| Monitoring and alerting | [OpenZeppelin Monitor](https://docs.openzeppelin.com/monitor) |
| Static / security analysis | [Scout](https://github.com/CoinFabrik/scout-soroban) |
| A block explorer or transaction dashboard | [Stellar Lab](https://lab.stellar.org), StellarExpert |
| Local transaction replay | [Erst](https://github.com/dotandev/hintents) |
