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
        let mut positionals = Vec::new();
        let mut it = args.into_iter();
        while let Some(tok) = it.next() {
            if let Some(name) = tok.strip_prefix("--") {
                if bool_flags.contains(&name) {
                    bools.insert(name.to_string());
                } else {
                    let value = it
                        .next()
                        .ok_or_else(|| format!("flag --{} requires a value", name))?;
                    map.insert(name.to_string(), value);
                }
            } else {
                positionals.push(tok);
            }
        }
        Ok(Flags {
            map,
            bools,
            positionals,
        })
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    fn get_or(&self, key: &str, default: &str) -> String {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn has(&self, key: &str) -> bool {
        self.bools.contains(key)
    }

    fn dir(&self) -> PathBuf {
        PathBuf::from(self.get_or("dir", ".recallweave"))
    }
}

fn parse_tags(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) if !s.is_empty() => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

// ---- commands --------------------------------------------------------------

fn cmd_add(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let kind = MemoryKind::parse(
        flags
            .get("kind")
            .ok_or("add requires --kind <episodic|semantic|procedural|preference>")?,
    )?;
    let content = flags
        .get("content")
        .or_else(|| flags.positionals.first().map(|s| s.as_str()))
        .ok_or("add requires --content <text> (or a positional content argument)")?
        .to_string();
    let confidence: f64 = flags
        .get_or("confidence", "1.0")
        .parse()
        .map_err(|_| "confidence must be a number in [0,1]".to_string())?;
    let ttl = match flags.get("ttl") {
        Some(v) => Some(
            v.parse::<u64>()
                .map_err(|_| "ttl must be a non-negative integer (seconds)".to_string())?,
        ),
        None => None,
    };
    let spec = AssertSpec {
        kind,
        content,
        provenance: Provenance {
            source: flags.get_or("source", "user"),
            detail: flags.get_or("detail", ""),
        },
        confidence,
        tags: parse_tags(flags.get("tags")),
        links: parse_tags(flags.get("links")),
        ttl_secs: ttl,
        supersedes: flags.get("supersedes").map(|s| s.to_string()),
    };

    let mut store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    let (id, deduped) = store.assert(spec, now)?;
    if flags.has("json") {
        let obj = recallweave::json::obj(vec![
            ("id", recallweave::json::s(&id)),
            ("deduped", Json::Bool(deduped)),
        ]);
        println!("{}", obj.to_compact());
    } else if deduped {
        println!("deduped -> existing memory {}", id);
    } else {
        println!("added {}", id);
    }
    Ok(())
}

fn cmd_list(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json", "all"])?;
    let store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    let mems = if flags.has("all") {
        store.all()
    } else {
        store.live(now)
    };
    print_memories(&mems, flags.has("json"));
    Ok(())
}

fn cmd_query(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json", "all"])?;
    let store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    let pool = if flags.has("all") {
        store.all()
    } else {
        store.live(now)
    };
    let query = Query {
        kind: match flags.get("kind") {
            Some(k) => Some(MemoryKind::parse(k)?),
            None => None,
        },
        tag: flags.get("tag").map(|s| s.to_string()),
        contains: flags.get("contains").map(|s| s.to_string()),
        min_confidence: match flags.get("min-confidence") {
            Some(v) => Some(
                v.parse()
                    .map_err(|_| "min-confidence must be a number".to_string())?,
            ),
            None => None,
        },
        source: flags.get("source").map(|s| s.to_string()),
        sort: match flags.get("sort") {
            Some(s) => Some(Sort::parse(s)?),
            None => None,
        },
        limit: match flags.get("limit") {
            Some(v) => Some(
                v.parse()
                    .map_err(|_| "limit must be an integer".to_string())?,
            ),
            None => None,
        },
    };
    let results = query.run(&pool);
    print_memories(&results, flags.has("json"));
    Ok(())
}

fn cmd_get(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let id = flags
        .positionals
        .first()
        .or_else(|| flags.map.get("id"))
        .ok_or("get requires an id (positional or --id)")?
        .clone();
    let store = Store::open(&flags.dir())?;
    let map = store.materialize();
    let mem = map
        .get(&id)
        .ok_or_else(|| format!("no memory with id '{}'", id))?;
    if flags.has("json") {
        println!("{}", mem.to_json().to_pretty());
    } else {
        print_memory_detail(mem);
    }
    Ok(())
}

fn cmd_supersede(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let old_id = flags
        .get("old")
        .ok_or("supersede requires --old <id>")?
        .to_string();
    let content = flags
        .get("content")
        .ok_or("supersede requires --content <text>")?
        .to_string();
    let store_dir = flags.dir();
    let mut store = Store::open(&store_dir)?;
    let now = clock(&flags);
    // Inherit kind from the old memory unless overridden.
    let old_kind = {
        let map = store.materialize();
        let m = map
            .get(&old_id)
            .ok_or_else(|| format!("no memory with id '{}'", old_id))?;
        m.kind
    };
    let kind = match flags.get("kind") {
        Some(k) => MemoryKind::parse(k)?,
        None => old_kind,
    };
    let confidence: f64 = flags
        .get_or("confidence", "1.0")
        .parse()
        .map_err(|_| "confidence must be a number in [0,1]".to_string())?;
    let spec = AssertSpec {
        kind,
        content,
        provenance: Provenance {
            source: flags.get_or("source", "user"),
            detail: flags.get_or("detail", ""),
        },
        confidence,
        tags: parse_tags(flags.get("tags")),
        links: parse_tags(flags.get("links")),
        ttl_secs: None,
        supersedes: Some(old_id.clone()),
    };
    let (new_id, _) = store.assert(spec, now)?;
    if flags.has("json") {
        let obj = recallweave::json::obj(vec![
            ("old", recallweave::json::s(&old_id)),
            ("new", recallweave::json::s(&new_id)),
        ]);
        println!("{}", obj.to_compact());
    } else {
        println!("{} superseded by {}", old_id, new_id);
    }
    Ok(())
}

fn cmd_forget(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let id = flags
        .positionals
        .first()
        .or_else(|| flags.map.get("id"))
        .ok_or("forget requires an id (positional or --id)")?
        .clone();
    let reason = flags.get_or("reason", "manual");
    let mut store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    store.tombstone(&id, &reason, now)?;
    if flags.has("json") {
        println!("{{\"forgotten\":\"{}\"}}", id);
    } else {
        println!("forgot {} ({})", id, reason);
    }
    Ok(())
}

fn cmd_forget_expired(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let mut store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    let forgotten = store.forget_expired(now)?;
    if flags.has("json") {
        let arr =
            recallweave::json::arr(forgotten.iter().map(|i| recallweave::json::s(i)).collect());
        println!("{}", arr.to_compact());
    } else if forgotten.is_empty() {
        println!("nothing expired");
    } else {
        println!("forgot {} expired memories:", forgotten.len());
        for id in forgotten {
            println!("  {}", id);
        }
    }
    Ok(())
}

fn cmd_conflicts(args: VecDeque<String>) -> Result<(), String> {
    let flags = Flags::parse(args, &["json"])?;
    let store = Store::open(&flags.dir())?;
    let now = clock(&flags);
    let conflicts = store.detect_conflicts(now);
    if flags.has("json") {
        let items: Vec<Json> = conflicts
            .iter()
            .map(|c| {
                recallweave::json::obj(vec![
                    ("a", recallweave::json::s(&c.a)),
