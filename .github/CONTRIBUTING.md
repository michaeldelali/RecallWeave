# Contributing to RecallWeave

Thank you for considering a contribution. RecallWeave is intentionally
dependency-free - the Rust engine builds on the standard library alone, and the
TypeScript viewer builds on plain `tsc`. Every rule below serves that goal.

## Ground rules

- **Stdlib only on the engine.** No new entries in `[dependencies]`. If a
  feature truly needs one, open a discussion first; the bar is "hand-rolled in
  a hundred lines or less".
- **The weave format is append-only.** Never change the meaning of an existing
  byte in `log.jsonl`. Backwards-compatible readers are a hard requirement;
  add new record kinds, don't mutate old ones.
- **Determinism is tested.** Dedupe, compaction, export and SVG rendering must
  produce byte-identical output for identical inputs. Extend
  `tests/lifecycle.rs` when you touch any of them.
- **Every lifecycle command ships with a CLI test.** New subcommands land with
  a case in `tests/cli.rs` and a section in the README command reference.

## Local workflow

```bash
make build     # cargo build + viewer tsc build
make test      # cargo test + viewer tests
cargo fmt      # formatting is checked in CI (cargo fmt --check)
cargo clippy --all-targets -- -D warnings   # warnings deny warnings
```

Please make sure CI's exact checks pass locally before opening a PR.

## Pull requests

- One topic per PR.
- Conventional commit messages (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).
- Update `CHANGELOG.md` under `[Unreleased]` as part of your change.
- Include a transcript of the command you exercised when the change touches
  CLI behavior.

## Reporting issues

Open a bug report with the store directory involved (or a trimmed
`memory-pack.json` that reproduces). For integrity-chain failures include the
`verify` output - that is the whole diagnosis.

## Code of conduct

The Contributor Covenant applies to everyone participating in this project -
see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
