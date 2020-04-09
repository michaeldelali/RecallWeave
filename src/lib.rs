//! `recallweave` — a local-first agent memory lifecycle engine.
//!
//! This crate is intentionally dependency-free (Rust standard library only). It
//! models an agent's long-term memory as an append-only, integrity-chained log
//! of typed memories and provides the full lifecycle around them: assertion,
//! deterministic dedupe, supersession, tombstoning, TTL-based forgetting,
//! compaction, conflict detection, integrity verification, query/filter, and a
//! portable export format ("memory-pack").
//!
