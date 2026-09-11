---
name: New failure rule
about: Propose or claim a rule that detects a specific Soroban failure mode
title: "rule: "
labels: ["rust", "soroban", "help wanted"]
---

## Failure mode

Which failure does this rule detect? Name the `CauseClass` it would produce, or
propose a new one.

## Evidence it would rely on

What in the transaction distinguishes this failure from the others? Be specific:
which diagnostic event, which auth entry, which footprint key, which result code.

## Fixture

**A rule without a fixture cannot be merged** — it would be untested by
construction.

- [ ] A fixture for this failure mode already exists: `fixtures/failed/...`
- [ ] I will capture one (say how: mainnet scan, or deliberately produced on testnet)
- [ ] I do not know how to produce this failure — **help wanted**

See [the taxonomy](../../docs/research/failure-taxonomy.md) for which categories
currently have no fixture.

## Expected output

Roughly what should `sdo explain` say? Include the remediation you would want to
read if this happened to you.

## Acceptance criteria

- [ ] Rule implemented in `crates/soroban-failure-analysis/src/rules/`
- [ ] Registered in `RuleRegistry::builtin()`
- [ ] Fires on its own fixture with cited evidence
- [ ] Stays silent on `fixtures/failed/classic-failed-no-diagnostics`
- [ ] Returns `None` when `has_diagnostic_evidence()` is false, if it reasons from events
- [ ] Taxonomy docs updated
