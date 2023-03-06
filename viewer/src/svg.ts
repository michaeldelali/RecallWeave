/**
 * Renders a memory-pack as a woven "tapestry" SVG.
 *
 * The metaphor is a loom: each memory kind is a horizontal *warp* band, and each
 * memory is a *weft* thread laid across it. Thread length encodes confidence,
 * colour encodes kind, and a small knot marks memories that carry links. Detected
 * conflicts are drawn as crossed threads between the two memories involved.
 *
 * The output is a standalone, dependency-free SVG string (no external fonts or
 * images) so it renders anywhere and can be committed to a repo.
 */

import type { MemoryPack, Memory, MemoryKind } from "./pack.js";
import { KINDS } from "./pack.js";

const KIND_COLOR: Record<MemoryKind, string> = {
  episodic: "#e07a5f", // terracotta
  semantic: "#3d8361", // moss
  procedural: "#5f7adb", // indigo
  preference: "#d4a017", // saffron
};

const WIDTH = 900;
const MARGIN = 60;
const BAND_HEIGHT = 90;
const HEADER = 120;
const FOOTER = 70;

function escapeXml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Deterministic pseudo-random in [0,1) from a string seed (FNV-1a based). */
function seededUnit(seed: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < seed.length; i++) {
    h ^= seed.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return (h % 100000) / 100000;
}

interface ThreadPos {
  id: string;
  x: number;
  y: number;
  color: string;
}

/**
 * Build the SVG string for a pack. The layout is fully deterministic given the
 * pack contents, so re-rendering the same pack yields byte-identical output.
 */
export function renderTapestry(pack: MemoryPack): string {
  const bands = KINDS.filter((k) => (pack.counts_by_kind[k] ?? 0) > 0);
  const height =
    HEADER + FOOTER + Math.max(1, bands.length) * (BAND_HEIGHT + 24);

  const parts: string[] = [];
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${height}" ` +
      `width="${WIDTH}" height="${height}" role="img" ` +
      `aria-label="recallweave memory tapestry with ${pack.live_count} live memories">`,
  );
  parts.push(defs());
  parts.push(
    `<rect x="0" y="0" width="${WIDTH}" height="${height}" fill="url(#linen)"/>`,
  );

  // Title + integrity head.
  parts.push(
    `<text x="${MARGIN}" y="48" font-family="Georgia, 'Times New Roman', serif" ` +
      `font-size="30" fill="#2b2118" font-weight="bold">recallweave tapestry</text>`,
  );
  parts.push(
    `<text x="${MARGIN}" y="76" font-family="monospace" font-size="13" fill="#6b5d4f">` +
      `${pack.live_count} live memories · ${pack.record_count} log records · head ` +
      `${escapeXml(pack.integrity_head.slice(0, 16))}…</text>`,
  );

  // Animated shuttle gliding across the top (the loom's shuttle passing yarn).
  const shuttleTop = 96;
  parts.push(
    `<g transform="translate(0,${shuttleTop})">` +
      `<rect x="-40" y="-7" width="40" height="14" rx="7" fill="#8a5a2b">` +
      `<animate attributeName="x" values="${MARGIN};${WIDTH - MARGIN - 40};${MARGIN}" ` +
      `dur="7s" repeatCount="indefinite"/></rect></g>`,
  );

  const positions: Record<string, ThreadPos> = {};
