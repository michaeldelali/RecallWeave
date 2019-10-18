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
