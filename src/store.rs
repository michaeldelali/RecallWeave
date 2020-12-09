//! The append-only memory store and its lifecycle operations.
//!
//! On-disk layout is a directory (default `.recallweave/`) containing a single
//! `log.jsonl` file: one JSON object per line, in append order. The engine never
//! rewrites earlier lines during normal operation — new facts, tombstones, and
//! supersessions are *appended*. Compaction is the sole operation that rewrites
//! the file, and it does so by producing a new, smaller log that preserves the
//! live state and the integrity chain.
//!
//! ## The pipeline
//!
//! ```text
//!   log.jsonl (events)  --replay-->  materialized state (Memory map)
//!         ^                                   |
//!         | append                            v
//!    assert/tombstone/supersede        query / verify / export / compact
//! ```
//!
//! Everything below builds on that fold-the-log model.

use crate::hash::{chain_digest, content_fingerprint};
use crate::json::Json;
use crate::model::{Event, LogRecord, Memory, MemoryKind, Provenance};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The zeroed digest that precedes the very first record.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The append-only store, bound to a directory on disk.
pub struct Store {
    dir: PathBuf,
    records: Vec<LogRecord>,
}

/// Parameters for asserting a new memory.
pub struct AssertSpec {
    pub kind: MemoryKind,
    pub content: String,
    pub provenance: Provenance,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub ttl_secs: Option<u64>,
    pub supersedes: Option<String>,
}

/// A detected conflict between two live memories.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub a: String,
    pub b: String,
    pub kind: String,
    pub explanation: String,
}

/// Summary returned by [`Store::compact`].
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionReport {
    pub before_records: usize,
    pub after_records: usize,
    pub dropped_tombstoned: usize,
    pub dropped_superseded: usize,
    pub dropped_expired: usize,
}

/// Result of [`Store::verify`].
#[derive(Debug, Clone, PartialEq)]
pub struct VerifyReport {
    pub records: usize,
    pub ok: bool,
    pub errors: Vec<String>,
}

impl Store {
    fn log_path(dir: &Path) -> PathBuf {
        dir.join("log.jsonl")
    }

    /// Open an existing store or create an empty one at `dir`.
