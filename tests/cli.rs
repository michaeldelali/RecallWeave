//! Black-box tests that invoke the compiled `recallweave` binary.
//!
//! Cargo sets `CARGO_BIN_EXE_recallweave` to the path of the built binary for
//! integration tests, so we can drive the real CLI and assert on its output and
//! exit codes. Each test uses an isolated `--dir` and a fixed `--now` so results
//! are deterministic.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_recallweave")
}

fn fresh_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut dir = env::temp_dir();
    dir.push(format!(
        "recallweave_cli_{}_{}_{}",
        tag,
        std::process::id(),
        nanos
    ));
    dir
}

struct Out {
