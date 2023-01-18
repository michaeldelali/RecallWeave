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
  integrity_head: string;
  record_count: number;
  live_count: number;
  counts_by_kind: Record<string, number>;
  tag_counts: Record<string, number>;
  conflicts: Conflict[];
  memories: Memory[];
}

/** Thrown when a file is not a valid recallweave pack. */
export class PackError extends Error {}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function asString(v: unknown, path: string): string {
  if (typeof v !== "string") throw new PackError(`${path}: expected string`);
  return v;
}

function asNumber(v: unknown, path: string): number {
  if (typeof v !== "number" || Number.isNaN(v)) {
    throw new PackError(`${path}: expected number`);
  }
  return v;
}

function asBool(v: unknown, path: string): boolean {
  if (typeof v !== "boolean") throw new PackError(`${path}: expected boolean`);
  return v;
}

