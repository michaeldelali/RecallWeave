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
