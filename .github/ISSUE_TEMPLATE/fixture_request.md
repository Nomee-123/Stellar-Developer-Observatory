---
name: Fixture contribution
about: Add a recorded transaction to the test corpus
title: "fixture: "
labels: ["fixtures", "good first issue"]
---

## Failure category

Which category from [the taxonomy](../../docs/research/failure-taxonomy.md) does
this cover? Categories with **no fixture at all** are the most valuable: auth,
footprint, archival, resource limit, resource fee.

## Transaction

- Hash:
- Network:
- Ledger:
- How it was produced (mainnet scan, or deliberately caused on testnet):

## Checklist

- [ ] Captured with `cargo run -p sdo-probe -- capture`, not hand-written
- [ ] `rpc-response.json` is verbatim and unedited
- [ ] `README.md` written, documenting provenance and why the fixture is useful
- [ ] `cargo test --workspace` passes
- [ ] Adds a category or shape the corpus does not already have
