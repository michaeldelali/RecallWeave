---
name: Bug report
about: The engine misbehaved on a store or pack
title: "[bug] "
labels: ""
assignees: ""
---

**What happened?**

A clear description: the command you ran, the lifecycle stage involved
(weave / dedupe / supersede / conflict / forget / compact / export / verify),
and what came out versus what you expected.

**Reproducer**

Attach the store directory or a trimmed `memory-pack.json`. Small files only -
trim to the records involved.

```text
recallweave --dir <store> <command> ...
```

**Version**

The git revision (or release tag) you ran.

**Expected vs actual**

- Expected:
- Actual (transcript trimmed):

---

*Note: if `verify` flags your store, attach the verify output - the integrity
chain points at the first bad record, which is the whole diagnosis.*
