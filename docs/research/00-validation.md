# Stellar Developer Observatory — Pre-Build Validation

**Date:** 2026-09-10
**Status:** Research complete. Product direction revised. Not yet building.
**Verdict:** The *category* is valid. The *proposed V1 is not* — it collides head-on with SDF's own funded roadmap. A revised V1 is proposed in §D.

---

## Executive summary

Three findings drove the revision:

1. **Stellar Lab's Transaction Dashboard already ships most of the proposed V1 UI.** It has Token Summary, Contracts, Events, State Change, Resource Profiler, Signatures, and Fee Breakdown tabs, and auto-detects classic vs. smart-contract transactions. A web page where you paste a hash and see contracts/events/resources is *done*.
2. **SDF has explicitly committed to going further.** The published roadmap states Lab 4.0 "will deliver deep insights into transaction-level simulation, debugging, and resource profiling." Competing with the foundation, on the foundation's flagship tool, using the foundation's grant money, is a losing position.
3. **But nobody has built the interpretation layer.** Every existing tool — Lab, StellarExpert, Soroscan, explorers — *renders* execution data. None of them state a **cause**. There is no tool, and no library, that takes a failed transaction and answers "this failed because the authorization entry for account X was missing." That gap is real, narrow, and defensible.

The revision: **stop building a viewer, build the analyzer other viewers can embed.**

---

## A. Competitor matrix

| Tool | What it actually does | Strengths | Weaknesses | Users | OSS? | Overlap | Gap it leaves |
|---|---|---|---|---|---|---|---|
| **Stellar Lab — Transaction Dashboard** | Per-transaction inspector. Tabs: Token Summary, Contracts, Events (decoded topics + payloads + emitting contract, chronological), State Change (storage read/write/delete, balance, trustline, contract instance, archival/restore, footprint), Resource Profiler (CPU insns, memory, read/write bytes, tx size, resource fee), Signatures, Fee Breakdown | Official, free, well-designed, decodes XDR properly, already covers ~70% of proposed V1 | **Single transaction only.** No aggregation, no history view, no API, no library, no alerting, no CI hook. **Renders data; does not diagnose.** Bound by RPC's 7-day window | All Stellar devs | Yes (`stellar/laboratory`) | **Very high** — this was the proposed V1 | Causal explanation; programmatic access |
| **Lab 4.0** (roadmapped Q4 2025, not shipped as of writing) | "Transaction-level simulation, debugging, and resource profiling" | SDF resources, distribution, default tool | Unshipped; scope unclear | All Stellar devs | Yes | **Existential to a UI-first plan** | Unknown — assume it closes the UI gap |
| **StellarExpert** | Block explorer with full history | Full historical range (beats RPC's 7 days), mature, trusted | Explorer-grade, not developer-debug grade. Oriented to accounts/assets/ops, not contract-execution failure analysis | End users, analysts, some devs | Partly | Low–medium | Developer-grade failure semantics |
| **Soroscan** | Event indexer → GraphQL / REST / webhooks | Real-time webhooks, queryable | Indexing/plumbing, not diagnosis. Success-path oriented | Devs needing event feeds | Yes | Low | Failure-path analysis |
| **Erst / `dotandev/hintents`** | Go CLI + Rust `erst-sim`; replays failed txs locally against `soroban-env-host`; colorized traces, flamegraphs, WASM→Rust source mapping, heuristic error suggestions | **Closest conceptual competitor.** Local replay + source mapping is genuinely strong. Active (created Jan 2026, pushed Sept 2026) | README admits simulation "pending" for MVP. Source mapping requires debug symbols — usually absent in deployed mainnet WASM. **13 stars / 257 forks / 50 open issues** — that ratio is a bounty/grant contributor pipeline, not adoption. Positioned as "premium" | Soroban contract devs | Apache-2.0 | **Medium–high on the failure-diagnosis niche** | Reproducibility-free analysis (no local replay needed); library form factor |
| **OpenZeppelin Monitor** | Production event/function monitoring + alerting for Soroban; uses P23 contract-spec named event params | Mature org, real alerting, Rust, self-hostable | Monitors *events you define*; doesn't diagnose *failures*. Config-driven, not diagnostic | Protocol teams in production | Yes | Low (kills the "Application Monitoring" pillar) | Failure classification as an alert signal |
| **Mercury / Retroshades (Xycloo)** | Stellar-native indexer; custom in-contract indexing logic via a forked SVM; Zephyr VM/SDK | Genuinely differentiated tech, Stellar-native team, mature | Commercial; indexing not diagnosis | Teams needing custom indexes | Partly | Low (kills the "build our own indexer" pillar) | — |
| **Solarkraft (freespek)** | Runtime monitoring vs. TLA+/Apalache specs | Rigorous, novel | **Dead** — last push Feb 2025, 12 stars, 1 fork. Requires writing formal specs | Researchers | Yes | Low | — |
| **Scout (CoinFabrik)**, **soroban-analyzer (Xycloo)** | Static analysis: security lints, gas inefficiency | Mature, useful | Pre-deployment source analysis; orthogonal to runtime failure | Auditors, devs | Yes | None | — |
| **Tenderly (EVM)** | Simulation, tx debugger with per-opcode traces, state diffs, alerting | The reference product for this category | EVM-only. Its power derives from EVM's opcode-level trace APIs (`debug_traceTransaction`) — **Soroban has no equivalent**, so the architecture does not port | EVM devs | No | Inspirational only | — |
| **Sentry (Web2)** | Error *grouping/fingerprinting*, regression detection, release correlation | The right mental model: the value is in **grouping and ranking errors**, not in displaying one | Not blockchain | All devs | Partly | Inspirational — see §C | — |

**Conclusion:** the ecosystem is crowded on *rendering*, *indexing*, *monitoring*, and *static analysis*. It is empty on **causal classification of failures**.

---

## B. Problem validation

**The pain point is real and documented by Stellar's own core team.**

`stellar/stellar-core#3816` ("Improve (Soroban) transaction submission error reporting", opened July 2023, now closed) states the problem precisely: developers receive a generic **function-trapped** error for a wide range of distinct causes — bad auth, wrong footprint, insufficient resources, insufficient fee. The issue asks that errors name the missing auth entry, the missing footprint entry, the depleted resource. It was closed without a linked implementation. **The underlying developer-experience gap was never fully closed at the protocol layer.**

Scored against the validation framework:

| Criterion | Score | Evidence |
|---|---|---|
| Problem severity | **High** | One opaque code maps to 4+ unrelated root causes |
| Frequency | **High** | Auth and footprint errors are the standard Soroban failure modes |
| Affected developers | **Medium** | Every Soroban dev — but the Soroban dev population is still modest; this caps the ceiling (see §H) |
| Existing solutions | **Partial** | Lab shows evidence; Erst replays locally; neither classifies |
| Gap | **Real** | No tool outputs a ranked cause with cited evidence |
| Stellar specificity | **Very high** | ScError semantics, auth entries, footprints, archival state, resource metering are Stellar-native. Not portable from EVM |
| Technical feasibility | **Medium** (see §E) | Gated on diagnostic-event availability |
| OSS potential | **High** | A classifier is a rule corpus — inherently collaborative |
| Contributor potential | **High** | "Add a rule for failure mode X" is a perfect issue shape |
| Ecosystem value | **High** | Every other tool can embed it |

**Weakest link: addressable developer count.** Be honest about this — see §H, risk 4.

---

## C. Differentiation

> **Existing tools show you *what* the transaction did. Stellar Developer Observatory tells you *why it failed*, cites the evidence, and does it in a library any tool can embed.**

Three concrete differentiators, in priority order:

1. **Causal classification, not rendering.** Input: a failed tx. Output: a ranked list of candidate causes, each with (a) a machine-readable class, (b) the specific diagnostic events / auth entries / footprint entries that support it, (c) a remediation. Lab shows you the auth section; we tell you *which entry is missing*.

2. **Contract-error-code resolution.** A Soroban contract error surfaces as `Error(Contract, #3)` — a bare `u32` with no name. The name lives in the contract's spec, retrievable on-chain via the contract's WASM. **Resolving `#3` → `InsufficientBalance` by fetching and parsing the contract spec is small, concretely valuable, uniquely Stellar-native, and — as far as this research found — nobody ships it.** This alone justifies a tool.

3. **Library-first form factor.** A Rust crate + CLI, not a website. This is the strategic wedge: a website competes with Lab; a crate is something **Lab itself could consume.** Being the dependency beats being the alternative. It also unlocks the CI/test-assertion use case nobody serves.

**Explicitly abandoned pillars** — each already well-served; do not build these:

- Application monitoring / alerting → OpenZeppelin Monitor
- Indexing / historical infrastructure → Mercury, Soroscan, and SDF's CDP + data-lake-backed RPC
- Static/security analysis → Scout, soroban-analyzer
- Generic dashboards / explorer → Lab, StellarExpert

---

## D. Revised V1

**Name the deliverable honestly:** this V1 is a **Soroban Failure Classifier**, not an "observatory." The observatory is the three-year story; keep it out of the README for now.

### Scope

A Rust workspace:

- **`soroban-failure-analysis`** (core crate, no I/O) — pure function:
  `(TransactionEnvelope, TransactionResult, TransactionMeta, Vec<DiagnosticEvent>, Option<ContractSpec>) -> Diagnosis`
- **`soroban-failure-rpc`** — thin RPC fetcher (`getTransaction`, `getLedgerEntries` for the contract spec)
- **`sdo` CLI** — `sdo explain <tx-hash> --network mainnet`, human output plus `--json`

### What `Diagnosis` contains

```
failure_stage:    Validation | Sequence | Fee | HostFunction | ContractExecution
                  | Auth | Footprint | StateArchival | ResourceLimit
error:            ScErrorType + code, OR contract error code resolved to its spec name
candidate_causes: ranked; each { class, confidence, evidence[], remediation }
evidence:         pointers to the exact diagnostic event / auth entry / footprint entry
call_stack:       contract invocation tree reconstructed from diagnostic call-stack events
resources:        consumed vs. declared limits
```

### V1 rule coverage — target 6 classes, done properly

1. Missing/invalid authorization entry (name the account and the required invocation)
2. Footprint entry missing (name the ledger key that was touched but not declared)
3. Archived ledger entry requiring restore (name the entry)
4. Resource limit exceeded (name *which* resource, declared vs. consumed)
5. Insufficient resource fee
6. Contract-defined error, **resolved to its spec name**

### Explicit non-goals for V1

No web UI. No database. No indexer. No alerting. No historical aggregation. No local replay — that is Erst's ground; interoperate later, don't duplicate.

### Success criterion

Run against 100 real failed mainnet Soroban transactions. Measure: % where the top-ranked cause is correct. **Publish that number, including the misses.** That honesty is itself a differentiator, and it is exactly the kind of artifact that establishes maintainer credibility.

---

## E. Technical feasibility — including two landmines

### Data sources

| Need | Source | Notes |
|---|---|---|
| Envelope, result, meta | `getTransaction` → `envelopeXdr`, `resultXdr`, `resultMetaXdr` | Present for both SUCCESS and FAILED |
| Diagnostic events | `getTransaction` → `diagnosticEventsXdr` | **Conditional — Landmine 1** |
| Contract events | `resultMetaXdr.sorobanMeta.events` | **Not populated on failure — Landmine 2** |
| Contract spec (error names) | `getLedgerEntries` → contract instance → WASM hash → contract code → parse spec | Core of differentiator #2 |
| Auth entries | `TransactionEnvelope` → `SorobanAuthorizationEntry[]` | Compare declared vs. required |
| Footprint | `SorobanTransactionData.resources.footprint` | Compare declared vs. touched |
| Resources | `SorobanTransactionData` + meta ext | Declared vs. consumed |

### 🔴 Landmine 1 — diagnostic events may not be available on public mainnet RPC

Diagnostic events are only returned if the node was started with `--enable-soroban-diagnostic-events`. It is **enabled by default only in local/quickstart mode.** Stellar's own guidance is to enable it only on *watcher* nodes (`NODE_IS_VALIDATOR=false`), because it makes the node diverge from normal execution paths. They are unmetered and **not part of consensus** — two nodes can legitimately return different diagnostic output.

**This is the single biggest risk to V1.** The entire product assumes diagnostic events are retrievable for arbitrary mainnet transactions.

**Mitigation — do this in week 1, before writing product code:** empirically probe each major public mainnet RPC provider with a known-failed Soroban transaction and record whether `diagnosticEventsXdr` is populated. Publish the results as a table in the repo. If coverage is poor, the fallback is a self-hosted watcher node with the flag on — which changes the project from "a crate" to "a crate plus infrastructure" and must be known *before* committing.

*(This probe is a genuinely useful public artifact for the ecosystem regardless of whether the project proceeds.)*

### 🟠 Landmine 2 — contract events are absent on failed transactions

`sorobanMeta.events` is populated **only when the transaction succeeds.** For failures, diagnostic events are the *sole* source of execution-flow evidence. This compounds Landmine 1: without diagnostic events there is essentially nothing to analyze beyond the result code and the envelope. Design the `Diagnosis` type so it degrades gracefully and states plainly what it could not determine.

### Historical data — resolved; don't build it

RPC's default retention is 120,960 ledgers (~7 days). This is **no longer a reason to build an indexer**: SDF has integrated RPC with a ledger data lake enabling queries back to genesis, and Galexie/CDP exists for self-hosting (full-history export ≈150 days on a single instance, ~4–5 days across 40–50 parallel instances). Use SDF's infrastructure. Building an indexer would be the single largest scope error available here.

### Language

Rust is correct, and not for aesthetic reasons: `stellar-xdr`, `soroban-env-common` / `soroban-env-host`, and the canonical `ScError` semantics are all Rust-first. `soroban_env_common::Error` already decodes the 28-bit type / 32-bit code pair. Working in TypeScript means reimplementing semantics that exist canonically in Rust.

---

## F. Architecture

Drop the original layered diagram — it presumes an indexer and a service. The correct V1 shape is a library with adapters:

```
                     ┌──────────────────────────────┐
  stellar RPC ─────► │  soroban-failure-rpc         │  fetch + decode XDR
  (getTransaction,   │  (I/O, network-aware)        │  fetch contract spec
   getLedgerEntries) └─────────────┬────────────────┘
                                   │  typed inputs
                                   ▼
                     ┌──────────────────────────────┐
                     │ soroban-failure-analysis     │  NO I/O. Pure. Deterministic.
                     │                              │
                     │   stage classifier           │  where in the pipeline it died
                     │   error resolver             │  ScError + spec → name
                     │   evidence extractor         │  diag events → call stack
                     │   rule engine  ◄── rules/    │  each rule: evidence → cause
                     │   ranker                     │  confidence ordering
                     └─────────────┬────────────────┘
                                   │  Diagnosis (serde)
                     ┌─────────────┴────────────────┐
                     ▼                              ▼
                 sdo CLI                      JSON / consumers
            (human + --json)         (CI, other tools, a future UI, Lab)
```

**Load-bearing design decisions:**

1. **The core crate performs no I/O.** Every rule is a pure function over decoded structures. This makes the whole rule corpus testable from fixture files with zero network — which is what makes outside contribution possible at all.
2. **Rules are a registry, not a match statement.** A `Rule` trait with `fn evaluate(&self, ctx: &FailureContext) -> Option<CandidateCause>`. Adding a failure mode = one file + one fixture. **This is the contributor pipeline, expressed in code.**
3. **Fixtures are captured real transactions.** A committed `fixtures/` directory of recorded RPC responses. Contributors need no mainnet access and no funded account.
4. **Ranked candidates, never a single verdict.** Diagnostic evidence is heuristic and non-consensus. Claiming certainty you don't have will destroy trust the first time it's wrong.

---

## G. Roadmap

**M0 — Feasibility probe (week 1). Gate: go/no-go.**
- Probe every public mainnet RPC provider for `diagnosticEventsXdr` on known-failed txs
- Collect ~100 real failed Soroban mainnet transactions into `fixtures/`
- Manually categorize their failure modes → this dataset *is* the product requirements
- **If diagnostic events are broadly unavailable and self-hosting is out of reach, stop here and reassess.** Publish the findings either way.

**M1 — Core decode (weeks 2–3).** Workspace scaffold; XDR decode of envelope/result/meta/diagnostics; failure-stage classifier; `sdo explain` printing structured raw facts. No rules yet.

**M2 — Error resolution (weeks 4–5).** `ScError` type/code decoding with human strings; contract-spec fetch + parse; **contract error code → name.** First genuinely novel output — ship and announce it.

**M3 — Rule engine + first 6 rules (weeks 6–9).** Rule trait, registry, ranker, evidence citation. Auth, footprint, archival, resource limit, fee, contract error.

**M4 — Accuracy report + v0.1.0 (week 10).** Run against the 100-tx corpus; publish accuracy including failures. Release to crates.io. Docs site. Announce on Stellar Dev Discord + SCF.

**M5 — Community (weeks 11–14).** CONTRIBUTING with a "how to add a rule" walkthrough; 10+ `good-first-issue` rules each with a linked fixture; issue/PR templates; CI (fmt, clippy, test, fixture regression).

**M6 — Reach (post-v0.1).** JSON schema for `Diagnosis`; WASM build for JS consumers; a GitHub Action asserting failure classes in CI; *then* consider a thin web UI. Approach the Lab, Erst, and OpenZeppelin teams about consuming the crate — adoption by an existing tool is worth more than any UI you could ship.

**Contributor issue ladder:**
- *Beginner:* one new rule + fixture; improve a remediation message; docs
- *Intermediate:* contract-spec parsing edge cases; call-stack reconstruction; JSON schema; CLI output formats
- *Advanced:* confidence calibration; multi-cause disambiguation; WASM bindings; Erst interop

---

## H. Risk assessment

| # | Risk | Severity | Mitigation |
|---|---|---|---|
| 1 | **Diagnostic events unavailable on public mainnet RPC** | 🔴 Critical | M0 gate. Probe first. Self-hosted watcher node as fallback. Do not write product code before this resolves |
| 2 | **SDF ships Lab 4.0 covering this** | 🔴 High | Library-first, not UI-first. Be the dependency, not the competitor. Actively offer the crate to the Lab team |
| 3 | **Erst/hintents gets there first** | 🟠 Medium-high | Different form factor (library vs. CLI product), different method (no local replay), different licence posture (fully OSS vs. "premium"). Their 257-fork/13-star profile suggests funded contributor churn rather than users — beatable on adoption, but do not underestimate a funded team |
| 4 | **Small addressable audience** — Soroban's dev population is still modest | 🟠 Medium | Accept it. A small, real, well-served audience beats a large imagined one. Ecosystem-infrastructure positioning (other tools depend on you) multiplies reach beyond direct users |
| 5 | **Heuristics are wrong and burn trust** | 🟠 Medium | Ranked candidates with confidence, never a bare verdict. Always cite evidence. Publish the accuracy number including misses |
| 6 | **Protocol changes invalidate rules** | 🟡 Medium | Version rules against protocol version; regression fixtures per protocol; CI |
| 7 | **No contributors show up** | 🟡 Medium | The rule-registry architecture is the mitigation — it makes a contribution genuinely 1 file + 1 fixture. Seed 10+ real issues before announcing |
| 8 | **Scope creep back toward "observatory"** | 🟡 Medium | The abandoned-pillars list in §C is binding. Re-read it before accepting any feature |
| 9 | **Solo-maintainer burnout** | 🟠 Medium | Narrow V1 (10 weeks, not 10 months). Ship v0.1.0 before seeking contributors |

---

## Recommendation

**Proceed — but with the revised, narrower V1, and only after the M0 feasibility gate passes.**

The original framing ("Sentry/Tenderly for Stellar," a transaction-debugger web UI) should be dropped. It is ~70% already shipped by Stellar Lab and 100% on SDF's roadmap. Building it would produce a project that is competent, redundant, and unadopted — the worst outcome for the maintainer goal, because credibility comes from being *depended upon*, not from having *built something*.

The revised framing — **the open-source engine that explains why Soroban transactions fail, that every other Stellar tool can embed** — is smaller, harder to dismiss, genuinely unoccupied, and far better suited to the actual objective.

**Next action: M0. Probe the RPC providers. Everything else waits on that answer.**

---

## Sources

- [Stellar Lab — Transaction Dashboard](https://developers.stellar.org/docs/tools/lab/transaction-dashboard)
- [Stellar Lab overview](https://developers.stellar.org/docs/tools/lab)
- [SDF Product Roadmap](https://stellar.org/foundation/roadmap)
- [getTransaction — Stellar RPC](https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getTransaction)
- [Diagnostic Events — Stellar Docs](https://developers.stellar.org/docs/tools/quickstart/debugging/diagnostic-events)
- [Events — Stellar Data Structures](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/events)
- [Indexers Overview — Stellar Docs](https://developers.stellar.org/docs/data/indexers)
- [Galexie full-history exporting](https://developers.stellar.org/docs/data/indexers/build-your-own/galexie/admin_guide/full-history-exporting)
- [stellar-core#3816 — Improve Soroban transaction submission error reporting](https://github.com/stellar/stellar-core/issues/3816)
- [dotandev/hintents (Erst)](https://github.com/dotandev/hintents)
- [freespek/solarkraft](https://github.com/freespek/solarkraft)
- [CoinFabrik/scout-soroban](https://github.com/CoinFabrik/scout-soroban)
- [xycloo/soroban-analyzer](https://github.com/xycloo/soroban-analyzer)
- [OpenZeppelin Monitor](https://docs.openzeppelin.com/monitor)
- [Retroshades — Xycloo Labs](https://blog.xycloo.com/blog/retroshades)
- [Mercury](https://mercurydata.app/)
- [soroban_env_host::Error](https://docs.rs/soroban-env-host/latest/soroban_env_host/struct.Error.html)
- [stellar/laboratory](https://github.com/stellar/laboratory)
