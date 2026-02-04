---
name: Feature request
about: A new lifecycle capability or viewer feature
title: "[feature] "
labels: ""
assignees: ""
---

**The memory-lifecycle gap**

Describe the real workflow that RecallWeave misses today (a new record kind,
a new query shape, a compaction policy, a viewer rendering).

**Proposed behavior**

Exact subcommand or flag, default values, and how it interacts with the
append-only weave. Remember the ground rules: the engine stays stdlib-only,
and existing store bytes never change meaning.

**Determinism statement**

How the feature keeps byte-identical output for identical inputs, and which
test file will pin it (`tests/cli.rs` or `tests/lifecycle.rs`).

**Alternatives considered**

What you tried with the current commands and why it is not enough.
