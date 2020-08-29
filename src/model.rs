//! The memory domain model.
//!
//! A *memory* is a typed, timestamped unit of knowledge an agent wants to keep.
//! Memories are never mutated in place; instead the engine records *events* in an
//! append-only log (see [`crate::store`]). The current state of the store is the
//! result of folding every event in order. This module defines the value types
//! and their conversion to/from the [`Json`] representation used on disk.

use crate::hash::content_fingerprint;
use crate::json::{arr, num, obj, s, Json};
use std::collections::BTreeMap;

/// The four memory kinds the engine understands.
///
/// * `Episodic`   — something that happened at a point in time ("the deploy on
///   Tuesday failed"). Naturally decays; good candidate for TTL.
/// * `Semantic`   — a durable fact ("the prod region is eu-west-1").
/// * `Procedural` — a how-to / recipe ("to rotate the token, run ...").
/// * `Preference` — a stated preference ("prefers concise answers"). These are
///   the most conflict-prone because they change over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Preference,
}

impl MemoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryKind::Episodic => "episodic",
            MemoryKind::Semantic => "semantic",
            MemoryKind::Procedural => "procedural",
            MemoryKind::Preference => "preference",
        }
    }

    pub fn parse(value: &str) -> Result<MemoryKind, String> {
        match value {
            "episodic" => Ok(MemoryKind::Episodic),
            "semantic" => Ok(MemoryKind::Semantic),
            "procedural" => Ok(MemoryKind::Procedural),
            "preference" => Ok(MemoryKind::Preference),
            other => Err(format!(
                "unknown memory kind '{}' (expected episodic|semantic|procedural|preference)",
                other
            )),
        }
    }
}

/// Where a memory came from. Provenance is first-class: an agent should always
/// be able to answer "why do I believe this?".
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    /// e.g. "user", "tool:web", "inference", "import"
    pub source: String,
    /// Free-form detail: a URL, a conversation id, a tool name, etc.
    pub detail: String,
}

impl Provenance {
    pub fn to_json(&self) -> Json {
        obj(vec![
            ("source", s(&self.source)),
            ("detail", s(&self.detail)),
        ])
    }

    pub fn from_json(v: &Json) -> Result<Provenance, String> {
        Ok(Provenance {
            source: v
                .get("source")
                .and_then(Json::as_str)
                .unwrap_or("unknown")
                .to_string(),
            detail: v
                .get("detail")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_string(),
        })
    }
}

/// A single memory record in its current, materialized form.
#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
    pub id: String,
    pub kind: MemoryKind,
    pub content: String,
