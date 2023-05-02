#!/bin/sh
# recallweave demo — builds a small memory store with the real binary, exercises
# the full lifecycle (assert, dedupe, supersede, TTL, forget, conflicts,
# compaction, verify) and exports a memory-pack.
#
# Run from the project root:  sh examples/demo.sh
# It is deterministic: every command pins --now so ids and output are stable.

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/recallweave"
STORE="$ROOT/examples/demo-store"
