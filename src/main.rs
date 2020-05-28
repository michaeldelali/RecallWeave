//! The `recallweave` command-line interface.
//!
//! A hand-rolled argument parser (no clap; stdlib only) exposes the memory
//! lifecycle as subcommands. Every command operates on a store directory chosen
//! with `--dir` (default `.recallweave`). Output is human-readable text by
//! default, or machine JSON with `--json`, so the CLI is scriptable.
//!
//! Run `recallweave help` for the full command list.

use recallweave::json::Json;
use recallweave::model::{MemoryKind, Provenance};
use recallweave::query::{Query, Sort};
use recallweave::store::{now_epoch, AssertSpec, Store};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut args: VecDeque<String> = args.into();
    if args.is_empty() {
        print_help();
        return Ok(());
    }
    // Global flags may appear before the subcommand (e.g. `--dir X list`).
    // We scan forward for the first token that is not a flag or a flag value,
    // treating that as the command and leaving all other tokens for the
    // per-command flag parser. `--json`, `--all` are known value-less flags.
    const VALUELESS: &[&str] = &["json", "all", "help", "version"];
    let mut leading: Vec<String> = Vec::new();
    let mut cmd: Option<String> = None;
    while let Some(tok) = args.pop_front() {
        if let Some(name) = tok.strip_prefix("--") {
            // A leading --help/--version acts immediately, regardless of order.
            if name == "help" {
                print_help();
                return Ok(());
            }
            if name == "version" {
                println!("recallweave {}", recallweave::VERSION);
                return Ok(());
            }
            leading.push(tok.clone());
            if !VALUELESS.contains(&name) {
                // consume this flag's value too
                if let Some(v) = args.pop_front() {
                    leading.push(v);
                }
            }
        } else if tok == "-h" {
            print_help();
            return Ok(());
        } else if tok == "-V" {
            println!("recallweave {}", recallweave::VERSION);
            return Ok(());
        } else {
            cmd = Some(tok);
            break;
        }
    }
    let cmd = match cmd {
        Some(c) => c,
        None => {
            // Only global flags were given; nothing to do but show help.
            print_help();
            return Ok(());
        }
    };
    // Re-attach the leading global flags in front of the remaining args so the
    // per-command parser sees them.
    for tok in leading.into_iter().rev() {
        args.push_front(tok);
    }
    match cmd.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("recallweave {}", recallweave::VERSION);
            Ok(())
        }
        "add" => cmd_add(args),
        "list" => cmd_list(args),
        "query" => cmd_query(args),
        "get" => cmd_get(args),
        "supersede" => cmd_supersede(args),
        "forget" => cmd_forget(args),
        "gc" | "forget-expired" => cmd_forget_expired(args),
        "conflicts" => cmd_conflicts(args),
        "compact" => cmd_compact(args),
        "verify" => cmd_verify(args),
        "export" => cmd_export(args),
        "stats" => cmd_stats(args),
        other => Err(format!(
            "unknown command '{}'. Try 'recallweave help'.",
            other
        )),
    }
}

// ---- flag parsing helpers --------------------------------------------------

/// A tiny option bag: pulls `--key value` and `--flag` out of the arg deque,
/// leaving positional arguments behind.
struct Flags {
    map: std::collections::HashMap<String, String>,
    bools: std::collections::HashSet<String>,
    positionals: Vec<String>,
}

impl Flags {
    fn parse(args: VecDeque<String>, bool_flags: &[&str]) -> Result<Flags, String> {
        let mut map = std::collections::HashMap::new();
        let mut bools = std::collections::HashSet::new();
