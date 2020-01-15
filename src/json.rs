//! A small, dependency-free JSON implementation.
//!
//! `recallweave` deliberately avoids third-party crates (see `docs/MEMORY.md`),
//! so we ship a compact hand-written JSON parser and serializer. It supports the
//! subset of JSON needed by the memory format: objects, arrays, strings, numbers
//! (integer and float), booleans, and null. It is not a general-purpose library
//! but it is correct for the data this engine produces and consumes, and it round
//! trips every value we emit.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// All numbers are stored as f64. Integer-valued numbers are serialized
    /// without a fractional part so IDs and counters stay stable.
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Objects use a BTreeMap so serialization is deterministic (sorted keys).
    /// Determinism matters: integrity hashes are computed over serialized bytes.
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Num(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Obj(o) => Some(o),
            _ => None,
        }
    }

    /// Convenience accessor for a field of an object.
    pub fn get(&self, key: &str) -> Option<&Json> {
        self.as_object().and_then(|o| o.get(key))
    }

