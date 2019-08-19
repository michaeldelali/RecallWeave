<div align="center">

# RecallWeave

[![CI](https://github.com/michaeldelali/RecallWeave/actions/workflows/ci.yml/badge.svg)](https://github.com/michaeldelali/RecallWeave/actions/workflows/ci.yml) ![license](https://img.shields.io/badge/license-MIT-blue) ![rust](https://img.shields.io/badge/rust-1.74%2B-DEA584)
### a local-first agent memory lifecycle engine

*Threads of memory, laid one pass at a time, into a cloth you can inspect,
mend, and fold — never a black box, never the cloud.*

![the memory loom](docs/assets/memory-loom.svg)

</div>

---

Every long-running agent accumulates a tangle of things it "knows": facts it was
told, preferences it inferred, procedures it was taught, events it witnessed.
Most memory tooling reaches straight for a vector database and calls the problem
solved. But a vector index answers exactly one question — *"what is similar to
this?"* — and stays silent on all the questions a memory actually has to answer
over its lifetime:

- Where did this belief come from, and how sure am I?
- This contradicts what I knew yesterday — which one wins?
- This was only ever true for an afternoon; when do I let it go?
- Has anyone tampered with what I remember?
- Can I hand my whole memory to another tool without leaking the machinery?

**recallweave is the loom, not the thread.** It is the *lifecycle* layer: it
gives every memory an identity, a provenance, a confidence, a time-to-live, and a
place in an append-only, tamper-evident weave. It dedupes, detects conflicts,
forgets on schedule, compacts, verifies its own integrity, and exports a portable
*memory-pack* that a companion viewer turns into a woven tapestry you can read at
a glance.

It is written in **Rust with the standard library only** — no `serde`, no
`clap`, no crypto crates — so the entire format, from the JSON codec to the hash
chain, is a few hundred lines you can audit in an afternoon. The companion
**tapestry viewer** is **TypeScript** and consumes only the exported pack.

> **What it is not.** recallweave does **no semantic search**. Matching is
> lexical. It is a lifecycle engine you run *beside* a vector store, not a
> replacement for one. The [honest limitations](#-honest-limitations) section
> means every word of that.

---

## Table of contents

- [The weaving metaphor](#-the-weaving-metaphor)
- [Quick start](#-quick-start)
- [The threads: memory kinds](#-the-threads-memory-kinds)
- [The loom: the append-only weave](#-the-loom-the-append-only-weave)
- [Tending the cloth: the lifecycle](#-tending-the-cloth-the-lifecycle)
- [Forgetting, on purpose](#-forgetting-on-purpose)
- [The tapestry viewer](#-the-tapestry-viewer)
- [Command reference](#-command-reference)
- [Worked example: an agent's week](#-worked-example-an-agents-week)
- [Design principles](#-design-principles)
- [Honest limitations](#-honest-limitations)
- [Building, testing, layout](#-building-testing-and-project-layout)
- [License](#-license)

---

## 🪡 The weaving metaphor

A loom holds a set of taut vertical threads — the **warp** — under tension. The
weaver passes a **shuttle** back and forth, trailing a horizontal thread — the
**weft** — over and under the warp. Row by row, the passes accumulate into cloth.
Once a row is beaten into place it is *fixed*; you do not go back and re-thread
it. To change the finished cloth you either weave *more* on top, or you cut a
thread and let it fray.

recallweave takes that literally:

| loom                        | recallweave                                              |
|-----------------------------|----------------------------------------------------------|
| a pass of the shuttle       | appending one event to the log                           |
| a weft thread               | a single memory                                          |
| the colour of a thread      | the memory's *kind*                                      |
| thread thickness / length   | *confidence*                                             |
| a finished, beaten-in row   | an immutable log record                                  |
| cutting a thread            | a **tombstone**                                          |
| re-weaving over an old row  | a **supersession**                                       |
| a thread that frays with time | a **TTL** expiry                                        |
| folding the cloth to store it | **compaction**                                         |
| checking the weave for slips | integrity **verification**                              |
| rolling the cloth off the loom to show someone | exporting a **memory-pack**             |

The two animated panels in this README (`docs/assets/memory-loom.svg` and
`docs/assets/forgetting-weave.svg`) are hand-authored SVGs that show the shuttle
weaving typed threads, and threads fading, being cut, and re-woven during
forgetting and compaction. They are local files — no external assets, no
tracking — so they render anywhere the SVG does.

---

## 🚀 Quick start

You need a Rust toolchain (1.74+). Node 18+ is only needed for the viewer.

```bash
# build the engine
cargo build --release

# point every command at a store directory with --dir (default: .recallweave)
alias rw='./target/release/recallweave --dir mymemory'

# weave in some memories
rw add --kind semantic   --content "prod region is eu-west-1" --tags infra,region
rw add --kind preference --content "prefers concise answers"  --tags style --confidence 0.8
rw add --kind procedural --content "rotate the API token via ops runbook step 3" --tags ops
rw add --kind episodic   --content "deploy failed at 14:03"    --tags infra --ttl 86400

# read the cloth
rw list
rw query --kind preference --contains concise
rw stats

# check the weave has not been tampered with
rw verify

# roll it off the loom into a portable pack
rw export --out memory-pack.json
```

Then turn the pack into a tapestry:

```bash
cd viewer
npm install
npm run build
node dist/cli.js ../memory-pack.json --svg tapestry.svg
```

---

## 🎨 The threads: memory kinds

recallweave is *typed on purpose*. A memory's kind changes how you should treat
it, so the type is first-class rather than a free-text tag.

- **🟢 semantic** — durable facts. *"The prod region is eu-west-1."* These are the
  backbone threads; they rarely expire, and when they change you usually
  **supersede** rather than delete.
- **🟡 preference** — stated or inferred preferences. *"Prefers concise answers."*
  The liveliest, most conflict-prone threads: people change their minds. This is
  the only kind the polarity conflict detector inspects.
- **🔵 procedural** — how-to knowledge. *"Rotate the token by running step 3 of the
  ops runbook."* Recipes worth keeping verbatim.
- **🔴 episodic** — things that happened at a moment. *"Deploy failed at 14:03."*
  The threads most likely to fray: give them a **TTL** and let the loom forget
  them on schedule.

Every thread also carries the metadata that makes a memory trustworthy:
**provenance** (`source` + `detail` — *why do I believe this?*), **confidence**
in `[0,1]`, **tags**, **links** to related memories, a **fingerprint** for
dedupe, and lifecycle fields (`ttl_secs`, `supersedes`, `superseded_by`,
`tombstoned`). The full schema lives in [`docs/MEMORY.md`](docs/MEMORY.md).

---

## 🧶 The loom: the append-only weave

The store is a directory (default `.recallweave/`) holding a single
`log.jsonl` — **one JSON event per line, in the order it happened.** Three event
types are ever written:

- `assert` — a new memory (a new weft thread).
- `tombstone` — retire a memory (cut a thread), with a reason.
- `supersede` — replace one memory with another (re-weave a row).

The **current state** is not stored anywhere. It is *derived* by replaying every
event in order:

```
   log.jsonl (events)  ──replay──▶  materialized state (id → Memory)
         ▲                                   │
         │ append                            ▼
    assert / tombstone / supersede    query · verify · export · compact
```

Because state is a pure function of the log, the log is the single source of
truth and nothing is silently lost. You can always ask *"how did I come to
believe this?"* and replay to find out.

### Tamper-evidence: the integrity chain

Each record commits to the one before it. It stores `prev` — the digest of the
previous record — and `digest`, a 256-bit hash over its own canonical payload
*chained* with `prev`. The first record's `prev` is 64 zeros (GENESIS). Alter any
byte of any record and every digest downstream stops matching:

```bash
$ rw verify
integrity OK: 7 records, chain intact

# ... someone edits log.jsonl by hand ...
$ rw verify
integrity FAILED (7 records):
  - seq 3: digest mismatch (record contents were altered)
```

This is a **checksum chain, not a signature.** It catches accidental corruption
and casual tampering. It does not stop a determined attacker who rewrites the
whole file — for that, sign the exported pack out of band. (Said plainly again in
the limitations.)

---

## 🧷 Tending the cloth: the lifecycle

