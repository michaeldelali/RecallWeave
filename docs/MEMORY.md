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
