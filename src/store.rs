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
    pub fn open(dir: &Path) -> Result<Store, String> {
        let path = Self::log_path(dir);
        let records = if path.exists() {
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("reading {}: {}", path.display(), e))?;
            let mut recs = Vec::new();
            for (i, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let value =
                    Json::parse(line).map_err(|e| format!("line {}: parse error: {}", i + 1, e))?;
                let rec =
                    LogRecord::from_json(&value).map_err(|e| format!("line {}: {}", i + 1, e))?;
                recs.push(rec);
            }
            recs
        } else {
            Vec::new()
        };
        Ok(Store {
            dir: dir.to_path_buf(),
            records,
        })
    }

    /// Number of log records currently held.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The digest of the most recent record (or GENESIS if empty).
    fn head_digest(&self) -> String {
        self.records
            .last()
            .map(|r| r.digest.clone())
            .unwrap_or_else(|| GENESIS.to_string())
    }

    fn next_seq(&self) -> u64 {
        self.records.last().map(|r| r.seq + 1).unwrap_or(1)
    }

    /// Build a record with proper chaining, append it in memory, and persist.
    fn append(&mut self, ts: u64, event: Event) -> Result<(), String> {
        let seq = self.next_seq();
        let prev = self.head_digest();
        // Compute the digest over the canonical payload.
        let mut partial = LogRecord {
            seq,
            ts,
            event,
            prev: prev.clone(),
            digest: String::new(),
        };
        let payload = partial.payload_json().to_compact();
        partial.digest = chain_digest(&prev, payload.as_bytes());
        self.records.push(partial);
        self.persist()
    }

    /// Write the entire log to disk atomically (write temp, then rename).
    fn persist(&self) -> Result<(), String> {
        fs::create_dir_all(&self.dir)
            .map_err(|e| format!("creating {}: {}", self.dir.display(), e))?;
        let path = Self::log_path(&self.dir);
        let tmp = self.dir.join("log.jsonl.tmp");
        let mut file = fs::File::create(&tmp).map_err(|e| format!("creating temp log: {}", e))?;
        for rec in &self.records {
            let line = rec.to_json().to_compact();
            file.write_all(line.as_bytes())
                .and_then(|_| file.write_all(b"\n"))
                .map_err(|e| format!("writing log: {}", e))?;
        }
        file.flush().map_err(|e| format!("flushing log: {}", e))?;
        drop(file);
        fs::rename(&tmp, &path).map_err(|e| format!("renaming temp log into place: {}", e))?;
        Ok(())
    }

    /// Fold every event into the current materialized memory map.
    ///
    /// This is the definitive interpreter of the log. Ordering matters:
    /// supersession and tombstone events are applied to whatever the map holds
    /// at that point, so replaying is fully deterministic.
    pub fn materialize(&self) -> BTreeMap<String, Memory> {
        let mut map: BTreeMap<String, Memory> = BTreeMap::new();
        for rec in &self.records {
            match &rec.event {
                Event::Assert(m) => {
                    map.insert(m.id.clone(), m.clone());
                }
                Event::Tombstone { id, reason } => {
                    if let Some(m) = map.get_mut(id) {
                        m.tombstoned = true;
                        m.tombstone_reason = Some(reason.clone());
                    }
                }
                Event::Supersede { old_id, new } => {
                    if let Some(old) = map.get_mut(old_id) {
                        old.superseded_by = Some(new.id.clone());
                    }
                    let mut new_mem = new.clone();
                    if new_mem.supersedes.is_none() {
                        new_mem.supersedes = Some(old_id.clone());
                    }
                    map.insert(new_mem.id.clone(), new_mem);
                }
            }
        }
        map
    }

    /// All live memories at time `now`, sorted by id for deterministic output.
    pub fn live(&self, now: u64) -> Vec<Memory> {
        let mut v: Vec<Memory> = self
            .materialize()
            .into_values()
            .filter(|m| m.is_live(now))
            .collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// All memories (including retired) sorted by creation then id.
    pub fn all(&self) -> Vec<Memory> {
        let mut v: Vec<Memory> = self.materialize().into_values().collect();
        v.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        v
    }

    /// Deterministically derive an id from content, kind and creation time.
    ///
    /// The id is stable: the same (kind, content, created_at) always yields the
    /// same id. That makes fixtures and tests reproducible and means importing
    /// the same memory twice collapses to one id.
    pub fn derive_id(kind: MemoryKind, content: &str, created_at: u64) -> String {
        let seed = format!(
            "{}|{}|{}",
            kind.as_str(),
            content_fingerprint(content),
            created_at
        );
        let fp = content_fingerprint(&seed);
        format!("mem_{}", &fp[..12])
    }

    /// Assert a new memory. Returns `(id, deduped)`.
    ///
    /// Deterministic dedupe: if a *live* memory of the same kind already has the
    /// same content fingerprint, no new record is written and the existing id is
    /// returned with `deduped = true`. This is exact/normalized-lexical dedupe,
    /// not semantic — see honest limitations in the docs.
    pub fn assert(&mut self, spec: AssertSpec, now: u64) -> Result<(String, bool), String> {
        let fingerprint = content_fingerprint(&spec.content);
        let existing = self.materialize();
        for m in existing.values() {
            if m.is_live(now) && m.kind == spec.kind && m.fingerprint == fingerprint {
                return Ok((m.id.clone(), true));
            }
        }
        let id = Self::derive_id(spec.kind, &spec.content, now);
        let mut tags = spec.tags.clone();
        tags.sort();
        tags.dedup();
        let mem = Memory {
            id: id.clone(),
            kind: spec.kind,
            content: spec.content,
            fingerprint,
            provenance: spec.provenance,
            confidence: spec.confidence.clamp(0.0, 1.0),
            tags,
            links: spec.links,
            created_at: now,
            ttl_secs: spec.ttl_secs,
            supersedes: spec.supersedes.clone(),
            superseded_by: None,
            tombstoned: false,
            tombstone_reason: None,
        };
        match spec.supersedes {
            Some(old_id) => self.append(now, Event::Supersede { old_id, new: mem })?,
            None => self.append(now, Event::Assert(mem))?,
        }
        Ok((id, false))
    }

    /// Retire a memory by id with a reason. No-op error if the id is unknown.
    pub fn tombstone(&mut self, id: &str, reason: &str, now: u64) -> Result<(), String> {
        let map = self.materialize();
        if !map.contains_key(id) {
            return Err(format!("no memory with id '{}'", id));
        }
        self.append(
            now,
            Event::Tombstone {
                id: id.to_string(),
