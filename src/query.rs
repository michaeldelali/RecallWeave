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
