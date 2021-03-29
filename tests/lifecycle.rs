//! End-to-end lifecycle tests exercised through the public library API.
//!
//! These tests build a store in a fresh temp directory, drive it through a full
//! lifecycle (assert -> dedupe -> supersede -> forget -> compact), and assert on
//! integrity and the exported pack at each stage. They complement the focused
//! unit tests inside each module.

use recallweave::json::Json;
use recallweave::model::{MemoryKind, Provenance};
use recallweave::query::{Query, Sort};
use recallweave::store::{AssertSpec, Store};
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn fresh_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut dir = env::temp_dir();
    dir.push(format!(
        "recallweave_it_{}_{}_{}",
        tag,
        std::process::id(),
        nanos
    ));
    dir
}

fn spec(kind: MemoryKind, content: &str, conf: f64, tags: &[&str]) -> AssertSpec {
    AssertSpec {
        kind,
        content: content.to_string(),
        provenance: Provenance {
