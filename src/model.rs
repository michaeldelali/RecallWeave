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

