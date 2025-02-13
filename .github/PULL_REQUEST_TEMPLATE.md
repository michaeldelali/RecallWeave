<!-- One topic per PR. -->

## What

<!-- What does this PR change? Engine lifecycle, viewer, docs... -->

## Why

<!-- The workflow gap this closes. -->

## Verification

- [ ] `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo test` + viewer tests green
- [ ] the weave format stays append-only (no meaning change for existing bytes)
- [ ] determinism: identical inputs still produce byte-identical output
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
