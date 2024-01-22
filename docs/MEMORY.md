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

