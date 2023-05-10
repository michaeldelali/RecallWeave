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
OLD_ID=$(rw query --kind semantic --json --now 1710000050 | grep -o 'mem_[0-9a-f]*' | head -n1)

echo "== the region changed: supersede the old fact =="
rw supersede --old "$OLD_ID" --content "prod region is eu-west-1" --confidence 0.99 --tags infra,region --now 1710000100

echo "== record some preferences (two of them will conflict) =="
rw add --kind preference --content "prefers concise answers" --tags style --confidence 0.8 --now 1710000200
