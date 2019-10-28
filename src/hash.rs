//! Hashing primitives implemented with the standard library only.
//!
//! Two hashes are used across the engine, and they serve different purposes:
//!
//! * [`content_fingerprint`] produces a short, stable hex string derived from the
//!   normalized *content* of a memory. It powers deterministic dedupe: two
//!   memories with the same normalized content share a fingerprint regardless of
//!   when or how they were recorded.
//!
//! * [`digest_hex`] produces a 256-bit digest over arbitrary bytes. It backs the
//!   integrity chain in the append-only log, where each record commits to the
//!   digest of the record before it (a hash chain / tamper-evident ledger).
//!
//! The 256-bit digest is a custom sponge-style construction built on a 64-bit
//! mixing permutation. It is *not* SHA-256 and makes no cryptographic security
//! claims; it is a strong, well-distributed checksum suitable for detecting
//! accidental corruption and casual tampering in a local-first file. This
//! limitation is documented honestly in `docs/MEMORY.md`.

/// FNV-1a 64-bit hash. Deterministic and fast; used for content fingerprints.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Produce a 16-character hex fingerprint from normalized content.
///
/// Normalization: trim, collapse internal whitespace runs to a single space,
/// and lowercase. This means "Loves  DARK\tMode" and "loves dark mode" collapse
/// to the same fingerprint, which is exactly the deterministic dedupe behavior
/// we want for near-identical restatements. It is purely lexical: it will *not*
/// detect paraphrases like "prefers dark theme" (see honest limitations).
pub fn content_fingerprint(content: &str) -> String {
    let normalized = normalize_content(content);
    format!("{:016x}", fnv1a_64(normalized.as_bytes()))
}

