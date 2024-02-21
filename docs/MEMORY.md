# MEMORY.md — the recallweave data model & lifecycle

This document is the reference for how `recallweave` represents an agent's
long‑term memory, how the append‑only log works, and — importantly — an honest
account of what the engine does **not** do.

If you only read one section, read [Honest limitations](#honest-limitations).

---

## 1. What a memory is

A **memory** is a typed, timestamped unit of knowledge. Every memory carries:

| field            | meaning                                                             |
|------------------|---------------------------------------------------------------------|
| `id`             | stable, content‑derived identifier (`mem_<12 hex>`)                 |
| `kind`           | `episodic` \| `semantic` \| `procedural` \| `preference`            |
| `content`        | the human‑readable text                                             |
| `fingerprint`    | 16‑hex lexical fingerprint of the *normalized* content (dedupe key) |
| `provenance`     | `{ source, detail }` — *why do I believe this?*                     |
| `confidence`     | subjective belief in `[0.0, 1.0]`                                   |
| `tags`           | sorted, de‑duplicated labels                                        |
| `links`          | ids of related memories (a light knowledge graph)                   |
| `created_at`     | logical creation time (epoch seconds)                               |
| `ttl_secs`       | optional time‑to‑live; `null` = keep indefinitely                   |
| `supersedes`     | id of a memory this one replaced (if any)                           |
| `superseded_by`  | id of the memory that replaced this one (derived)                   |
| `tombstoned`     | `true` once retired                                                 |
| `tombstone_reason` | why it was retired (`ttl-expired`, `manual`, …)                   |

### The four kinds

- **episodic** — something that happened at a time ("deploy failed at 14:03").
  Naturally decays; the most common candidate for a TTL.
- **semantic** — a durable fact ("prod region is eu‑west‑1").
- **procedural** — a how‑to / recipe ("rotate the token by running …").
- **preference** — a stated preference ("prefers concise answers"). The most
  conflict‑prone, because preferences change over time.

---

## 2. The append‑only log

State lives in a directory (default `.recallweave/`) with a single file,
`log.jsonl`: **one JSON object per line, in append order.** Nothing earlier is
mutated during normal operation. There are three event types:

- `assert` — a new memory is recorded.
- `tombstone` — an existing memory is retired (`{ id, reason }`).
- `supersede` — a new memory replaces an old one (`{ old_id, new }`).

The **current state** is the deterministic fold of every event in order:

```
log.jsonl (events)  ──replay──▶  materialized state (id → Memory)
      ▲                                   │
      │ append                            ▼
 assert/tombstone/supersede    query / verify / export / compact
```

Because state is *derived*, the log is the single source of truth and history is
never lost until you explicitly compact.

### Integrity chain (tamper‑evidence)

Each record stores `prev` (the digest of the previous record) and `digest` (a
256‑bit hash over the record's canonical payload chained with `prev`). The first
record's `prev` is 64 zeros (GENESIS). `recallweave verify` recomputes the whole
chain and reports:

- sequence gaps,
- broken `prev` links,
- any record whose contents no longer match its digest.

This detects accidental corruption and casual tampering. **It is not a
cryptographic signature** — see limitations.

---

## 3. Lifecycle operations

| operation      | command                | effect                                                            |
|----------------|------------------------|-------------------------------------------------------------------|
| assert         | `add`                  | append an `assert` (or `supersede` if `--supersedes` given)       |
| dedupe         | *(automatic on add)*   | identical normalized content + same kind ⇒ reuse existing id      |
| supersede      | `supersede`            | replace an old memory; old becomes `superseded_by`                |
| forget (one)   | `forget <id>`          | append a `tombstone`                                              |
| forget (TTL)   | `gc`                   | tombstone every expired memory                                    |
| conflicts      | `conflicts`            | report (never mutate) detected conflicts                          |
| compaction     | `compact`              | rewrite the log, dropping retired records, rebuilding the chain   |
| verify         | `verify`               | check the integrity chain                                         |
| query / filter | `list`, `query`, `get` | read the materialized state                                       |
| export         | `export`               | write a portable **memory‑pack** for viewers                      |

### Deterministic dedupe

Content is **normalized** before fingerprinting: trim → collapse internal
whitespace → lowercase. So `"Loves  DARK\tmode"` and `"loves dark mode"` collapse
to the same fingerprint. On `add`, if a **live** memory of the **same kind** has
that fingerprint, no new record is written and the existing id is returned. This
is exact/normalized‑lexical dedupe — deterministic and explainable.

