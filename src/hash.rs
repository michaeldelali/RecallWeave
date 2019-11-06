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

/// The normalization applied before fingerprinting. Exposed for tests and for
/// the conflict detector, which reasons about normalized content too.
pub fn normalize_content(content: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for c in content.trim().chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            prev_space = false;
        }
    }
    out
}

/// A 256-bit digest represented as four 64-bit lanes.
struct Digest256 {
    state: [u64; 4],
}

impl Digest256 {
    fn new() -> Self {
        // Distinct, high-entropy initial constants for each lane (fractional
        // bits of square roots, a common way to pick "nothing up my sleeve"
        // numbers).
        Digest256 {
            state: [
                0x6a09e667f3bcc908,
                0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1,
            ],
        }
    }

    fn absorb(&mut self, bytes: &[u8]) {
        let mut counter: u64 = 0;
        for &b in bytes {
            let lane = (counter % 4) as usize;
            self.state[lane] ^= (b as u64).wrapping_add(counter);
            self.state[lane] = mix64(self.state[lane]);
            // Cross-diffuse into the next lane so byte position matters.
            let next = ((counter + 1) % 4) as usize;
            self.state[next] = self.state[next].rotate_left(17) ^ self.state[lane];
