/**
 * Tests for the tapestry viewer, using Node's built-in assert (no test runner
 * dependency). Run with `npm test`, which compiles then executes this file.
 * The script exits non-zero if any assertion fails.
 */

import assert from "node:assert/strict";
import { parsePackText, PackError, type MemoryPack } from "./pack.js";
import { renderReport } from "./report.js";
import { renderTapestry } from "./svg.js";

let passed = 0;
function test(name: string, fn: () => void): void {
  try {
    fn();
    passed++;
    process.stdout.write(`ok   - ${name}\n`);
  } catch (e) {
    process.stdout.write(`FAIL - ${name}\n  ${(e as Error).message}\n`);
    process.exitCode = 1;
  }
}

function samplePackText(): string {
  const pack: MemoryPack = {
    format: "recallweave-pack",
    format_version: 1,
    generated_at: 1710000500,
    integrity_head: "abc123def456abc123def456abc123def456abc123def456abc123def4560000",
    record_count: 4,
    live_count: 3,
    counts_by_kind: { semantic: 1, preference: 2, episodic: 0, procedural: 0 },
    tag_counts: { ui: 2, infra: 1 },
    conflicts: [
      {
        a: "mem_a",
        b: "mem_b",
        kind: "preference-polarity",
        explanation: "preferences appear to conflict on 'dark' vs 'light'",
      },
    ],
    memories: [
      {
        id: "mem_region",
        kind: "semantic",
        content: "prod region is eu-west-1",
