# Contributing to RecallWeave

Thank you for considering a contribution. RecallWeave is intentionally
dependency-free - the Rust engine builds on the standard library alone, and the
TypeScript viewer builds on plain `tsc`. Every rule below serves that goal.

## Ground rules

- **Stdlib only on the engine.** No new entries in `[dependencies]`. If a
  feature truly needs one, open a discussion first; the bar is "hand-rolled in
