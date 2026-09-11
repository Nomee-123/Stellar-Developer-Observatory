# Architecture

## Shape

```
                     ┌──────────────────────────────┐
  Stellar RPC ─────► │  soroban-failure-rpc         │
  getTransaction     │                              │
  getLedgerEntries   │   client.rs   ← the only     │
                     │               networking     │
                     │   decode.rs   ← pure         │
                     │   fixture.rs  ← pure         │
                     └─────────────┬────────────────┘
                                   │  AnalysisInput (typed, decoded)
                                   ▼
                     ┌──────────────────────────────┐
                     │ soroban-failure-analysis     │
                     │                              │
                     │   taxonomy.rs  stage + cause │
                     │   input.rs     the boundary  │
                     │   rule.rs      Rule trait    │
                     │   diagnosis.rs the output    │
                     │   lib.rs       ranking       │
                     │                              │
                     │   NO I/O. Deterministic.     │
                     └─────────────┬────────────────┘
                                   │  Diagnosis
                     ┌─────────────┴────────────────┐
                     ▼                              ▼
                 sdo CLI                     other consumers
                                      (CI, explorers, a future UI)
```

## The four decisions that matter

### 1. The core crate performs no I/O

Every rule is a pure function over decoded structures, so the whole corpus is
testable from committed fixtures with no network. This is the decision the rest
of the project hangs off. It has its own document:
[purity.md](purity.md).

### 2. Rules are a registry, not a match statement

A `Rule` is a trait with `evaluate(&FailureContext) -> Option<CandidateCause>`.
Rules are registered in `RuleRegistry` and never inspect each other.

Adding a failure mode is therefore **one file, one fixture, one test** — not a
change to a growing central `match` that every contributor has to understand and
that every PR conflicts on. This is the contributor pipeline expressed in code.

The registry currently ships **empty**. Rules are milestone M4.

### 3. Stage is observed; cause is claimed

Two separate enums, deliberately:

- `FailureStage` is read mechanically from the transaction result. Right or
  wrong, no judgement.
- `CauseClass` is a rule's interpretation, and always arrives wrapped in a
  `CandidateCause` carrying a `Confidence` and `Evidence`.

Diagnostic events are unmetered and **not part of consensus**. A tool that
presents an inference with the same authority as a result code will eventually
be confidently wrong, and will deserve to lose its users. The type system keeps
the two apart so that cannot happen by accident.

### 4. Absence of evidence is not evidence of absence

`AnalysisInput` carries `diagnostics_enabled` alongside `diagnostic_events`.
An empty event list means two very different things:

| `diagnostics_enabled` | `diagnostic_events` | Meaning |
|---|---|---|
| `false` | empty | The node does not emit them. **Infer nothing.** |
| `true` | empty | The node emits them and this transaction produced none. **A real observation.** |

Rules must call `has_diagnostic_evidence()` before reasoning from events. This
exists because M0 established that diagnostic events depend on node
configuration, so their absence genuinely can be meaningless.

## Output

```
Diagnosis {
    transaction_hash,
    stage:            FailureStage,            // observed
    candidate_causes: Vec<CandidateCause>,     // ranked, strongest confidence first
    limitations:      Vec<String>,             // what could NOT be determined
    rules_evaluated:  usize,
}
```

`limitations` is not decoration. A diagnostic tool that degrades silently is
worse than one that refuses, so the engine always states what it could not do —
including, right now, that it has no rules.

Ranking is stable: `sort_by_key` with `Reverse(confidence)` preserves
registration order within a confidence band, so output does not shift between
runs.

## Testing

| Layer | Where | Network? |
|---|---|---|
| Unit | alongside each module | never |
| Fixture / integration | `crates/sdo/tests/fixtures.rs` | never |
| Live measurement | `tools/rpc-probe`, run by hand | yes — that is its purpose |

Integration tests also guard the corpus itself: required files, hash consistency
between `probe.json` and `rpc-response.json`, reproducibility of analysis, and
that no fixture contains a secret.

CI never contacts an RPC provider. A test that needs the network is a bug.

## Further reading

- [The purity rule](purity.md)
- [Failure taxonomy](../research/failure-taxonomy.md)
- [M0 feasibility report](../research/m0-report.md)
- [Pre-build validation](../research/00-validation.md) — why library-first
