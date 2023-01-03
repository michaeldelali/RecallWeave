/**
 * Typed model and validating parser for a recallweave memory-pack.
 *
 * The pack format is produced by the Rust engine's `export` command. This module
 * is the single point where the viewer validates that a file really is a pack
 * (checking `format` and `format_version`) and narrows it into strongly typed
 * structures. Everything downstream — the report and the SVG tapestry — consumes
 * these types, never raw JSON.
 */

export type MemoryKind = "episodic" | "semantic" | "procedural" | "preference";

export const KINDS: readonly MemoryKind[] = [
  "episodic",
  "semantic",
  "procedural",
  "preference",
];

