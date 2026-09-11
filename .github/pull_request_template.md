## What this changes

## Why

## Milestone

Which milestone does this belong to? See [ROADMAP.md](../ROADMAP.md).

## Checklist

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] No test I added requires network access
- [ ] Docs updated in this PR if behaviour changed
- [ ] If this removes a limitation, the corresponding text in `analyze` / README is removed too

## If this touches `soroban-failure-analysis`

- [ ] No I/O was added — no network, filesystem, clock, env, or randomness
      (see [purity.md](../docs/architecture/purity.md))

## If this adds a rule

- [ ] A fixture exhibiting the failure is included
- [ ] The rule cites evidence for anything above `Confidence::Possible`
- [ ] The rule stays silent on the negative-control fixture
