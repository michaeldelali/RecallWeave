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
PACK="$ROOT/examples/memory-pack.json"

if [ ! -x "$BIN" ]; then
  echo "building release binary..."
  ( cd "$ROOT" && cargo build --release )
fi

rw() { "$BIN" --dir "$STORE" "$@"; }

rm -rf "$STORE"

echo "== weaving semantic + preference + procedural + episodic threads =="
rw add --kind semantic   --content "prod region is us-east-1" --tags infra,region --source user --now 1710000000
# capture the id of that semantic memory for supersession
