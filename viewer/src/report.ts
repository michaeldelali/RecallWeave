/**
 * Renders a memory-pack as a plain-text report for the terminal.
 *
 * This is the default output of the `tapestry` CLI when no `--svg` is requested.
 * It is intentionally compact and dependency-free (no colour libraries) so it
 * behaves in any terminal and in CI logs.
 */

import type { MemoryPack } from "./pack.js";
import { KINDS } from "./pack.js";

function bar(count: number, max: number, width = 24): string {
  if (max <= 0) return "";
  const filled = Math.round((count / max) * width);
  return "█".repeat(filled) + "·".repeat(Math.max(0, width - filled));
}

export function renderReport(pack: MemoryPack): string {
  const lines: string[] = [];
  lines.push("recallweave memory tapestry");
