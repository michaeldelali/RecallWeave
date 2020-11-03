//! Query and filtering over the materialized memory set.
//!
//! Queries are deterministic and purely lexical. A [`Query`] combines optional
//! constraints (kind, tag, substring, minimum confidence, source) that are ANDed
//! together, plus sorting and a limit. There is no ranking model and no semantic
//! matching — substring search means substring search. This is a deliberate,
//! honest design choice documented in `docs/MEMORY.md`.

use crate::hash::normalize_content;
use crate::model::{Memory, MemoryKind};

/// Sort order for query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// Newest first (descending creation time).
    Newest,
    /// Oldest first (ascending creation time).
    Oldest,
    /// Highest confidence first.
    Confidence,
    /// By id, ascending (fully deterministic tiebreak-free ordering).
    Id,
}

impl Sort {
    pub fn parse(v: &str) -> Result<Sort, String> {
        match v {
            "newest" => Ok(Sort::Newest),
            "oldest" => Ok(Sort::Oldest),
            "confidence" => Ok(Sort::Confidence),
            "id" => Ok(Sort::Id),
            other => Err(format!(
                "unknown sort '{}' (expected newest|oldest|confidence|id)",
                other
            )),
        }
    }
}

/// A conjunctive query over memories.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub kind: Option<MemoryKind>,
    pub tag: Option<String>,
    /// Case-insensitive substring match against normalized content.
    pub contains: Option<String>,
    pub min_confidence: Option<f64>,
    pub source: Option<String>,
    pub sort: Option<Sort>,
    pub limit: Option<usize>,
}

impl Query {
