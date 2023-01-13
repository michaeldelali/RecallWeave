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

export interface Provenance {
  source: string;
  detail: string;
}

export interface Memory {
  id: string;
  kind: MemoryKind;
  content: string;
  fingerprint: string;
  provenance: Provenance;
  confidence: number;
  tags: string[];
  links: string[];
  created_at: number;
  ttl_secs: number | null;
  supersedes: string | null;
  superseded_by: string | null;
  tombstoned: boolean;
  tombstone_reason: string | null;
}

export interface Conflict {
  a: string;
  b: string;
  kind: string;
  explanation: string;
}

export interface MemoryPack {
  format: "recallweave-pack";
  format_version: number;
  generated_at: number;
